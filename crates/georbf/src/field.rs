//! Symmetric dense assembly for hard-equality scalar-field problems.
//!
//! [`FieldProblem`] keeps observation functionals and center representers as
//! distinct inputs. An all-representer problem must align them explicitly;
//! assembly never converts one role into the other. Mixed value and
//! directional-derivative expressions are supported for exactly D=1, D=2,
//! and D=3.
//!
//! ```compile_fail
//! use georbf::FieldProblem;
//!
//! fn unsupported(problem: FieldProblem<4>) {
//!     let _ = problem;
//! }
//! ```

use std::error::Error;
use std::fmt;
use std::num::NonZeroUsize;

use crate::Point;
use crate::cpd::{CpdError, CpdMatrix, CpdNullSpace};
use crate::dimension::{Dim, SupportedDimension};
use crate::functional::{
    CenterRepresenter, FunctionalError, KernelActionError, ObservationFunctional,
};
use crate::kernel::{
    KernelDefiniteness, KernelDerivativeCapability, KernelDerivativeOrder, KernelMetadata,
};
use crate::kernel_calculus::SpatialKernelJetPrefix;
use crate::polynomial::{PolynomialSpace, PolynomialSpaceError};
use crate::problem_ir::{
    AffineExpression, AffineTerm, CanonicalProblem, CanonicalizationError, Enforcement,
    ProblemIrError, SemanticProblemIr, SemanticRelation, VariableBlock,
};

const SYMMETRY_TOLERANCE_FACTOR: f64 = 64.0;

/// Storage category used by fallible field-assembly allocation diagnostics.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FieldAssemblyStorage {
    /// Owned center-representer collection.
    Centers,
    /// Symmetric kernel-action matrix.
    KernelActions,
    /// Observation-side polynomial-action matrix.
    PolynomialActions,
    /// Sparse affine terms for one canonical equality.
    AffineTerms,
    /// Full augmented dense matrix.
    DenseMatrix,
    /// Augmented equality right-hand side.
    RightHandSide,
    /// Canonical variable-block names.
    VariableBlockName,
    /// Temporary copy used by CPD projection diagnostics.
    ProjectionInput,
}

/// Error returned while constructing an all-representer [`FieldProblem`].
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum FieldProblemError {
    /// No center representer was supplied.
    EmptyCenters,
    /// The observation and center collections have different lengths.
    ObservationCenterCountMismatch {
        /// Number of semantic observations.
        observations: usize,
        /// Number of center representers.
        centers: usize,
    },
    /// A semantic relation was not an equality.
    NonEqualityConstraint {
        /// Constraint index in semantic insertion order.
        constraint_index: usize,
    },
    /// A semantic equality requested soft enforcement.
    SoftEqualityConstraint {
        /// Constraint index in semantic insertion order.
        constraint_index: usize,
    },
    /// The observation expression and same-index center expression differ.
    ObservationCenterExpressionMismatch {
        /// Mismatched all-representer index.
        index: usize,
    },
    /// Storage could not be reserved.
    AllocationFailed {
        /// Storage category.
        storage: FieldAssemblyStorage,
        /// Exact or minimum number of entries requested.
        requested: usize,
    },
}

impl fmt::Display for FieldProblemError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyCenters => formatter.write_str("field problem requires at least one center"),
            Self::ObservationCenterCountMismatch {
                observations,
                centers,
            } => write!(
                formatter,
                "field problem has {observations} observations but {centers} centers"
            ),
            Self::NonEqualityConstraint { constraint_index } => write!(
                formatter,
                "field constraint {constraint_index} is not an equality"
            ),
            Self::SoftEqualityConstraint { constraint_index } => write!(
                formatter,
                "field equality {constraint_index} is soft; this assembly preserves hard equalities only"
            ),
            Self::ObservationCenterExpressionMismatch { index } => write!(
                formatter,
                "observation and center expressions differ at all-representer index {index}"
            ),
            Self::AllocationFailed { storage, requested } => write!(
                formatter,
                "could not reserve {requested} entries for field assembly storage {storage:?}"
            ),
        }
    }
}

impl Error for FieldProblemError {}

