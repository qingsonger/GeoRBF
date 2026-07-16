//! Checked dense equality solvers and numerical diagnostics.
//!
//! The public contract owns all matrices and diagnostics. Nalgebra remains a
//! private factorization and rank-review adapter. Every solve selects checked
//! Cholesky or symmetric-pivoted Bunch--Kaufman LBLT explicitly, performs
//! RRQR screening and bounded SVD review, and rejects hidden fallback or
//! unrequested regularization.
//!
//! Assembled field systems cross the checked [`try_solve_field`] boundary;
//! callers cannot bypass its retained field execution limit through a direct
//! solver-owned conversion.
//!
//! ```compile_fail
//! use georbf::DenseEqualitySystem;
//!
//! let _ = DenseEqualitySystem::try_from_field::<1>;
//! ```

use std::error::Error;
use std::fmt;
use std::num::NonZeroUsize;

use nalgebra::{
    DMatrix, DVector, Dyn,
    linalg::{Cholesky, LBLT, SVD},
};

use crate::dimension::{Dim, SupportedDimension};
use crate::field::DenseFieldSystem;

const EQUILIBRATION_PASSES: usize = 8;
const SVD_MAX_ITERATIONS: usize = 10_000;
const AMBIGUITY_FACTOR: f64 = 16.0;
const RESIDUAL_FACTOR: f64 = 128.0;
const MAX_REFINEMENT_STEPS: usize = 8;
const PEAK_MATRIX_BUFFERS: usize = 6;
const PEAK_VECTOR_BUFFERS: usize = 32;
const PEAK_INDEX_PAIR_BUFFERS: usize = 2;

/// Explicit dense factorization used for a square equality system.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[must_use]
pub enum DenseFactorization {
    /// Checked Cholesky for a caller-asserted symmetric positive-definite system.
    Cholesky,
    /// Symmetric-pivoted Bunch--Kaufman LBLT for an indefinite system.
    PivotedLblt,
}

impl fmt::Display for DenseFactorization {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cholesky => formatter.write_str("checked Cholesky"),
            Self::PivotedLblt => formatter.write_str("pivoted LBLT"),
        }
    }
}

/// Explicit diagonal regularization policy.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub enum Regularization {
    /// Preserve the input matrix exactly.
    None,
    /// Add this positive finite value to every diagonal entry.
    Explicit(f64),
}

impl Regularization {
    fn amount(self) -> f64 {
        match self {
            Self::None => 0.0,
            Self::Explicit(value) => value,
        }
    }
}

/// Dimensionless condition-warning and rejection thresholds.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct ConditionPolicy {
    warning_threshold: f64,
    error_threshold: Option<f64>,
}

impl ConditionPolicy {
    /// Constructs a policy with a mandatory warning threshold and optional error threshold.
    ///
    /// # Errors
    ///
    /// Returns an error unless thresholds are finite, at least one, and the
    /// optional error threshold is strictly greater than the warning threshold.
    pub fn try_new(
        warning_threshold: f64,
        error_threshold: Option<f64>,
    ) -> Result<Self, DenseSolverConfigurationError> {
        if !warning_threshold.is_finite() || warning_threshold < 1.0 {
            return Err(DenseSolverConfigurationError::InvalidConditionWarning {
                value: warning_threshold,
            });
        }
        if let Some(error) = error_threshold
            && (!error.is_finite() || error <= warning_threshold)
        {
            return Err(DenseSolverConfigurationError::InvalidConditionError {
                warning: warning_threshold,
                value: error,
            });
        }
        Ok(Self {
            warning_threshold,
            error_threshold,
        })
    }

    /// Returns the dimensionless warning threshold.
    #[must_use]
    pub const fn warning_threshold(self) -> f64 {
        self.warning_threshold
    }

    /// Returns the optional dimensionless rejection threshold.
    #[must_use]
    pub const fn error_threshold(self) -> Option<f64> {
        self.error_threshold
    }
}

impl Default for ConditionPolicy {
    fn default() -> Self {
        Self {
            warning_threshold: 1.0 / f64::EPSILON.sqrt(),
            error_threshold: None,
        }
    }
}

/// Complete explicit policy for one dense solve.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct DenseSolveOptions {
    factorization: DenseFactorization,
    regularization: Regularization,
    condition_policy: ConditionPolicy,
    maximum_refinement_steps: usize,
    memory_limit_bytes: NonZeroUsize,
}

impl DenseSolveOptions {
    /// Constructs checked solve options.
    ///
    /// # Errors
    ///
    /// Rejects a nonpositive or nonfinite explicit regularization amount and
    /// an unbounded refinement count. The nonzero memory limit is mandatory so
    /// no backend allocation path is dispatched without a finite policy cap.
    pub fn try_new(
        factorization: DenseFactorization,
        regularization: Regularization,
        condition_policy: ConditionPolicy,
        maximum_refinement_steps: usize,
        memory_limit_bytes: NonZeroUsize,
    ) -> Result<Self, DenseSolverConfigurationError> {
        if let Regularization::Explicit(value) = regularization
            && (!value.is_finite() || value <= 0.0)
        {
            return Err(DenseSolverConfigurationError::InvalidRegularization { value });
        }
        if maximum_refinement_steps > MAX_REFINEMENT_STEPS {
            return Err(DenseSolverConfigurationError::TooManyRefinementSteps {
                requested: maximum_refinement_steps,
                maximum: MAX_REFINEMENT_STEPS,
            });
        }
        Ok(Self {
            factorization,
            regularization,
            condition_policy,
            maximum_refinement_steps,
            memory_limit_bytes,
        })
    }

    /// Returns the explicitly requested factorization.
    pub const fn factorization(self) -> DenseFactorization {
        self.factorization
    }

    /// Returns the explicit regularization policy.
    pub const fn regularization(self) -> Regularization {
        self.regularization
    }

    /// Returns the condition policy.
    pub const fn condition_policy(self) -> ConditionPolicy {
        self.condition_policy
    }

    /// Returns the bounded refinement-step count.
    #[must_use]
    pub const fn maximum_refinement_steps(self) -> usize {
        self.maximum_refinement_steps
    }

    /// Returns the explicit peak-working-set limit in bytes.
    #[must_use]
    pub const fn memory_limit_bytes(self) -> NonZeroUsize {
        self.memory_limit_bytes
    }

    const fn with_memory_limit_bytes(mut self, memory_limit_bytes: NonZeroUsize) -> Self {
        self.memory_limit_bytes = memory_limit_bytes;
        self
    }
}

/// Invalid user-supplied dense-solver policy.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub enum DenseSolverConfigurationError {
    /// The condition warning threshold is not finite and at least one.
    InvalidConditionWarning {
        /// Rejected value.
        value: f64,
    },
    /// The condition error threshold is not finite and above the warning threshold.
    InvalidConditionError {
        /// Accepted warning threshold.
        warning: f64,
        /// Rejected error threshold.
        value: f64,
    },
    /// The explicit diagonal regularization is not positive and finite.
    InvalidRegularization {
        /// Rejected value.
        value: f64,
    },
    /// The refinement count exceeds the fixed public bound.
    TooManyRefinementSteps {
        /// Requested count.
        requested: usize,
        /// Maximum accepted count.
        maximum: usize,
    },
}

impl fmt::Display for DenseSolverConfigurationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConditionWarning { value } => write!(
                formatter,
                "condition warning threshold must be finite and at least one, got {value}"
            ),
            Self::InvalidConditionError { warning, value } => write!(
                formatter,
                "condition error threshold must be finite and greater than warning threshold {warning}, got {value}"
            ),
            Self::InvalidRegularization { value } => write!(
                formatter,
                "explicit diagonal regularization must be positive and finite, got {value}"
            ),
            Self::TooManyRefinementSteps { requested, maximum } => write!(
                formatter,
                "requested {requested} refinement steps exceeds the bound {maximum}"
            ),
        }
    }
}

impl Error for DenseSolverConfigurationError {}

/// Immutable finite symmetric square equality system owned by `GeoRBF`.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct DenseEqualitySystem {
    dimension: usize,
    matrix: Vec<f64>,
    rhs: Vec<f64>,
}

