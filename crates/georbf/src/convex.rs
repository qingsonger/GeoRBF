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
const REVIEW_FACTOR: f64 = 64.0;
const ITERATIVE_REFINEMENT_STEPS: u32 = 10;
const EQUILIBRATION_STEPS: u32 = 10;
const PEAK_STORAGE_COPIES: usize = 16;
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
    /// Explicit iteration limit.
    pub maximum_iterations: u32,
    /// Optional time limit in seconds.
    pub time_limit_seconds: Option<f64>,
    /// Explicit serial backend thread count.
    pub thread_count: u32,
    /// Whether backend equilibration was enabled.
    pub equilibration: ConvexSettingState,
    /// Fixed maximum equilibration passes.
    pub equilibration_steps: u32,
    /// Whether backend iterative refinement was enabled.
    pub iterative_refinement: ConvexSettingState,
    /// Fixed maximum refinement steps.
    pub iterative_refinement_steps: u32,
    /// Presolve is always disabled by this adapter.
    pub presolve: ConvexSettingState,
    /// Static KKT regularization is always disabled by this adapter.
    pub static_regularization: ConvexSettingState,
    /// Dynamic KKT regularization is always disabled by this adapter.
    pub dynamic_regularization: ConvexSettingState,
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
    /// Independently recomputed primal objective.
    pub original_objective: f64,
    /// Backend-reported primal objective.
    pub backend_primal_objective: f64,
    /// Backend-reported dual objective.
    pub backend_dual_objective: f64,
    /// Absolute backend duality gap.
    pub duality_gap: f64,
}

