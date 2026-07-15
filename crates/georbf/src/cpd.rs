//! Rank diagnosis and polynomial null-space construction for CPD systems.
//!
//! The public matrices in this module are GeoRBF-owned row-major values. The
//! internal numerical adapter uses nalgebra only for column-pivoted QR, SVD,
//! and orthogonalization; no third-party matrix type crosses the API boundary.

use std::error::Error;
use std::fmt;

use nalgebra::{DMatrix, linalg::SVD};

use crate::dimension::{Dim, SupportedDimension};
use crate::functional::{CenterRepresenter, FunctionalError, FunctionalProvenance};
use crate::polynomial::PolynomialSpace;

const EQUILIBRATION_PASSES: usize = 8;
const SVD_MAX_ITERATIONS: usize = 10_000;
const AMBIGUITY_FACTOR: f64 = 16.0;
const RESIDUAL_TOLERANCE_FACTOR: f64 = 64.0;

/// A finite, owned row-major matrix used by the CPD API.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct CpdMatrix {
    rows: usize,
    columns: usize,
    values: Vec<f64>,
}

impl CpdMatrix {
    /// Constructs a finite row-major matrix.
    ///
    /// Zero-sized output matrices are permitted when their data length is
    /// zero. CPD action and energy inputs apply their own nonempty constraints.
    ///
    /// # Errors
    ///
    /// Returns a structured error for shape overflow, a data-length mismatch,
    /// or a non-finite entry.
    pub fn try_from_row_major(
        rows: usize,
        columns: usize,
        values: Vec<f64>,
    ) -> Result<Self, CpdError> {
        let expected = rows
            .checked_mul(columns)
            .ok_or(CpdError::MatrixShapeOverflow { rows, columns })?;
        if values.len() != expected {
            return Err(CpdError::MatrixLengthMismatch {
                rows,
                columns,
                expected,
                actual: values.len(),
            });
        }
        if let Some((index, value)) = values
            .iter()
            .copied()
            .enumerate()
            .find(|(_, value)| !value.is_finite())
        {
            return Err(CpdError::NonFiniteMatrixEntry {
                row: index / columns.max(1),
                column: index % columns.max(1),
                value,
            });
        }
        Ok(Self {
            rows,
            columns,
            values,
        })
    }

    /// Returns the row count.
    #[must_use]
    pub const fn rows(&self) -> usize {
        self.rows
    }

    /// Returns the column count.
    #[must_use]
    pub const fn columns(&self) -> usize {
        self.columns
    }

    /// Borrows all entries in row-major order.
    #[must_use]
    pub fn values(&self) -> &[f64] {
        &self.values
    }

    /// Borrows one complete row, or returns `None` for an invalid row index.
    #[must_use]
    pub fn row(&self, row: usize) -> Option<&[f64]> {
        let start = row.checked_mul(self.columns)?;
        self.values.get(start..start.checked_add(self.columns)?)
    }

    /// Returns one entry, or `None` for an invalid index.
    #[must_use]
    pub fn get(&self, row: usize, column: usize) -> Option<f64> {
        if row >= self.rows || column >= self.columns {
            return None;
        }
        self.values.get(row * self.columns + column).copied()
    }
}

/// Recorded norms for one matrix representation.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct CpdMatrixNorms {
    /// Largest absolute entry.
    pub max_absolute: f64,
    /// Maximum absolute row sum.
    pub infinity: f64,
}

/// Final result of the scale-aware rank decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CpdRankDecision {
    /// Both rank reviews agree on clear full column rank.
    FullRank,
    /// Both rank reviews agree on clear column-rank deficiency.
    Deficient,
    /// A singular value lies in the guard band or the reviews disagree.
    Ambiguous,
}

/// Complete recorded evidence for a CPD polynomial-action rank decision.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct CpdRankDiagnostics {
    /// Number of center-functional rows.
    pub rows: usize,
    /// Number of complete polynomial-space columns.
    pub columns: usize,
    /// Fixed count of alternating infinity-norm equilibration passes.
    pub equilibration_passes: usize,
    /// Multipliers satisfying `scaled = diag(row_scales) Q diag(column_scales)`.
    pub row_scales: Vec<f64>,
    /// Polynomial-column multipliers used by the equilibration.
    pub column_scales: Vec<f64>,
    /// Rows that are exactly zero before equilibration.
    pub zero_rows: Vec<usize>,
    /// Columns that are exactly zero before equilibration.
    pub zero_columns: Vec<usize>,
    /// Norms of the original polynomial-action matrix.
    pub original_norms: CpdMatrixNorms,
    /// Norms after dimensionless equilibration.
    pub scaled_norms: CpdMatrixNorms,
    /// Absolute diagonal of the column-pivoted QR `R` factor.
    pub rrqr_diagonal: Vec<f64>,
    /// RRQR threshold `max(rows, columns) * eps * max(abs(diag(R)))`.
    pub rrqr_threshold: f64,
    /// Effective RRQR rank using a strict threshold comparison.
    pub rrqr_rank: usize,
    /// Singular values from the bounded SVD review in descending order.
    pub singular_values: Vec<f64>,
    /// SVD threshold `max(rows, columns) * eps * sigma_max`.
    pub svd_threshold: f64,
    /// Effective SVD rank using a strict threshold comparison.
    pub svd_rank: usize,
    /// Lower edge of the multiplicative ambiguity guard band.
    pub ambiguity_lower: f64,
    /// Upper edge of the multiplicative ambiguity guard band.
    pub ambiguity_upper: f64,
    /// Whether any singular value falls inside the closed ambiguity band.
    pub threshold_adjacent: bool,
    /// Whether RRQR and SVD produced different effective ranks.
    pub rank_disagreement: bool,
    /// Scaled spectral condition estimate, or infinity for a deficient matrix.
    pub condition_estimate: f64,
    /// Conservative final classification.
    pub decision: CpdRankDecision,
}