impl DenseEqualitySystem {
    /// Validates a nonempty finite symmetric row-major matrix and right-hand side.
    ///
    /// # Errors
    ///
    /// Returns structured shape, finite-value, allocation, or symmetry errors.
    pub fn try_from_row_major(
        dimension: usize,
        matrix: Vec<f64>,
        rhs: Vec<f64>,
    ) -> Result<Self, DenseEqualitySystemError> {
        if dimension == 0 {
            return Err(DenseEqualitySystemError::Empty);
        }
        let entries = dimension
            .checked_mul(dimension)
            .ok_or(DenseEqualitySystemError::ShapeOverflow { dimension })?;
        if matrix.len() != entries || rhs.len() != dimension {
            return Err(DenseEqualitySystemError::ShapeMismatch {
                dimension,
                matrix_entries: matrix.len(),
                rhs_entries: rhs.len(),
            });
        }
        if let Some((index, value)) = matrix
            .iter()
            .copied()
            .enumerate()
            .find(|(_, value)| !value.is_finite())
        {
            return Err(DenseEqualitySystemError::NonFiniteMatrix { index, value });
        }
        if let Some((index, value)) = rhs
            .iter()
            .copied()
            .enumerate()
            .find(|(_, value)| !value.is_finite())
        {
            return Err(DenseEqualitySystemError::NonFiniteRightHandSide { index, value });
        }
        exact_symmetry_review(&matrix, dimension)?;
        Ok(Self {
            dimension,
            matrix,
            rhs,
        })
    }

    /// Copies an assembled field system after the caller has enforced the
    /// field-context peak estimate and effective execution limit.
    ///
    /// # Errors
    ///
    /// Returns a structured allocation or validation error.
    fn try_from_field<const D: usize>(
        system: &DenseFieldSystem<D>,
    ) -> Result<Self, DenseEqualitySystemError>
    where
        Dim<D>: SupportedDimension,
    {
        let matrix = try_copy(system.matrix().values(), DenseSolverStorage::InputMatrix)?;
        let rhs = try_copy(system.rhs(), DenseSolverStorage::InputRightHandSide)?;
        Self::try_from_row_major(system.matrix().dimension(), matrix, rhs)
    }

    /// Returns the equal row and column count.
    #[must_use]
    pub const fn dimension(&self) -> usize {
        self.dimension
    }

    /// Borrows the row-major matrix.
    #[must_use]
    pub fn matrix(&self) -> &[f64] {
        &self.matrix
    }

    /// Borrows the right-hand side.
    #[must_use]
    pub fn rhs(&self) -> &[f64] {
        &self.rhs
    }

    /// Returns the checked conservative peak-working-set estimate for a direct solve.
    ///
    /// The estimate includes GeoRBF-owned input, equilibration and refinement
    /// buffers plus nalgebra RRQR, SVD, factorization, and solve storage.
    ///
    /// # Errors
    ///
    /// Returns [`DenseSolveError::MemoryEstimateOverflow`] if the byte
    /// arithmetic cannot be represented by `usize`.
    pub fn try_estimated_peak_memory_bytes(&self) -> Result<usize, DenseSolveError> {
        estimate_peak_memory_bytes(self.dimension, SolveMemoryContext::OwnedSystem)
    }

    /// Solves this system with one explicit numerical policy.
    ///
    /// # Errors
    ///
    /// Returns structured rank, conditioning, scaling, factorization, finite-
    /// result, or residual-review errors without changing the requested problem.
    pub fn try_solve(&self, options: DenseSolveOptions) -> Result<DenseSolution, DenseSolveError> {
        let estimated_peak_memory_bytes = self.try_estimated_peak_memory_bytes()?;
        enforce_memory_limit(estimated_peak_memory_bytes, options.memory_limit_bytes)?;
        solve_validated(self, options, estimated_peak_memory_bytes)
    }
}

/// Invalid dense equality system input.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub enum DenseEqualitySystemError {
    /// A zero-dimensional system was supplied.
    Empty,
    /// The square entry count overflowed.
    ShapeOverflow {
        /// Requested dimension.
        dimension: usize,
    },
    /// Matrix or right-hand-side length does not match the dimension.
    ShapeMismatch {
        /// Requested dimension.
        dimension: usize,
        /// Supplied matrix entries.
        matrix_entries: usize,
        /// Supplied right-hand-side entries.
        rhs_entries: usize,
    },
    /// A matrix entry is nonfinite.
    NonFiniteMatrix {
        /// Row-major entry index.
        index: usize,
        /// Rejected value.
        value: f64,
    },
    /// A right-hand-side entry is nonfinite.
    NonFiniteRightHandSide {
        /// Entry index.
        index: usize,
        /// Rejected value.
        value: f64,
    },
    /// The matrix is not exactly symmetric.
    NotSymmetric {
        /// Row index.
        row: usize,
        /// Column index.
        column: usize,
        /// Entry at `(row, column)`.
        upper: f64,
        /// Entry at `(column, row)`.
        lower: f64,
    },
    /// A solver-owned input allocation failed.
    AllocationFailed {
        /// Storage role.
        storage: DenseSolverStorage,
        /// Requested entries.
        requested: usize,
    },
}

impl fmt::Display for DenseEqualitySystemError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("dense equality system must be nonempty"),
            Self::ShapeOverflow { dimension } => {
                write!(
                    formatter,
                    "dense square shape {dimension} by {dimension} overflows"
                )
            }
            Self::ShapeMismatch {
                dimension,
                matrix_entries,
                rhs_entries,
            } => write!(
                formatter,
                "dimension {dimension} requires {} matrix and {dimension} right-hand-side entries, got {matrix_entries} and {rhs_entries}",
                dimension.saturating_mul(*dimension)
            ),
            Self::NonFiniteMatrix { index, value } => {
                write!(formatter, "matrix entry {index} is nonfinite: {value}")
            }
            Self::NonFiniteRightHandSide { index, value } => {
                write!(
                    formatter,
                    "right-hand-side entry {index} is nonfinite: {value}"
                )
            }
            Self::NotSymmetric {
                row,
                column,
                upper,
                lower,
            } => write!(
                formatter,
                "matrix entries ({row}, {column})={upper} and ({column}, {row})={lower} are not exactly symmetric"
            ),
            Self::AllocationFailed { storage, requested } => {
                write!(
                    formatter,
                    "could not allocate {requested} entries for {storage}"
                )
            }
        }
    }
}

impl Error for DenseEqualitySystemError {}

/// Solver-owned allocation role used by structured errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[must_use]
pub enum DenseSolverStorage {
    /// Copied field matrix.
    InputMatrix,
    /// Copied field right-hand side.
    InputRightHandSide,
    /// Effective regularized matrix.
    EffectiveMatrix,
    /// Equilibrated rank-review matrix.
    RankReviewMatrix,
    /// Rank-review row scales.
    RankRowScales,
    /// Rank-review column scales.
    RankColumnScales,
    /// RRQR diagonal evidence.
    RrqrDiagonal,
    /// SVD singular-value evidence.
    SingularValues,
    /// Symmetric factorization scales.
    FactorizationScales,
    /// Symmetrically scaled matrix.
    ScaledMatrix,
    /// Scaled right-hand side.
    ScaledRightHandSide,
    /// Solution or refinement vector.
    Solution,
    /// Residual vector.
    Residual,
}

impl fmt::Display for DenseSolverStorage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InputMatrix => "input matrix",
            Self::InputRightHandSide => "input right-hand side",
            Self::EffectiveMatrix => "effective matrix",
            Self::RankReviewMatrix => "rank-review matrix",
            Self::RankRowScales => "rank-review row scales",
            Self::RankColumnScales => "rank-review column scales",
            Self::RrqrDiagonal => "RRQR diagonal",
            Self::SingularValues => "SVD singular values",
            Self::FactorizationScales => "factorization scales",
            Self::ScaledMatrix => "scaled matrix",
            Self::ScaledRightHandSide => "scaled right-hand side",
            Self::Solution => "solution",
            Self::Residual => "residual",
        })
    }
}