/// Internal linearizer diagnostic retained by canonicalization errors.
#[derive(Clone, Debug, PartialEq)]
pub enum FieldLinearizationError {
    /// A checked canonical row index was unavailable.
    MissingPreassembledRow {
        /// Requested row index.
        row: usize,
        /// Available row count.
        rows: usize,
    },
    /// Sparse affine-term storage could not be reserved.
    AllocationFailed {
        /// Exact capacity requested.
        requested: usize,
    },
    /// Construction of one affine term or expression failed.
    Ir(ProblemIrError),
}

impl fmt::Display for FieldLinearizationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingPreassembledRow { row, rows } => {
                write!(
                    formatter,
                    "preassembled field row {row} is outside {rows} rows"
                )
            }
            Self::AllocationFailed { requested } => write!(
                formatter,
                "could not reserve {requested} sparse field coefficients"
            ),
            Self::Ir(source) => source.fmt(formatter),
        }
    }
}

impl Error for FieldLinearizationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Ir(source) => Some(source),
            Self::MissingPreassembledRow { .. } | Self::AllocationFailed { .. } => None,
        }
    }
}

/// Scale-derived symmetry evidence for an assembled dense system.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct FieldAssemblyDiagnostics {
    /// Dimension of the square augmented system.
    pub system_dimension: usize,
    /// Number of kernel entries evaluated before symmetric reflection.
    pub kernel_entry_evaluations: usize,
    /// Largest absolute entry in the augmented matrix.
    pub maximum_absolute_entry: f64,
    /// Maximum absolute asymmetry divided by the maximum absolute entry.
    pub normalized_asymmetry: f64,
    /// Acceptance threshold `64 * system_dimension * f64::EPSILON`.
    pub symmetry_tolerance: f64,
}

/// Error returned while assembling a hard-equality field system.
#[derive(Debug)]
pub enum FieldAssemblyError<E> {
    /// Kernel metadata does not support the compile-time dimension.
    UnsupportedKernelDimension {
        /// Compile-time dimension.
        dimension: usize,
    },
    /// A term pair demands an unavailable derivative.
    UnsupportedDerivativeCapability {
        /// Observation row index.
        observation_index: usize,
        /// Observation term index.
        observation_term_index: usize,
        /// Center column index.
        center_index: usize,
        /// Center term index.
        center_term_index: usize,
        /// Observation derivative order.
        observation_order: KernelDerivativeOrder,
        /// Center derivative order.
        center_order: KernelDerivativeOrder,
        /// Metadata classification for the combined demand.
        capability: KernelDerivativeCapability,
        /// Whether the term points coincide exactly.
        coincident: bool,
    },
    /// Kernel evaluation or contraction failed for one upper-triangle entry.
    KernelAction {
        /// Observation row index.
        observation_index: usize,
        /// Center column index.
        center_index: usize,
        /// Provenance-bearing kernel action diagnostic.
        source: Box<KernelActionError<E>>,
    },
    /// Observation-side complete-polynomial action failed.
    PolynomialAction {
        /// Observation row index.
        observation_index: usize,
        /// Functional diagnostic.
        source: FunctionalError,
    },
    /// Complete polynomial-space construction failed.
    PolynomialSpace(PolynomialSpaceError),
    /// CPD rank, null-space, or projected-energy construction failed.
    Cpd(CpdError),
    /// Semantic-to-canonical hard-equality compilation failed.
    Canonicalization(CanonicalizationError<FieldLinearizationError>),
    /// Canonical variable-block construction failed.
    Ir(ProblemIrError),
    /// A checked matrix or evaluation count overflowed `usize`.
    CountOverflow,
    /// A validated field problem no longer contained an expected equality row.
    InvalidProblemState {
        /// Missing observation index.
        observation_index: usize,
    },
    /// Storage could not be reserved.
    AllocationFailed {
        /// Storage category.
        storage: FieldAssemblyStorage,
        /// Exact or minimum entry count requested.
        requested: usize,
    },
    /// The augmented all-representer matrix failed symmetry review.
    NotSymmetric {
        /// Recorded scale-derived evidence.
        diagnostics: FieldAssemblyDiagnostics,
    },
}