/// Scale-aware quality evidence for the constructed null-space basis.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct CpdNullSpaceQuality {
    /// Normalized infinity residual of `Q^T Z` after polynomial-column scaling.
    pub side_condition_residual: f64,
    /// Absolute infinity residual of `Q^T Z` in the original action units.
    pub original_side_condition_residual: f64,
    /// Infinity norm of `Z^T Z - I`.
    pub orthonormality_residual: f64,
    /// Shared dimension-aware verification tolerance.
    pub tolerance: f64,
}

/// Immutable CPD polynomial side-condition and its orthonormal null space.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct CpdNullSpace {
    actions: CpdMatrix,
    basis: CpdMatrix,
    diagnostics: CpdRankDiagnostics,
    center_provenance: Vec<FunctionalProvenance>,
    center_provenance_offsets: Vec<usize>,
    quality: CpdNullSpaceQuality,
}

impl CpdNullSpace {
    /// Assembles `Q[j, alpha] = M_j p_alpha`, diagnoses full column rank, and
    /// constructs an orthonormal basis for `null(Q^T)`.
    ///
    /// The rank review always performs the recorded RRQR screen and bounded
    /// SVD review. Deficient or threshold-ambiguous inputs fail explicitly;
    /// no coordinate perturbation, jitter, pseudoinverse, or minimum-norm
    /// fallback is used.
    ///
    /// # Errors
    ///
    /// Returns structured assembly, allocation, equilibration, backend, rank,
    /// ambiguity, or null-space verification diagnostics.
    pub fn try_from_centers<const D: usize>(
        centers: &[CenterRepresenter<D>],
        space: &PolynomialSpace<D>,
    ) -> Result<Self, CpdError>
    where
        Dim<D>: SupportedDimension,
    {
        if centers.is_empty() {
            return Err(CpdError::EmptyCenters);
        }
        let rows = centers.len();
        let columns = space.term_count();
        let entry_count = rows
            .checked_mul(columns)
            .ok_or(CpdError::MatrixShapeOverflow { rows, columns })?;
        let mut values = try_zeroed(entry_count, CpdStorage::PolynomialActions)?;
        let mut polynomial_values = try_zeroed(columns, CpdStorage::PolynomialValues)?;
        let mut polynomial_gradients =
            try_filled(columns, [0.0; D], CpdStorage::PolynomialGradients)?;
        let provenance_count = centers.iter().try_fold(0_usize, |count, center| {
            count.checked_add(center.expression().term_count())
        });
        let Some(provenance_count) = provenance_count else {
            return Err(CpdError::ProvenanceCountOverflow);
        };
        let mut center_provenance = Vec::new();
        center_provenance
            .try_reserve_exact(provenance_count)
            .map_err(|_| CpdError::AllocationFailed {
                storage: CpdStorage::CenterProvenance,
                requested: provenance_count,
            })?;
        let offset_count = rows
            .checked_add(1)
            .ok_or(CpdError::ProvenanceCountOverflow)?;
        let mut center_provenance_offsets = Vec::new();
        center_provenance_offsets
            .try_reserve_exact(offset_count)
            .map_err(|_| CpdError::AllocationFailed {
                storage: CpdStorage::CenterProvenance,
                requested: offset_count,
            })?;
        center_provenance_offsets.push(0);

        for (center_index, center) in centers.iter().enumerate() {
            let start = center_index * columns;
            center
                .expression()
                .try_apply_polynomial_with_scratch(
                    space,
                    &mut values[start..start + columns],
                    &mut polynomial_values,
                    &mut polynomial_gradients,
                )
                .map_err(|source| CpdError::PolynomialAction {
                    center_index,
                    source,
                })?;
            center_provenance.extend(
                center
                    .expression()
                    .terms()
                    .iter()
                    .map(|term| term.atom().provenance()),
            );
            center_provenance_offsets.push(center_provenance.len());
        }

        let actions = CpdMatrix::try_from_row_major(rows, columns, values)?;
        let diagnostics = diagnose_rank(&actions)?;
        match diagnostics.decision {
            CpdRankDecision::FullRank => {}
            CpdRankDecision::Deficient => {
                return Err(CpdError::RankDeficient {
                    diagnostics: Box::new(diagnostics),
                });
            }
            CpdRankDecision::Ambiguous => {
                return Err(CpdError::AmbiguousRank {
                    diagnostics: Box::new(diagnostics),
                });
            }
        }

        let basis = construct_null_space(&actions)?;
        let quality = verify_null_space(&actions, &basis);
        if quality.side_condition_residual > quality.tolerance
            || quality.orthonormality_residual > quality.tolerance
        {
            return Err(CpdError::NullSpaceVerificationFailed { quality });
        }
        Ok(Self {
            actions,
            basis,
            diagnostics,
            center_provenance,
            center_provenance_offsets,
            quality,
        })
    }

