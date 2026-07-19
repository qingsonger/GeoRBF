//! Checked canonical QP and SOCP dispatch.
//!
//! `GeoRBF` owns canonical compilation, sparse storage, provenance, settings,
//! memory policy, and all acceptance reviews. Clarabel is confined to this
//! module and receives only `P`, `q`, `A`, `b`, and an ordered product cone.
//! Presolve and KKT regularization are disabled; reduced-accuracy and limit
//! statuses are never accepted as successful solves.

use std::error::Error;
use std::fmt;
use std::num::{NonZeroU32, NonZeroUsize};

use clarabel::{
    algebra::CscMatrix,
    solver::{
        DefaultSettings, DefaultSolver, IPSolver, NonnegativeConeT, SecondOrderConeT, SolverStatus,
        SupportedConeT, ZeroConeT,
    },
};

use crate::execution::{
    ExecutionControl, ExecutionError, ExecutionOperation, ExecutionStage, ProgressTracker,
};
use crate::problem_ir::{
    AffineExpression, CanonicalProblem, CanonicalSoftRelation, ExecutionOptions,
    SemanticProvenance, SoftLoss,
};

const BACKEND_VERSION: &str = "clarabel-0.11.1";
const MAX_TOLERANCE: f64 = 1.0e-6;
const ITERATIVE_REFINEMENT_STEPS: u32 = 10;
const EQUILIBRATION_STEPS: u32 = 10;
const DIRECT_SOLVER: &str = "qdldl";
const MAX_STEP_FRACTION: f64 = 0.99;
const EQUILIBRATION_MIN_SCALING: f64 = 1.0e-4;
const EQUILIBRATION_MAX_SCALING: f64 = 1.0e4;
const LINESEARCH_BACKTRACK_STEP: f64 = 0.8;
const MIN_SWITCH_STEP_LENGTH: f64 = 1.0e-1;
const MIN_TERMINATE_STEP_LENGTH: f64 = 1.0e-4;
const STATIC_REGULARIZATION_CONSTANT: f64 = 1.0e-8;
const STATIC_REGULARIZATION_PROPORTIONAL: f64 = f64::EPSILON * f64::EPSILON;
const DYNAMIC_REGULARIZATION_EPSILON: f64 = 1.0e-13;
const DYNAMIC_REGULARIZATION_DELTA: f64 = 2.0e-7;
const ITERATIVE_REFINEMENT_RELATIVE_TOLERANCE: f64 = 1.0e-13;
const ITERATIVE_REFINEMENT_ABSOLUTE_TOLERANCE: f64 = 1.0e-12;
const ITERATIVE_REFINEMENT_STOP_RATIO: f64 = 5.0;
const GEO_OWNED_STORAGE_COPIES: usize = 16;
const BACKEND_DENSE_FILL_COPIES: usize = 8;
const SOLVE_PROGRESS_STEPS: usize = 4;

/// Exact public terminal-status vocabulary, independent of the backend type.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub enum ConvexBackendStatus {
    /// An exact requested-tolerance solution was reported.
    Solved,
    /// A primal-infeasibility certificate was reported.
    PrimalInfeasible,
    /// A dual-infeasibility certificate was reported.
    DualInfeasible,
    /// Only a reduced-accuracy solution was reported.
    ReducedAccuracy,
    /// Only a reduced-accuracy infeasibility result was reported.
    ReducedAccuracyInfeasible,
    /// The explicit iteration limit was reached.
    MaximumIterations,
    /// The explicit time limit was reached.
    MaximumTime,
    /// The backend reported a numerical error.
    NumericalError,
    /// The backend reported insufficient progress.
    InsufficientProgress,
    /// A callback terminated the backend.
    CallbackTerminated,
    /// The backend returned without a terminal decision.
    Unsolved,
}

impl fmt::Display for ConvexBackendStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Solved => "solved",
            Self::PrimalInfeasible => "primal infeasible",
            Self::DualInfeasible => "dual infeasible",
            Self::ReducedAccuracy => "reduced accuracy",
            Self::ReducedAccuracyInfeasible => "reduced-accuracy infeasible",
            Self::MaximumIterations => "maximum iterations",
            Self::MaximumTime => "maximum time",
            Self::NumericalError => "numerical error",
            Self::InsufficientProgress => "insufficient progress",
            Self::CallbackTerminated => "callback terminated",
            Self::Unsolved => "unsolved",
        })
    }
}

/// Explicit, bounded policy for one canonical convex solve.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct ConvexSolveOptions {
    tolerance: f64,
    maximum_iterations: NonZeroU32,
    time_limit_seconds: Option<f64>,
    memory_limit_bytes: NonZeroUsize,
}

impl ConvexSolveOptions {
    /// Constructs a strict solve policy.
    ///
    /// `tolerance` controls feasibility, gap, and infeasibility checks and may
    /// not exceed `1e-6`. An absent time limit is recorded as unbounded; the
    /// iteration and memory limits remain mandatory.
    ///
    /// # Errors
    ///
    /// Rejects a nonpositive, non-finite, or overly loose tolerance and a
    /// nonpositive or non-finite time limit.
    pub fn try_new(
        tolerance: f64,
        maximum_iterations: NonZeroU32,
        time_limit_seconds: Option<f64>,
        memory_limit_bytes: NonZeroUsize,
    ) -> Result<Self, ConvexSolverConfigurationError> {
        if !tolerance.is_finite() || tolerance <= 0.0 || tolerance > MAX_TOLERANCE {
            return Err(ConvexSolverConfigurationError::InvalidTolerance { value: tolerance });
        }
        if let Some(value) = time_limit_seconds
            && (!value.is_finite() || value <= 0.0)
        {
            return Err(ConvexSolverConfigurationError::InvalidTimeLimit { value });
        }
        Ok(Self {
            tolerance,
            maximum_iterations,
            time_limit_seconds,
            memory_limit_bytes,
        })
    }

    /// Returns the requested feasibility, gap, and certificate tolerance.
    #[must_use]
    pub const fn tolerance(self) -> f64 {
        self.tolerance
    }

    /// Returns the mandatory iteration limit.
    #[must_use]
    pub const fn maximum_iterations(self) -> NonZeroU32 {
        self.maximum_iterations
    }

    /// Returns the optional positive time limit in seconds.
    #[must_use]
    pub const fn time_limit_seconds(self) -> Option<f64> {
        self.time_limit_seconds
    }

    /// Returns the mandatory peak-working-set limit.
    #[must_use]
    pub const fn memory_limit_bytes(self) -> NonZeroUsize {
        self.memory_limit_bytes
    }
}

/// Invalid convex-solver policy.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub enum ConvexSolverConfigurationError {
    /// The requested tolerance is not positive, finite, and at most `1e-6`.
    InvalidTolerance {
        /// Rejected value.
        value: f64,
    },
    /// The requested time limit is not positive and finite.
    InvalidTimeLimit {
        /// Rejected value.
        value: f64,
    },
}

impl fmt::Display for ConvexSolverConfigurationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTolerance { value } => write!(
                formatter,
                "convex tolerance must be positive, finite, and at most {MAX_TOLERANCE}, got {value}"
            ),
            Self::InvalidTimeLimit { value } => write!(
                formatter,
                "convex time limit must be positive and finite, got {value}"
            ),
        }
    }
}

impl Error for ConvexSolverConfigurationError {}

/// Canonical relation represented by one reviewed diagnostic entry.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub enum ConvexConstraintKind {
    /// Hard equality.
    Equality,
    /// Hard lower bound.
    LowerBound,
    /// Hard upper bound.
    UpperBound,
    /// Hard second-order cone.
    SecondOrderCone,
    /// Soft objective contribution.
    SoftObjective,
    /// Compiler-owned epigraph row associated with a soft objective.
    Epigraph,
}

/// Original-unit residual and complete provenance for one relation.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct ConvexConstraintDiagnostics {
    /// Relation family.
    pub kind: ConvexConstraintKind,
    /// Complete semantic source provenance.
    pub provenance: SemanticProvenance,
    /// Raw residual or nonnegative violation in original units.
    pub original_residual: f64,
    /// Dimensionless residual divided by a scale-aware magnitude.
    pub normalized_residual: f64,
}

/// Every solver setting that can change termination or mathematical handling.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct ConvexSettingsDiagnostics {
    /// Requested feasibility tolerance.
    pub feasibility_tolerance: f64,
    /// Requested absolute gap tolerance.
    pub absolute_gap_tolerance: f64,
    /// Requested relative gap tolerance.
    pub relative_gap_tolerance: f64,
    /// Requested absolute infeasibility tolerance.
    pub absolute_infeasibility_tolerance: f64,
    /// Requested relative infeasibility tolerance.
    pub relative_infeasibility_tolerance: f64,
    /// Explicit homogeneous kappa/tau termination tolerance.
    pub kappa_tau_tolerance: f64,
    /// Reduced-status absolute gap tolerance; reduced statuses remain rejected.
    pub reduced_absolute_gap_tolerance: f64,
    /// Reduced-status relative gap tolerance; reduced statuses remain rejected.
    pub reduced_relative_gap_tolerance: f64,
    /// Reduced-status feasibility tolerance; reduced statuses remain rejected.
    pub reduced_feasibility_tolerance: f64,
    /// Reduced-status absolute infeasibility tolerance.
    pub reduced_absolute_infeasibility_tolerance: f64,
    /// Reduced-status relative infeasibility tolerance.
    pub reduced_relative_infeasibility_tolerance: f64,
    /// Reduced-status kappa/tau tolerance.
    pub reduced_kappa_tau_tolerance: f64,
    /// Explicit iteration limit.
    pub maximum_iterations: u32,
    /// Optional time limit in seconds.
    pub time_limit_seconds: Option<f64>,
    /// Explicit serial backend thread count.
    pub thread_count: u32,
    /// Maximum interior-point step fraction.
    pub maximum_step_fraction: f64,
    /// Whether backend equilibration was enabled.
    pub equilibration: ConvexSettingState,
    /// Fixed maximum equilibration passes.
    pub equilibration_steps: u32,
    /// Smallest permitted equilibration multiplier.
    pub equilibration_minimum_scaling: f64,
    /// Largest permitted equilibration multiplier.
    pub equilibration_maximum_scaling: f64,
    /// Line-search backtracking multiplier.
    pub line_search_backtrack_step: f64,
    /// Minimum asymmetric-cone scaling-switch step.
    pub minimum_switch_step_length: f64,
    /// Minimum termination step length.
    pub minimum_terminate_step_length: f64,
    /// Direct KKT solving is required.
    pub direct_kkt_solver: ConvexSettingState,
    /// Explicit selected direct factorization backend.
    pub direct_solver: &'static str,
    /// Whether backend iterative refinement was enabled.
    pub iterative_refinement: ConvexSettingState,
    /// Fixed maximum refinement steps.
    pub iterative_refinement_steps: u32,
    /// Iterative-refinement relative tolerance.
    pub iterative_refinement_relative_tolerance: f64,
    /// Iterative-refinement absolute tolerance.
    pub iterative_refinement_absolute_tolerance: f64,
    /// Iterative-refinement stalling ratio.
    pub iterative_refinement_stop_ratio: f64,
    /// Presolve is always disabled by this adapter.
    pub presolve: ConvexSettingState,
    /// Static KKT regularization is always disabled by this adapter.
    pub static_regularization: ConvexSettingState,
    /// Recorded inactive static-regularization constant.
    pub static_regularization_constant: f64,
    /// Recorded inactive proportional static-regularization constant.
    pub static_regularization_proportional: f64,
    /// Dynamic KKT regularization is always disabled by this adapter.
    pub dynamic_regularization: ConvexSettingState,
    /// Recorded inactive dynamic-regularization threshold.
    pub dynamic_regularization_epsilon: f64,
    /// Recorded inactive dynamic-regularization shift.
    pub dynamic_regularization_delta: f64,
    /// Structural sparse zeros are retained explicitly.
    pub input_sparse_drop_zeros: ConvexSettingState,
    /// Exact dimensionless tolerance used by independent reviews.
    pub independent_review_tolerance: f64,
    /// No multiplier or unrecorded absolute floor is applied to reviews.
    pub independent_review_tolerance_multiplier: u32,
}

/// Recorded binary state for a backend setting.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub enum ConvexSettingState {
    /// The setting was enabled.
    Enabled,
    /// The setting was disabled.
    Disabled,
}