impl<E> fmt::Display for FieldAssemblyError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedKernelDimension { dimension } => write!(
                formatter,
                "kernel metadata does not support field dimension D={dimension}"
            ),
            Self::UnsupportedDerivativeCapability {
                observation_index,
                observation_term_index,
                center_index,
                center_term_index,
                observation_order,
                center_order,
                capability,
                coincident,
            } => write!(
                formatter,
                "field entry ({observation_index}, {center_index}) term pair ({observation_term_index}, {center_term_index}) demands {observation_order:?}+{center_order:?}, classified {capability:?} with coincident={coincident}"
            ),
            Self::KernelAction {
                observation_index,
                center_index,
                source,
            } => write!(
                formatter,
                "kernel action failed at field entry ({observation_index}, {center_index}): {source}"
            ),
            Self::PolynomialAction {
                observation_index,
                source,
            } => write!(
                formatter,
                "polynomial action failed for field observation {observation_index}: {source}"
            ),
            Self::PolynomialSpace(source) => source.fmt(formatter),
            Self::Cpd(source) => source.fmt(formatter),
            Self::Canonicalization(source) => source.fmt(formatter),
            Self::Ir(source) => source.fmt(formatter),
            Self::CountOverflow => formatter.write_str("field assembly count overflowed"),
            Self::InvalidProblemState { observation_index } => write!(
                formatter,
                "validated field problem is missing equality observation {observation_index}"
            ),
            Self::AllocationFailed { storage, requested } => write!(
                formatter,
                "could not reserve {requested} entries for field assembly storage {storage:?}"
            ),
            Self::NotSymmetric { diagnostics } => write!(
                formatter,
                "field matrix normalized asymmetry {} exceeds tolerance {}",
                diagnostics.normalized_asymmetry, diagnostics.symmetry_tolerance
            ),
        }
    }
}

impl<E> Error for FieldAssemblyError<E>
where
    E: Error + 'static,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::KernelAction { source, .. } => Some(source.as_ref()),
            Self::PolynomialAction { source, .. } => Some(source),
            Self::PolynomialSpace(source) => Some(source),
            Self::Cpd(source) => Some(source),
            Self::Canonicalization(source) => Some(source),
            Self::Ir(source) => Some(source),
            Self::UnsupportedKernelDimension { .. }
            | Self::UnsupportedDerivativeCapability { .. }
            | Self::CountOverflow
            | Self::InvalidProblemState { .. }
            | Self::AllocationFailed { .. }
            | Self::NotSymmetric { .. } => None,
        }
    }
}

/// An immutable finite square row-major field matrix.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct DenseFieldMatrix {
    dimension: usize,
    values: Vec<f64>,
}

impl DenseFieldMatrix {
    /// Returns the equal row and column count.
    #[must_use]
    pub const fn dimension(&self) -> usize {
        self.dimension
    }

    /// Borrows every entry in row-major order.
    #[must_use]
    pub fn values(&self) -> &[f64] {
        &self.values
    }

    /// Borrows one complete row, or returns `None` for an invalid row.
    #[must_use]
    pub fn row(&self, row: usize) -> Option<&[f64]> {
        let start = row.checked_mul(self.dimension)?;
        self.values.get(start..start.checked_add(self.dimension)?)
    }

    /// Returns one entry, or `None` for an invalid row or column.
    #[must_use]
    pub fn get(&self, row: usize, column: usize) -> Option<f64> {
        if row >= self.dimension || column >= self.dimension {
            return None;
        }
        self.values.get(row * self.dimension + column).copied()
    }
}

/// CPD-specific assembly evidence attached to an augmented field system.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct CpdFieldAssembly<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    polynomial_space: PolynomialSpace<D>,
    null_space: CpdNullSpace,
    projected_energy: CpdMatrix,
}

impl<const D: usize> CpdFieldAssembly<D>
where
    Dim<D>: SupportedDimension,
{
    /// Borrows the deterministic complete polynomial space.
    pub const fn polynomial_space(&self) -> &PolynomialSpace<D> {
        &self.polynomial_space
    }

    /// Borrows rank, provenance, and verified `null(Q^T)` evidence.
    pub const fn null_space(&self) -> &CpdNullSpace {
        &self.null_space
    }

    /// Borrows the projected symmetric energy `Z^T K Z`.
    pub const fn projected_energy(&self) -> &CpdMatrix {
        &self.projected_energy
    }
}

/// Immutable canonical and dense forms of one assembled field problem.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct DenseFieldSystem<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    canonical: CanonicalProblem,
    matrix: DenseFieldMatrix,
    rhs: Vec<f64>,
    center_count: usize,
    diagnostics: FieldAssemblyDiagnostics,
    cpd: Option<CpdFieldAssembly<D>>,
}