    /// Borrows the original row-major polynomial-action matrix `Q`.
    pub const fn actions(&self) -> &CpdMatrix {
        &self.actions
    }

    /// Borrows the orthonormal row-major null-space basis `Z`.
    pub const fn basis(&self) -> &CpdMatrix {
        &self.basis
    }

    /// Borrows the complete rank-decision evidence.
    pub const fn diagnostics(&self) -> &CpdRankDiagnostics {
        &self.diagnostics
    }

    /// Borrows atomic-term provenance for one center in expression order.
    #[must_use]
    pub fn center_provenance(&self, center_index: usize) -> Option<&[FunctionalProvenance]> {
        let start = *self.center_provenance_offsets.get(center_index)?;
        let end = *self
            .center_provenance_offsets
            .get(center_index.checked_add(1)?)?;
        self.center_provenance.get(start..end)
    }

    /// Returns null-space residual and orthonormality evidence.
    pub const fn quality(&self) -> CpdNullSpaceQuality {
        self.quality
    }

    /// Expands reduced coordinates as `w = Z y` and verifies `Q^T w = 0`.
    ///
    /// # Errors
    ///
    /// Returns a length, non-finite, allocation, or side-condition error.
    pub fn try_expand_weights(&self, reduced: &[f64]) -> Result<CpdWeights, CpdError> {
        let expected = self.basis.columns;
        if reduced.len() != expected {
            return Err(CpdError::ReducedLengthMismatch {
                expected,
                actual: reduced.len(),
            });
        }
        if let Some((index, value)) = reduced
            .iter()
            .copied()
            .enumerate()
            .find(|(_, value)| !value.is_finite())
        {
            return Err(CpdError::NonFiniteReducedCoordinate { index, value });
        }
        let mut weights = try_zeroed(self.basis.rows, CpdStorage::Weights)?;
        for (row, weight) in weights.iter_mut().enumerate() {
            *weight = (0..self.basis.columns)
                .map(|column| {
                    self.basis.values[row * self.basis.columns + column] * reduced[column]
                })
                .sum();
            if !weight.is_finite() {
                return Err(CpdError::NonFiniteWeight { center_index: row });
            }
        }
        let (residual, original_residual) = weight_residuals(&self.actions, &weights);
        let tolerance = verification_tolerance(self.actions.rows);
        if residual > tolerance {
            return Err(CpdError::WeightSideConditionFailed {
                residual,
                tolerance,
            });
        }
        Ok(CpdWeights {
            values: weights,
            origin: CpdWeightOrigin::PolynomialNullSpace,
            side_condition_residual: residual,
            original_side_condition_residual: original_residual,
            tolerance,
        })
    }