/// Independent primal, dual, objective, and cone review.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct ConvexKktDiagnostics {
    /// Raw infinity norm of `A*x + s - b`.
    pub primal_residual_infinity: f64,
    /// Scale-aware primal residual.
    pub normalized_primal_residual: f64,
    /// Raw infinity norm of `P*x + q + A^T*z`.
    pub dual_residual_infinity: f64,
    /// Scale-aware dual residual.
    pub normalized_dual_residual: f64,
    /// Maximum normalized distance of `s` from the primal product cone.
    pub primal_cone_violation: f64,
    /// Maximum normalized distance of `z` from the dual product cone.
    pub dual_cone_violation: f64,
    /// Absolute complementarity `|s^T z|`.
    pub complementarity: f64,
    /// Independently recomputed semantic primal objective.
    pub original_objective: f64,
    /// Primal objective reconstructed from the compiled `P` and `q`.
    pub compiled_primal_objective: f64,
    /// Backend-reported primal objective.
    pub backend_primal_objective: f64,
    /// Backend-reported dual objective.
    pub backend_dual_objective: f64,
    /// Dual objective independently reconstructed from `P`, `x`, `b`, and `z`.
    pub reconstructed_dual_objective: f64,
    /// Absolute semantic-primal/reconstructed-dual gap.
    pub duality_gap: f64,
    /// Homogeneous normalized complementarity.
    pub normalized_complementarity: f64,
    /// Homogeneous normalized semantic primal-dual gap.
    pub normalized_duality_gap: f64,
}

/// Evidence for an independently accepted primal-infeasibility certificate.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct ConvexCertificateDiagnostics {
    /// Infinity norm of `A^T z` after certificate normalization.
    pub stationarity_residual_infinity: f64,
    /// Maximum componentwise homogeneous `A^T z` residual.
    pub normalized_stationarity_residual_infinity: f64,
    /// Maximum normalized dual-cone violation.
    pub dual_cone_violation: f64,
    /// Strict separating value `b^T z` in original compiled units.
    pub separating_value: f64,
    /// Scale used for the strict-separator decision.
    pub separator_scale: f64,
}

/// Reviewed primal-infeasibility certificate in GeoRBF-owned types.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct ConvexInfeasibilityCertificate {
    /// Exact terminal status.
    pub status: ConvexBackendStatus,
    /// Infinity-normalized dual vector in canonical row order.
    pub normalized_dual: Vec<f64>,
    /// Provenance and relation kind for every canonical row.
    pub rows: Vec<(ConvexConstraintKind, SemanticProvenance)>,
    /// Independent original-data certificate review.
    pub diagnostics: ConvexCertificateDiagnostics,
    /// Backend iteration count.
    pub iterations: u32,
}

/// Complete accepted-solve evidence.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct ConvexSolveDiagnostics {
    /// Private backend version label.
    pub backend: &'static str,
    /// Exact accepted terminal status.
    pub status: ConvexBackendStatus,
    /// Explicit settings used for dispatch.
    pub settings: ConvexSettingsDiagnostics,
    /// Conservative peak-working-set estimate.
    pub estimated_peak_memory_bytes: usize,
    /// Convex-option memory limit before combination with execution policy.
    pub convex_memory_limit_bytes: usize,
    /// Optional execution-level memory limit before combination.
    pub execution_memory_limit_bytes: Option<usize>,
    /// Effective smaller nonzero peak-working-set limit.
    pub memory_limit_bytes: usize,
    /// Original canonical variable count.
    pub original_variable_count: usize,
    /// Compiler-owned auxiliary variable count.
    pub auxiliary_variable_count: usize,
    /// Backend row count.
    pub row_count: usize,
    /// Backend sparse coefficient count across `P` and `A`.
    pub coefficient_count: usize,
    /// Ordered product-cone block count.
    pub cone_count: usize,
    /// Backend iteration count.
    pub iterations: u32,
    /// Backend solve duration in seconds.
    pub solve_time_seconds: f64,
    /// Independent KKT and objective review.
    pub kkt: ConvexKktDiagnostics,
    /// Provenance-preserving original-unit relation reviews.
    pub constraints: Vec<ConvexConstraintDiagnostics>,
}

/// Accepted immutable solution for the original canonical variables.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct ConvexSolution {
    values: Vec<f64>,
    diagnostics: ConvexSolveDiagnostics,
}

impl ConvexSolution {
    /// Borrows values in canonical variable-block order.
    #[must_use]
    pub fn values(&self) -> &[f64] {
        &self.values
    }

    /// Borrows complete solve evidence.
    pub const fn diagnostics(&self) -> &ConvexSolveDiagnostics {
        &self.diagnostics
    }
}

/// Structured canonical convex-solve failure.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub enum ConvexSolveError {
    /// Caller execution policy or cancellation rejected the operation.
    Execution(ExecutionError),
    /// Checked arithmetic could not represent the conservative estimate.
    MemoryEstimateOverflow,
    /// The conservative estimate exceeds the explicit limit.
    MemoryLimitExceeded {
        /// Estimated peak bytes.
        estimated_peak_bytes: usize,
        /// Explicit limit bytes.
        limit_bytes: usize,
    },
    /// Compiler arithmetic produced a non-finite value before dispatch.
    NonFiniteCompilation {
        /// Human-readable compiler field.
        field: &'static str,
    },
    /// Allocation for GeoRBF-owned adapter storage failed.
    AllocationFailed {
        /// Storage category.
        storage: &'static str,
        /// Requested element count.
        requested: usize,
    },
    /// The backend rejected validated problem setup.
    BackendSetup {
        /// Backend-owned message copied into `GeoRBF` storage.
        message: String,
    },
    /// Backend result vectors do not match the compiled dimensions.
    BackendDimensionMismatch {
        /// Expected primal-variable count.
        expected_variables: usize,
        /// Actual primal-variable count.
        actual_variables: usize,
        /// Expected row-vector count.
        expected_rows: usize,
        /// Actual primal-slack count.
        actual_slacks: usize,
        /// Actual dual count.
        actual_duals: usize,
    },
    /// A primal-infeasibility report passed independent certificate review.
    PrimalInfeasible {
        /// Reviewed certificate.
        certificate: ConvexInfeasibilityCertificate,
    },
    /// A primal-infeasibility report failed independent review.
    InvalidInfeasibilityCertificate {
        /// Failed review reason.
        reason: &'static str,
    },
    /// An exact-success status failed independent solution review.
    SolutionReviewFailed {
        /// Failed review reason.
        reason: &'static str,
        /// Measured value.
        value: f64,
        /// Accepted upper bound.
        tolerance: f64,
    },
    /// Any non-success status other than reviewed primal infeasibility.
    UnacceptedStatus {
        /// GeoRBF-owned status.
        status: ConvexBackendStatus,
        /// Iterations completed.
        iterations: u32,
    },
}

impl fmt::Display for ConvexSolveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Execution(source) => source.fmt(formatter),
            Self::MemoryEstimateOverflow => {
                formatter.write_str("canonical convex peak-memory estimate overflowed")
            }
            Self::MemoryLimitExceeded {
                estimated_peak_bytes,
                limit_bytes,
            } => write!(
                formatter,
                "estimated convex peak working set {estimated_peak_bytes} bytes exceeds explicit limit {limit_bytes} bytes"
            ),
            Self::NonFiniteCompilation { field } => {
                write!(
                    formatter,
                    "canonical convex compilation made {field} non-finite"
                )
            }
            Self::AllocationFailed { storage, requested } => {
                write!(
                    formatter,
                    "failed to allocate {requested} entries for {storage}"
                )
            }
            Self::BackendSetup { message } => {
                write!(formatter, "{BACKEND_VERSION} setup failed: {message}")
            }
            Self::BackendDimensionMismatch {
                expected_variables,
                actual_variables,
                expected_rows,
                actual_slacks,
                actual_duals,
            } => write!(
                formatter,
                "{BACKEND_VERSION} returned dimensions x={actual_variables}, s={actual_slacks}, z={actual_duals}; expected x={expected_variables}, s=z={expected_rows}"
            ),
            Self::PrimalInfeasible { .. } => formatter.write_str(
                "canonical convex problem is primal infeasible with a reviewed certificate",
            ),
            Self::InvalidInfeasibilityCertificate { reason } => write!(
                formatter,
                "backend primal-infeasibility certificate failed independent review: {reason}"
            ),
            Self::SolutionReviewFailed {
                reason,
                value,
                tolerance,
            } => write!(
                formatter,
                "backend solution failed {reason}: {value:.17e} exceeds {tolerance:.17e}"
            ),
            Self::UnacceptedStatus { status, iterations } => write!(
                formatter,
                "{BACKEND_VERSION} returned unaccepted status {status} after {iterations} iterations"
            ),
        }
    }
}

impl Error for ConvexSolveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Execution(source) => Some(source),
            _ => None,
        }
    }
}

impl From<ExecutionError> for ConvexSolveError {
    fn from(source: ExecutionError) -> Self {
        Self::Execution(source)
    }
}

#[derive(Clone, Debug)]
struct SlackExpression {
    terms: Vec<(usize, f64)>,
    constant: f64,
    kind: ConvexConstraintKind,
    provenance: SemanticProvenance,
}

#[derive(Clone, Debug)]
struct ConeBlock {
    rows: Vec<SlackExpression>,
    kind: ConeKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ConeKind {
    Zero,
    Nonnegative,
    SecondOrder,
}

#[derive(Clone, Debug)]
struct CompiledProblem {
    original_variables: usize,
    variables: usize,
    p_diagonal: Vec<f64>,
    q: Vec<f64>,
    blocks: Vec<ConeBlock>,
}

impl CompiledProblem {
    fn rows(&self) -> impl Iterator<Item = &SlackExpression> {
        self.blocks.iter().flat_map(|block| block.rows.iter())
    }

    fn row_count(&self) -> usize {
        self.blocks.iter().map(|block| block.rows.len()).sum()
    }

    fn coefficient_count(&self) -> usize {
        self.p_diagonal
            .iter()
            .filter(|value| **value != 0.0)
            .count()
            + self.rows().map(|row| row.terms.len()).sum::<usize>()
    }
}

struct Compiler {
    original_variables: usize,
    next_variable: usize,
    p_diagonal: Vec<f64>,
    q: Vec<f64>,
    zero_rows: Vec<SlackExpression>,
    nonnegative_rows: Vec<SlackExpression>,
    second_order_blocks: Vec<ConeBlock>,
}

impl Compiler {
    fn new(problem: &CanonicalProblem) -> Result<Self, ConvexSolveError> {
        let count = problem.variable_count();
        let mut p_diagonal = Vec::new();
        try_reserve(&mut p_diagonal, count, "quadratic diagonal")?;
        p_diagonal.resize(count, 0.0);
        let mut q = Vec::new();
        try_reserve(&mut q, count, "linear objective")?;
        q.resize(count, 0.0);
        Ok(Self {
            original_variables: count,
            next_variable: count,
            p_diagonal,
            q,
            zero_rows: Vec::new(),
            nonnegative_rows: Vec::new(),
            second_order_blocks: Vec::new(),
        })
    }

    fn auxiliary(&mut self) -> Result<usize, ConvexSolveError> {
        let index = self.next_variable;
        self.next_variable = self
            .next_variable
            .checked_add(1)
            .ok_or(ConvexSolveError::MemoryEstimateOverflow)?;
        self.p_diagonal
            .try_reserve(1)
            .map_err(|_| allocation("quadratic diagonal", self.next_variable))?;
        self.q
            .try_reserve(1)
            .map_err(|_| allocation("linear objective", self.next_variable))?;
        self.p_diagonal.push(0.0);
        self.q.push(0.0);
        Ok(index)
    }