impl<const D: usize> DenseFieldSystem<D>
where
    Dim<D>: SupportedDimension,
{
    /// Borrows canonical observation equalities before CPD side-row augmentation.
    pub const fn canonical_problem(&self) -> &CanonicalProblem {
        &self.canonical
    }

    /// Borrows the symmetric dense matrix, including CPD polynomial rows.
    pub const fn matrix(&self) -> &DenseFieldMatrix {
        &self.matrix
    }

    /// Borrows the equality right-hand side, including CPD side-condition zeros.
    #[must_use]
    pub fn rhs(&self) -> &[f64] {
        &self.rhs
    }

    /// Returns the number of center weights at the beginning of the variable vector.
    #[must_use]
    pub const fn center_count(&self) -> usize {
        self.center_count
    }

    /// Returns the number of polynomial coefficients after the center weights.
    #[must_use]
    pub fn polynomial_count(&self) -> usize {
        self.cpd
            .as_ref()
            .map_or(0, |cpd| cpd.polynomial_space.term_count())
    }

    /// Borrows CPD rank/null-space evidence, or `None` for a strictly PD kernel.
    #[must_use]
    pub const fn cpd(&self) -> Option<&CpdFieldAssembly<D>> {
        self.cpd.as_ref()
    }

    /// Returns recorded symmetry and work-count evidence.
    pub const fn diagnostics(&self) -> FieldAssemblyDiagnostics {
        self.diagnostics
    }
}

/// One immutable all-representer hard-equality scalar-field problem.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct FieldProblem<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    semantic: SemanticProblemIr<D>,
    centers: Vec<CenterRepresenter<D>>,
}