    /// Forms `Z^T K Z` for a finite symmetric center-energy matrix.
    ///
    /// Symmetry is validated at a dimension-aware scaled tolerance. This
    /// helper performs no regularization and does not classify positive
    /// definiteness, which belongs to later solver assembly.
    ///
    /// # Errors
    ///
    /// Returns a shape, symmetry, allocation, or non-finite arithmetic error.
    pub fn try_project_symmetric_energy(&self, energy: &CpdMatrix) -> Result<CpdMatrix, CpdError> {
        let centers = self.actions.rows;
        if energy.rows != centers || energy.columns != centers {
            return Err(CpdError::EnergyShapeMismatch {
                expected: centers,
                rows: energy.rows,
                columns: energy.columns,
            });
        }
        let symmetry_tolerance = verification_tolerance(centers);
        let max_absolute = matrix_norms(energy).max_absolute;
        let mut asymmetry = 0.0_f64;
        for row in 0..centers {
            for column in 0..row {
                asymmetry = asymmetry.max(
                    (energy.values[row * centers + column] - energy.values[column * centers + row])
                        .abs(),
                );
            }
        }
        let normalized_asymmetry = if max_absolute == 0.0 {
            asymmetry
        } else {
            asymmetry / max_absolute
        };
        if normalized_asymmetry > symmetry_tolerance {
            return Err(CpdError::EnergyNotSymmetric {
                normalized_asymmetry,
                tolerance: symmetry_tolerance,
            });
        }

        let reduced = self.basis.columns;
        let workspace_count = checked_entries(centers, reduced)?;
        let mut energy_times_basis = try_zeroed(workspace_count, CpdStorage::ProjectionWorkspace)?;
        for row in 0..centers {
            for column in 0..reduced {
                let value = (0..centers)
                    .map(|inner| {
                        energy.values[row * centers + inner]
                            * self.basis.values[inner * reduced + column]
                    })
                    .sum::<f64>();
                if !value.is_finite() {
                    return Err(CpdError::NonFiniteProjection { row, column });
                }
                energy_times_basis[row * reduced + column] = value;
            }
        }
        let projected_count = checked_entries(reduced, reduced)?;
        let mut projected = try_zeroed(projected_count, CpdStorage::ProjectedEnergy)?;
        for row in 0..reduced {
            for column in 0..reduced {
                let value = (0..centers)
                    .map(|inner| {
                        self.basis.values[inner * reduced + row]
                            * energy_times_basis[inner * reduced + column]
                    })
                    .sum::<f64>();
                if !value.is_finite() {
                    return Err(CpdError::NonFiniteProjection { row, column });
                }
                projected[row * reduced + column] = value;
            }
        }
        CpdMatrix::try_from_row_major(reduced, reduced, projected)
    }
}

/// Provenance attached to expanded CPD weights.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CpdWeightOrigin {
    /// The weights were constructed as `w = Z y` from the recorded basis.
    PolynomialNullSpace,
}

/// Expanded immutable center weights with verified polynomial side condition.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct CpdWeights {
    values: Vec<f64>,
    origin: CpdWeightOrigin,
    side_condition_residual: f64,
    original_side_condition_residual: f64,
    tolerance: f64,
}

impl CpdWeights {
    /// Borrows center weights in original center order.
    #[must_use]
    pub fn values(&self) -> &[f64] {
        &self.values
    }

    /// Returns the mathematical construction that guarantees the side condition.
    #[must_use]
    pub const fn origin(&self) -> CpdWeightOrigin {
        self.origin
    }

    /// Returns the normalized residual of `Q^T w`.
    #[must_use]
    pub const fn side_condition_residual(&self) -> f64 {
        self.side_condition_residual
    }

    /// Returns the absolute `Q^T w` residual in original action units.
    #[must_use]
    pub const fn original_side_condition_residual(&self) -> f64 {
        self.original_side_condition_residual
    }

    /// Returns the tolerance used to verify the side condition.
    #[must_use]
    pub const fn tolerance(&self) -> f64 {
        self.tolerance
    }
}

/// Internal or output storage that could not be reserved.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CpdStorage {
    /// Polynomial-action matrix.
    PolynomialActions,
    /// Polynomial value scratch.
    PolynomialValues,
    /// Polynomial gradient scratch.
    PolynomialGradients,
    /// Center provenance.
    CenterProvenance,
    /// Equilibrated matrix.
    EquilibratedMatrix,
    /// Null-space basis.
    NullSpaceBasis,
    /// Expanded weights.
    Weights,
    /// Intermediate `K Z` projection.
    ProjectionWorkspace,
    /// Projected energy output.
    ProjectedEnergy,
}