    fn push_nonnegative(&mut self, row: SlackExpression) -> Result<(), ConvexSolveError> {
        self.nonnegative_rows
            .try_reserve(1)
            .map_err(|_| allocation("nonnegative rows", self.nonnegative_rows.len() + 1))?;
        self.nonnegative_rows.push(row);
        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    fn compile(mut self, problem: &CanonicalProblem) -> Result<CompiledProblem, ConvexSolveError> {
        for equality in problem.equalities() {
            self.zero_rows
                .try_reserve(1)
                .map_err(|_| allocation("zero-cone rows", self.zero_rows.len() + 1))?;
            self.zero_rows.push(slack_from_affine(
                equality.row(),
                -1.0,
                equality.rhs(),
                ConvexConstraintKind::Equality,
                equality.provenance(),
            )?);
        }
        for bound in problem.linear_bounds() {
            if let Some(lower) = bound.lower() {
                self.push_nonnegative(slack_from_affine(
                    bound.row(),
                    1.0,
                    -lower,
                    ConvexConstraintKind::LowerBound,
                    bound.provenance(),
                )?)?;
            }
            if let Some(upper) = bound.upper() {
                self.push_nonnegative(slack_from_affine(
                    bound.row(),
                    -1.0,
                    upper,
                    ConvexConstraintKind::UpperBound,
                    bound.provenance(),
                )?)?;
            }
        }
        for cone in problem.second_order_cones() {
            let mut rows = Vec::new();
            try_reserve(&mut rows, cone.lhs().len() + 1, "second-order cone rows")?;
            rows.push(slack_from_affine(
                cone.rhs(),
                1.0,
                0.0,
                ConvexConstraintKind::SecondOrderCone,
                cone.provenance(),
            )?);
            for lhs in cone.lhs() {
                rows.push(slack_from_affine(
                    lhs,
                    1.0,
                    0.0,
                    ConvexConstraintKind::SecondOrderCone,
                    cone.provenance(),
                )?);
            }
            self.second_order_blocks.try_reserve(1).map_err(|_| {
                allocation(
                    "second-order cone blocks",
                    self.second_order_blocks.len() + 1,
                )
            })?;
            self.second_order_blocks.push(ConeBlock {
                rows,
                kind: ConeKind::SecondOrder,
            });
        }
        for objective in problem.soft_objectives() {
            let violation = self.auxiliary()?;
            self.push_nonnegative(auxiliary_nonnegative(violation, objective.provenance())?)?;
            match objective.relation() {
                CanonicalSoftRelation::Equality(relation) => {
                    let inverse = checked_inverse(objective.scale())?;
                    let residual_constant = -relation.rhs() * inverse;
                    self.push_nonnegative(epigraph_row(
                        relation.row(),
                        -inverse,
                        -residual_constant,
                        violation,
                        objective.provenance(),
                    )?)?;
                    self.push_nonnegative(epigraph_row(
                        relation.row(),
                        inverse,
                        residual_constant,
                        violation,
                        objective.provenance(),
                    )?)?;
                }
                CanonicalSoftRelation::LinearBound(relation) => {
                    let inverse = checked_inverse(objective.scale())?;
                    if let Some(lower) = relation.lower() {
                        self.push_nonnegative(epigraph_row(
                            relation.row(),
                            inverse,
                            -lower * inverse,
                            violation,
                            objective.provenance(),
                        )?)?;
                    }
                    if let Some(upper) = relation.upper() {
                        self.push_nonnegative(epigraph_row(
                            relation.row(),
                            -inverse,
                            upper * inverse,
                            violation,
                            objective.provenance(),
                        )?)?;
                    }
                }
                CanonicalSoftRelation::SecondOrderCone(relation) => {
                    let inverse = checked_inverse(objective.scale())?;
                    let mut rows = Vec::new();
                    try_reserve(
                        &mut rows,
                        relation.lhs().len() + 1,
                        "soft second-order cone rows",
                    )?;
                    let mut rhs = slack_from_affine(
                        relation.rhs(),
                        inverse,
                        0.0,
                        ConvexConstraintKind::SoftObjective,
                        objective.provenance(),
                    )?;
                    rhs.terms
                        .try_reserve(1)
                        .map_err(|_| allocation("soft cone auxiliary term", rhs.terms.len() + 1))?;
                    rhs.terms.push((violation, 1.0));
                    rows.push(rhs);
                    for lhs in relation.lhs() {
                        rows.push(slack_from_affine(
                            lhs,
                            inverse,
                            0.0,
                            ConvexConstraintKind::SoftObjective,
                            objective.provenance(),
                        )?);
                    }
                    self.second_order_blocks.try_reserve(1).map_err(|_| {
                        allocation(
                            "second-order cone blocks",
                            self.second_order_blocks.len() + 1,
                        )
                    })?;
                    self.second_order_blocks.push(ConeBlock {
                        rows,
                        kind: ConeKind::SecondOrder,
                    });
                }
            }
            match objective.loss() {
                SoftLoss::SquaredL2 => self.p_diagonal[violation] = 2.0,
                SoftLoss::AbsoluteL1 => self.q[violation] = 1.0,
                SoftLoss::Huber { delta } => {
                    let quadratic = self.auxiliary()?;
                    let linear = self.auxiliary()?;
                    self.p_diagonal[quadratic] = 1.0;
                    self.q[linear] = delta;
                    self.push_nonnegative(auxiliary_nonnegative(
                        quadratic,
                        objective.provenance(),
                    )?)?;
                    self.push_nonnegative(auxiliary_nonnegative(linear, objective.provenance())?)?;
                    let mut terms = Vec::new();
                    try_reserve(&mut terms, 3, "Huber epigraph terms")?;
                    terms.extend([(violation, -1.0), (quadratic, 1.0), (linear, 1.0)]);
                    self.push_nonnegative(SlackExpression {
                        terms,
                        constant: 0.0,
                        kind: ConvexConstraintKind::Epigraph,
                        provenance: try_clone_provenance(objective.provenance())?,
                    })?;
                }
            }
        }
        validate_objective(&self.p_diagonal, &self.q)?;
        let mut blocks = Vec::new();
        let capacity = 2_usize
            .checked_add(self.second_order_blocks.len())
            .ok_or(ConvexSolveError::MemoryEstimateOverflow)?;
        try_reserve(&mut blocks, capacity, "cone blocks")?;
        if !self.zero_rows.is_empty() {
            blocks.push(ConeBlock {
                rows: self.zero_rows,
                kind: ConeKind::Zero,
            });
        }
        if !self.nonnegative_rows.is_empty() {
            blocks.push(ConeBlock {
                rows: self.nonnegative_rows,
                kind: ConeKind::Nonnegative,
            });
        }
        blocks.extend(self.second_order_blocks);
        Ok(CompiledProblem {
            original_variables: self.original_variables,
            variables: self.next_variable,
            p_diagonal: self.p_diagonal,
            q: self.q,
            blocks,
        })
    }
}

/// Solves a canonical problem using default serial execution metadata.
///
/// # Errors
///
/// Returns structured compilation, memory, backend-status, certificate, or
/// independent-review failures. Hard constraints are never relaxed.
pub fn try_solve_canonical(
    problem: &CanonicalProblem,
    options: ConvexSolveOptions,
) -> Result<ConvexSolution, ConvexSolveError> {
    try_solve_canonical_with_control(
        problem,
        options,
        ExecutionOptions::default(),
        ExecutionControl::default(),
    )
}

/// Solves a canonical problem with explicit execution metadata and controls.
///
/// The selected QDLDL path is serial. Cancellation is observed before and
/// after the indivisible backend call; no partial solution is returned.
///
/// # Errors
///
/// Returns the same failures as [`try_solve_canonical`] plus execution-policy
/// and cancellation failures.
pub fn try_solve_canonical_with_control(
    problem: &CanonicalProblem,
    options: ConvexSolveOptions,
    execution: ExecutionOptions,
    control: ExecutionControl<'_>,
) -> Result<ConvexSolution, ConvexSolveError> {
    let memory_limit = effective_memory_limit(options, execution);
    let memory_plan = preflight_memory(problem)?;
    if memory_plan.estimated_peak_bytes > memory_limit {
        return Err(ConvexSolveError::MemoryLimitExceeded {
            estimated_peak_bytes: memory_plan.estimated_peak_bytes,
            limit_bytes: memory_limit,
        });
    }
    let mut progress = ProgressTracker::try_new(
        control,
        ExecutionOperation::ConvexSolve,
        execution,
        SOLVE_PROGRESS_STEPS,
    )?;
    let compiled = Compiler::new(problem)?.compile(problem)?;
    memory_plan.verify(&compiled)?;
    progress.advance(ExecutionStage::MemoryReview)?;
    let matrices = build_matrices(&compiled)?;
    let settings = settings(options)?;
    let mut solver = DefaultSolver::new(
        &matrices.quadratic,
        &compiled.q,
        &matrices.constraints,
        &matrices.rhs,
        &matrices.cones,
        settings,
    )
    .map_err(|error| ConvexSolveError::BackendSetup {
        message: error.to_string(),
    })?;
    solver.solve();
    progress.advance(ExecutionStage::BackendSolve)?;
    let status = map_status(solver.solution.status);
    if status == ConvexBackendStatus::PrimalInfeasible {
        let certificate = review_certificate(
            &compiled,
            &matrices,
            &solver.solution.z,
            options,
            solver.solution.iterations,
        )?;
        progress.advance(ExecutionStage::CertificateReview)?;
        return Err(ConvexSolveError::PrimalInfeasible { certificate });
    }
    require_solved_status(status, solver.solution.iterations)?;
    let review = review_solution(
        problem,
        &compiled,
        &matrices,
        &solver.solution.x,
        &solver.solution.s,
        &solver.solution.z,
        solver.solution.obj_val,
        solver.solution.obj_val_dual,
        options,
    )?;
    progress.advance(ExecutionStage::ResidualReview)?;
    let values = try_copy(
        &solver.solution.x[..compiled.original_variables],
        "accepted original-variable solution",
    )?;
    let diagnostics = ConvexSolveDiagnostics {
        backend: BACKEND_VERSION,
        status,
        settings: settings_diagnostics(options),
        estimated_peak_memory_bytes: memory_plan.estimated_peak_bytes,
        convex_memory_limit_bytes: options.memory_limit_bytes().get(),
        execution_memory_limit_bytes: execution.memory_limit_bytes().map(NonZeroUsize::get),
        memory_limit_bytes: memory_limit,
        original_variable_count: compiled.original_variables,
        auxiliary_variable_count: compiled.variables - compiled.original_variables,
        row_count: compiled.row_count(),
        coefficient_count: compiled.coefficient_count(),
        cone_count: compiled.blocks.len(),
        iterations: solver.solution.iterations,
        solve_time_seconds: solver.solution.solve_time,
        kkt: review.kkt,
        constraints: review.constraints,
    };
    progress.complete()?;
    Ok(ConvexSolution {
        values,
        diagnostics,
    })
}

struct Matrices {
    quadratic: CscMatrix<f64>,
    constraints: CscMatrix<f64>,
    rhs: Vec<f64>,
    cones: Vec<SupportedConeT<f64>>,
}

fn build_matrices(compiled: &CompiledProblem) -> Result<Matrices, ConvexSolveError> {
    let quadratic_nonzeros = compiled
        .p_diagonal
        .iter()
        .filter(|value| **value != 0.0)
        .count();
    let mut p_colptr = Vec::new();
    let mut p_rowval = Vec::new();
    let mut p_values = Vec::new();
    try_reserve(&mut p_colptr, compiled.variables + 1, "P column pointers")?;
    try_reserve(&mut p_rowval, quadratic_nonzeros, "P row indices")?;
    try_reserve(&mut p_values, quadratic_nonzeros, "P values")?;
    p_colptr.push(0);
    for (index, value) in compiled.p_diagonal.iter().copied().enumerate() {
        if value != 0.0 {
            p_rowval.push(index);
            p_values.push(value);
        }
        p_colptr.push(p_rowval.len());
    }
    let row_count = compiled.row_count();
    let constraint_nonzeros = compiled
        .coefficient_count()
        .checked_sub(quadratic_nonzeros)
        .ok_or(ConvexSolveError::MemoryEstimateOverflow)?;
    let mut column_counts = Vec::new();
    try_reserve(&mut column_counts, compiled.variables, "A column counts")?;
    column_counts.resize(compiled.variables, 0_usize);
    for row in compiled.rows() {
        for &(column, _) in &row.terms {
            column_counts[column] = column_counts[column]
                .checked_add(1)
                .ok_or(ConvexSolveError::MemoryEstimateOverflow)?;
        }
    }
    let mut a_colptr = Vec::new();
    try_reserve(&mut a_colptr, compiled.variables + 1, "A column pointers")?;
    a_colptr.push(0_usize);
    for count in column_counts {
        let next = a_colptr[a_colptr.len() - 1]
            .checked_add(count)
            .ok_or(ConvexSolveError::MemoryEstimateOverflow)?;
        a_colptr.push(next);
    }
    let mut a_rowval = Vec::new();
    try_reserve(&mut a_rowval, constraint_nonzeros, "A row indices")?;
    a_rowval.resize(constraint_nonzeros, 0_usize);
    let mut a_values = Vec::new();
    try_reserve(&mut a_values, constraint_nonzeros, "A values")?;
    a_values.resize(constraint_nonzeros, 0.0);
    let mut cursor = try_copy(&a_colptr[..compiled.variables], "A column assembly cursors")?;
    let mut rhs = Vec::new();
    try_reserve(&mut rhs, row_count, "constraint right-hand side")?;
    for (row_index, row) in compiled.rows().enumerate() {
        if !row.constant.is_finite() {
            return Err(ConvexSolveError::NonFiniteCompilation {
                field: "constraint right-hand side",
            });
        }
        rhs.push(row.constant);
        for &(column, coefficient) in &row.terms {
            let position = cursor[column];
            a_rowval[position] = row_index;
            a_values[position] = -coefficient;
            cursor[column] += 1;
        }
    }
    let mut cones = Vec::new();
    try_reserve(&mut cones, compiled.blocks.len(), "cone descriptors")?;
    for block in &compiled.blocks {
        cones.push(match block.kind {
            ConeKind::Zero => ZeroConeT(block.rows.len()),
            ConeKind::Nonnegative => NonnegativeConeT(block.rows.len()),
            ConeKind::SecondOrder => SecondOrderConeT(block.rows.len()),
        });
    }
    Ok(Matrices {
        quadratic: CscMatrix::new(
            compiled.variables,
            compiled.variables,
            p_colptr,
            p_rowval,
            p_values,
        ),
        constraints: CscMatrix::new(row_count, compiled.variables, a_colptr, a_rowval, a_values),
        rhs,
        cones,
    })
}

fn settings(options: ConvexSolveOptions) -> Result<DefaultSettings<f64>, ConvexSolveError> {
    let mut direct_solve_method = String::new();
    direct_solve_method
        .try_reserve_exact(DIRECT_SOLVER.len())
        .map_err(|_| allocation("direct-solver setting", DIRECT_SOLVER.len()))?;
    direct_solve_method.push_str(DIRECT_SOLVER);
    Ok(DefaultSettings {
        verbose: false,
        max_iter: options.maximum_iterations().get(),
        time_limit: options.time_limit_seconds().unwrap_or(f64::INFINITY),
        max_step_fraction: MAX_STEP_FRACTION,
        max_threads: 1,
        tol_gap_abs: options.tolerance(),
        tol_gap_rel: options.tolerance(),
        tol_feas: options.tolerance(),
        tol_infeas_abs: options.tolerance(),
        tol_infeas_rel: options.tolerance(),
        tol_ktratio: options.tolerance(),
        reduced_tol_gap_abs: options.tolerance(),
        reduced_tol_gap_rel: options.tolerance(),
        reduced_tol_feas: options.tolerance(),
        reduced_tol_infeas_abs: options.tolerance(),
        reduced_tol_infeas_rel: options.tolerance(),
        reduced_tol_ktratio: options.tolerance(),
        equilibrate_enable: true,
        equilibrate_max_iter: EQUILIBRATION_STEPS,
        equilibrate_min_scaling: EQUILIBRATION_MIN_SCALING,
        equilibrate_max_scaling: EQUILIBRATION_MAX_SCALING,
        linesearch_backtrack_step: LINESEARCH_BACKTRACK_STEP,
        min_switch_step_length: MIN_SWITCH_STEP_LENGTH,
        min_terminate_step_length: MIN_TERMINATE_STEP_LENGTH,
        direct_kkt_solver: true,
        direct_solve_method,
        static_regularization_enable: false,
        static_regularization_constant: STATIC_REGULARIZATION_CONSTANT,
        static_regularization_proportional: STATIC_REGULARIZATION_PROPORTIONAL,
        dynamic_regularization_enable: false,
        dynamic_regularization_eps: DYNAMIC_REGULARIZATION_EPSILON,
        dynamic_regularization_delta: DYNAMIC_REGULARIZATION_DELTA,
        iterative_refinement_enable: true,
        iterative_refinement_reltol: ITERATIVE_REFINEMENT_RELATIVE_TOLERANCE,
        iterative_refinement_abstol: ITERATIVE_REFINEMENT_ABSOLUTE_TOLERANCE,
        iterative_refinement_max_iter: ITERATIVE_REFINEMENT_STEPS,
        iterative_refinement_stop_ratio: ITERATIVE_REFINEMENT_STOP_RATIO,
        presolve_enable: false,
        input_sparse_dropzeros: false,
    })
}

const fn settings_diagnostics(options: ConvexSolveOptions) -> ConvexSettingsDiagnostics {
    ConvexSettingsDiagnostics {
        feasibility_tolerance: options.tolerance(),
        absolute_gap_tolerance: options.tolerance(),
        relative_gap_tolerance: options.tolerance(),
        absolute_infeasibility_tolerance: options.tolerance(),
        relative_infeasibility_tolerance: options.tolerance(),
        kappa_tau_tolerance: options.tolerance(),
        reduced_absolute_gap_tolerance: options.tolerance(),
        reduced_relative_gap_tolerance: options.tolerance(),
        reduced_feasibility_tolerance: options.tolerance(),
        reduced_absolute_infeasibility_tolerance: options.tolerance(),
        reduced_relative_infeasibility_tolerance: options.tolerance(),
        reduced_kappa_tau_tolerance: options.tolerance(),
        maximum_iterations: options.maximum_iterations().get(),
        time_limit_seconds: options.time_limit_seconds(),
        thread_count: 1,
        maximum_step_fraction: MAX_STEP_FRACTION,
        equilibration: ConvexSettingState::Enabled,
        equilibration_steps: EQUILIBRATION_STEPS,
        equilibration_minimum_scaling: EQUILIBRATION_MIN_SCALING,
        equilibration_maximum_scaling: EQUILIBRATION_MAX_SCALING,
        line_search_backtrack_step: LINESEARCH_BACKTRACK_STEP,
        minimum_switch_step_length: MIN_SWITCH_STEP_LENGTH,
        minimum_terminate_step_length: MIN_TERMINATE_STEP_LENGTH,
        direct_kkt_solver: ConvexSettingState::Enabled,
        direct_solver: DIRECT_SOLVER,
        iterative_refinement: ConvexSettingState::Enabled,
        iterative_refinement_steps: ITERATIVE_REFINEMENT_STEPS,
        iterative_refinement_relative_tolerance: ITERATIVE_REFINEMENT_RELATIVE_TOLERANCE,
        iterative_refinement_absolute_tolerance: ITERATIVE_REFINEMENT_ABSOLUTE_TOLERANCE,
        iterative_refinement_stop_ratio: ITERATIVE_REFINEMENT_STOP_RATIO,
        presolve: ConvexSettingState::Disabled,
        static_regularization: ConvexSettingState::Disabled,
        static_regularization_constant: STATIC_REGULARIZATION_CONSTANT,
        static_regularization_proportional: STATIC_REGULARIZATION_PROPORTIONAL,
        dynamic_regularization: ConvexSettingState::Disabled,
        dynamic_regularization_epsilon: DYNAMIC_REGULARIZATION_EPSILON,
        dynamic_regularization_delta: DYNAMIC_REGULARIZATION_DELTA,
        input_sparse_drop_zeros: ConvexSettingState::Disabled,
        independent_review_tolerance: options.tolerance(),
        independent_review_tolerance_multiplier: 1,
    }
}

struct SolutionReview {
    kkt: ConvexKktDiagnostics,
    constraints: Vec<ConvexConstraintDiagnostics>,
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn review_solution(
    problem: &CanonicalProblem,
    compiled: &CompiledProblem,
    matrices: &Matrices,
    x: &[f64],
    s: &[f64],
    z: &[f64],
    backend_primal: f64,
    backend_dual: f64,
    options: ConvexSolveOptions,
) -> Result<SolutionReview, ConvexSolveError> {
    require_finite(x, "primal solution")?;
    require_finite(s, "primal slack")?;
    require_finite(z, "dual solution")?;
    require_finite(&[backend_primal, backend_dual], "backend objectives")?;
    if x.len() != compiled.variables || s.len() != compiled.row_count() || z.len() != s.len() {
        return Err(ConvexSolveError::BackendDimensionMismatch {
            expected_variables: compiled.variables,
            actual_variables: x.len(),
            expected_rows: compiled.row_count(),
            actual_slacks: s.len(),
            actual_duals: z.len(),
        });
    }
    let original_x = &x[..compiled.original_variables];
    let (original_objective, semantic_objective_scale) = semantic_objective(problem, original_x)?;
    let (compiled_objective, compiled_objective_scale) =
        objective_value_and_scale(&compiled.p_diagonal, &compiled.q, x)?;
    // Every canonical soft loss is dimensionless and has unit weight.  Its
    // count is therefore a homogeneous objective-unit reference, including
    // at an exact zero-residual optimum; it is not a row-unit absolute floor.
    let objective_count = u32::try_from(problem.soft_objectives().len())
        .map_err(|_| ConvexSolveError::MemoryEstimateOverflow)?;
    let natural_objective_scale = f64::from(objective_count);
    let objective_scale = semantic_objective_scale
        .max(compiled_objective_scale)
        .max(natural_objective_scale);
    let tolerance = options.tolerance();
    check_review(
        "semantic/compiled objective reconstruction",
        homogeneous_ratio(
            (original_objective - compiled_objective).abs(),
            objective_scale,
        ),
        tolerance,
    )?;
    check_review(
        "semantic/backend primal objective reconstruction",
        homogeneous_ratio(
            (original_objective - backend_primal).abs(),
            objective_scale.max(backend_primal.abs()),
        ),
        tolerance,
    )?;
    let mut primal_inf = 0.0_f64;
    let mut normalized_primal = 0.0_f64;
    for (row_index, row) in compiled.rows().enumerate() {
        let expression = evaluate_slack(row, x)?;
        let residual = (expression - s[row_index]).abs();
        primal_inf = primal_inf.max(residual);
        let mut scale = row.constant.abs() + s[row_index].abs();
        for &(column, coefficient) in &row.terms {
            scale = checked_sum(
                scale,
                (coefficient * x[column]).abs(),
                "primal review scale",
            )?;
        }
        if matches!(
            row.kind,
            ConvexConstraintKind::SoftObjective | ConvexConstraintKind::Epigraph
        ) {
            scale = scale.max(1.0);
        }
        normalized_primal = normalized_primal.max(homogeneous_ratio(residual, scale));
    }
    check_review("primal residual review", normalized_primal, tolerance)?;
    let mut stationarity = try_copy(&compiled.q, "dual stationarity")?;
    let mut dual_scales = Vec::new();
    try_reserve(&mut dual_scales, compiled.variables, "dual review scales")?;
    dual_scales.extend(compiled.q.iter().map(|value| value.abs()));
    for (index, value) in x.iter().copied().enumerate() {
        let contribution = compiled.p_diagonal[index] * value;
        stationarity[index] = finite_value(
            contribution + stationarity[index],
            "dual stationarity accumulation",
        )?;
        dual_scales[index] =
            checked_sum(dual_scales[index], contribution.abs(), "dual review scale")?;
    }
    for (row_index, row) in compiled.rows().enumerate() {
        for &(column, coefficient) in &row.terms {
            let contribution = -coefficient * z[row_index];
            stationarity[column] = finite_value(
                stationarity[column] + contribution,
                "dual stationarity accumulation",
            )?;
            dual_scales[column] =
                checked_sum(dual_scales[column], contribution.abs(), "dual review scale")?;
        }
    }
    let dual_inf = infinity_norm(&stationarity)?;
    let normalized_dual = stationarity
        .iter()
        .zip(&dual_scales)
        .map(|(residual, scale)| homogeneous_ratio(residual.abs(), *scale))
        .fold(0.0_f64, f64::max);
    check_review("dual stationarity review", normalized_dual, tolerance)?;
    let (primal_cone, dual_cone) = cone_reviews(compiled, s, z)?;
    check_review("primal cone review", primal_cone, tolerance)?;
    check_review("dual cone review", dual_cone, tolerance)?;
    let complementarity = dot(s, z)?.abs();
    let quadratic_half = half_quadratic_value(&compiled.p_diagonal, x)?;
    let rhs_dual = dot(&matrices.rhs, z)?;
    let reconstructed_dual = finite_value(-quadratic_half - rhs_dual, "dual objective")?;
    let dual_scale = quadratic_half.abs() + rhs_dual.abs();
    check_review(
        "reconstructed/backend dual objective",
        homogeneous_ratio(
            (reconstructed_dual - backend_dual).abs(),
            objective_scale.max(dual_scale).max(backend_dual.abs()),
        ),
        tolerance,
    )?;
    let primal_dual_scale = objective_scale.max(dual_scale);
    let complementarity_normalized = homogeneous_ratio(complementarity, primal_dual_scale);
    check_review(
        "complementarity review",
        complementarity_normalized,
        tolerance,
    )?;
    let duality_gap = (original_objective - reconstructed_dual).abs();
    let normalized_gap = homogeneous_ratio(duality_gap, primal_dual_scale);
    check_review("duality-gap review", normalized_gap, tolerance)?;
    let constraints = review_original_relations(problem, original_x, tolerance)?;
    Ok(SolutionReview {
        kkt: ConvexKktDiagnostics {
            primal_residual_infinity: primal_inf,
            normalized_primal_residual: normalized_primal,
            dual_residual_infinity: dual_inf,
            normalized_dual_residual: normalized_dual,
            primal_cone_violation: primal_cone,
            dual_cone_violation: dual_cone,
            complementarity,
            original_objective,
            compiled_primal_objective: compiled_objective,
            backend_primal_objective: backend_primal,
            backend_dual_objective: backend_dual,
            reconstructed_dual_objective: reconstructed_dual,
            duality_gap,
            normalized_complementarity: complementarity_normalized,
            normalized_duality_gap: normalized_gap,
        },
        constraints,
    })
}

fn review_original_relations(
    problem: &CanonicalProblem,
    x: &[f64],
    tolerance: f64,
) -> Result<Vec<ConvexConstraintDiagnostics>, ConvexSolveError> {
    let capacity = problem
        .equalities()
        .len()
        .checked_add(problem.linear_bounds().len().saturating_mul(2))
        .and_then(|value| value.checked_add(problem.second_order_cones().len()))
        .and_then(|value| value.checked_add(problem.soft_objectives().len()))
        .ok_or(ConvexSolveError::MemoryEstimateOverflow)?;
    let mut diagnostics = Vec::new();
    try_reserve(&mut diagnostics, capacity, "constraint diagnostics")?;
    for relation in problem.equalities() {
        let (actual, expression_scale) = evaluate_affine_and_scale(relation.row(), x)?;
        let residual = (actual - relation.rhs()).abs();
        let normalized = homogeneous_ratio(residual, expression_scale + relation.rhs().abs());
        check_review("hard equality original-unit review", normalized, tolerance)?;
        diagnostics.push(constraint_diagnostic(
            ConvexConstraintKind::Equality,
            relation.provenance(),
            residual,
            normalized,
        )?);
    }
    for relation in problem.linear_bounds() {
        let (actual, expression_scale) = evaluate_affine_and_scale(relation.row(), x)?;
        if let Some(lower) = relation.lower() {
            let violation = (lower - actual).max(0.0);
            let normalized = homogeneous_ratio(violation, expression_scale + lower.abs());
            check_review(
                "hard lower-bound original-unit review",
                normalized,
                tolerance,
            )?;
            diagnostics.push(constraint_diagnostic(
                ConvexConstraintKind::LowerBound,
                relation.provenance(),
                violation,
                normalized,
            )?);
        }
        if let Some(upper) = relation.upper() {
            let violation = (actual - upper).max(0.0);
            let normalized = homogeneous_ratio(violation, expression_scale + upper.abs());
            check_review(
                "hard upper-bound original-unit review",
                normalized,
                tolerance,
            )?;
            diagnostics.push(constraint_diagnostic(
                ConvexConstraintKind::UpperBound,
                relation.provenance(),
                violation,
                normalized,
            )?);
        }
    }
    for relation in problem.second_order_cones() {
        let (rhs, rhs_scale) = evaluate_affine_and_scale(relation.rhs(), x)?;
        let lhs_norm = stable_norm_iter(
            relation
                .lhs()
                .iter()
                .map(|expression| evaluate_affine(expression, x)),
        )?;
        let violation = (lhs_norm - rhs).max(0.0);
        let normalized = homogeneous_ratio(violation, lhs_norm + rhs_scale);
        check_review("hard cone original-unit review", normalized, tolerance)?;
        diagnostics.push(constraint_diagnostic(
            ConvexConstraintKind::SecondOrderCone,
            relation.provenance(),
            violation,
            normalized,
        )?);
    }
    for objective in problem.soft_objectives() {
        let violation = relation_violation(objective.relation(), x)?;
        let normalized = violation / objective.scale();
        if !normalized.is_finite() {
            return Err(ConvexSolveError::NonFiniteCompilation {
                field: "soft objective review",
            });
        }
        diagnostics.push(constraint_diagnostic(
            ConvexConstraintKind::SoftObjective,
            objective.provenance(),
            violation,
            normalized,
        )?);
    }
    Ok(diagnostics)
}

fn review_certificate(
    compiled: &CompiledProblem,
    matrices: &Matrices,
    certificate: &[f64],
    options: ConvexSolveOptions,
    iterations: u32,
) -> Result<ConvexInfeasibilityCertificate, ConvexSolveError> {
    require_finite(certificate, "infeasibility certificate")?;
    if certificate.len() != compiled.row_count() {
        return Err(ConvexSolveError::InvalidInfeasibilityCertificate {
            reason: "certificate length differs from canonical row count",
        });
    }
    let magnitude = infinity_norm(certificate)?;
    if magnitude == 0.0 {
        return Err(ConvexSolveError::InvalidInfeasibilityCertificate {
            reason: "certificate is the zero vector",
        });
    }
    let mut normalized = Vec::new();
    try_reserve(&mut normalized, certificate.len(), "normalized certificate")?;
    normalized.extend(certificate.iter().map(|value| value / magnitude));
    require_finite(&normalized, "normalized certificate")?;
    let (stationarity_inf, normalized_stationarity) =
        certificate_stationarity_metrics(compiled, &normalized)?;
    let dual_cone_violation = dual_cone_review(compiled, &normalized)?;
    let (separating_value, separator_scale, scaled_separator, scaled_separator_scale) =
        scaled_separator(&matrices.rhs, &normalized)?;
    let tolerance = options.tolerance();
    if normalized_stationarity > tolerance {
        return Err(ConvexSolveError::InvalidInfeasibilityCertificate {
            reason: "certificate stationarity residual is too large",
        });
    }
    if dual_cone_violation > tolerance {
        return Err(ConvexSolveError::InvalidInfeasibilityCertificate {
            reason: "certificate leaves the dual product cone",
        });
    }
    if scaled_separator >= -options.tolerance() * scaled_separator_scale {
        return Err(ConvexSolveError::InvalidInfeasibilityCertificate {
            reason: "certificate lacks a strict scale-aware separator",
        });
    }
    let mut rows = Vec::new();
    try_reserve(&mut rows, normalized.len(), "certificate provenance")?;
    for row in compiled.rows() {
        rows.push((row.kind, try_clone_provenance(&row.provenance)?));
    }
    Ok(ConvexInfeasibilityCertificate {
        status: ConvexBackendStatus::PrimalInfeasible,
        normalized_dual: normalized,
        rows,
        diagnostics: ConvexCertificateDiagnostics {
            stationarity_residual_infinity: stationarity_inf,
            normalized_stationarity_residual_infinity: normalized_stationarity,
            dual_cone_violation,
            separating_value,
            separator_scale,
        },
        iterations,
    })
}

fn cone_reviews(
    compiled: &CompiledProblem,
    primal: &[f64],
    dual: &[f64],
) -> Result<(f64, f64), ConvexSolveError> {
    let mut offset = 0_usize;
    let mut primal_max = 0.0_f64;
    let mut dual_max = 0.0_f64;
    for block in &compiled.blocks {
        let end = offset + block.rows.len();
        let primal_block = &primal[offset..end];
        let dual_block = &dual[offset..end];
        match block.kind {
            ConeKind::Zero => {
                let residual = infinity_norm(primal_block)?;
                primal_max = primal_max.max(homogeneous_ratio(residual, residual));
            }
            ConeKind::Nonnegative => {
                for (value, row) in primal_block.iter().zip(&block.rows) {
                    let scale = if matches!(
                        row.kind,
                        ConvexConstraintKind::SoftObjective | ConvexConstraintKind::Epigraph
                    ) {
                        value.abs().max(1.0)
                    } else {
                        value.abs()
                    };
                    primal_max = primal_max.max(homogeneous_ratio((-value).max(0.0), scale));
                }
                for (value, row) in dual_block.iter().zip(&block.rows) {
                    let scale = if matches!(
                        row.kind,
                        ConvexConstraintKind::SoftObjective | ConvexConstraintKind::Epigraph
                    ) {
                        value.abs().max(1.0)
                    } else {
                        value.abs()
                    };
                    dual_max = dual_max.max(homogeneous_ratio((-value).max(0.0), scale));
                }
            }
            ConeKind::SecondOrder => {
                let primal_norm = stable_norm(&primal_block[1..])?;
                let dual_norm = stable_norm(&dual_block[1..])?;
                primal_max = primal_max.max(homogeneous_ratio(
                    (primal_norm - primal_block[0]).max(0.0),
                    if block.rows[0].kind == ConvexConstraintKind::SoftObjective {
                        (primal_norm + primal_block[0].abs()).max(1.0)
                    } else {
                        primal_norm + primal_block[0].abs()
                    },
                ));
                dual_max = dual_max.max(homogeneous_ratio(
                    (dual_norm - dual_block[0]).max(0.0),
                    if block.rows[0].kind == ConvexConstraintKind::SoftObjective {
                        (dual_norm + dual_block[0].abs()).max(1.0)
                    } else {
                        dual_norm + dual_block[0].abs()
                    },
                ));
            }
        }
        offset = end;
    }
    Ok((primal_max, dual_max))
}

fn dual_cone_review(compiled: &CompiledProblem, dual: &[f64]) -> Result<f64, ConvexSolveError> {
    let mut offset = 0_usize;
    let mut maximum = 0.0_f64;
    for block in &compiled.blocks {
        let end = offset
            .checked_add(block.rows.len())
            .ok_or(ConvexSolveError::MemoryEstimateOverflow)?;
        let values = &dual[offset..end];
        match block.kind {
            ConeKind::Zero => {}
            ConeKind::Nonnegative => {
                for (value, row) in values.iter().zip(&block.rows) {
                    let scale = if matches!(
                        row.kind,
                        ConvexConstraintKind::SoftObjective | ConvexConstraintKind::Epigraph
                    ) {
                        value.abs().max(1.0)
                    } else {
                        value.abs()
                    };
                    maximum = maximum.max(homogeneous_ratio((-value).max(0.0), scale));
                }
            }
            ConeKind::SecondOrder => {
                let tail_norm = stable_norm(&values[1..])?;
                maximum = maximum.max(homogeneous_ratio(
                    (tail_norm - values[0]).max(0.0),
                    if block.rows[0].kind == ConvexConstraintKind::SoftObjective {
                        (tail_norm + values[0].abs()).max(1.0)
                    } else {
                        tail_norm + values[0].abs()
                    },
                ));
            }
        }
        offset = end;
    }
    Ok(maximum)
}

fn certificate_stationarity_metrics(
    compiled: &CompiledProblem,
    normalized_dual: &[f64],
) -> Result<(f64, f64), ConvexSolveError> {
    let mut raw_maximum = 0.0_f64;
    let mut maximum = 0.0_f64;
    for column in 0..compiled.variables {
        let coefficient_scale = compiled
            .rows()
            .enumerate()
            .flat_map(|(row_index, row)| {
                row.terms
                    .iter()
                    .filter_map(move |&(candidate, coefficient)| {
                        (candidate == column && normalized_dual[row_index] != 0.0)
                            .then_some(coefficient.abs())
                    })
            })
            .fold(0.0_f64, f64::max);
        if coefficient_scale == 0.0 {
            continue;
        }
        let mut residual = 0.0_f64;
        let mut scale = 0.0_f64;
        for (row_index, row) in compiled.rows().enumerate() {
            for &(candidate, coefficient) in &row.terms {
                if candidate != column || normalized_dual[row_index] == 0.0 {
                    continue;
                }
                let contribution = (coefficient / coefficient_scale) * normalized_dual[row_index];
                if contribution == 0.0 {
                    return Err(ConvexSolveError::InvalidInfeasibilityCertificate {
                        reason: "certificate stationarity normalization became unrepresentable",
                    });
                }
                residual = finite_value(
                    residual - contribution,
                    "normalized certificate stationarity",
                )?;
                scale = checked_sum(
                    scale,
                    contribution.abs(),
                    "normalized certificate stationarity scale",
                )?;
            }
        }
        let raw_residual = coefficient_scale * residual.abs();
        if (residual != 0.0 && raw_residual == 0.0) || !raw_residual.is_finite() {
            return Err(ConvexSolveError::InvalidInfeasibilityCertificate {
                reason: "certificate stationarity is not representable in original units",
            });
        }
        raw_maximum = raw_maximum.max(raw_residual);
        maximum = maximum.max(homogeneous_ratio(residual.abs(), scale));
    }
    Ok((raw_maximum, maximum))
}

fn scaled_separator(
    rhs: &[f64],
    normalized_dual: &[f64],
) -> Result<(f64, f64, f64, f64), ConvexSolveError> {
    let coefficient_scale = rhs
        .iter()
        .zip(normalized_dual)
        .filter_map(|(coefficient, dual)| (*dual != 0.0).then_some(coefficient.abs()))
        .fold(0.0_f64, f64::max);
    if coefficient_scale == 0.0 {
        return Ok((0.0, 0.0, 0.0, 0.0));
    }
    let mut scaled_value = 0.0_f64;
    let mut scaled_scale = 0.0_f64;
    for (coefficient, dual) in rhs.iter().zip(normalized_dual) {
        if *coefficient == 0.0 || *dual == 0.0 {
            continue;
        }
        let contribution = (*coefficient / coefficient_scale) * *dual;
        if contribution == 0.0 {
            return Err(ConvexSolveError::InvalidInfeasibilityCertificate {
                reason: "certificate separator normalization became unrepresentable",
            });
        }
        scaled_value = finite_value(
            scaled_value + contribution,
            "normalized certificate separator",
        )?;
        scaled_scale = checked_sum(
            scaled_scale,
            contribution.abs(),
            "normalized certificate separator scale",
        )?;
    }
    let value = coefficient_scale * scaled_value;
    let scale = coefficient_scale * scaled_scale;
    if (scaled_value != 0.0 && value == 0.0)
        || (scaled_scale != 0.0 && scale == 0.0)
        || !value.is_finite()
        || !scale.is_finite()
    {
        return Err(ConvexSolveError::InvalidInfeasibilityCertificate {
            reason: "certificate separator is not representable in original units",
        });
    }
    Ok((value, scale, scaled_value, scaled_scale))
}

fn relation_violation(
    relation: &CanonicalSoftRelation,
    x: &[f64],
) -> Result<f64, ConvexSolveError> {
    match relation {
        CanonicalSoftRelation::Equality(relation) => {
            Ok((evaluate_affine(relation.row(), x)? - relation.rhs()).abs())
        }
        CanonicalSoftRelation::LinearBound(relation) => {
            let value = evaluate_affine(relation.row(), x)?;
            let lower = relation
                .lower()
                .map_or(0.0, |bound| (bound - value).max(0.0));
            let upper = relation
                .upper()
                .map_or(0.0, |bound| (value - bound).max(0.0));
            Ok(lower.max(upper))
        }
        CanonicalSoftRelation::SecondOrderCone(relation) => {
            let lhs = stable_norm_iter(
                relation
                    .lhs()
                    .iter()
                    .map(|expression| evaluate_affine(expression, x)),
            )?;
            Ok((lhs - evaluate_affine(relation.rhs(), x)?).max(0.0))
        }
    }
}

fn semantic_objective(
    problem: &CanonicalProblem,
    x: &[f64],
) -> Result<(f64, f64), ConvexSolveError> {
    let mut value = 0.0_f64;
    let mut scale = 0.0_f64;
    for objective in problem.soft_objectives() {
        let normalized = relation_violation(objective.relation(), x)? / objective.scale();
        let contribution = match objective.loss() {
            SoftLoss::SquaredL2 => normalized * normalized,
            SoftLoss::AbsoluteL1 => normalized,
            SoftLoss::Huber { delta } if normalized <= delta => 0.5 * normalized * normalized,
            SoftLoss::Huber { delta } => delta * (normalized - 0.5 * delta),
        };
        value = finite_value(value + contribution, "semantic objective")?;
        scale = checked_sum(scale, contribution.abs(), "semantic objective scale")?;
    }
    Ok((value, scale))
}

fn slack_from_affine(
    expression: &AffineExpression,
    scale: f64,
    constant_addition: f64,
    kind: ConvexConstraintKind,
    provenance: &SemanticProvenance,
) -> Result<SlackExpression, ConvexSolveError> {
    let constant = scale.mul_add(expression.constant(), constant_addition);
    if !constant.is_finite() {
        return Err(ConvexSolveError::NonFiniteCompilation {
            field: "affine constant",
        });
    }
    let mut terms = Vec::new();
    try_reserve(
        &mut terms,
        expression.terms().len(),
        "compiled affine terms",
    )?;
    for term in expression.terms() {
        let coefficient = scale * term.coefficient();
        if !coefficient.is_finite() {
            return Err(ConvexSolveError::NonFiniteCompilation {
                field: "affine coefficient",
            });
        }
        terms.push((term.variable(), coefficient));
    }
    Ok(SlackExpression {
        terms,
        constant,
        kind,
        provenance: try_clone_provenance(provenance)?,
    })
}

fn epigraph_row(
    expression: &AffineExpression,
    scale: f64,
    constant_addition: f64,
    auxiliary: usize,
    provenance: &SemanticProvenance,
) -> Result<SlackExpression, ConvexSolveError> {
    let mut row = slack_from_affine(
        expression,
        scale,
        constant_addition,
        ConvexConstraintKind::Epigraph,
        provenance,
    )?;
    row.terms
        .try_reserve(1)
        .map_err(|_| allocation("epigraph auxiliary term", row.terms.len() + 1))?;
    row.terms.push((auxiliary, 1.0));
    Ok(row)
}

fn auxiliary_nonnegative(
    variable: usize,
    provenance: &SemanticProvenance,
) -> Result<SlackExpression, ConvexSolveError> {
    let mut terms = Vec::new();
    try_reserve(&mut terms, 1, "auxiliary nonnegative term")?;
    terms.push((variable, 1.0));
    Ok(SlackExpression {
        terms,
        constant: 0.0,
        kind: ConvexConstraintKind::Epigraph,
        provenance: try_clone_provenance(provenance)?,
    })
}

fn checked_inverse(value: f64) -> Result<f64, ConvexSolveError> {
    let inverse = value.recip();
    if inverse.is_finite() {
        Ok(inverse)
    } else {
        Err(ConvexSolveError::NonFiniteCompilation {
            field: "soft residual scale inverse",
        })
    }
}

fn validate_objective(p: &[f64], q: &[f64]) -> Result<(), ConvexSolveError> {
    if p.len() != q.len() || p.iter().any(|value| !value.is_finite() || *value < 0.0) {
        return Err(ConvexSolveError::NonFiniteCompilation {
            field: "PSD quadratic objective",
        });
    }
    require_finite(q, "linear objective")
}

#[derive(Clone, Copy, Debug)]
struct MemoryPlan {
    variables: usize,
    rows: usize,
    coefficients: usize,
    cone_blocks: usize,
    estimated_peak_bytes: usize,
}

impl MemoryPlan {
    fn verify(self, compiled: &CompiledProblem) -> Result<(), ConvexSolveError> {
        if self.variables == compiled.variables
            && self.rows == compiled.row_count()
            && self.coefficients == compiled.coefficient_count()
            && self.cone_blocks == compiled.blocks.len()
        {
            Ok(())
        } else {
            Err(ConvexSolveError::MemoryEstimateOverflow)
        }
    }
}

const fn effective_memory_limit(options: ConvexSolveOptions, execution: ExecutionOptions) -> usize {
    match execution.memory_limit_bytes() {
        Some(limit) => {
            let execution_limit = limit.get();
            let convex_limit = options.memory_limit_bytes().get();
            if execution_limit < convex_limit {
                execution_limit
            } else {
                convex_limit
            }
        }
        None => options.memory_limit_bytes().get(),
    }
}

#[allow(clippy::too_many_lines)]
fn preflight_memory(problem: &CanonicalProblem) -> Result<MemoryPlan, ConvexSolveError> {
    let mut variables = problem.variable_count();
    let mut rows = 0_usize;
    let mut coefficients = 0_usize;
    let mut second_order_blocks = 0_usize;
    let mut nonnegative_rows = 0_usize;
    let mut provenance_bytes = 0_usize;

    for relation in problem.equalities() {
        checked_accumulate(&mut rows, 1)?;
        checked_accumulate(&mut coefficients, relation.row().terms().len())?;
        checked_accumulate(
            &mut provenance_bytes,
            checked_product(provenance_storage_bytes(relation.provenance())?, 4)?,
        )?;
    }
    for relation in problem.linear_bounds() {
        let sides =
            usize::from(relation.lower().is_some()) + usize::from(relation.upper().is_some());
        checked_accumulate(&mut rows, sides)?;
        checked_accumulate(&mut nonnegative_rows, sides)?;
        checked_accumulate(
            &mut coefficients,
            checked_product(relation.row().terms().len(), sides)?,
        )?;
        checked_accumulate(
            &mut provenance_bytes,
            checked_product(
                provenance_storage_bytes(relation.provenance())?,
                2 * (sides + 1),
            )?,
        )?;
    }
    for relation in problem.second_order_cones() {
        let block_rows = relation
            .lhs()
            .len()
            .checked_add(1)
            .ok_or(ConvexSolveError::MemoryEstimateOverflow)?;
        checked_accumulate(&mut rows, block_rows)?;
        checked_accumulate(&mut coefficients, relation.rhs().terms().len())?;
        for expression in relation.lhs() {
            checked_accumulate(&mut coefficients, expression.terms().len())?;
        }
        checked_accumulate(&mut second_order_blocks, 1)?;
        checked_accumulate(
            &mut provenance_bytes,
            checked_product(
                provenance_storage_bytes(relation.provenance())?,
                2 * (block_rows + 1),
            )?,
        )?;
    }
    for objective in problem.soft_objectives() {
        checked_accumulate(&mut variables, 1)?;
        checked_accumulate(&mut rows, 1)?;
        checked_accumulate(&mut nonnegative_rows, 1)?;
        checked_accumulate(&mut coefficients, 1)?;
        let relation_rows = match objective.relation() {
            CanonicalSoftRelation::Equality(relation) => {
                checked_accumulate(&mut rows, 2)?;
                checked_accumulate(&mut nonnegative_rows, 2)?;
                checked_accumulate(
                    &mut coefficients,
                    checked_product(
                        relation
                            .row()
                            .terms()
                            .len()
                            .checked_add(1)
                            .ok_or(ConvexSolveError::MemoryEstimateOverflow)?,
                        2,
                    )?,
                )?;
                2
            }
            CanonicalSoftRelation::LinearBound(relation) => {
                let sides = usize::from(relation.lower().is_some())
                    + usize::from(relation.upper().is_some());
                checked_accumulate(&mut rows, sides)?;
                checked_accumulate(&mut nonnegative_rows, sides)?;
                checked_accumulate(
                    &mut coefficients,
                    checked_product(
                        relation
                            .row()
                            .terms()
                            .len()
                            .checked_add(1)
                            .ok_or(ConvexSolveError::MemoryEstimateOverflow)?,
                        sides,
                    )?,
                )?;
                sides
            }
            CanonicalSoftRelation::SecondOrderCone(relation) => {
                let block_rows = relation
                    .lhs()
                    .len()
                    .checked_add(1)
                    .ok_or(ConvexSolveError::MemoryEstimateOverflow)?;
                checked_accumulate(&mut rows, block_rows)?;
                checked_accumulate(
                    &mut coefficients,
                    relation
                        .rhs()
                        .terms()
                        .len()
                        .checked_add(1)
                        .ok_or(ConvexSolveError::MemoryEstimateOverflow)?,
                )?;
                for expression in relation.lhs() {
                    checked_accumulate(&mut coefficients, expression.terms().len())?;
                }
                checked_accumulate(&mut second_order_blocks, 1)?;
                block_rows
            }
        };
        if matches!(
            objective.loss(),
            SoftLoss::SquaredL2 | SoftLoss::Huber { .. }
        ) {
            checked_accumulate(&mut coefficients, 1)?;
        }
        if matches!(objective.loss(), SoftLoss::Huber { .. }) {
            checked_accumulate(&mut variables, 2)?;
            checked_accumulate(&mut rows, 3)?;
            checked_accumulate(&mut nonnegative_rows, 3)?;
            checked_accumulate(&mut coefficients, 5)?;
        }
        checked_accumulate(
            &mut provenance_bytes,
            checked_product(
                provenance_storage_bytes(objective.provenance())?,
                2 * (relation_rows + 5),
            )?,
        )?;
    }

    let cone_blocks = usize::from(!problem.equalities().is_empty())
        .checked_add(usize::from(nonnegative_rows != 0))
        .and_then(|value| value.checked_add(second_order_blocks))
        .ok_or(ConvexSolveError::MemoryEstimateOverflow)?;
    let scalar_and_index_bytes = std::mem::size_of::<f64>() + std::mem::size_of::<usize>();
    let sparse_bytes = checked_product(coefficients, scalar_and_index_bytes)?;
    let vector_entries = variables
        .checked_mul(12)
        .and_then(|value| value.checked_add(rows.checked_mul(14)?))
        .and_then(|value| value.checked_add(cone_blocks.checked_mul(4)?))
        .ok_or(ConvexSolveError::MemoryEstimateOverflow)?;
    let vector_bytes = checked_product(vector_entries, scalar_and_index_bytes)?;
    let geo_owned = problem
        .memory_estimate()
        .numeric_bytes
        .checked_add(provenance_bytes)
        .and_then(|value| value.checked_add(sparse_bytes))
        .and_then(|value| value.checked_add(vector_bytes))
        .and_then(|value| value.checked_mul(GEO_OWNED_STORAGE_COPIES))
        .ok_or(ConvexSolveError::MemoryEstimateOverflow)?;

    // QDLDL fill depends on sparsity ordering. Bound it before dispatch by the
    // dense lower triangle of the full primal/constraint KKT dimension, plus
    // independent symbolic/numeric/work-vector copies.
    let kkt_dimension = variables
        .checked_add(rows)
        .ok_or(ConvexSolveError::MemoryEstimateOverflow)?;
    let dense_fill_entries = kkt_dimension
        .checked_mul(
            kkt_dimension
                .checked_add(1)
                .ok_or(ConvexSolveError::MemoryEstimateOverflow)?,
        )
        .and_then(|value| value.checked_div(2))
        .ok_or(ConvexSolveError::MemoryEstimateOverflow)?;
    let backend_fill = dense_fill_entries
        .checked_mul(std::mem::size_of::<f64>() + 2 * std::mem::size_of::<usize>())
        .and_then(|value| value.checked_mul(BACKEND_DENSE_FILL_COPIES))
        .ok_or(ConvexSolveError::MemoryEstimateOverflow)?;
    let estimated_peak_bytes = geo_owned
        .checked_add(backend_fill)
        .ok_or(ConvexSolveError::MemoryEstimateOverflow)?;
    Ok(MemoryPlan {
        variables,
        rows,
        coefficients,
        cone_blocks,
        estimated_peak_bytes,
    })
}

fn provenance_storage_bytes(provenance: &SemanticProvenance) -> Result<usize, ConvexSolveError> {
    std::mem::size_of::<SemanticProvenance>()
        .checked_add(provenance.source().path().len())
        .and_then(|value| value.checked_add(provenance.original_units().len()))
        .and_then(|value| value.checked_add(provenance.field_path().len()))
        .and_then(|value| value.checked_add(provenance.constraint_group().map_or(0, str::len)))
        .ok_or(ConvexSolveError::MemoryEstimateOverflow)
}

fn checked_accumulate(target: &mut usize, amount: usize) -> Result<(), ConvexSolveError> {
    *target = target
        .checked_add(amount)
        .ok_or(ConvexSolveError::MemoryEstimateOverflow)?;
    Ok(())
}

fn checked_product(left: usize, right: usize) -> Result<usize, ConvexSolveError> {
    left.checked_mul(right)
        .ok_or(ConvexSolveError::MemoryEstimateOverflow)
}

fn evaluate_affine(expression: &AffineExpression, x: &[f64]) -> Result<f64, ConvexSolveError> {
    let mut value = expression.constant();
    for term in expression.terms() {
        value = term.coefficient().mul_add(x[term.variable()], value);
    }
    finite_value(value, "affine evaluation")
}

fn evaluate_affine_and_scale(
    expression: &AffineExpression,
    x: &[f64],
) -> Result<(f64, f64), ConvexSolveError> {
    let mut value = expression.constant();
    let mut scale = expression.constant().abs();
    for term in expression.terms() {
        let contribution = term.coefficient() * x[term.variable()];
        value = finite_value(value + contribution, "affine evaluation")?;
        scale = checked_sum(scale, contribution.abs(), "affine review scale")?;
    }
    Ok((value, scale))
}

fn evaluate_slack(expression: &SlackExpression, x: &[f64]) -> Result<f64, ConvexSolveError> {
    let mut value = expression.constant;
    for &(variable, coefficient) in &expression.terms {
        value = coefficient.mul_add(x[variable], value);
    }
    finite_value(value, "compiled slack evaluation")
}

fn objective_value_and_scale(
    p: &[f64],
    q: &[f64],
    x: &[f64],
) -> Result<(f64, f64), ConvexSolveError> {
    let mut value = 0.0_f64;
    let mut scale = 0.0_f64;
    for ((quadratic, linear), variable) in p.iter().zip(q).zip(x) {
        let contribution = 0.5 * quadratic * variable * variable + linear * variable;
        value = finite_value(value + contribution, "objective evaluation")?;
        scale = checked_sum(scale, contribution.abs(), "compiled objective scale")?;
    }
    Ok((value, scale))
}

fn half_quadratic_value(p: &[f64], x: &[f64]) -> Result<f64, ConvexSolveError> {
    let mut value = 0.0_f64;
    for (quadratic, variable) in p.iter().zip(x) {
        value = finite_value(
            value + 0.5 * quadratic * variable * variable,
            "dual quadratic objective",
        )?;
    }
    Ok(value)
}

fn stable_norm(values: &[f64]) -> Result<f64, ConvexSolveError> {
    require_finite(values, "norm input")?;
    let scale = infinity_norm(values)?;
    if scale == 0.0 {
        return Ok(0.0);
    }
    let scaled = values
        .iter()
        .map(|value| (value / scale).powi(2))
        .sum::<f64>()
        .sqrt();
    finite_value(scale * scaled, "stable norm")
}

fn stable_norm_iter(
    values: impl IntoIterator<Item = Result<f64, ConvexSolveError>>,
) -> Result<f64, ConvexSolveError> {
    let mut scale = 0.0_f64;
    let mut sum_squares = 1.0_f64;
    for value in values {
        let absolute = value?.abs();
        if absolute == 0.0 {
            continue;
        }
        if scale < absolute {
            let ratio = scale / absolute;
            sum_squares = 1.0 + sum_squares * ratio * ratio;
            scale = absolute;
        } else {
            let ratio = absolute / scale;
            sum_squares += ratio * ratio;
        }
    }
    if scale == 0.0 {
        Ok(0.0)
    } else {
        finite_value(scale * sum_squares.sqrt(), "stable iterator norm")
    }
}

fn homogeneous_ratio(residual: f64, scale: f64) -> f64 {
    if residual == 0.0 {
        0.0
    } else if scale > 0.0 && scale.is_finite() {
        residual / scale
    } else {
        f64::INFINITY
    }
}

fn checked_sum(left: f64, right: f64, field: &'static str) -> Result<f64, ConvexSolveError> {
    finite_value(left + right, field)
}

fn infinity_norm(values: &[f64]) -> Result<f64, ConvexSolveError> {
    require_finite(values, "infinity norm input")?;
    Ok(values.iter().map(|value| value.abs()).fold(0.0, f64::max))
}

fn dot(left: &[f64], right: &[f64]) -> Result<f64, ConvexSolveError> {
    if left.len() != right.len() {
        return Err(ConvexSolveError::NonFiniteCompilation {
            field: "dot-product dimensions",
        });
    }
    let value = left
        .iter()
        .zip(right)
        .map(|(left, right)| left * right)
        .sum::<f64>();
    finite_value(value, "dot product")
}

fn finite_value(value: f64, field: &'static str) -> Result<f64, ConvexSolveError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(ConvexSolveError::NonFiniteCompilation { field })
    }
}

fn require_finite(values: &[f64], field: &'static str) -> Result<(), ConvexSolveError> {
    if values.iter().all(|value| value.is_finite()) {
        Ok(())
    } else {
        Err(ConvexSolveError::NonFiniteCompilation { field })
    }
}

fn check_review(reason: &'static str, value: f64, tolerance: f64) -> Result<(), ConvexSolveError> {
    if value.is_finite() && value <= tolerance {
        Ok(())
    } else {
        Err(ConvexSolveError::SolutionReviewFailed {
            reason,
            value,
            tolerance,
        })
    }
}

fn constraint_diagnostic(
    kind: ConvexConstraintKind,
    provenance: &SemanticProvenance,
    original_residual: f64,
    normalized_residual: f64,
) -> Result<ConvexConstraintDiagnostics, ConvexSolveError> {
    Ok(ConvexConstraintDiagnostics {
        kind,
        provenance: try_clone_provenance(provenance)?,
        original_residual,
        normalized_residual,
    })
}

fn try_clone_provenance(
    provenance: &SemanticProvenance,
) -> Result<SemanticProvenance, ConvexSolveError> {
    provenance.try_clone_for_canonical().map_err(|_| {
        allocation(
            "semantic provenance",
            provenance_storage_bytes(provenance).unwrap_or(usize::MAX),
        )
    })
}

fn try_reserve<T>(
    values: &mut Vec<T>,
    count: usize,
    storage: &'static str,
) -> Result<(), ConvexSolveError> {
    values
        .try_reserve_exact(count)
        .map_err(|_| allocation(storage, count))
}

fn try_copy<T: Clone>(values: &[T], storage: &'static str) -> Result<Vec<T>, ConvexSolveError> {
    let mut copy = Vec::new();
    try_reserve(&mut copy, values.len(), storage)?;
    copy.extend_from_slice(values);
    Ok(copy)
}

const fn allocation(storage: &'static str, requested: usize) -> ConvexSolveError {
    ConvexSolveError::AllocationFailed { storage, requested }
}

const fn map_status(status: SolverStatus) -> ConvexBackendStatus {
    match status {
        SolverStatus::Solved => ConvexBackendStatus::Solved,
        SolverStatus::PrimalInfeasible => ConvexBackendStatus::PrimalInfeasible,
        SolverStatus::DualInfeasible => ConvexBackendStatus::DualInfeasible,
        SolverStatus::AlmostSolved => ConvexBackendStatus::ReducedAccuracy,
        SolverStatus::AlmostPrimalInfeasible | SolverStatus::AlmostDualInfeasible => {
            ConvexBackendStatus::ReducedAccuracyInfeasible
        }
        SolverStatus::MaxIterations => ConvexBackendStatus::MaximumIterations,
        SolverStatus::MaxTime => ConvexBackendStatus::MaximumTime,
        SolverStatus::NumericalError => ConvexBackendStatus::NumericalError,
        SolverStatus::InsufficientProgress => ConvexBackendStatus::InsufficientProgress,
        SolverStatus::CallbackTerminated => ConvexBackendStatus::CallbackTerminated,
        SolverStatus::Unsolved => ConvexBackendStatus::Unsolved,
    }
}

fn require_solved_status(
    status: ConvexBackendStatus,
    iterations: u32,
) -> Result<(), ConvexSolveError> {
    if status == ConvexBackendStatus::Solved {
        Ok(())
    } else {
        Err(ConvexSolveError::UnacceptedStatus { status, iterations })
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::float_cmp)]
mod tests {
    use super::*;
    use crate::problem_ir::{
        AffineTerm, CanonicalEquality, CanonicalLinearBound, CanonicalSoftObjective, ObservationId,
        SourceLocation, VariableBlock,
    };