impl<const D: usize> FieldProblem<D>
where
    Dim<D>: SupportedDimension,
{
    /// Validates distinct but same-index observation and center expressions.
    ///
    /// Every semantic relation must be a hard equality. The observation and
    /// center collections remain differently typed, but an all-representer
    /// symmetric system requires exact expression alignment at each index.
    ///
    /// # Errors
    ///
    /// Returns a structured empty, count, relation, enforcement, expression,
    /// or allocation diagnostic without returning a partial problem.
    pub fn try_new(
        semantic: SemanticProblemIr<D>,
        centers: impl IntoIterator<Item = CenterRepresenter<D>>,
    ) -> Result<Self, FieldProblemError> {
        let iterator = centers.into_iter();
        let minimum = iterator.size_hint().0;
        let mut stored = Vec::new();
        stored
            .try_reserve_exact(minimum)
            .map_err(|_| FieldProblemError::AllocationFailed {
                storage: FieldAssemblyStorage::Centers,
                requested: minimum,
            })?;
        for center in iterator {
            if stored.len() == stored.capacity() {
                let requested = stored.len().saturating_add(1);
                stored
                    .try_reserve(1)
                    .map_err(|_| FieldProblemError::AllocationFailed {
                        storage: FieldAssemblyStorage::Centers,
                        requested,
                    })?;
            }
            stored.push(center);
        }
        if stored.is_empty() {
            return Err(FieldProblemError::EmptyCenters);
        }
        let constraints = semantic.constraints();
        if constraints.len() != stored.len() {
            return Err(FieldProblemError::ObservationCenterCountMismatch {
                observations: constraints.len(),
                centers: stored.len(),
            });
        }
        for (index, (constraint, center)) in constraints.iter().zip(&stored).enumerate() {
            let SemanticRelation::Equality { expression, .. } = constraint.relation() else {
                return Err(FieldProblemError::NonEqualityConstraint {
                    constraint_index: index,
                });
            };
            if constraint.enforcement() != Enforcement::Hard {
                return Err(FieldProblemError::SoftEqualityConstraint {
                    constraint_index: index,
                });
            }
            if expression.functional().expression() != center.expression() {
                return Err(FieldProblemError::ObservationCenterExpressionMismatch { index });
            }
        }
        Ok(Self {
            semantic,
            centers: stored,
        })
    }

    /// Borrows the provenance-bearing semantic hard equalities.
    pub const fn semantic_problem(&self) -> &SemanticProblemIr<D> {
        &self.semantic
    }

    /// Borrows center representers in deterministic all-representer order.
    pub fn centers(&self) -> &[CenterRepresenter<D>] {
        &self.centers
    }

    /// Assembles a symmetric dense equality system without solving it.
    ///
    /// Only the upper kernel triangle is evaluated; exact same-index
    /// observation/center expression validation makes reflection explicit,
    /// not a silent compatibility repair. CPD metadata automatically selects
    /// the complete polynomial space, performs rank/null-space review, adds
    /// `Q^T w = 0` rows, and records `Z^T K Z`. No jitter, regularization,
    /// pseudoinverse, factorization, or constraint relaxation is performed.
    ///
    /// The evaluator receives validated query and center points plus the exact
    /// combined derivative demand for each atomic term pair.
    ///
    /// # Errors
    ///
    /// Returns structured dimension, derivative-capability, kernel action,
    /// polynomial, CPD, canonicalization, allocation, or symmetry diagnostics.
    #[allow(clippy::too_many_lines)]
    pub fn try_assemble<E>(
        &self,
        metadata: KernelMetadata<'_>,
        mut evaluator: impl FnMut(
            Point<D>,
            Point<D>,
            KernelDerivativeOrder,
        ) -> Result<SpatialKernelJetPrefix<D>, E>,
    ) -> Result<DenseFieldSystem<D>, FieldAssemblyError<E>> {
        if !metadata.dimensions().supports::<D>() {
            return Err(FieldAssemblyError::UnsupportedKernelDimension { dimension: D });
        }
        let centers = self.centers.len();
        let kernel_entries = checked_square(centers)?;
        let mut kernel_actions = try_zeroed(kernel_entries, FieldAssemblyStorage::KernelActions)?;
        let kernel_entry_evaluations = centers
            .checked_mul(
                centers
                    .checked_add(1)
                    .ok_or(FieldAssemblyError::CountOverflow)?,
            )
            .and_then(|count| count.checked_div(2))
            .ok_or(FieldAssemblyError::CountOverflow)?;

        for row in 0..centers {
            let observation = observation_at(&self.semantic, row).ok_or(
                FieldAssemblyError::InvalidProblemState {
                    observation_index: row,
                },
            )?;
            for column in row..centers {
                let center = &self.centers[column];
                validate_capabilities(metadata, row, observation, column, center)?;
                let value = observation
                    .try_apply_kernel(center, &mut evaluator)
                    .map_err(|source| FieldAssemblyError::KernelAction {
                        observation_index: row,
                        center_index: column,
                        source: Box::new(source),
                    })?;
                kernel_actions[row * centers + column] = value;
                kernel_actions[column * centers + row] = value;
            }
        }

        let cpd_state = match metadata.definiteness() {
            KernelDefiniteness::StrictlyPositiveDefinite => None,
            KernelDefiniteness::ConditionallyPositiveDefinite { order } => {
                let polynomial_space = PolynomialSpace::<D>::try_new(order.get())
                    .map_err(FieldAssemblyError::PolynomialSpace)?;
                let null_space = CpdNullSpace::try_from_centers(&self.centers, &polynomial_space)
                    .map_err(FieldAssemblyError::Cpd)?;
                Some((polynomial_space, null_space))
            }
        };
        let polynomial_count = cpd_state
            .as_ref()
            .map_or(0, |(space, _)| space.term_count());
        let polynomial_entries = centers
            .checked_mul(polynomial_count)
            .ok_or(FieldAssemblyError::CountOverflow)?;
        let mut observation_polynomial =
            try_zeroed(polynomial_entries, FieldAssemblyStorage::PolynomialActions)?;
        if let Some((space, _)) = &cpd_state {
            for row in 0..centers {
                let values = observation_at(&self.semantic, row)
                    .ok_or(FieldAssemblyError::InvalidProblemState {
                        observation_index: row,
                    })?
                    .expression()
                    .try_apply_polynomial(space)
                    .map_err(|source| FieldAssemblyError::PolynomialAction {
                        observation_index: row,
                        source,
                    })?;
                observation_polynomial[row * polynomial_count..(row + 1) * polynomial_count]
                    .copy_from_slice(&values);
            }
        }

        let variable_count = centers
            .checked_add(polynomial_count)
            .ok_or(FieldAssemblyError::CountOverflow)?;
        let mut blocks = Vec::new();
        blocks
            .try_reserve_exact(usize::from(polynomial_count != 0) + 1)
            .map_err(|_| FieldAssemblyError::AllocationFailed {
                storage: FieldAssemblyStorage::AffineTerms,
                requested: usize::from(polynomial_count != 0) + 1,
            })?;
        blocks.push(
            VariableBlock::try_new(
                try_owned_name("center_weights")?,
                NonZeroUsize::new(centers).ok_or(FieldAssemblyError::CountOverflow)?,
            )
            .map_err(FieldAssemblyError::Ir)?,
        );
        if polynomial_count != 0 {
            blocks.push(
                VariableBlock::try_new(
                    try_owned_name("polynomial_coefficients")?,
                    NonZeroUsize::new(polynomial_count).ok_or(FieldAssemblyError::CountOverflow)?,
                )
                .map_err(FieldAssemblyError::Ir)?,
            );
        }

        let mut next_row = 0_usize;
        let canonical = self
            .semantic
            .try_compile(blocks, |_, _| {
                let row = next_row;
                next_row = next_row.saturating_add(1);
                if row >= centers {
                    return Err(FieldLinearizationError::MissingPreassembledRow {
                        row,
                        rows: centers,
                    });
                }
                let mut terms = Vec::new();
                terms.try_reserve_exact(variable_count).map_err(|_| {
                    FieldLinearizationError::AllocationFailed {
                        requested: variable_count,
                    }
                })?;
                for column in 0..centers {
                    let coefficient = kernel_actions[row * centers + column];
                    if coefficient != 0.0 {
                        terms.push(
                            AffineTerm::try_new(column, coefficient)
                                .map_err(FieldLinearizationError::Ir)?,
                        );
                    }
                }
                for column in 0..polynomial_count {
                    let coefficient = observation_polynomial[row * polynomial_count + column];
                    if coefficient != 0.0 {
                        terms.push(
                            AffineTerm::try_new(centers + column, coefficient)
                                .map_err(FieldLinearizationError::Ir)?,
                        );
                    }
                }
                AffineExpression::try_new(terms, 0.0).map_err(FieldLinearizationError::Ir)
            })
            .map_err(FieldAssemblyError::Canonicalization)?;

        let system_dimension = variable_count;
        let dense_entries = checked_square(system_dimension)?;
        let mut dense = try_zeroed(dense_entries, FieldAssemblyStorage::DenseMatrix)?;
        let mut rhs = try_zeroed(system_dimension, FieldAssemblyStorage::RightHandSide)?;
        for (row, equality) in canonical.equalities().iter().enumerate() {
            rhs[row] = equality.rhs();
            for term in equality.row().terms() {
                dense[row * system_dimension + term.variable()] = term.coefficient();
            }
        }
        if let Some((_, null_space)) = &cpd_state {
            let actions = null_space.actions();
            for polynomial in 0..polynomial_count {
                let row = centers + polynomial;
                for center in 0..centers {
                    dense[row * system_dimension + center] = actions
                        .get(center, polynomial)
                        .ok_or(FieldAssemblyError::CountOverflow)?;
                }
            }
        }

        let diagnostics = symmetry_diagnostics(&dense, system_dimension, kernel_entry_evaluations);
        if diagnostics.normalized_asymmetry > diagnostics.symmetry_tolerance {
            return Err(FieldAssemblyError::NotSymmetric { diagnostics });
        }

        let projected_energy = if let Some((_, null_space)) = &cpd_state {
            let energy_values = try_copy(&kernel_actions, FieldAssemblyStorage::ProjectionInput)?;
            let energy = CpdMatrix::try_from_row_major(centers, centers, energy_values)
                .map_err(FieldAssemblyError::Cpd)?;
            Some(
                null_space
                    .try_project_symmetric_energy(&energy)
                    .map_err(FieldAssemblyError::Cpd)?,
            )
        } else {
            None
        };
        let cpd = match (cpd_state, projected_energy) {
            (Some((polynomial_space, null_space)), Some(projected_energy)) => {
                Some(CpdFieldAssembly {
                    polynomial_space,
                    null_space,
                    projected_energy,
                })
            }
            _ => None,
        };

        Ok(DenseFieldSystem {
            canonical,
            matrix: DenseFieldMatrix {
                dimension: system_dimension,
                values: dense,
            },
            rhs,
            center_count: centers,
            diagnostics,
            cpd,
        })
    }
}