/// Error returned by CPD action assembly, rank diagnosis, or projection.
#[derive(Debug)]
#[non_exhaustive]
pub enum CpdError {
    /// At least one center representer is required.
    EmptyCenters,
    /// Matrix shape arithmetic overflowed.
    MatrixShapeOverflow {
        /// Row count.
        rows: usize,
        /// Column count.
        columns: usize,
    },
    /// The flattened atomic-provenance count overflowed.
    ProvenanceCountOverflow,
    /// Row-major data did not match its shape.
    MatrixLengthMismatch {
        /// Row count.
        rows: usize,
        /// Column count.
        columns: usize,
        /// Required data length.
        expected: usize,
        /// Supplied data length.
        actual: usize,
    },
    /// A matrix entry was NaN or infinite.
    NonFiniteMatrixEntry {
        /// Entry row.
        row: usize,
        /// Entry column.
        column: usize,
        /// Rejected value.
        value: f64,
    },
    /// Owned storage could not be reserved.
    AllocationFailed {
        /// Storage role.
        storage: CpdStorage,
        /// Requested element count.
        requested: usize,
    },
    /// One center expression could not act on the polynomial space.
    PolynomialAction {
        /// Center index in caller order.
        center_index: usize,
        /// Functional action diagnostic.
        source: FunctionalError,
    },
    /// A cumulative row or column equilibration multiplier was unrepresentable.
    UnrepresentableEquilibrationScale {
        /// `true` for a row scale and `false` for a column scale.
        row_scale: bool,
        /// Row or column index.
        index: usize,
        /// Zero-based equilibration pass.
        pass: usize,
    },
    /// The bounded SVD review did not converge.
    SvdDidNotConverge {
        /// Recorded finite iteration limit.
        maximum_iterations: usize,
    },
    /// The polynomial-action matrix is clearly rank deficient.
    RankDeficient {
        /// Complete rank evidence.
        diagnostics: Box<CpdRankDiagnostics>,
    },
    /// The rank decision falls in the ambiguity guard band or reviews disagree.
    AmbiguousRank {
        /// Complete rank evidence.
        diagnostics: Box<CpdRankDiagnostics>,
    },
    /// The backend-supported null-space construction could not find all columns.
    NullSpaceConstructionFailed {
        /// Required nullity.
        expected: usize,
        /// Constructed column count.
        actual: usize,
    },
    /// The constructed basis failed residual or orthonormality verification.
    NullSpaceVerificationFailed {
        /// Recorded verification evidence.
        quality: CpdNullSpaceQuality,
    },
    /// Reduced coordinates did not match the nullity.
    ReducedLengthMismatch {
        /// Required reduced length.
        expected: usize,
        /// Supplied reduced length.
        actual: usize,
    },
    /// A reduced coordinate was NaN or infinite.
    NonFiniteReducedCoordinate {
        /// Coordinate index.
        index: usize,
        /// Rejected value.
        value: f64,
    },
    /// Weight expansion produced a non-finite value.
    NonFiniteWeight {
        /// Center index.
        center_index: usize,
    },
    /// Expanded weights failed the polynomial side-condition review.
    WeightSideConditionFailed {
        /// Normalized residual.
        residual: f64,
        /// Verification tolerance.
        tolerance: f64,
    },
    /// The center-energy matrix had the wrong shape.
    EnergyShapeMismatch {
        /// Required square dimension.
        expected: usize,
        /// Supplied rows.
        rows: usize,
        /// Supplied columns.
        columns: usize,
    },
    /// The center-energy matrix was not symmetric to scaled tolerance.
    EnergyNotSymmetric {
        /// Maximum asymmetry divided by the largest absolute entry.
        normalized_asymmetry: f64,
        /// Symmetry tolerance.
        tolerance: f64,
    },
    /// Projection arithmetic produced a non-finite output.
    NonFiniteProjection {
        /// Output row.
        row: usize,
        /// Output column.
        column: usize,
    },
}

impl CpdError {
    /// Returns rank evidence when rank classification caused this error.
    #[must_use]
    pub fn rank_diagnostics(&self) -> Option<&CpdRankDiagnostics> {
        match self {
            Self::RankDeficient { diagnostics } | Self::AmbiguousRank { diagnostics } => {
                Some(diagnostics)
            }
            _ => None,
        }
    }
}

impl fmt::Display for CpdError {
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyCenters => formatter.write_str("CPD assembly requires at least one center"),
            Self::MatrixShapeOverflow { rows, columns } => {
                write!(formatter, "matrix shape {rows}x{columns} overflows")
            }
            Self::ProvenanceCountOverflow => {
                formatter.write_str("center atomic-provenance count overflows")
            }
            Self::MatrixLengthMismatch {
                rows,
                columns,
                expected,
                actual,
            } => write!(
                formatter,
                "matrix shape {rows}x{columns} requires {expected} entries, got {actual}"
            ),
            Self::NonFiniteMatrixEntry { row, column, value } => write!(
                formatter,
                "matrix entry ({row}, {column}) must be finite, got {value}"
            ),
            Self::AllocationFailed { storage, requested } => {
                write!(
                    formatter,
                    "could not reserve {storage:?} storage for {requested} entries"
                )
            }
            Self::PolynomialAction {
                center_index,
                source,
            } => {
                write!(
                    formatter,
                    "center {center_index} polynomial action failed: {source}"
                )
            }
            Self::UnrepresentableEquilibrationScale {
                row_scale,
                index,
                pass,
            } => write!(
                formatter,
                "{} equilibration scale {index} became unrepresentable on pass {pass}",
                if *row_scale { "row" } else { "column" }
            ),
            Self::SvdDidNotConverge { maximum_iterations } => write!(
                formatter,
                "SVD rank review did not converge within {maximum_iterations} iterations"
            ),
            Self::RankDeficient { diagnostics } => write!(
                formatter,
                "CPD polynomial actions have effective rank {}, require {}",
                diagnostics.svd_rank, diagnostics.columns
            ),
            Self::AmbiguousRank { diagnostics } => write!(
                formatter,
                "CPD polynomial-action rank is ambiguous (RRQR {}, SVD {}, required {})",
                diagnostics.rrqr_rank, diagnostics.svd_rank, diagnostics.columns
            ),
            Self::NullSpaceConstructionFailed { expected, actual } => write!(
                formatter,
                "null-space construction produced {actual} columns, require {expected}"
            ),
            Self::NullSpaceVerificationFailed { quality } => write!(
                formatter,
                "null-space verification failed: scaled side residual {}, original-unit side residual {}, orthonormality residual {}, tolerance {}",
                quality.side_condition_residual,
                quality.original_side_condition_residual,
                quality.orthonormality_residual,
                quality.tolerance
            ),
            Self::ReducedLengthMismatch { expected, actual } => {
                write!(
                    formatter,
                    "reduced coordinates require length {expected}, got {actual}"
                )
            }
            Self::NonFiniteReducedCoordinate { index, value } => {
                write!(
                    formatter,
                    "reduced coordinate {index} must be finite, got {value}"
                )
            }
            Self::NonFiniteWeight { center_index } => {
                write!(formatter, "expanded weight {center_index} is not finite")
            }
            Self::WeightSideConditionFailed {
                residual,
                tolerance,
            } => write!(
                formatter,
                "expanded weights violate Q^T w = 0: residual {residual}, tolerance {tolerance}"
            ),
            Self::EnergyShapeMismatch {
                expected,
                rows,
                columns,
            } => write!(
                formatter,
                "center energy must be {expected}x{expected}, got {rows}x{columns}"
            ),
            Self::EnergyNotSymmetric {
                normalized_asymmetry,
                tolerance,
            } => write!(
                formatter,
                "center energy asymmetry {normalized_asymmetry} exceeds tolerance {tolerance}"
            ),
            Self::NonFiniteProjection { row, column } => {
                write!(
                    formatter,
                    "projected energy entry ({row}, {column}) is not finite"
                )
            }
        }
    }
}