/// Evidence for an independently accepted primal-infeasibility certificate.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct ConvexCertificateDiagnostics {
    /// Infinity norm of `A^T z` after certificate normalization.
    pub stationarity_residual_infinity: f64,
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
    /// Explicit peak-working-set limit.
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
            self.second_order_blocks.push(ConeBlock {
                rows,
                kind: ConeKind::SecondOrder,
            });
        }
        for objective in problem.soft_objectives() {
            let violation = self.auxiliary()?;
            self.push_nonnegative(auxiliary_nonnegative(violation, objective.provenance()))?;
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
                    ))?;
                    self.push_nonnegative(auxiliary_nonnegative(linear, objective.provenance()))?;
                    self.push_nonnegative(SlackExpression {
                        terms: vec![(violation, -1.0), (quadratic, 1.0), (linear, 1.0)],
                        constant: 0.0,
                        kind: ConvexConstraintKind::Epigraph,
                        provenance: objective.provenance().clone(),
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
    let mut progress = ProgressTracker::try_new(
        control,
        ExecutionOperation::ConvexSolve,
        execution,
        SOLVE_PROGRESS_STEPS,
    )?;
    let compiled = Compiler::new(problem)?.compile(problem)?;
    let estimated = estimate_peak_memory(problem, &compiled)?;
    if estimated > options.memory_limit_bytes().get() {
        return Err(ConvexSolveError::MemoryLimitExceeded {
            estimated_peak_bytes: estimated,
            limit_bytes: options.memory_limit_bytes().get(),
        });
    }
    progress.advance(ExecutionStage::MemoryReview)?;
    let matrices = build_matrices(&compiled)?;
    let settings = settings(options);
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
    if status != ConvexBackendStatus::Solved {
        return Err(ConvexSolveError::UnacceptedStatus {
            status,
            iterations: solver.solution.iterations,
        });
    }
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
        estimated_peak_memory_bytes: estimated,
        memory_limit_bytes: options.memory_limit_bytes().get(),
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
    let mut column_counts = vec![0_usize; compiled.variables];
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
    let mut a_rowval = vec![0_usize; constraint_nonzeros];
    let mut a_values = vec![0.0; constraint_nonzeros];
    let mut cursor = a_colptr[..compiled.variables].to_vec();
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

fn settings(options: ConvexSolveOptions) -> DefaultSettings<f64> {
    DefaultSettings {
        verbose: false,
        max_iter: options.maximum_iterations().get(),
        time_limit: options.time_limit_seconds().unwrap_or(f64::INFINITY),
        max_threads: 1,
        tol_gap_abs: options.tolerance(),
        tol_gap_rel: options.tolerance(),
        tol_feas: options.tolerance(),
        tol_infeas_abs: options.tolerance(),
        tol_infeas_rel: options.tolerance(),
        equilibrate_enable: true,
        equilibrate_max_iter: EQUILIBRATION_STEPS,
        direct_solve_method: "qdldl".to_owned(),
        static_regularization_enable: false,
        dynamic_regularization_enable: false,
        iterative_refinement_enable: true,
        iterative_refinement_max_iter: ITERATIVE_REFINEMENT_STEPS,
        presolve_enable: false,
        ..DefaultSettings::default()
    }
}

const fn settings_diagnostics(options: ConvexSolveOptions) -> ConvexSettingsDiagnostics {
    ConvexSettingsDiagnostics {
        feasibility_tolerance: options.tolerance(),
        absolute_gap_tolerance: options.tolerance(),
        relative_gap_tolerance: options.tolerance(),
        absolute_infeasibility_tolerance: options.tolerance(),
        relative_infeasibility_tolerance: options.tolerance(),
        maximum_iterations: options.maximum_iterations().get(),
        time_limit_seconds: options.time_limit_seconds(),
        thread_count: 1,
        equilibration: ConvexSettingState::Enabled,
        equilibration_steps: EQUILIBRATION_STEPS,
        iterative_refinement: ConvexSettingState::Enabled,
        iterative_refinement_steps: ITERATIVE_REFINEMENT_STEPS,
        presolve: ConvexSettingState::Disabled,
        static_regularization: ConvexSettingState::Disabled,
        dynamic_regularization: ConvexSettingState::Disabled,
    }
}

struct SolutionReview {
    kkt: ConvexKktDiagnostics,
    constraints: Vec<ConvexConstraintDiagnostics>,
}

#[allow(clippy::too_many_arguments)]
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
    let original_objective = objective_value(&compiled.p_diagonal, &compiled.q, x)?;
    let objective_scale = 1.0_f64
        .max(original_objective.abs())
        .max(backend_primal.abs());
    let objective_difference = (original_objective - backend_primal).abs() / objective_scale;
    let tolerance = REVIEW_FACTOR * options.tolerance();
    check_review("objective reconstruction", objective_difference, tolerance)?;
    let mut primal_inf = 0.0_f64;
    let mut primal_scale = 1.0_f64;
    let rows = compiled.rows().collect::<Vec<_>>();
    for (row_index, row) in rows.iter().enumerate() {
        let expression = evaluate_slack(row, x)?;
        primal_inf = primal_inf.max((expression - s[row_index]).abs());
        primal_scale = primal_scale
            .max(expression.abs())
            .max(s[row_index].abs())
            .max(matrices.rhs[row_index].abs());
    }
    let normalized_primal = primal_inf / primal_scale;
    check_review("primal residual review", normalized_primal, tolerance)?;
    let mut stationarity = compiled.q.clone();
    for (index, value) in x.iter().copied().enumerate() {
        stationarity[index] = compiled.p_diagonal[index].mul_add(value, stationarity[index]);
    }
    let mut dual_scale = stationarity
        .iter()
        .map(|value| value.abs())
        .fold(1.0_f64, f64::max);
    for (row_index, row) in rows.iter().enumerate() {
        for &(column, coefficient) in &row.terms {
            let contribution = -coefficient * z[row_index];
            stationarity[column] += contribution;
            dual_scale = dual_scale.max(contribution.abs());
        }
    }
    let dual_inf = infinity_norm(&stationarity)?;
    let normalized_dual = dual_inf / dual_scale;
    check_review("dual stationarity review", normalized_dual, tolerance)?;
    let (primal_cone, dual_cone) = cone_reviews(compiled, s, z)?;
    check_review("primal cone review", primal_cone, tolerance)?;
    check_review("dual cone review", dual_cone, tolerance)?;
    let complementarity = dot(s, z)?.abs();
    let complementarity_normalized = complementarity
        / 1.0_f64
            .max(original_objective.abs())
            .max(backend_dual.abs());
    check_review(
        "complementarity review",
        complementarity_normalized,
        tolerance,
    )?;
    let duality_gap = (backend_primal - backend_dual).abs();
    check_review(
        "duality-gap review",
        duality_gap / objective_scale,
        tolerance,
    )?;
    let original_x = &x[..compiled.original_variables];
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
            backend_primal_objective: backend_primal,
            backend_dual_objective: backend_dual,
            duality_gap,
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
        let actual = evaluate_affine(relation.row(), x)?;
        let residual = (actual - relation.rhs()).abs();
        let normalized = residual / 1.0_f64.max(actual.abs()).max(relation.rhs().abs());
        check_review("hard equality original-unit review", normalized, tolerance)?;
        diagnostics.push(constraint_diagnostic(
            ConvexConstraintKind::Equality,
            relation.provenance(),
            residual,
            normalized,
        ));
    }
    for relation in problem.linear_bounds() {
        let actual = evaluate_affine(relation.row(), x)?;
        if let Some(lower) = relation.lower() {
            let violation = (lower - actual).max(0.0);
            let normalized = violation / 1.0_f64.max(actual.abs()).max(lower.abs());
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
            ));
        }
        if let Some(upper) = relation.upper() {
            let violation = (actual - upper).max(0.0);
            let normalized = violation / 1.0_f64.max(actual.abs()).max(upper.abs());
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
            ));
        }
    }
    for relation in problem.second_order_cones() {
        let rhs = evaluate_affine(relation.rhs(), x)?;
        let lhs_norm = stable_norm(
            relation
                .lhs()
                .iter()
                .map(|expression| evaluate_affine(expression, x))
                .collect::<Result<Vec<_>, _>>()?
                .as_slice(),
        )?;
        let violation = (lhs_norm - rhs).max(0.0);
        let normalized = violation / 1.0_f64.max(lhs_norm).max(rhs.abs());
        check_review("hard cone original-unit review", normalized, tolerance)?;
        diagnostics.push(constraint_diagnostic(
            ConvexConstraintKind::SecondOrderCone,
            relation.provenance(),
            violation,
            normalized,
        ));
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
        ));
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
    let mut stationarity = vec![0.0; compiled.variables];
    for (row_index, row) in compiled.rows().enumerate() {
        for &(column, coefficient) in &row.terms {
            stationarity[column] =
                (-coefficient).mul_add(normalized[row_index], stationarity[column]);
        }
    }
    let stationarity_inf = infinity_norm(&stationarity)?;
    let (_, dual_cone_violation) =
        cone_reviews(compiled, &vec![0.0; normalized.len()], &normalized)?;
    let separating_value = dot(&matrices.rhs, &normalized)?;
    let separator_scale = matrices
        .rhs
        .iter()
        .zip(&normalized)
        .map(|(rhs, dual)| (rhs * dual).abs())
        .sum::<f64>()
        .max(1.0);
    let tolerance = REVIEW_FACTOR * options.tolerance();
    if stationarity_inf > tolerance {
        return Err(ConvexSolveError::InvalidInfeasibilityCertificate {
            reason: "certificate stationarity residual is too large",
        });
    }
    if dual_cone_violation > tolerance {
        return Err(ConvexSolveError::InvalidInfeasibilityCertificate {
            reason: "certificate leaves the dual product cone",
        });
    }
    if separating_value >= -options.tolerance() * separator_scale {
        return Err(ConvexSolveError::InvalidInfeasibilityCertificate {
            reason: "certificate lacks a strict scale-aware separator",
        });
    }
    let mut rows = Vec::new();
    try_reserve(&mut rows, normalized.len(), "certificate provenance")?;
    rows.extend(
        compiled
            .rows()
            .map(|row| (row.kind, row.provenance.clone())),
    );
    Ok(ConvexInfeasibilityCertificate {
        status: ConvexBackendStatus::PrimalInfeasible,
        normalized_dual: normalized,
        rows,
        diagnostics: ConvexCertificateDiagnostics {
            stationarity_residual_infinity: stationarity_inf,
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
                primal_max = primal_max.max(infinity_norm(primal_block)?);
            }
            ConeKind::Nonnegative => {
                for value in primal_block {
                    primal_max = primal_max.max((-value).max(0.0) / value.abs().max(1.0));
                }
                for value in dual_block {
                    dual_max = dual_max.max((-value).max(0.0) / value.abs().max(1.0));
                }
            }
            ConeKind::SecondOrder => {
                let primal_norm = stable_norm(&primal_block[1..])?;
                let dual_norm = stable_norm(&dual_block[1..])?;
                primal_max = primal_max.max(
                    (primal_norm - primal_block[0]).max(0.0)
                        / 1.0_f64.max(primal_norm).max(primal_block[0].abs()),
                );
                dual_max = dual_max.max(
                    (dual_norm - dual_block[0]).max(0.0)
                        / 1.0_f64.max(dual_norm).max(dual_block[0].abs()),
                );
            }
        }
        offset = end;
    }
    Ok((primal_max, dual_max))
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
            let lhs = relation
                .lhs()
                .iter()
                .map(|expression| evaluate_affine(expression, x))
                .collect::<Result<Vec<_>, _>>()?;
            Ok((stable_norm(&lhs)? - evaluate_affine(relation.rhs(), x)?).max(0.0))
        }
    }
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
        provenance: provenance.clone(),
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
    row.terms.push((auxiliary, 1.0));
    Ok(row)
}