/// Matrix norms retained before and after equilibration.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct DenseMatrixNorms {
    /// Maximum absolute column sum.
    pub one: f64,
    /// Maximum absolute row sum.
    pub infinity: f64,
    /// Stable Frobenius norm.
    pub frobenius: f64,
}

/// Complete RRQR/SVD rank classification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[must_use]
pub enum DenseRankDecision {
    /// Both reviews agree that every column is independent.
    FullRank,
    /// Both reviews identify rank deficiency away from the threshold guard band.
    Deficient,
    /// Reviews disagree or a singular value lies in the closed ambiguity band.
    Ambiguous,
}

/// Complete scale-aware RRQR and bounded-SVD evidence.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct DenseRankDiagnostics {
    /// Square matrix dimension.
    pub dimension: usize,
    /// Fixed alternating equilibration pass count.
    pub equilibration_passes: usize,
    /// Cumulative row multipliers.
    pub row_scales: Vec<f64>,
    /// Cumulative column multipliers.
    pub column_scales: Vec<f64>,
    /// Norms in original matrix units.
    pub original_norms: DenseMatrixNorms,
    /// Norms after dimensionless equilibration.
    pub scaled_norms: DenseMatrixNorms,
    /// Absolute diagonal of the column-pivoted QR R factor.
    pub rrqr_diagonal: Vec<f64>,
    /// Dimension-times-epsilon RRQR threshold.
    pub rrqr_threshold: f64,
    /// Strict RRQR rank.
    pub rrqr_rank: usize,
    /// Descending singular values from bounded SVD review.
    pub singular_values: Vec<f64>,
    /// Dimension-times-epsilon SVD threshold.
    pub svd_threshold: f64,
    /// Strict SVD rank.
    pub svd_rank: usize,
    /// Lower closed ambiguity-band boundary.
    pub ambiguity_lower: f64,
    /// Upper closed ambiguity-band boundary.
    pub ambiguity_upper: f64,
    /// Whether a singular value is threshold-adjacent.
    pub threshold_adjacent: bool,
    /// Whether RRQR and SVD disagree.
    pub rank_disagreement: bool,
    /// Scale-aware condition estimate from the largest and smallest singular values.
    pub condition_estimate: f64,
    /// Final combined decision.
    pub decision: DenseRankDecision,
}

/// Retained equilibration and RRQR evidence after bounded SVD non-convergence.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct DenseIncompleteRankDiagnostics {
    /// Square matrix dimension.
    pub dimension: usize,
    /// Fixed alternating equilibration pass count.
    pub equilibration_passes: usize,
    /// Cumulative row multipliers.
    pub row_scales: Vec<f64>,
    /// Cumulative column multipliers.
    pub column_scales: Vec<f64>,
    /// Norms in original matrix units.
    pub original_norms: DenseMatrixNorms,
    /// Norms after dimensionless equilibration.
    pub scaled_norms: DenseMatrixNorms,
    /// Absolute RRQR diagonal.
    pub rrqr_diagonal: Vec<f64>,
    /// RRQR threshold.
    pub rrqr_threshold: f64,
    /// Strict RRQR rank.
    pub rrqr_rank: usize,
    /// Recorded bounded SVD iteration limit.
    pub maximum_svd_iterations: usize,
}

/// Scaled and original-unit residual evidence for one solution state.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct DenseResidualDiagnostics {
    /// Infinity norm of the dimensionless scaled residual.
    pub scaled_infinity: f64,
    /// Normwise backward error of the scaled system.
    pub scaled_backward_error: f64,
    /// Infinity norm computed with exact products and sums in original units.
    pub original_infinity: f64,
    /// Normwise backward error in original units.
    pub original_backward_error: f64,
}

/// Complete numerical evidence for an accepted dense solution.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct DenseSolveDiagnostics {
    /// Conservative peak-working-set estimate checked before backend dispatch.
    pub estimated_peak_memory_bytes: usize,
    /// Effective explicit memory limit, after applying a field execution limit.
    pub memory_limit_bytes: usize,
    /// Caller-requested factorization.
    pub requested_factorization: DenseFactorization,
    /// Actually used factorization; always identical without fallback.
    pub actual_factorization: DenseFactorization,
    /// Caller-requested regularization.
    pub requested_regularization: Regularization,
    /// Diagonal amount actually added.
    pub applied_regularization: f64,
    /// Rank review of the unmodified input matrix.
    pub original_rank: DenseRankDiagnostics,
    /// Rank review of the matrix actually factorized.
    pub effective_rank: DenseRankDiagnostics,
    /// Symmetric congruence scales used for factorization.
    pub factorization_scales: Vec<f64>,
    /// Whether pivoted LBLT used at least one nonzero 2-by-2 D block.
    pub has_two_by_two_pivot: bool,
    /// Whether the effective condition estimate crossed the warning threshold.
    pub condition_warning: bool,
    /// Residual evidence before refinement.
    pub initial_residual: DenseResidualDiagnostics,
    /// Residual evidence after accepted refinement corrections.
    pub final_residual: DenseResidualDiagnostics,
    /// Original-system residual after solving an explicitly regularized system.
    pub unregularized_final_residual: DenseResidualDiagnostics,
    /// Strictly decreasing corrections that were accepted.
    pub accepted_refinement_steps: usize,
    /// Requested refinement-step bound.
    pub maximum_refinement_steps: usize,
    /// Dimension-derived backward-error acceptance tolerance.
    pub residual_tolerance: f64,
}

/// Immutable accepted dense solution and diagnostics.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct DenseSolution {
    values: Vec<f64>,
    diagnostics: DenseSolveDiagnostics,
}

impl DenseSolution {
    /// Borrows solution values in variable order.
    #[must_use]
    pub fn values(&self) -> &[f64] {
        &self.values
    }

    /// Borrows complete numerical evidence.
    pub const fn diagnostics(&self) -> &DenseSolveDiagnostics {
        &self.diagnostics
    }
}

/// Structured failure from rank review, factorization, refinement, or residual checks.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub enum DenseSolveError {
    /// An assembled field system could not cross the solver-owned boundary.
    InvalidSystem(DenseEqualitySystemError),
    /// A solver-owned working allocation failed.
    AllocationFailed {
        /// Storage role.
        storage: DenseSolverStorage,
        /// Requested entries.
        requested: usize,
    },
    /// Checked peak-working-set byte arithmetic overflowed.
    MemoryEstimateOverflow {
        /// Matrix dimension whose estimate could not be represented.
        dimension: usize,
    },
    /// The conservative peak-working-set estimate exceeds the effective limit.
    MemoryLimitExceeded {
        /// Checked conservative peak estimate.
        estimated_peak_bytes: usize,
        /// Explicit effective policy limit.
        limit_bytes: usize,
    },
    /// A rank threshold dimension could not be represented safely.
    DimensionTooLarge {
        /// Matrix dimension.
        dimension: usize,
    },
    /// Equilibration produced an unrepresentable multiplier or matrix entry.
    UnrepresentableEquilibration {
        /// Whether the failure occurred on a row multiplier.
        row: bool,
        /// Row or column index.
        index: usize,
        /// Zero-based pass.
        pass: usize,
    },
    /// Required matrix norms overflowed despite finite entries.
    UnrepresentableMatrixNorms,
    /// RRQR or SVD returned nonfinite diagnostic evidence.
    NonFiniteRankEvidence,
    /// Bounded SVD did not converge; completed RRQR evidence is retained.
    SvdDidNotConverge {
        /// Evidence available before SVD.
        diagnostics: Box<DenseIncompleteRankDiagnostics>,
    },
    /// The effective system is rank-deficient.
    RankDeficient {
        /// Complete evidence.
        diagnostics: Box<DenseRankDiagnostics>,
    },
    /// The effective rank decision is threshold-ambiguous.
    RankAmbiguous {
        /// Complete evidence.
        diagnostics: Box<DenseRankDiagnostics>,
    },
    /// The explicit condition rejection threshold was crossed.
    ConditionLimitExceeded {
        /// Condition estimate.
        estimate: f64,
        /// Policy limit.
        limit: f64,
        /// Complete rank evidence.
        diagnostics: Box<DenseRankDiagnostics>,
    },
    /// Explicit regularization made a diagonal entry unrepresentable.
    UnrepresentableRegularization {
        /// Diagonal index.
        index: usize,
        /// Original entry.
        original: f64,
        /// Requested amount.
        amount: f64,
    },
    /// Symmetric congruence scaling was not representable.
    UnrepresentableFactorizationScaling {
        /// Row index.
        row: usize,
        /// Column index.
        column: usize,
    },
    /// Checked Cholesky rejected a non-SPD effective system.
    CholeskyRejected,
    /// Pivoted LBLT reported a zero pivot.
    LbltZeroPivot,
    /// A backend solve produced a nonfinite value.
    NonFiniteSolution {
        /// Solution entry index.
        index: usize,
        /// Rejected value.
        value: f64,
    },
    /// An exact original-unit residual could not be rounded representably.
    UnrepresentableOriginalResidual {
        /// Residual row.
        row: usize,
    },
    /// Final original-unit backward error exceeds the fixed tolerance.
    ResidualRejected {
        /// Complete solve evidence.
        diagnostics: Box<DenseSolveDiagnostics>,
    },
}