impl Error for CpdError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::PolynomialAction { source, .. } => Some(source),
            _ => None,
        }
    }
}

struct EquilibratedMatrix {
    matrix: CpdMatrix,
    row_scales: Vec<f64>,
    column_scales: Vec<f64>,
    zero_rows: Vec<usize>,
    zero_columns: Vec<usize>,
}

fn diagnose_rank(actions: &CpdMatrix) -> Result<CpdRankDiagnostics, CpdError> {
    let equilibrated = equilibrate(actions)?;
    let matrix = DMatrix::from_row_slice(
        equilibrated.matrix.rows,
        equilibrated.matrix.columns,
        &equilibrated.matrix.values,
    );
    let qr = matrix.clone().col_piv_qr();
    let r = qr.r();
    let diagonal_count = actions.rows.min(actions.columns);
    let rrqr_diagonal = (0..diagonal_count)
        .map(|index| r[(index, index)].abs())
        .collect::<Vec<_>>();
    let dimension = actions.rows.max(actions.columns);
    let rrqr_threshold = rank_threshold(&rrqr_diagonal, dimension)?;
    let rrqr_rank = rrqr_diagonal
        .iter()
        .filter(|value| **value > rrqr_threshold)
        .count();

    let svd = SVD::try_new(matrix, false, false, 5.0 * f64::EPSILON, SVD_MAX_ITERATIONS).ok_or(
        CpdError::SvdDidNotConverge {
            maximum_iterations: SVD_MAX_ITERATIONS,
        },
    )?;
    let singular_values = svd.singular_values.iter().copied().collect::<Vec<_>>();
    let svd_threshold = rank_threshold(&singular_values, dimension)?;
    let svd_rank = singular_values
        .iter()
        .filter(|value| **value > svd_threshold)
        .count();
    let ambiguity_lower = svd_threshold / AMBIGUITY_FACTOR;
    let ambiguity_upper = svd_threshold * AMBIGUITY_FACTOR;
    let threshold_adjacent = svd_threshold > 0.0
        && singular_values
            .iter()
            .any(|value| *value >= ambiguity_lower && *value <= ambiguity_upper);
    let rank_disagreement = rrqr_rank != svd_rank;
    let decision = if threshold_adjacent || rank_disagreement {
        CpdRankDecision::Ambiguous
    } else if svd_rank == actions.columns {
        CpdRankDecision::FullRank
    } else {
        CpdRankDecision::Deficient
    };
    let condition_estimate = match (singular_values.first(), singular_values.last()) {
        (Some(largest), Some(smallest)) if svd_rank == actions.columns && *smallest > 0.0 => {
            largest / smallest
        }
        _ => f64::INFINITY,
    };
    let diagnostics = CpdRankDiagnostics {
        rows: actions.rows,
        columns: actions.columns,
        equilibration_passes: EQUILIBRATION_PASSES,
        row_scales: equilibrated.row_scales,
        column_scales: equilibrated.column_scales,
        zero_rows: equilibrated.zero_rows,
        zero_columns: equilibrated.zero_columns,
        original_norms: matrix_norms(actions),
        scaled_norms: matrix_norms(&equilibrated.matrix),
        rrqr_diagonal,
        rrqr_threshold,
        rrqr_rank,
        singular_values,
        svd_threshold,
        svd_rank,
        ambiguity_lower,
        ambiguity_upper,
        threshold_adjacent,
        rank_disagreement,
        condition_estimate,
        decision,
    };
    Ok(diagnostics)
}