    type TestResult = Result<(), Box<dyn Error>>;

    fn test_options() -> Result<ConvexSolveOptions, ConvexSolverConfigurationError> {
        ConvexSolveOptions::try_new(
            1.0e-9,
            NonZeroU32::new(200).expect("nonzero test iteration count"),
            Some(5.0),
            NonZeroUsize::new(64 * 1024 * 1024).expect("nonzero test memory limit"),
        )
    }

    fn test_provenance(
        identifier: u64,
        payload: usize,
    ) -> Result<SemanticProvenance, Box<dyn Error>> {
        let path = "p".repeat(payload.max(1));
        Ok(SemanticProvenance::try_new(
            ObservationId::new(identifier),
            SourceLocation::try_new(path, NonZeroUsize::new(1).expect("nonzero source line"))?,
            "m".to_owned(),
            format!("test[{identifier}]"),
            Some("convex-private".to_owned()),
        )?)
    }

    fn one_variable_block(
        count: usize,
    ) -> Result<VariableBlock, crate::problem_ir::ProblemIrError> {
        VariableBlock::try_new(
            "x".to_owned(),
            NonZeroUsize::new(count).expect("positive test variable count"),
        )
    }

    fn row(coefficient: f64) -> Result<AffineExpression, crate::problem_ir::ProblemIrError> {
        AffineExpression::try_new([AffineTerm::try_new(0, coefficient)?], 0.0)
    }