fn auxiliary_nonnegative(variable: usize, provenance: &SemanticProvenance) -> SlackExpression {
    SlackExpression {
        terms: vec![(variable, 1.0)],
        constant: 0.0,
        kind: ConvexConstraintKind::Epigraph,
        provenance: provenance.clone(),
    }
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

fn estimate_peak_memory(
    problem: &CanonicalProblem,
    compiled: &CompiledProblem,
) -> Result<usize, ConvexSolveError> {
    let rows = compiled.row_count();
    let coefficients = compiled.coefficient_count();
    let scalar_vectors = compiled
        .variables
        .checked_mul(10)
        .and_then(|value| value.checked_add(rows.checked_mul(12)?))
        .ok_or(ConvexSolveError::MemoryEstimateOverflow)?;
    let sparse_bytes = coefficients
        .checked_mul(std::mem::size_of::<f64>() + std::mem::size_of::<usize>())
        .and_then(|value| {
            value.checked_add((compiled.variables + 1).checked_mul(std::mem::size_of::<usize>())?)
        })
        .ok_or(ConvexSolveError::MemoryEstimateOverflow)?;
    let vector_bytes = scalar_vectors
        .checked_mul(std::mem::size_of::<f64>())
        .ok_or(ConvexSolveError::MemoryEstimateOverflow)?;
    problem
        .memory_estimate()
        .numeric_bytes
        .checked_add(sparse_bytes)
        .and_then(|value| value.checked_add(vector_bytes))
        .and_then(|value| value.checked_mul(PEAK_STORAGE_COPIES))
        .ok_or(ConvexSolveError::MemoryEstimateOverflow)
}

fn evaluate_affine(expression: &AffineExpression, x: &[f64]) -> Result<f64, ConvexSolveError> {
    let mut value = expression.constant();
    for term in expression.terms() {
        value = term.coefficient().mul_add(x[term.variable()], value);
    }
    finite_value(value, "affine evaluation")
}

fn evaluate_slack(expression: &SlackExpression, x: &[f64]) -> Result<f64, ConvexSolveError> {
    let mut value = expression.constant;
    for &(variable, coefficient) in &expression.terms {
        value = coefficient.mul_add(x[variable], value);
    }
    finite_value(value, "compiled slack evaluation")
}

fn objective_value(p: &[f64], q: &[f64], x: &[f64]) -> Result<f64, ConvexSolveError> {
    let value = p
        .iter()
        .zip(q)
        .zip(x)
        .map(|((quadratic, linear), variable)| {
            0.5 * quadratic * variable * variable + linear * variable
        })
        .sum::<f64>();
    finite_value(value, "objective evaluation")
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
) -> ConvexConstraintDiagnostics {
    ConvexConstraintDiagnostics {
        kind,
        provenance: provenance.clone(),
        original_residual,
        normalized_residual,
    }
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
