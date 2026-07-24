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

#[cfg(test)]
use std::cell::Cell;

use crate::Point;
use crate::cpd::{CpdError, CpdMatrix, CpdNullSpace};
use crate::dimension::{Dim, SupportedDimension};
use crate::execution::{
    ExecutionControl, ExecutionError, ExecutionOperation, ExecutionStage, ProgressTracker,
};
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
    ExecutionOptions, ProblemIrError, SemanticProblemIr, SemanticRelation, VariableBlock,
};

const SYMMETRY_TOLERANCE_FACTOR: f64 = 64.0;

/// Fixed representer-block edge used by deterministic dense assembly.
///
/// Only upper-triangle blocks are traversed. Every kernel action is evaluated
/// once and off-diagonal values are reflected into the symmetric position.
pub const DENSE_ASSEMBLY_BLOCK_SIZE: usize = 32;

/// Storage category used by fallible field-assembly allocation diagnostics.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FieldAssemblyStorage {
    /// Owned center-representer collection.
    Centers,
    /// Symmetric kernel-action matrix.
    KernelActions,
    /// Observation-side polynomial-action matrix.
    PolynomialActions,
    /// Reused polynomial-value scratch for observation actions.
    PolynomialValues,
    /// Reused polynomial-gradient scratch for observation actions.
    PolynomialGradients,
    /// Sparse affine terms for one canonical equality.
    AffineTerms,
    /// Canonical variable-block collection.
    VariableBlocks,
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
    /// Fixed representer-block edge used for kernel assembly.
    pub assembly_block_size: usize,
    /// Number of upper-triangle representer blocks traversed.
    pub upper_triangle_blocks: usize,
    /// Number of kernel entries evaluated before symmetric reflection.
    pub kernel_entry_evaluations: usize,
    /// Number of off-diagonal kernel entries filled by exact reflection.
    pub reflected_kernel_entries: usize,
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
    /// Caller execution policy was unsupported or cancellation was requested.
    Execution(ExecutionError),
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
        source: KernelActionError<E>,
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
            Self::Execution(source) => source.fmt(formatter),
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
            Self::Execution(source) => Some(source),
            Self::KernelAction { source, .. } => Some(source),
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
    execution: ExecutionOptions,
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

    /// Returns the execution limits retained from the semantic problem.
    pub const fn execution_options(&self) -> ExecutionOptions {
        self.execution
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

    pub(crate) fn into_model_parts(
        self,
    ) -> (FieldAssemblyDiagnostics, Option<CpdFieldAssembly<D>>) {
        (self.diagnostics, self.cpd)
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

    pub(crate) fn into_centers(self) -> Vec<CenterRepresenter<D>> {
        self.centers
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
        evaluator: impl FnMut(
            Point<D>,
            Point<D>,
            KernelDerivativeOrder,
        ) -> Result<SpatialKernelJetPrefix<D>, E>,
    ) -> Result<DenseFieldSystem<D>, FieldAssemblyError<E>> {
        let control = ExecutionControl::default();
        self.try_assemble_with_control(metadata, evaluator, control)
    }

    /// Assembles a symmetric dense equality system with caller execution controls.
    ///
    /// Progress is synchronous and deterministic. Cancellation is checked
    /// before work, after each upper-triangle kernel action and polynomial row,
    /// and around each CPD, canonicalization, symmetry, and projection boundary.
    /// A cancellation or unsupported explicit thread count returns no partial
    /// system. The current dense assembly implementation is serial and accepts
    /// only an absent thread count or an explicit count of one.
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`Self::try_assemble`], plus structured
    /// execution-policy and cancellation failures.
    #[allow(clippy::too_many_lines)]
    pub fn try_assemble_with_control<E>(
        &self,
        metadata: KernelMetadata<'_>,
        mut evaluator: impl FnMut(
            Point<D>,
            Point<D>,
            KernelDerivativeOrder,
        ) -> Result<SpatialKernelJetPrefix<D>, E>,
        control: ExecutionControl<'_>,
    ) -> Result<DenseFieldSystem<D>, FieldAssemblyError<E>> {
        if !metadata.dimensions().supports::<D>() {
            return Err(FieldAssemblyError::UnsupportedKernelDimension { dimension: D });
        }
        let centers = self.centers.len();
        let kernel_entries = checked_square(centers)?;
        let kernel_entry_evaluations = centers
            .checked_mul(
                centers
                    .checked_add(1)
                    .ok_or(FieldAssemblyError::CountOverflow)?,
            )
            .and_then(|count| count.checked_div(2))
            .ok_or(FieldAssemblyError::CountOverflow)?;
        let block_count = centers.div_ceil(DENSE_ASSEMBLY_BLOCK_SIZE);
        let upper_triangle_blocks = block_count
            .checked_mul(
                block_count
                    .checked_add(1)
                    .ok_or(FieldAssemblyError::CountOverflow)?,
            )
            .and_then(|count| count.checked_div(2))
            .ok_or(FieldAssemblyError::CountOverflow)?;
        let reflected_kernel_entries = centers
            .checked_mul(
                centers
                    .checked_sub(1)
                    .ok_or(FieldAssemblyError::CountOverflow)?,
            )
            .and_then(|count| count.checked_div(2))
            .ok_or(FieldAssemblyError::CountOverflow)?;
        let cpd_progress = match metadata.definiteness() {
            KernelDefiniteness::StrictlyPositiveDefinite => 0,
            KernelDefiniteness::ConditionallyPositiveDefinite { .. } => centers
                .checked_add(2)
                .ok_or(FieldAssemblyError::CountOverflow)?,
        };
        let total_progress = kernel_entry_evaluations
            .checked_add(cpd_progress)
            .and_then(|count| count.checked_add(2))
            .ok_or(FieldAssemblyError::CountOverflow)?;
        let mut progress = ProgressTracker::try_new(
            control,
            ExecutionOperation::FieldAssembly,
            self.semantic.execution_options(),
            total_progress,
        )
        .map_err(FieldAssemblyError::Execution)?;
        let mut kernel_actions = try_zeroed(kernel_entries, FieldAssemblyStorage::KernelActions)?;

        for block_row in 0..block_count {
            let row_start = block_row * DENSE_ASSEMBLY_BLOCK_SIZE;
            let row_end = row_start
                .saturating_add(DENSE_ASSEMBLY_BLOCK_SIZE)
                .min(centers);
            for block_column in block_row..block_count {
                let column_start = block_column * DENSE_ASSEMBLY_BLOCK_SIZE;
                let column_end = column_start
                    .saturating_add(DENSE_ASSEMBLY_BLOCK_SIZE)
                    .min(centers);
                for row in row_start..row_end {
                    let observation = observation_at(&self.semantic, row).ok_or(
                        FieldAssemblyError::InvalidProblemState {
                            observation_index: row,
                        },
                    )?;
                    let first_column = if block_row == block_column {
                        row
                    } else {
                        column_start
                    };
                    for column in first_column..column_end {
                        let center = &self.centers[column];
                        validate_capabilities(metadata, row, observation, column, center)?;
                        let value = observation
                            .try_apply_kernel(center, &mut evaluator)
                            .map_err(|source| FieldAssemblyError::KernelAction {
                                observation_index: row,
                                center_index: column,
                                source,
                            })?;
                        kernel_actions[row * centers + column] = value;
                        kernel_actions[column * centers + row] = value;
                        progress
                            .advance(ExecutionStage::KernelAssembly)
                            .map_err(FieldAssemblyError::Execution)?;
                    }
                }
            }
        }

        let cpd_state = match metadata.definiteness() {
            KernelDefiniteness::StrictlyPositiveDefinite => None,
            KernelDefiniteness::ConditionallyPositiveDefinite { order } => {
                let polynomial_space = PolynomialSpace::<D>::try_new(order.get())
                    .map_err(FieldAssemblyError::PolynomialSpace)?;
                let null_space = CpdNullSpace::try_from_centers(&self.centers, &polynomial_space)
                    .map_err(FieldAssemblyError::Cpd)?;
                progress
                    .advance(ExecutionStage::CpdConstruction)
                    .map_err(FieldAssemblyError::Execution)?;
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
            let mut polynomial_values =
                try_zeroed(polynomial_count, FieldAssemblyStorage::PolynomialValues)?;
            let mut polynomial_gradients = try_filled(
                polynomial_count,
                [0.0; D],
                FieldAssemblyStorage::PolynomialGradients,
            )?;
            for row in 0..centers {
                let start = row * polynomial_count;
                observation_at(&self.semantic, row)
                    .ok_or(FieldAssemblyError::InvalidProblemState {
                        observation_index: row,
                    })?
                    .expression()
                    .try_apply_polynomial_with_scratch(
                        space,
                        &mut observation_polynomial[start..start + polynomial_count],
                        &mut polynomial_values,
                        &mut polynomial_gradients,
                    )
                    .map_err(|source| FieldAssemblyError::PolynomialAction {
                        observation_index: row,
                        source,
                    })?;
                progress
                    .advance(ExecutionStage::PolynomialAssembly)
                    .map_err(FieldAssemblyError::Execution)?;
            }
        }

        let variable_count = centers
            .checked_add(polynomial_count)
            .ok_or(FieldAssemblyError::CountOverflow)?;
        let mut blocks = Vec::new();
        try_reserve_exact(
            &mut blocks,
            usize::from(polynomial_count != 0) + 1,
            FieldAssemblyStorage::VariableBlocks,
        )?;
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
        progress
            .advance(ExecutionStage::Canonicalization)
            .map_err(FieldAssemblyError::Execution)?;

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

        let diagnostics = symmetry_diagnostics(
            &dense,
            system_dimension,
            upper_triangle_blocks,
            kernel_entry_evaluations,
            reflected_kernel_entries,
        );
        if diagnostics.normalized_asymmetry > diagnostics.symmetry_tolerance {
            return Err(FieldAssemblyError::NotSymmetric { diagnostics });
        }
        progress
            .advance(ExecutionStage::SymmetryReview)
            .map_err(FieldAssemblyError::Execution)?;

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
        if projected_energy.is_some() {
            progress
                .advance(ExecutionStage::ProjectedEnergy)
                .map_err(FieldAssemblyError::Execution)?;
        }
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

        let system = DenseFieldSystem {
            canonical,
            execution: self.semantic.execution_options(),
            matrix: DenseFieldMatrix {
                dimension: system_dimension,
                values: dense,
            },
            rhs,
            center_count: centers,
            diagnostics,
            cpd,
        };
        progress.complete().map_err(FieldAssemblyError::Execution)?;
        Ok(system)
    }
}

pub(crate) fn observation_at<const D: usize>(
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

pub(crate) fn validate_capabilities<const D: usize, E>(
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
    try_filled(count, 0.0, storage)
}

fn try_filled<T: Clone, E>(
    count: usize,
    value: T,
    storage: FieldAssemblyStorage,
) -> Result<Vec<T>, FieldAssemblyError<E>> {
    #[cfg(test)]
    test_allocation_attempt(storage, count)?;
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| FieldAssemblyError::AllocationFailed {
            storage,
            requested: count,
        })?;
    values.resize(count, value);
    Ok(values)
}

fn try_reserve_exact<T, E>(
    values: &mut Vec<T>,
    count: usize,
    storage: FieldAssemblyStorage,
) -> Result<(), FieldAssemblyError<E>> {
    #[cfg(test)]
    test_allocation_attempt(storage, count)?;
    values
        .try_reserve_exact(count)
        .map_err(|_| FieldAssemblyError::AllocationFailed {
            storage,
            requested: count,
        })
}

fn try_copy<E>(
    values: &[f64],
    storage: FieldAssemblyStorage,
) -> Result<Vec<f64>, FieldAssemblyError<E>> {
    #[cfg(test)]
    test_allocation_attempt(storage, values.len())?;
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
    #[cfg(test)]
    test_allocation_attempt(FieldAssemblyStorage::VariableBlockName, value.len())?;
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

#[cfg(test)]
std::thread_local! {
    static FORCED_ALLOCATION_FAILURE: Cell<Option<FieldAssemblyStorage>> = const { Cell::new(None) };
    static FORCE_NEXT_ALLOCATION_FAILURE: Cell<bool> = const { Cell::new(false) };
    static POLYNOMIAL_VALUE_ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
    static POLYNOMIAL_GRADIENT_ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}

#[cfg(test)]
fn test_allocation_attempt<E>(
    storage: FieldAssemblyStorage,
    requested: usize,
) -> Result<(), FieldAssemblyError<E>> {
    let force_next = FORCE_NEXT_ALLOCATION_FAILURE.with(|force| force.replace(false));
    let force_storage = FORCED_ALLOCATION_FAILURE.with(|force| {
        if force.get() == Some(storage) {
            force.set(None);
            true
        } else {
            false
        }
    });
    if force_next || force_storage {
        return Err(FieldAssemblyError::AllocationFailed { storage, requested });
    }
    match storage {
        FieldAssemblyStorage::PolynomialValues => {
            POLYNOMIAL_VALUE_ALLOCATIONS.with(|count| count.set(count.get().saturating_add(1)));
        }
        FieldAssemblyStorage::PolynomialGradients => {
            POLYNOMIAL_GRADIENT_ALLOCATIONS.with(|count| count.set(count.get().saturating_add(1)));
        }
        _ => {}
    }
    Ok(())
}

fn symmetry_diagnostics(
    matrix: &[f64],
    dimension: usize,
    upper_triangle_blocks: usize,
    kernel_entry_evaluations: usize,
    reflected_kernel_entries: usize,
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
        assembly_block_size: DENSE_ASSEMBLY_BLOCK_SIZE,
        upper_triangle_blocks,
        kernel_entry_evaluations,
        reflected_kernel_entries,
        maximum_absolute_entry,
        normalized_asymmetry,
        symmetry_tolerance: SYMMETRY_TOLERANCE_FACTOR * dimension_scale * f64::EPSILON,
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fmt;
    use std::io;

    use super::*;
    use crate::{
        Enforcement, ExecutionOptions, FunctionalAtom, FunctionalExpr, FunctionalProvenance,
        FunctionalTerm, Gaussian, ObservationId, PolyharmonicSpline, RadialSeparation,
        SemanticConstraint, SemanticExpression, SemanticProvenance, SourceLocation,
        SpatialKernelJet,
    };

    type TestResult = Result<(), Box<dyn Error>>;

    #[derive(Debug)]
    struct EvaluatorFailure;

    impl fmt::Display for EvaluatorFailure {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("forced evaluator failure")
        }
    }

    impl Error for EvaluatorFailure {}

    struct ForcedStorageAllocationFailure;

    impl ForcedStorageAllocationFailure {
        fn new(storage: FieldAssemblyStorage) -> Self {
            FORCED_ALLOCATION_FAILURE.with(|force| force.set(Some(storage)));
            Self
        }
    }

    impl Drop for ForcedStorageAllocationFailure {
        fn drop(&mut self) {
            FORCED_ALLOCATION_FAILURE.with(|force| force.set(None));
        }
    }

    fn value_problem(count: usize) -> Result<FieldProblem<1>, Box<dyn Error>> {
        let mut constraints = Vec::new();
        let mut centers = Vec::new();
        constraints.try_reserve_exact(count)?;
        centers.try_reserve_exact(count)?;
        for index in 0..count {
            let coordinate = f64::from(u32::try_from(index)?);
            let term_provenance = FunctionalProvenance::new(u64::try_from(index)? + 100);
            let expression = FunctionalExpr::try_new([FunctionalTerm::try_new(
                1.0,
                FunctionalAtom::value(Point::try_new([coordinate])?, term_provenance),
            )?])?;
            let observation = ObservationFunctional::new(expression.clone());
            centers.push(CenterRepresenter::new(expression));
            let line = NonZeroUsize::new(index.checked_add(1).ok_or("line overflow")?)
                .ok_or("zero line")?;
            let semantic_provenance = SemanticProvenance::try_new(
                ObservationId::new(u64::try_from(index)?),
                SourceLocation::try_new("field-unit.csv".to_owned(), line)?,
                "m".to_owned(),
                format!("field.equalities[{index}]"),
                Some("field-unit".to_owned()),
            )?;
            constraints.push(SemanticConstraint::try_new(
                semantic_provenance,
                SemanticRelation::Equality {
                    expression: SemanticExpression::try_new(observation, 0.0)?,
                    target: 0.0,
                },
                Enforcement::Hard,
            )?);
        }
        let semantic = SemanticProblemIr::try_new(constraints, ExecutionOptions::default())?;
        Ok(FieldProblem::try_new(semantic, centers)?)
    }

    fn assemble_cpd(
        problem: &FieldProblem<1>,
        kernel: PolyharmonicSpline,
    ) -> Result<DenseFieldSystem<1>, FieldAssemblyError<io::Error>> {
        problem.try_assemble(kernel.metadata(), |query, center, demand| {
            let separation = RadialSeparation::try_new(query, center)
                .map_err(|error| io::Error::other(error.to_string()))?;
            if separation.is_center() {
                let value = kernel
                    .radial_value(0.0)
                    .map_err(|error| io::Error::other(error.to_string()))?;
                return match demand {
                    KernelDerivativeOrder::Value => {
                        SpatialKernelJetPrefix::try_center_value(separation, value)
                            .map_err(|error| io::Error::other(error.to_string()))
                    }
                    KernelDerivativeOrder::First => {
                        SpatialKernelJetPrefix::try_center_through_first(separation, value)
                            .map_err(|error| io::Error::other(error.to_string()))
                    }
                    KernelDerivativeOrder::Second => {
                        let second = kernel
                            .radial_derivative(0.0, KernelDerivativeOrder::Second)
                            .map_err(|error| io::Error::other(error.to_string()))?
                            .ok_or_else(|| io::Error::other("missing center Hessian"))?;
                        SpatialKernelJetPrefix::try_center_through_second(separation, value, second)
                            .map_err(|error| io::Error::other(error.to_string()))
                    }
                    KernelDerivativeOrder::Third => {
                        Err(io::Error::other("unexpected third-order demand"))
                    }
                };
            }
            let radial = kernel
                .radial_jet(separation)
                .map_err(|error| io::Error::other(error.to_string()))?;
            Ok(SpatialKernelJet::try_new(separation, radial)
                .map_err(|error| io::Error::other(error.to_string()))?
                .into())
        })
    }

    fn scratch_allocation_counts(problem: &FieldProblem<1>) -> TestResult {
        let kernel = PolyharmonicSpline::try_new(3)?;
        POLYNOMIAL_VALUE_ALLOCATIONS.with(|count| count.set(0));
        POLYNOMIAL_GRADIENT_ALLOCATIONS.with(|count| count.set(0));
        let _system = assemble_cpd(problem, kernel)?;
        let values = POLYNOMIAL_VALUE_ALLOCATIONS.with(Cell::get);
        let gradients = POLYNOMIAL_GRADIENT_ALLOCATIONS.with(Cell::get);
        assert_eq!((values, gradients), (1, 1));
        Ok(())
    }

    #[test]
    fn evaluator_error_mapping_allocates_nothing_and_retains_provenance() -> TestResult {
        let problem = value_problem(1)?;
        let kernel = Gaussian::try_new(1.0)?;
        let result = problem.try_assemble(kernel.metadata(), |_, _, _| {
            FORCE_NEXT_ALLOCATION_FAILURE.with(|force| force.set(true));
            Err::<SpatialKernelJetPrefix<1>, _>(EvaluatorFailure)
        });
        let failpoint_remained_armed =
            FORCE_NEXT_ALLOCATION_FAILURE.with(|force| force.replace(false));
        assert!(failpoint_remained_armed);
        let Err(FieldAssemblyError::KernelAction {
            observation_index: 0,
            center_index: 0,
            source:
                KernelActionError::Evaluation {
                    observation_term_index: 0,
                    observation_provenance,
                    center_term_index: 0,
                    center_provenance,
                    source: EvaluatorFailure,
                },
        }) = result
        else {
            return Err("unexpected evaluator result".into());
        };
        assert_eq!(observation_provenance.identifier(), 100);
        assert_eq!(center_provenance.identifier(), 100);
        Ok(())
    }

    #[test]
    fn observation_polynomial_scratch_allocation_is_constant_in_row_count() -> TestResult {
        scratch_allocation_counts(&value_problem(3)?)?;
        scratch_allocation_counts(&value_problem(17)?)?;
        Ok(())
    }

    #[test]
    fn variable_block_reservation_failure_reports_exact_storage_and_count() -> TestResult {
        let problem = value_problem(3)?;
        let kernel = PolyharmonicSpline::try_new(3)?;
        let _failure = ForcedStorageAllocationFailure::new(FieldAssemblyStorage::VariableBlocks);
        assert!(matches!(
            assemble_cpd(&problem, kernel),
            Err(FieldAssemblyError::AllocationFailed {
                storage: FieldAssemblyStorage::VariableBlocks,
                requested: 2,
            })
        ));
        Ok(())
    }
}