    fn equality_problem(
        coefficient: f64,
        payload: usize,
    ) -> Result<CanonicalProblem, Box<dyn Error>> {
        Ok(CanonicalProblem::try_from_linear_parts_and_objectives(
            [one_variable_block(1)?],
            vec![CanonicalEquality::from_parts(
                row(coefficient)?,
                0.0,
                test_provenance(1, payload)?,
            )],
            Vec::new(),
            Vec::new(),
        )?)
    }

    #[test]
    fn certificate_stationarity_is_componentwise_and_scale_invariant() -> TestResult {
        let feasible_scaled = CanonicalProblem::try_from_linear_parts_and_objectives(
            [one_variable_block(1)?],
            Vec::new(),
            vec![CanonicalLinearBound::from_parts(
                row(1.0e-12)?,
                None,
                Some(-1.0),
                test_provenance(2, 1)?,
            )],
            Vec::new(),
        )?;
        let compiled = Compiler::new(&feasible_scaled)?.compile(&feasible_scaled)?;
        let matrices = build_matrices(&compiled)?;
        assert!(matches!(
            review_certificate(&compiled, &matrices, &[1.0], test_options()?, 0),
            Err(ConvexSolveError::InvalidInfeasibilityCertificate {
                reason: "certificate stationarity residual is too large"
            })
        ));

        for scale in [1.0e-12, 1.0, 1.0e12] {
            let compiled = CompiledProblem {
                original_variables: 1,
                variables: 1,
                p_diagonal: vec![0.0],
                q: vec![0.0],
                blocks: vec![ConeBlock {
                    rows: vec![
                        SlackExpression {
                            terms: vec![(0, scale)],
                            constant: -scale,
                            kind: ConvexConstraintKind::LowerBound,
                            provenance: test_provenance(3, 1)?,
                        },
                        SlackExpression {
                            terms: vec![(0, -scale)],
                            constant: 0.0,
                            kind: ConvexConstraintKind::UpperBound,
                            provenance: test_provenance(4, 1)?,
                        },
                    ],
                    kind: ConeKind::Nonnegative,
                }],
            };
            let matrices = build_matrices(&compiled)?;
            let certificate =
                review_certificate(&compiled, &matrices, &[1.0, 1.0], test_options()?, 0)?;
            assert_eq!(
                certificate
                    .diagnostics
                    .normalized_stationarity_residual_infinity,
                0.0
            );
            assert!(certificate.diagnostics.separating_value < 0.0);
        }
        Ok(())
    }