impl fmt::Display for DenseSolveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSystem(error) => {
                write!(formatter, "invalid dense equality system: {error}")
            }
            Self::AllocationFailed { storage, requested } => {
                write!(
                    formatter,
                    "could not allocate {requested} entries for {storage}"
                )
            }
            Self::MemoryEstimateOverflow { dimension } => write!(
                formatter,
                "peak-working-set estimate for dimension {dimension} overflows"
            ),
            Self::MemoryLimitExceeded {
                estimated_peak_bytes,
                limit_bytes,
            } => write!(
                formatter,
                "estimated peak working set {estimated_peak_bytes} bytes exceeds explicit limit {limit_bytes} bytes"
            ),
            Self::DimensionTooLarge { dimension } => {
                write!(
                    formatter,
                    "matrix dimension {dimension} exceeds diagnostic limits"
                )
            }
            Self::UnrepresentableEquilibration { row, index, pass } => write!(
                formatter,
                "{} equilibration scale {index} became unrepresentable on pass {pass}",
                if *row { "row" } else { "column" }
            ),
            Self::UnrepresentableMatrixNorms => {
                formatter.write_str("matrix norms are not finitely representable")
            }
            Self::NonFiniteRankEvidence => {
                formatter.write_str("rank-review backend returned nonfinite evidence")
            }
            Self::SvdDidNotConverge { diagnostics } => write!(
                formatter,
                "bounded SVD did not converge within {} iterations",
                diagnostics.maximum_svd_iterations
            ),
            Self::RankDeficient { diagnostics } => write!(
                formatter,
                "dense system is rank deficient (RRQR {}, SVD {}, dimension {})",
                diagnostics.rrqr_rank, diagnostics.svd_rank, diagnostics.dimension
            ),
            Self::RankAmbiguous { diagnostics } => write!(
                formatter,
                "dense system rank is ambiguous (RRQR {}, SVD {}, dimension {})",
                diagnostics.rrqr_rank, diagnostics.svd_rank, diagnostics.dimension
            ),
            Self::ConditionLimitExceeded {
                estimate, limit, ..
            } => write!(
                formatter,
                "condition estimate {estimate} exceeds explicit limit {limit}"
            ),
            Self::UnrepresentableRegularization {
                index,
                original,
                amount,
            } => write!(
                formatter,
                "adding regularization {amount} to diagonal {index} value {original} is unrepresentable"
            ),
            Self::UnrepresentableFactorizationScaling { row, column } => write!(
                formatter,
                "symmetric scaling of factorization entry ({row}, {column}) is unrepresentable"
            ),
            Self::CholeskyRejected => {
                formatter.write_str("checked Cholesky rejected the effective matrix")
            }
            Self::LbltZeroPivot => formatter.write_str("pivoted LBLT reported a zero pivot"),
            Self::NonFiniteSolution { index, value } => {
                write!(
                    formatter,
                    "backend solution entry {index} is nonfinite: {value}"
                )
            }
            Self::UnrepresentableOriginalResidual { row } => write!(
                formatter,
                "original-unit residual row {row} cannot be represented without overflow or underflow"
            ),
            Self::ResidualRejected { diagnostics } => write!(
                formatter,
                "final original-unit backward error {} exceeds tolerance {}",
                diagnostics.final_residual.original_backward_error, diagnostics.residual_tolerance
            ),
        }
    }
}

impl Error for DenseSolveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidSystem(error) => Some(error),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
enum SolveMemoryContext {
    OwnedSystem,
    FieldSystem,
}

fn estimate_peak_memory_bytes(
    dimension: usize,
    context: SolveMemoryContext,
) -> Result<usize, DenseSolveError> {
    let overflow = || DenseSolveError::MemoryEstimateOverflow { dimension };
    let entries = dimension.checked_mul(dimension).ok_or_else(overflow)?;
    let matrix_bytes = entries.checked_mul(size_of::<f64>()).ok_or_else(overflow)?;
    let vector_bytes = dimension
        .checked_mul(size_of::<f64>())
        .ok_or_else(overflow)?;
    let matrix_payload = matrix_bytes
        .checked_mul(PEAK_MATRIX_BUFFERS)
        .ok_or_else(overflow)?;
    let vector_payload = vector_bytes
        .checked_mul(PEAK_VECTOR_BUFFERS)
        .ok_or_else(overflow)?;
    let index_payload = dimension
        .checked_mul(size_of::<(usize, usize)>())
        .and_then(|bytes| bytes.checked_mul(PEAK_INDEX_PAIR_BUFFERS))
        .ok_or_else(overflow)?;
    let fixed_payload = size_of::<DenseRankDiagnostics>()
        .checked_mul(2)
        .and_then(|bytes| bytes.checked_add(size_of::<DenseSolveDiagnostics>()))
        .and_then(|bytes| bytes.checked_add(size_of::<ExactDotAccumulator>()))
        .ok_or_else(overflow)?;
    let field_source_payload = match context {
        SolveMemoryContext::OwnedSystem => 0,
        SolveMemoryContext::FieldSystem => matrix_bytes
            .checked_add(vector_bytes)
            .ok_or_else(overflow)?,
    };

    matrix_payload
        .checked_add(vector_payload)
        .and_then(|bytes| bytes.checked_add(index_payload))
        .and_then(|bytes| bytes.checked_add(fixed_payload))
        .and_then(|bytes| bytes.checked_add(field_source_payload))
        .ok_or_else(overflow)
}

fn enforce_memory_limit(
    estimated_peak_bytes: usize,
    memory_limit_bytes: NonZeroUsize,
) -> Result<(), DenseSolveError> {
    let limit_bytes = memory_limit_bytes.get();
    if estimated_peak_bytes > limit_bytes {
        return Err(DenseSolveError::MemoryLimitExceeded {
            estimated_peak_bytes,
            limit_bytes,
        });
    }
    Ok(())
}

/// Solves one assembled field system through the solver-owned matrix boundary.
///
/// # Errors
///
/// Returns structured boundary, rank, condition, factorization, or residual errors.
pub fn try_solve_field<const D: usize>(
    system: &DenseFieldSystem<D>,
    options: DenseSolveOptions,
) -> Result<DenseSolution, DenseSolveError>
where
    Dim<D>: SupportedDimension,
{
    let field_memory_limit = system.execution_options().memory_limit_bytes();
    let effective_memory_limit = field_memory_limit.map_or(options.memory_limit_bytes, |field| {
        if field.get() < options.memory_limit_bytes.get() {
            field
        } else {
            options.memory_limit_bytes
        }
    });
    let options = options.with_memory_limit_bytes(effective_memory_limit);
    let estimated_peak_memory_bytes =
        estimate_peak_memory_bytes(system.matrix().dimension(), SolveMemoryContext::FieldSystem)?;
    enforce_memory_limit(estimated_peak_memory_bytes, effective_memory_limit)?;
    let owned =
        DenseEqualitySystem::try_from_field(system).map_err(DenseSolveError::InvalidSystem)?;
    solve_validated(&owned, options, estimated_peak_memory_bytes)
}