fn observation_at<const D: usize>(
    semantic: &SemanticProblemIr<D>,
    index: usize,
) -> Option<&ObservationFunctional<D>>
where
    Dim<D>: SupportedDimension,
{
    let constraint = semantic.constraints().get(index)?;
    let SemanticRelation::Equality { expression, .. } = constraint.relation() else {
        return None;
    };
    Some(expression.functional())
}

fn validate_capabilities<const D: usize, E>(
    metadata: KernelMetadata<'_>,
    observation_index: usize,
    observation: &ObservationFunctional<D>,
    center_index: usize,
    center: &CenterRepresenter<D>,
) -> Result<(), FieldAssemblyError<E>>
where
    Dim<D>: SupportedDimension,
{
    for (observation_term_index, observation_term) in
        observation.expression().terms().iter().enumerate()
    {
        for (center_term_index, center_term) in center.expression().terms().iter().enumerate() {
            let observation_atom = observation_term.atom();
            let center_atom = center_term.atom();
            let observation_order = observation_atom.derivative_order();
            let center_order = center_atom.derivative_order();
            let capability = metadata
                .derivatives()
                .matrix_capability(observation_order, center_order);
            let coincident = observation_atom.point() == center_atom.point();
            if capability == KernelDerivativeCapability::Unsupported
                || (capability == KernelDerivativeCapability::SupportedAwayFromCenters
                    && coincident)
            {
                return Err(FieldAssemblyError::UnsupportedDerivativeCapability {
                    observation_index,
                    observation_term_index,
                    center_index,
                    center_term_index,
                    observation_order,
                    center_order,
                    capability,
                    coincident,
                });
            }
        }
    }
    Ok(())
}