    #[test]
    fn hard_relation_review_decision_is_invariant_to_nonzero_row_scaling() -> TestResult {
        for scale in [1.0e-12, 1.0, 1.0e12] {
            let problem = equality_problem(scale, 1)?;
            let accepted = review_original_relations(&problem, &[0.0], 1.0e-9)?;
            assert_eq!(accepted[0].normalized_residual, 0.0);
            assert!(matches!(
                review_original_relations(&problem, &[1.0e-5], 1.0e-9),
                Err(ConvexSolveError::SolutionReviewFailed {
                    reason: "hard equality original-unit review",
                    ..
                })
            ));
        }
        Ok(())
    }

    #[test]
    fn semantic_objective_rejects_compiler_perturbation() -> TestResult {
        let equality = CanonicalEquality::from_parts(row(1.0)?, 0.0, test_provenance(5, 1)?);
        let problem = CanonicalProblem::try_from_linear_parts_and_objectives(
            [one_variable_block(1)?],
            Vec::new(),
            Vec::new(),
            vec![CanonicalSoftObjective::from_parts(
                CanonicalSoftRelation::Equality(equality),
                2.0,
                SoftLoss::SquaredL2,
            )],
        )?;
        let mut compiled = Compiler::new(&problem)?.compile(&problem)?;
        let x = [2.0, 1.0];
        let (semantic, semantic_scale) = semantic_objective(&problem, &x[..1])?;
        let (compiled_value, compiled_scale) =
            objective_value_and_scale(&compiled.p_diagonal, &compiled.q, &x)?;
        assert_eq!(semantic, 1.0);
        assert_eq!(compiled_value, 1.0);
        compiled.p_diagonal[1] = 4.0;
        let (perturbed, perturbed_scale) =
            objective_value_and_scale(&compiled.p_diagonal, &compiled.q, &x)?;
        assert!(
            check_review(
                "semantic/compiled objective reconstruction",
                homogeneous_ratio(
                    (semantic - perturbed).abs(),
                    semantic_scale.max(compiled_scale).max(perturbed_scale),
                ),
                test_options()?.tolerance(),
            )
            .is_err()
        );
        Ok(())
    }