#[derive(Clone, Debug)]
struct RankReview {
    diagnostics: DenseRankDiagnostics,
}

#[derive(Clone, Copy)]
enum SvdReviewMode {
    Bounded,
    #[cfg(test)]
    ForceNonConvergence,
}

#[allow(clippy::too_many_lines)]
fn solve_validated(
    system: &DenseEqualitySystem,
    options: DenseSolveOptions,
    estimated_peak_memory_bytes: usize,
) -> Result<DenseSolution, DenseSolveError> {
    let original_rank = diagnose_rank(&system.matrix, system.dimension, SvdReviewMode::Bounded)?;
    let amount = options.regularization.amount();
    let mut effective = try_copy_solve(&system.matrix, DenseSolverStorage::EffectiveMatrix)?;
    if amount != 0.0 {
        for index in 0..system.dimension {
            let offset = index * system.dimension + index;
            let original = effective[offset];
            let updated = original + amount;
            if !updated.is_finite() || updated.to_bits() == original.to_bits() {
                return Err(DenseSolveError::UnrepresentableRegularization {
                    index,
                    original,
                    amount,
                });
            }
            effective[offset] = updated;
        }
    }
    let effective_rank = if amount == 0.0 {
        original_rank.clone()
    } else {
        diagnose_rank(&effective, system.dimension, SvdReviewMode::Bounded)?
    };
    match effective_rank.diagnostics.decision {
        DenseRankDecision::FullRank => {}
        DenseRankDecision::Deficient => {
            return Err(DenseSolveError::RankDeficient {
                diagnostics: Box::new(effective_rank.diagnostics),
            });
        }
        DenseRankDecision::Ambiguous => {
            return Err(DenseSolveError::RankAmbiguous {
                diagnostics: Box::new(effective_rank.diagnostics),
            });
        }
    }
    if let Some(limit) = options.condition_policy.error_threshold
        && effective_rank.diagnostics.condition_estimate > limit
    {
        return Err(DenseSolveError::ConditionLimitExceeded {
            estimate: effective_rank.diagnostics.condition_estimate,
            limit,
            diagnostics: Box::new(effective_rank.diagnostics),
        });
    }

    let scales = symmetric_scales(&effective_rank.diagnostics)?;
    let scaled_matrix = symmetrically_scale_matrix(&effective, system.dimension, &scales)?;
    let scaled_rhs = scale_vector(
        &system.rhs,
        &scales,
        DenseSolverStorage::ScaledRightHandSide,
    )?;
    let factor = Factorization::try_new(options.factorization, &scaled_matrix, system.dimension)?;
    let has_two_by_two_pivot = factor.has_two_by_two_pivot();
    let mut scaled_solution = factor.solve(&scaled_rhs)?;
    let mut solution = scale_vector(&scaled_solution, &scales, DenseSolverStorage::Solution)?;
    validate_solution(&solution)?;
    let initial_residual = residual_diagnostics(
        &effective,
        &system.rhs,
        &solution,
        &scaled_matrix,
        &scaled_rhs,
        &scaled_solution,
        system.dimension,
    )?;
    let mut final_residual = initial_residual;
    let mut accepted_refinement_steps = 0;

    for _ in 0..options.maximum_refinement_steps {
        if final_residual.original_infinity == 0.0 {
            break;
        }
        let residual = floating_residual(&effective, &system.rhs, &solution, system.dimension)?;
        let scaled_residual =
            scale_vector(&residual, &scales, DenseSolverStorage::ScaledRightHandSide)?;
        let delta_scaled = factor.solve(&scaled_residual)?;
        let delta = scale_vector(&delta_scaled, &scales, DenseSolverStorage::Solution)?;
        let mut candidate = try_copy_solve(&solution, DenseSolverStorage::Solution)?;
        let mut candidate_scaled = try_copy_solve(&scaled_solution, DenseSolverStorage::Solution)?;
        for ((value, delta), (scaled, delta_scaled)) in candidate
            .iter_mut()
            .zip(delta)
            .zip(candidate_scaled.iter_mut().zip(delta_scaled))
        {
            *value += delta;
            *scaled += delta_scaled;
        }
        validate_solution(&candidate)?;
        validate_solution(&candidate_scaled)?;
        let candidate_residual = residual_diagnostics(
            &effective,
            &system.rhs,
            &candidate,
            &scaled_matrix,
            &scaled_rhs,
            &candidate_scaled,
            system.dimension,
        )?;
        if candidate_residual.original_infinity >= final_residual.original_infinity {
            break;
        }
        solution = candidate;
        scaled_solution = candidate_scaled;
        final_residual = candidate_residual;
        accepted_refinement_steps += 1;
    }

    let unregularized_scaled_matrix =
        symmetrically_scale_matrix(&system.matrix, system.dimension, &scales)?;
    let unregularized_final_residual = residual_diagnostics(
        &system.matrix,
        &system.rhs,
        &solution,
        &unregularized_scaled_matrix,
        &scaled_rhs,
        &scaled_solution,
        system.dimension,
    )?;
    let dimension =
        u32::try_from(system.dimension).map_err(|_| DenseSolveError::DimensionTooLarge {
            dimension: system.dimension,
        })?;
    let residual_tolerance = RESIDUAL_FACTOR * f64::from(dimension) * f64::EPSILON;
    let condition_warning =
        effective_rank.diagnostics.condition_estimate > options.condition_policy.warning_threshold;
    let diagnostics = DenseSolveDiagnostics {
        estimated_peak_memory_bytes,
        memory_limit_bytes: options.memory_limit_bytes.get(),
        requested_factorization: options.factorization,
        actual_factorization: options.factorization,
        requested_regularization: options.regularization,
        applied_regularization: amount,
        original_rank: original_rank.diagnostics,
        effective_rank: effective_rank.diagnostics,
        factorization_scales: scales,
        has_two_by_two_pivot,
        condition_warning,
        initial_residual,
        final_residual,
        unregularized_final_residual,
        accepted_refinement_steps,
        maximum_refinement_steps: options.maximum_refinement_steps,
        residual_tolerance,
    };
    if diagnostics.final_residual.original_backward_error > residual_tolerance {
        return Err(DenseSolveError::ResidualRejected {
            diagnostics: Box::new(diagnostics),
        });
    }
    Ok(DenseSolution {
        values: solution,
        diagnostics,
    })
}

fn exact_symmetry_review(matrix: &[f64], dimension: usize) -> Result<(), DenseEqualitySystemError> {
    for row in 0..dimension {
        for column in row + 1..dimension {
            let upper = matrix[row * dimension + column];
            let lower = matrix[column * dimension + row];
            let upper_is_zero = upper.to_bits().trailing_zeros() >= 63;
            let lower_is_zero = lower.to_bits().trailing_zeros() >= 63;
            if upper.to_bits() != lower.to_bits() && !(upper_is_zero && lower_is_zero) {
                return Err(DenseEqualitySystemError::NotSymmetric {
                    row,
                    column,
                    upper,
                    lower,
                });
            }
        }
    }
    Ok(())
}

fn diagnose_rrqr(
    backend: &DMatrix<f64>,
    dimension: usize,
) -> Result<(Vec<f64>, f64, usize), DenseSolveError> {
    let qr = backend.clone().col_piv_qr();
    let r = qr.r();
    let mut diagonal = try_with_capacity(dimension, DenseSolverStorage::RrqrDiagonal)?;
    diagonal.extend((0..dimension).map(|index| r[(index, index)].abs()));
    if diagonal.iter().any(|value| !value.is_finite()) {
        return Err(DenseSolveError::NonFiniteRankEvidence);
    }
    let threshold = rank_threshold(&diagonal, dimension)?;
    let rank = diagonal.iter().filter(|value| **value > threshold).count();
    Ok((diagonal, threshold, rank))
}