fn checked_square<E>(dimension: usize) -> Result<usize, FieldAssemblyError<E>> {
    dimension
        .checked_mul(dimension)
        .ok_or(FieldAssemblyError::CountOverflow)
}

fn try_zeroed<E>(
    count: usize,
    storage: FieldAssemblyStorage,
) -> Result<Vec<f64>, FieldAssemblyError<E>> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| FieldAssemblyError::AllocationFailed {
            storage,
            requested: count,
        })?;
    values.resize(count, 0.0);
    Ok(values)
}

fn try_copy<E>(
    values: &[f64],
    storage: FieldAssemblyStorage,
) -> Result<Vec<f64>, FieldAssemblyError<E>> {
    let mut copied = Vec::new();
    copied
        .try_reserve_exact(values.len())
        .map_err(|_| FieldAssemblyError::AllocationFailed {
            storage,
            requested: values.len(),
        })?;
    copied.extend_from_slice(values);
    Ok(copied)
}

fn try_owned_name<E>(value: &str) -> Result<String, FieldAssemblyError<E>> {
    let mut owned = String::new();
    owned
        .try_reserve_exact(value.len())
        .map_err(|_| FieldAssemblyError::AllocationFailed {
            storage: FieldAssemblyStorage::VariableBlockName,
            requested: value.len(),
        })?;
    owned.push_str(value);
    Ok(owned)
}

fn symmetry_diagnostics(
    matrix: &[f64],
    dimension: usize,
    kernel_entry_evaluations: usize,
) -> FieldAssemblyDiagnostics {
    let maximum_absolute_entry = matrix
        .iter()
        .map(|value| value.abs())
        .fold(0.0_f64, f64::max);
    let mut maximum_asymmetry = 0.0_f64;
    for row in 0..dimension {
        for column in 0..row {
            maximum_asymmetry = maximum_asymmetry
                .max((matrix[row * dimension + column] - matrix[column * dimension + row]).abs());
        }
    }
    let normalized_asymmetry = if maximum_absolute_entry == 0.0 {
        maximum_asymmetry
    } else {
        maximum_asymmetry / maximum_absolute_entry
    };
    let dimension_scale = u32::try_from(dimension).map_or(f64::INFINITY, f64::from);
    FieldAssemblyDiagnostics {
        system_dimension: dimension,
        kernel_entry_evaluations,
        maximum_absolute_entry,
        normalized_asymmetry,
        symmetry_tolerance: SYMMETRY_TOLERANCE_FACTOR * dimension_scale * f64::EPSILON,
    }
}