fn equilibrate(actions: &CpdMatrix) -> Result<EquilibratedMatrix, CpdError> {
    let mut values = try_zeroed(actions.values.len(), CpdStorage::EquilibratedMatrix)?;
    values.copy_from_slice(&actions.values);
    let mut row_scales = vec![1.0; actions.rows];
    let mut column_scales = vec![1.0; actions.columns];
    let zero_rows = (0..actions.rows)
        .filter(|row| {
            values[*row * actions.columns..(*row + 1) * actions.columns]
                .iter()
                .all(|value| *value == 0.0)
        })
        .collect();
    let zero_columns = (0..actions.columns)
        .filter(|column| {
            (0..actions.rows).all(|row| values[row * actions.columns + *column] == 0.0)
        })
        .collect();

    for pass in 0..EQUILIBRATION_PASSES {
        for (row, row_scale) in row_scales.iter_mut().enumerate() {
            let start = row * actions.columns;
            let end = start + actions.columns;
            let scale = values[start..end]
                .iter()
                .map(|value| value.abs())
                .fold(0.0_f64, f64::max);
            if scale > 0.0 {
                let cumulative = *row_scale / scale;
                if !cumulative.is_finite() || cumulative == 0.0 {
                    return Err(CpdError::UnrepresentableEquilibrationScale {
                        row_scale: true,
                        index: row,
                        pass,
                    });
                }
                *row_scale = cumulative;
                for value in &mut values[start..end] {
                    *value /= scale;
                }
            }
        }
        for column in 0..actions.columns {
            let scale = (0..actions.rows)
                .map(|row| values[row * actions.columns + column].abs())
                .fold(0.0_f64, f64::max);
            if scale > 0.0 {
                let cumulative = column_scales[column] / scale;
                if !cumulative.is_finite() || cumulative == 0.0 {
                    return Err(CpdError::UnrepresentableEquilibrationScale {
                        row_scale: false,
                        index: column,
                        pass,
                    });
                }
                column_scales[column] = cumulative;
                for row in 0..actions.rows {
                    values[row * actions.columns + column] /= scale;
                }
            }
        }
    }
    Ok(EquilibratedMatrix {
        matrix: CpdMatrix {
            rows: actions.rows,
            columns: actions.columns,
            values,
        },
        row_scales,
        column_scales,
        zero_rows,
        zero_columns,
    })
}

fn construct_null_space(actions: &CpdMatrix) -> Result<CpdMatrix, CpdError> {
    let nullity = actions.rows - actions.columns;
    if nullity == 0 {
        return CpdMatrix::try_from_row_major(actions.rows, 0, Vec::new());
    }

    // Row equilibration changes null(Q^T). Build a well-scaled basis for the
    // scaled matrix, map it back with the recorded row operation implicitly by
    // using the original column space, and reorthogonalize deterministically.
    // The original QR is safe here because rank classification has already
    // rejected marginal systems; final scale-aware residual checks are binding.
    let matrix = DMatrix::from_row_slice(actions.rows, actions.columns, &actions.values);
    let column_basis = matrix.qr().q();
    let basis_count = checked_entries(actions.rows, nullity)?;
    let mut columns = try_zeroed(basis_count, CpdStorage::NullSpaceBasis)?;
    let mut candidate = vec![0.0; actions.rows];
    let acceptance = verification_tolerance(actions.rows);
    let mut constructed = 0;
    for axis in 0..actions.rows {
        candidate.fill(0.0);
        candidate[axis] = 1.0;
        for _ in 0..2 {
            for column in 0..actions.columns {
                let projection = (0..actions.rows)
                    .map(|row| candidate[row] * column_basis[(row, column)])
                    .sum::<f64>();
                for row in 0..actions.rows {
                    candidate[row] -= projection * column_basis[(row, column)];
                }
            }
            for column in 0..constructed {
                let offset = column * actions.rows;
                let projection = (0..actions.rows)
                    .map(|row| candidate[row] * columns[offset + row])
                    .sum::<f64>();
                for row in 0..actions.rows {
                    candidate[row] -= projection * columns[offset + row];
                }
            }
        }
        let norm = candidate
            .iter()
            .map(|value| value * value)
            .sum::<f64>()
            .sqrt();
        if norm <= acceptance {
            continue;
        }
        let offset = constructed * actions.rows;
        for row in 0..actions.rows {
            columns[offset + row] = candidate[row] / norm;
        }
        constructed += 1;
        if constructed == nullity {
            break;
        }
    }
    if constructed != nullity {
        return Err(CpdError::NullSpaceConstructionFailed {
            expected: nullity,
            actual: constructed,
        });
    }
    let mut row_major = try_zeroed(basis_count, CpdStorage::NullSpaceBasis)?;
    for row in 0..actions.rows {
        for column in 0..nullity {
            row_major[row * nullity + column] = columns[column * actions.rows + row];
        }
    }
    CpdMatrix::try_from_row_major(actions.rows, nullity, row_major)
}