fn diagnose_rank(
    matrix: &[f64],
    dimension: usize,
    svd_mode: SvdReviewMode,
) -> Result<RankReview, DenseSolveError> {
    let original_norms = matrix_norms(matrix, dimension);
    let equilibrated = equilibrate(matrix, dimension)?;
    let scaled_norms = matrix_norms(&equilibrated.values, dimension);
    if !norms_are_finite(original_norms) || !norms_are_finite(scaled_norms) {
        return Err(DenseSolveError::UnrepresentableMatrixNorms);
    }
    let backend = DMatrix::from_row_slice(dimension, dimension, &equilibrated.values);
    let (rrqr_diagonal, rrqr_threshold, rrqr_rank) = diagnose_rrqr(&backend, dimension)?;
    let svd = match svd_mode {
        SvdReviewMode::Bounded => SVD::try_new(
            backend,
            false,
            false,
            5.0 * f64::EPSILON,
            SVD_MAX_ITERATIONS,
        ),
        #[cfg(test)]
        SvdReviewMode::ForceNonConvergence => None,
    };
    let Some(svd) = svd else {
        return Err(DenseSolveError::SvdDidNotConverge {
            diagnostics: Box::new(DenseIncompleteRankDiagnostics {
                dimension,
                equilibration_passes: EQUILIBRATION_PASSES,
                row_scales: equilibrated.row_scales,
                column_scales: equilibrated.column_scales,
                original_norms,
                scaled_norms,
                rrqr_diagonal,
                rrqr_threshold,
                rrqr_rank,
                maximum_svd_iterations: SVD_MAX_ITERATIONS,
            }),
        });
    };
    let mut singular_values = try_with_capacity(dimension, DenseSolverStorage::SingularValues)?;
    singular_values.extend(svd.singular_values.iter().copied());
    if singular_values.iter().any(|value| !value.is_finite()) {
        return Err(DenseSolveError::NonFiniteRankEvidence);
    }
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
        DenseRankDecision::Ambiguous
    } else if svd_rank == dimension {
        DenseRankDecision::FullRank
    } else {
        DenseRankDecision::Deficient
    };
    let condition_estimate = match (singular_values.first(), singular_values.last()) {
        (Some(largest), Some(smallest))
            if decision == DenseRankDecision::FullRank && *smallest > 0.0 =>
        {
            largest / smallest
        }
        _ => f64::INFINITY,
    };
    Ok(RankReview {
        diagnostics: DenseRankDiagnostics {
            dimension,
            equilibration_passes: EQUILIBRATION_PASSES,
            row_scales: equilibrated.row_scales,
            column_scales: equilibrated.column_scales,
            original_norms,
            scaled_norms,
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
        },
    })
}

struct EquilibratedMatrix {
    values: Vec<f64>,
    row_scales: Vec<f64>,
    column_scales: Vec<f64>,
}

fn equilibrate(matrix: &[f64], dimension: usize) -> Result<EquilibratedMatrix, DenseSolveError> {
    let mut values = try_copy_solve(matrix, DenseSolverStorage::RankReviewMatrix)?;
    let mut row_scales = try_filled(dimension, 1.0, DenseSolverStorage::RankRowScales)?;
    let mut column_scales = try_filled(dimension, 1.0, DenseSolverStorage::RankColumnScales)?;
    for pass in 0..EQUILIBRATION_PASSES {
        for (row, cumulative) in row_scales.iter_mut().enumerate() {
            let range = row * dimension..(row + 1) * dimension;
            let scale = values[range.clone()]
                .iter()
                .map(|value| value.abs())
                .fold(0.0_f64, f64::max);
            if scale > 0.0 {
                let updated = *cumulative / scale;
                if !updated.is_finite() || updated == 0.0 {
                    return Err(DenseSolveError::UnrepresentableEquilibration {
                        row: true,
                        index: row,
                        pass,
                    });
                }
                *cumulative = updated;
                for value in &mut values[range] {
                    let original = *value;
                    *value /= scale;
                    if original != 0.0 && *value == 0.0 {
                        return Err(DenseSolveError::UnrepresentableEquilibration {
                            row: true,
                            index: row,
                            pass,
                        });
                    }
                }
            }
        }
        for column in 0..dimension {
            let scale = (0..dimension)
                .map(|row| values[row * dimension + column].abs())
                .fold(0.0_f64, f64::max);
            if scale > 0.0 {
                let updated = column_scales[column] / scale;
                if !updated.is_finite() || updated == 0.0 {
                    return Err(DenseSolveError::UnrepresentableEquilibration {
                        row: false,
                        index: column,
                        pass,
                    });
                }
                column_scales[column] = updated;
                for row in 0..dimension {
                    let index = row * dimension + column;
                    let original = values[index];
                    values[index] /= scale;
                    if original != 0.0 && values[index] == 0.0 {
                        return Err(DenseSolveError::UnrepresentableEquilibration {
                            row: false,
                            index: column,
                            pass,
                        });
                    }
                }
            }
        }
    }
    Ok(EquilibratedMatrix {
        values,
        row_scales,
        column_scales,
    })
}

fn matrix_norms(matrix: &[f64], dimension: usize) -> DenseMatrixNorms {
    let mut one = 0.0_f64;
    let mut infinity = 0.0_f64;
    let mut frobenius = 0.0_f64;
    for row in 0..dimension {
        let mut row_sum = 0.0;
        for column in 0..dimension {
            let value = matrix[row * dimension + column].abs();
            row_sum += value;
            frobenius = frobenius.hypot(value);
        }
        infinity = infinity.max(row_sum);
    }
    for column in 0..dimension {
        let column_sum = (0..dimension)
            .map(|row| matrix[row * dimension + column].abs())
            .sum::<f64>();
        one = one.max(column_sum);
    }
    DenseMatrixNorms {
        one,
        infinity,
        frobenius,
    }
}

const fn norms_are_finite(norms: DenseMatrixNorms) -> bool {
    norms.one.is_finite() && norms.infinity.is_finite() && norms.frobenius.is_finite()
}

fn rank_threshold(values: &[f64], dimension: usize) -> Result<f64, DenseSolveError> {
    let dimension_u32 =
        u32::try_from(dimension).map_err(|_| DenseSolveError::DimensionTooLarge { dimension })?;
    Ok(f64::from(dimension_u32) * f64::EPSILON * values.iter().copied().fold(0.0_f64, f64::max))
}

fn symmetric_scales(diagnostics: &DenseRankDiagnostics) -> Result<Vec<f64>, DenseSolveError> {
    let mut scales = try_with_capacity(
        diagnostics.dimension,
        DenseSolverStorage::FactorizationScales,
    )?;
    for (index, (row, column)) in diagnostics
        .row_scales
        .iter()
        .copied()
        .zip(diagnostics.column_scales.iter().copied())
        .enumerate()
    {
        let scale = row.sqrt() * column.sqrt();
        if !scale.is_finite() || scale == 0.0 {
            return Err(DenseSolveError::UnrepresentableFactorizationScaling {
                row: index,
                column: index,
            });
        }
        scales.push(scale);
    }
    Ok(scales)
}

fn symmetrically_scale_matrix(
    matrix: &[f64],
    dimension: usize,
    scales: &[f64],
) -> Result<Vec<f64>, DenseSolveError> {
    let entries = dimension
        .checked_mul(dimension)
        .ok_or(DenseSolveError::DimensionTooLarge { dimension })?;
    let mut output = try_filled(entries, 0.0, DenseSolverStorage::ScaledMatrix)?;
    for row in 0..dimension {
        for column in row..dimension {
            let original = matrix[row * dimension + column];
            let value = original * scales[row] * scales[column];
            if !value.is_finite() || (original != 0.0 && value == 0.0) {
                return Err(DenseSolveError::UnrepresentableFactorizationScaling { row, column });
            }
            output[row * dimension + column] = value;
            output[column * dimension + row] = value;
        }
    }
    Ok(output)
}