    #[test]
    fn execution_limit_and_memory_preflight_cover_metadata_and_dense_fill() -> TestResult {
        let problem = equality_problem(1.0, 4096)?;
        let plan = preflight_memory(&problem)?;
        assert!(plan.estimated_peak_bytes > 4096 * GEO_OWNED_STORAGE_COPIES);
        let kkt_dimension = plan.variables + plan.rows;
        let dense_entries = kkt_dimension * (kkt_dimension + 1) / 2;
        let dense_fill_bytes = dense_entries
            * (std::mem::size_of::<f64>() + 2 * std::mem::size_of::<usize>())
            * BACKEND_DENSE_FILL_COPIES;
        assert!(plan.estimated_peak_bytes >= dense_fill_bytes);
        let execution = ExecutionOptions::new(true, None, Some(NonZeroUsize::MIN));
        assert_eq!(effective_memory_limit(test_options()?, execution), 1);
        let metadata_limit = ConvexSolveOptions::try_new(
            1.0e-9,
            NonZeroU32::MIN,
            None,
            NonZeroUsize::new(4096).ok_or("metadata memory limit")?,
        )?;
        assert!(matches!(
            try_solve_canonical(&problem, metadata_limit),
            Err(ConvexSolveError::MemoryLimitExceeded { .. })
        ));

        let sparse_problem = CanonicalProblem::try_from_linear_parts_and_objectives(
            [one_variable_block(128)?],
            vec![CanonicalEquality::from_parts(
                row(1.0)?,
                0.0,
                test_provenance(6, 1)?,
            )],
            Vec::new(),
            Vec::new(),
        )?;
        let sparse_plan = preflight_memory(&sparse_problem)?;
        assert_eq!(sparse_plan.variables, 128);
        assert_eq!(sparse_plan.coefficients, 1);
        let sparse_kkt_dimension = sparse_plan.variables + sparse_plan.rows;
        let sparse_dense_entries = sparse_kkt_dimension * (sparse_kkt_dimension + 1) / 2;
        let sparse_fill_bytes = sparse_dense_entries
            * (std::mem::size_of::<f64>() + 2 * std::mem::size_of::<usize>())
            * BACKEND_DENSE_FILL_COPIES;
        assert!(sparse_plan.estimated_peak_bytes >= sparse_fill_bytes);
        Ok(())
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn settings_snapshot_and_status_routing_are_complete() -> TestResult {
        let options = test_options()?;
        let backend = settings(options)?;
        let recorded = settings_diagnostics(options);
        assert_eq!(backend.max_iter, recorded.maximum_iterations);
        assert_eq!(
            backend.time_limit,
            recorded.time_limit_seconds.unwrap_or(f64::INFINITY)
        );
        assert_eq!(backend.max_step_fraction, recorded.maximum_step_fraction);
        assert_eq!(backend.tol_gap_abs, recorded.absolute_gap_tolerance);
        assert_eq!(backend.tol_gap_rel, recorded.relative_gap_tolerance);
        assert_eq!(backend.tol_feas, recorded.feasibility_tolerance);
        assert_eq!(
            backend.tol_infeas_abs,
            recorded.absolute_infeasibility_tolerance
        );
        assert_eq!(
            backend.tol_infeas_rel,
            recorded.relative_infeasibility_tolerance
        );
        assert_eq!(backend.tol_ktratio, recorded.kappa_tau_tolerance);
        assert_eq!(
            backend.reduced_tol_gap_abs,
            recorded.reduced_absolute_gap_tolerance
        );
        assert_eq!(
            backend.reduced_tol_gap_rel,
            recorded.reduced_relative_gap_tolerance
        );
        assert_eq!(
            backend.reduced_tol_feas,
            recorded.reduced_feasibility_tolerance
        );
        assert_eq!(
            backend.reduced_tol_infeas_abs,
            recorded.reduced_absolute_infeasibility_tolerance
        );
        assert_eq!(
            backend.reduced_tol_infeas_rel,
            recorded.reduced_relative_infeasibility_tolerance
        );
        assert_eq!(
            backend.reduced_tol_ktratio,
            recorded.reduced_kappa_tau_tolerance
        );
        assert_eq!(
            backend.equilibrate_enable,
            recorded.equilibration == ConvexSettingState::Enabled
        );
        assert_eq!(backend.equilibrate_max_iter, recorded.equilibration_steps);
        assert_eq!(
            backend.equilibrate_min_scaling,
            recorded.equilibration_minimum_scaling
        );
        assert_eq!(
            backend.equilibrate_max_scaling,
            recorded.equilibration_maximum_scaling
        );
        assert_eq!(
            backend.linesearch_backtrack_step,
            recorded.line_search_backtrack_step
        );
        assert_eq!(
            backend.min_switch_step_length,
            recorded.minimum_switch_step_length
        );
        assert_eq!(
            backend.min_terminate_step_length,
            recorded.minimum_terminate_step_length
        );
        assert_eq!(backend.max_threads, recorded.thread_count);
        assert_eq!(
            backend.direct_kkt_solver,
            recorded.direct_kkt_solver == ConvexSettingState::Enabled
        );
        assert_eq!(backend.direct_solve_method, recorded.direct_solver);
        assert!(!backend.static_regularization_enable);
        assert_eq!(
            backend.static_regularization_constant,
            recorded.static_regularization_constant
        );
        assert_eq!(
            backend.static_regularization_proportional,
            recorded.static_regularization_proportional
        );
        assert!(!backend.dynamic_regularization_enable);
        assert_eq!(
            backend.dynamic_regularization_eps,
            recorded.dynamic_regularization_epsilon
        );
        assert_eq!(
            backend.dynamic_regularization_delta,
            recorded.dynamic_regularization_delta
        );
        assert_eq!(
            backend.iterative_refinement_enable,
            recorded.iterative_refinement == ConvexSettingState::Enabled
        );
        assert_eq!(
            backend.iterative_refinement_reltol,
            recorded.iterative_refinement_relative_tolerance
        );
        assert_eq!(
            backend.iterative_refinement_abstol,
            recorded.iterative_refinement_absolute_tolerance
        );
        assert_eq!(
            backend.iterative_refinement_max_iter,
            recorded.iterative_refinement_steps
        );
        assert_eq!(
            backend.iterative_refinement_stop_ratio,
            recorded.iterative_refinement_stop_ratio
        );
        assert!(!backend.presolve_enable);
        assert!(!backend.input_sparse_dropzeros);
        assert_eq!(recorded.independent_review_tolerance, options.tolerance());
        assert_eq!(recorded.independent_review_tolerance_multiplier, 1);

        assert_eq!(
            map_status(SolverStatus::Solved),
            ConvexBackendStatus::Solved
        );
        assert_eq!(
            map_status(SolverStatus::AlmostSolved),
            ConvexBackendStatus::ReducedAccuracy
        );
        assert_eq!(
            map_status(SolverStatus::MaxIterations),
            ConvexBackendStatus::MaximumIterations
        );
        assert_eq!(
            map_status(SolverStatus::MaxTime),
            ConvexBackendStatus::MaximumTime
        );
        assert_eq!(
            map_status(SolverStatus::NumericalError),
            ConvexBackendStatus::NumericalError
        );
        assert_eq!(
            map_status(SolverStatus::InsufficientProgress),
            ConvexBackendStatus::InsufficientProgress
        );
        assert_eq!(
            map_status(SolverStatus::CallbackTerminated),
            ConvexBackendStatus::CallbackTerminated
        );
        assert_eq!(
            map_status(SolverStatus::Unsolved),
            ConvexBackendStatus::Unsolved
        );
        assert!(matches!(
            require_solved_status(ConvexBackendStatus::ReducedAccuracy, 7),
            Err(ConvexSolveError::UnacceptedStatus {
                status: ConvexBackendStatus::ReducedAccuracy,
                iterations: 7,
            })
        ));
        Ok(())
    }
}