fn verify_null_space(actions: &CpdMatrix, basis: &CpdMatrix) -> CpdNullSpaceQuality {
    let mut side_condition_residual = 0.0_f64;
    let mut original_side_condition_residual = 0.0_f64;
    for polynomial in 0..actions.columns {
        let column_scale = (0..actions.rows)
            .map(|row| actions.values[row * actions.columns + polynomial].abs())
            .fold(0.0_f64, f64::max);
        if column_scale == 0.0 {
            continue;
        }
        for basis_column in 0..basis.columns {
            let original_residual = (0..actions.rows)
                .map(|row| {
                    actions.values[row * actions.columns + polynomial]
                        * basis.values[row * basis.columns + basis_column]
                })
                .sum::<f64>()
                .abs();
            let residual = (0..actions.rows)
                .map(|row| {
                    actions.values[row * actions.columns + polynomial] / column_scale
                        * basis.values[row * basis.columns + basis_column]
                })
                .sum::<f64>()
                .abs();
            side_condition_residual = side_condition_residual.max(residual);
            original_side_condition_residual =
                original_side_condition_residual.max(original_residual);
        }
    }
    let mut orthonormality_residual = 0.0_f64;
    for left in 0..basis.columns {
        for right in 0..basis.columns {
            let product = (0..basis.rows)
                .map(|row| {
                    basis.values[row * basis.columns + left]
                        * basis.values[row * basis.columns + right]
                })
                .sum::<f64>();
            orthonormality_residual =
                orthonormality_residual.max((product - f64::from(left == right)).abs());
        }
    }
    CpdNullSpaceQuality {
        side_condition_residual,
        original_side_condition_residual,
        orthonormality_residual,
        tolerance: verification_tolerance(actions.rows),
    }
}

fn weight_residuals(actions: &CpdMatrix, weights: &[f64]) -> (f64, f64) {
    let weight_scale = weights
        .iter()
        .map(|value| value.abs())
        .fold(0.0_f64, f64::max);
    if weight_scale == 0.0 {
        return (0.0, 0.0);
    }
    let mut residual = 0.0_f64;
    let mut original_residual = 0.0_f64;
    for polynomial in 0..actions.columns {
        let column_scale = (0..actions.rows)
            .map(|row| actions.values[row * actions.columns + polynomial].abs())
            .fold(0.0_f64, f64::max);
        if column_scale == 0.0 {
            continue;
        }
        let original_value = (0..actions.rows)
            .map(|row| actions.values[row * actions.columns + polynomial] * weights[row])
            .sum::<f64>()
            .abs();
        let value = (0..actions.rows)
            .map(|row| {
                actions.values[row * actions.columns + polynomial] / column_scale
                    * (weights[row] / weight_scale)
            })
            .sum::<f64>()
            .abs();
        residual = residual.max(value);
        original_residual = original_residual.max(original_value);
    }
    (residual, original_residual)
}

fn rank_threshold(values: &[f64], dimension: usize) -> Result<f64, CpdError> {
    let dimension = u32::try_from(dimension).map_err(|_| CpdError::MatrixShapeOverflow {
        rows: dimension,
        columns: dimension,
    })?;
    let leading = values.iter().copied().fold(0.0_f64, f64::max);
    Ok(f64::from(dimension) * f64::EPSILON * leading)
}

fn verification_tolerance(dimension: usize) -> f64 {
    let dimension = u32::try_from(dimension).map_or(f64::INFINITY, f64::from);
    RESIDUAL_TOLERANCE_FACTOR * dimension * f64::EPSILON
}

fn checked_entries(rows: usize, columns: usize) -> Result<usize, CpdError> {
    rows.checked_mul(columns)
        .ok_or(CpdError::MatrixShapeOverflow { rows, columns })
}

fn matrix_norms(matrix: &CpdMatrix) -> CpdMatrixNorms {
    let max_absolute = matrix
        .values
        .iter()
        .map(|value| value.abs())
        .fold(0.0_f64, f64::max);
    let infinity = (0..matrix.rows)
        .map(|row| {
            matrix.values[row * matrix.columns..(row + 1) * matrix.columns]
                .iter()
                .map(|value| value.abs())
                .sum()
        })
        .fold(0.0_f64, f64::max);
    CpdMatrixNorms {
        max_absolute,
        infinity,
    }
}

fn try_zeroed(count: usize, storage: CpdStorage) -> Result<Vec<f64>, CpdError> {
    try_filled(count, 0.0, storage)
}

fn try_filled<T: Clone>(count: usize, value: T, storage: CpdStorage) -> Result<Vec<T>, CpdError> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| CpdError::AllocationFailed {
            storage,
            requested: count,
        })?;
    values.resize(count, value);
    Ok(values)
}