fn scale_vector(
    values: &[f64],
    scales: &[f64],
    storage: DenseSolverStorage,
) -> Result<Vec<f64>, DenseSolveError> {
    let mut output = try_with_capacity(values.len(), storage)?;
    for (index, (value, scale)) in values
        .iter()
        .copied()
        .zip(scales.iter().copied())
        .enumerate()
    {
        let product = value * scale;
        if !product.is_finite() || (value != 0.0 && product == 0.0) {
            return Err(DenseSolveError::UnrepresentableFactorizationScaling {
                row: index,
                column: index,
            });
        }
        output.push(product);
    }
    Ok(output)
}

enum Factorization {
    Cholesky(Cholesky<f64, Dyn>),
    PivotedLblt {
        factor: LBLT<f64, Dyn>,
        has_two_by_two_pivot: bool,
    },
}

impl Factorization {
    fn try_new(
        requested: DenseFactorization,
        matrix: &[f64],
        dimension: usize,
    ) -> Result<Self, DenseSolveError> {
        let matrix = DMatrix::from_row_slice(dimension, dimension, matrix);
        match requested {
            DenseFactorization::Cholesky => matrix
                .cholesky()
                .map(Self::Cholesky)
                .ok_or(DenseSolveError::CholeskyRejected),
            DenseFactorization::PivotedLblt => {
                let factor = LBLT::new(matrix);
                let d = factor.d();
                let has_two_by_two_pivot =
                    (0..dimension.saturating_sub(1)).any(|index| d[(index + 1, index)] != 0.0);
                Ok(Self::PivotedLblt {
                    factor,
                    has_two_by_two_pivot,
                })
            }
        }
    }

    fn solve(&self, rhs: &[f64]) -> Result<Vec<f64>, DenseSolveError> {
        let rhs = DVector::from_column_slice(rhs);
        let solution = match self {
            Self::Cholesky(factor) => factor.solve(&rhs),
            Self::PivotedLblt { factor, .. } => {
                factor.solve(&rhs).ok_or(DenseSolveError::LbltZeroPivot)?
            }
        };
        let values = try_copy_solve(solution.as_slice(), DenseSolverStorage::Solution)?;
        validate_solution(&values)?;
        Ok(values)
    }

    const fn has_two_by_two_pivot(&self) -> bool {
        match self {
            Self::Cholesky(_) => false,
            Self::PivotedLblt {
                has_two_by_two_pivot,
                ..
            } => *has_two_by_two_pivot,
        }
    }
}

fn validate_solution(solution: &[f64]) -> Result<(), DenseSolveError> {
    if let Some((index, value)) = solution
        .iter()
        .copied()
        .enumerate()
        .find(|(_, value)| !value.is_finite())
    {
        return Err(DenseSolveError::NonFiniteSolution { index, value });
    }
    Ok(())
}

fn floating_residual(
    matrix: &[f64],
    rhs: &[f64],
    solution: &[f64],
    dimension: usize,
) -> Result<Vec<f64>, DenseSolveError> {
    let mut residual = try_filled(dimension, 0.0, DenseSolverStorage::Residual)?;
    for row in 0..dimension {
        let mut value = rhs[row];
        for column in 0..dimension {
            value = (-matrix[row * dimension + column]).mul_add(solution[column], value);
        }
        if !value.is_finite() {
            return Err(DenseSolveError::UnrepresentableOriginalResidual { row });
        }
        residual[row] = value;
    }
    Ok(residual)
}

fn residual_diagnostics(
    original_matrix: &[f64],
    original_rhs: &[f64],
    original_solution: &[f64],
    scaled_matrix: &[f64],
    scaled_rhs: &[f64],
    scaled_solution: &[f64],
    dimension: usize,
) -> Result<DenseResidualDiagnostics, DenseSolveError> {
    let scaled_residual = floating_residual(scaled_matrix, scaled_rhs, scaled_solution, dimension)?;
    let scaled_infinity = infinity_norm(&scaled_residual);
    let scaled_backward_error = backward_error(
        scaled_infinity,
        matrix_norms(scaled_matrix, dimension).infinity,
        infinity_norm(scaled_solution),
        infinity_norm(scaled_rhs),
    )?;
    let original_infinity =
        exact_original_residual_inf(original_matrix, original_rhs, original_solution, dimension)?;
    let original_backward_error = backward_error(
        original_infinity,
        matrix_norms(original_matrix, dimension).infinity,
        infinity_norm(original_solution),
        infinity_norm(original_rhs),
    )?;
    Ok(DenseResidualDiagnostics {
        scaled_infinity,
        scaled_backward_error,
        original_infinity,
        original_backward_error,
    })
}

fn backward_error(
    residual: f64,
    matrix_norm: f64,
    solution_norm: f64,
    rhs_norm: f64,
) -> Result<f64, DenseSolveError> {
    let denominator = matrix_norm.mul_add(solution_norm, rhs_norm);
    if !denominator.is_finite() {
        return Err(DenseSolveError::UnrepresentableOriginalResidual { row: 0 });
    }
    let value = if denominator == 0.0 {
        residual
    } else {
        residual / denominator
    };
    if !value.is_finite() {
        return Err(DenseSolveError::UnrepresentableOriginalResidual { row: 0 });
    }
    Ok(value)
}

fn infinity_norm(values: &[f64]) -> f64 {
    values
        .iter()
        .map(|value| value.abs())
        .fold(0.0_f64, f64::max)
}

fn exact_original_residual_inf(
    matrix: &[f64],
    rhs: &[f64],
    solution: &[f64],
    dimension: usize,
) -> Result<f64, DenseSolveError> {
    let mut norm = 0.0_f64;
    for row in 0..dimension {
        let mut accumulator = ExactDotAccumulator::default();
        accumulator
            .try_add_product(rhs[row], 1.0)
            .ok_or(DenseSolveError::UnrepresentableOriginalResidual { row })?;
        for column in 0..dimension {
            accumulator
                .try_add_product(-matrix[row * dimension + column], solution[column])
                .ok_or(DenseSolveError::UnrepresentableOriginalResidual { row })?;
        }
        let residual = accumulator
            .try_abs_f64()
            .ok_or(DenseSolveError::UnrepresentableOriginalResidual { row })?;
        norm = norm.max(residual);
    }
    Ok(norm)
}

fn try_copy(
    values: &[f64],
    storage: DenseSolverStorage,
) -> Result<Vec<f64>, DenseEqualitySystemError> {
    let mut copied = Vec::new();
    copied.try_reserve_exact(values.len()).map_err(|_| {
        DenseEqualitySystemError::AllocationFailed {
            storage,
            requested: values.len(),
        }
    })?;
    copied.extend_from_slice(values);
    Ok(copied)
}

fn try_copy_solve(
    values: &[f64],
    storage: DenseSolverStorage,
) -> Result<Vec<f64>, DenseSolveError> {
    let mut copied = try_with_capacity(values.len(), storage)?;
    copied.extend_from_slice(values);
    Ok(copied)
}

fn try_with_capacity(
    requested: usize,
    storage: DenseSolverStorage,
) -> Result<Vec<f64>, DenseSolveError> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(requested)
        .map_err(|_| DenseSolveError::AllocationFailed { storage, requested })?;
    Ok(values)
}

fn try_filled(
    requested: usize,
    value: f64,
    storage: DenseSolverStorage,
) -> Result<Vec<f64>, DenseSolveError> {
    let mut values = try_with_capacity(requested, storage)?;
    values.resize(requested, value);
    Ok(values)
}

const EXACT_PRODUCT_MIN_EXPONENT: i32 = -2148;
const EXACT_DOT_LIMBS: usize = 67;

#[derive(Clone)]
struct ExactDotAccumulator {
    words: [u64; EXACT_DOT_LIMBS],
}

impl Default for ExactDotAccumulator {
    fn default() -> Self {
        Self {
            words: [0; EXACT_DOT_LIMBS],
        }
    }
}

impl ExactDotAccumulator {
    fn try_add_product(&mut self, first: f64, second: f64) -> Option<()> {
        if first == 0.0 || second == 0.0 {
            return Some(());
        }
        let (first_negative, first_significand, first_exponent) = exact_factor(first)?;
        let (second_negative, second_significand, second_exponent) = exact_factor(second)?;
        let product = u128::from(first_significand) * u128::from(second_significand);
        let exponent = first_exponent.checked_add(second_exponent)?;
        let shift = usize::try_from(exponent.checked_sub(EXACT_PRODUCT_MIN_EXPONENT)?).ok()?;
        add_shifted_signed_product(
            &mut self.words,
            product,
            shift,
            first_negative != second_negative,
        )
    }

    fn try_abs_f64(&self) -> Option<f64> {
        let mut magnitude = self.words;
        if magnitude[EXACT_DOT_LIMBS - 1] >> 63 != 0 {
            twos_complement_in_place(&mut magnitude);
        }
        let Some(highest_bit) = highest_set_bit(&magnitude) else {
            return Some(0.0);
        };
        let exact_exponent = i32::try_from(highest_bit)
            .ok()?
            .checked_add(EXACT_PRODUCT_MIN_EXPONENT)?;
        if exact_exponent > 1023 {
            return None;
        }
        let normal = exact_exponent >= -1022;
        let retained_exponent = if normal { exact_exponent - 52 } else { -1074 };
        let shift = usize::try_from(retained_exponent - EXACT_PRODUCT_MIN_EXPONENT).ok()?;
        let mut significand = shifted_low_u64(&magnitude, shift);
        let halfway = shift.checked_sub(1)?;
        let round_up = bit_is_set(&magnitude, halfway)
            && (any_bits_below(&magnitude, halfway) || significand & 1 == 1);
        if round_up {
            significand = significand.checked_add(1)?;
        }
        if normal {
            let mut rounded_exponent = exact_exponent;
            if significand == 1_u64 << 53 {
                significand >>= 1;
                rounded_exponent += 1;
            }
            if rounded_exponent > 1023 || !(1_u64 << 52..1_u64 << 53).contains(&significand) {
                return None;
            }
            let biased = u64::try_from(rounded_exponent + 1023).ok()?;
            Some(f64::from_bits(
                (biased << 52) | (significand - (1_u64 << 52)),
            ))
        } else if significand == 0 || significand > 1_u64 << 52 {
            None
        } else if significand == 1_u64 << 52 {
            Some(f64::MIN_POSITIVE)
        } else {
            Some(f64::from_bits(significand))
        }
    }
}

fn exact_factor(value: f64) -> Option<(bool, u64, i32)> {
    if !value.is_finite() || value == 0.0 {
        return None;
    }
    let bits = value.to_bits();
    let negative = bits >> 63 != 0;
    let magnitude = bits & 0x7fff_ffff_ffff_ffff;
    let biased_exponent = ((magnitude >> 52) & 0x7ff) as i32;
    let fraction = magnitude & 0x000f_ffff_ffff_ffff;
    if biased_exponent == 0 {
        Some((negative, fraction, -1074))
    } else {
        Some((
            negative,
            (1_u64 << 52) | fraction,
            biased_exponent - 1023 - 52,
        ))
    }
}

fn add_shifted_signed_product(
    words: &mut [u64; EXACT_DOT_LIMBS],
    product: u128,
    shift: usize,
    negative: bool,
) -> Option<()> {
    let word = shift / 64;
    let offset = shift % 64;
    let low = u64::try_from(product & u128::from(u64::MAX)).ok()?;
    let high = u64::try_from(product >> 64).ok()?;
    let parts = if offset == 0 {
        [low, high, 0]
    } else {
        [
            low << offset,
            (low >> (64 - offset)) | (high << offset),
            high >> (64 - offset),
        ]
    };
    words.get(word.checked_add(2)?)?;
    let mut carry_or_borrow = false;
    for (index, target) in words.iter_mut().enumerate().skip(word) {
        let part = parts.get(index - word).copied().unwrap_or(0);
        if negative {
            let (difference, first_borrow) = target.overflowing_sub(part);
            let (difference, second_borrow) =
                difference.overflowing_sub(u64::from(carry_or_borrow));
            *target = difference;
            carry_or_borrow = first_borrow || second_borrow;
        } else {
            let (sum, first_carry) = target.overflowing_add(part);
            let (sum, second_carry) = sum.overflowing_add(u64::from(carry_or_borrow));
            *target = sum;
            carry_or_borrow = first_carry || second_carry;
        }
        if index >= word + 2 && !carry_or_borrow {
            break;
        }
    }
    Some(())
}

fn twos_complement_in_place(words: &mut [u64; EXACT_DOT_LIMBS]) {
    for word in words.iter_mut() {
        *word = !*word;
    }
    let mut carry = true;
    for word in words.iter_mut() {
        let (sum, next_carry) = word.overflowing_add(u64::from(carry));
        *word = sum;
        carry = next_carry;
        if !carry {
            break;
        }
    }
}

fn highest_set_bit(words: &[u64; EXACT_DOT_LIMBS]) -> Option<usize> {
    words
        .iter()
        .rposition(|word| *word != 0)
        .map(|word| word * 64 + (63 - words[word].leading_zeros() as usize))
}

fn shifted_low_u64(words: &[u64; EXACT_DOT_LIMBS], shift: usize) -> u64 {
    let word = shift / 64;
    let offset = shift % 64;
    let low = words.get(word).copied().unwrap_or(0) >> offset;
    if offset == 0 {
        low
    } else {
        low | (words.get(word + 1).copied().unwrap_or(0) << (64 - offset))
    }
}

fn bit_is_set(words: &[u64; EXACT_DOT_LIMBS], bit: usize) -> bool {
    words
        .get(bit / 64)
        .is_some_and(|word| word & (1_u64 << (bit % 64)) != 0)
}

fn any_bits_below(words: &[u64; EXACT_DOT_LIMBS], exclusive: usize) -> bool {
    let full_words = exclusive / 64;
    if words[..full_words].iter().any(|word| *word != 0) {
        return true;
    }
    let remainder = exclusive % 64;
    remainder != 0
        && words
            .get(full_words)
            .is_some_and(|word| word & ((1_u64 << remainder) - 1) != 0)
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::{
        DenseRankDecision, DenseSolveError, SolveMemoryContext, SvdReviewMode, diagnose_rank,
        estimate_peak_memory_bytes,
    };

    #[test]
    fn peak_memory_estimate_overflow_is_structured() {
        assert!(matches!(
            estimate_peak_memory_bytes(usize::MAX, SolveMemoryContext::OwnedSystem),
            Err(DenseSolveError::MemoryEstimateOverflow {
                dimension: usize::MAX
            })
        ));
    }

    #[test]
    fn forced_svd_nonconvergence_retains_rrqr_evidence() -> Result<(), Box<dyn Error>> {
        let result = diagnose_rank(&[2.0, 0.0, 0.0, 3.0], 2, SvdReviewMode::ForceNonConvergence);
        let diagnostics = match result {
            Err(DenseSolveError::SvdDidNotConverge { diagnostics }) => diagnostics,
            other => return Err(format!("unexpected forced-SVD result {other:?}").into()),
        };
        assert_eq!(diagnostics.dimension, 2);
        assert_eq!(diagnostics.rrqr_rank, 2);
        assert_eq!(diagnostics.rrqr_diagonal.len(), 2);
        assert_eq!(diagnostics.maximum_svd_iterations, 10_000);
        Ok(())
    }

    #[test]
    fn independent_nonzero_row_scaling_preserves_rank_classification() -> Result<(), Box<dyn Error>>
    {
        let original = diagnose_rank(&[4.0, 1.0, 1.0, 3.0], 2, SvdReviewMode::Bounded)?;
        let row_scaled = diagnose_rank(&[16.0, 4.0, 0.125, 0.375], 2, SvdReviewMode::Bounded)?;
        assert_eq!(original.diagnostics.decision, DenseRankDecision::FullRank);
        assert_eq!(
            original.diagnostics.decision,
            row_scaled.diagnostics.decision
        );
        assert_eq!(
            original.diagnostics.rrqr_rank,
            row_scaled.diagnostics.rrqr_rank
        );
        assert_eq!(
            original.diagnostics.svd_rank,
            row_scaled.diagnostics.svd_rank
        );
        Ok(())
    }
}
