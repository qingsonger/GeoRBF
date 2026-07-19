//! Caller-owned cancellation and progress controls for long core operations.
//!
//! Controls are borrowed for one synchronous operation. `GeoRBF` never stores a
//! progress sink in a fitted model or global state, and never creates a global
//! thread pool. Progress callbacks run synchronously at deterministic work
//! boundaries and must return promptly without panicking.

use std::error::Error;
use std::fmt;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::problem_ir::ExecutionOptions;

const SERIAL_THREAD_COUNT: NonZeroUsize = NonZeroUsize::MIN;

/// Cloneable, thread-safe caller cancellation state.
///
/// Cancelling any clone cancels every clone. Cancellation is sticky and uses
/// acquire/release atomic ordering so a requesting thread can safely publish
/// the request to an operation thread. Dropping a token never cancels work.
#[derive(Clone, Default)]
#[must_use]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    /// Creates a token in the non-cancelled state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Requests cancellation for every clone of this token.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    /// Returns whether cancellation has been requested.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
}

impl fmt::Debug for CancellationToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CancellationToken")
            .field("cancelled", &self.is_cancelled())
            .finish_non_exhaustive()
    }
}

/// Long-running core operation reported by execution controls.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub enum ExecutionOperation {
    /// Symmetric hard-equality field assembly.
    FieldAssembly,
    /// Checked dense equality solving.
    DenseSolve,
    /// Canonical convex QP or SOCP solving.
    ConvexSolve,
}

impl fmt::Display for ExecutionOperation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::FieldAssembly => "field assembly",
            Self::DenseSolve => "dense solve",
            Self::ConvexSolve => "convex solve",
        })
    }
}

/// Deterministic progress or cancellation boundary within an operation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub enum ExecutionStage {
    /// The operation passed its pre-start cancellation and execution-policy checks.
    Started,
    /// One upper-triangle kernel entry was assembled.
    KernelAssembly,
    /// CPD polynomial rank and null-space evidence was constructed.
    CpdConstruction,
    /// One observation row was applied to the complete polynomial basis.
    PolynomialAssembly,
    /// Semantic equalities were compiled to the canonical numerical form.
    Canonicalization,
    /// The dense matrix passed its symmetry review.
    SymmetryReview,
    /// Projected CPD energy evidence was constructed.
    ProjectedEnergy,
    /// The solve passed its checked peak-memory policy.
    MemoryReview,
    /// RRQR and bounded-SVD rank evidence was reviewed.
    RankReview,
    /// The explicitly selected factorization was constructed.
    Factorization,
    /// The canonical QP or SOCP backend completed one indivisible solve.
    BackendSolve,
    /// A reported infeasibility certificate was independently reviewed.
    CertificateReview,
    /// One bounded iterative-refinement candidate was reviewed.
    Refinement,
    /// Original-unit residual evidence was reviewed.
    ResidualReview,
    /// The operation completed successfully.
    Completed,
}

/// One deterministic, monotonic progress observation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub struct ProgressEvent {
    operation: ExecutionOperation,
    stage: ExecutionStage,
    completed: usize,
    total: usize,
    deterministic: bool,
    thread_count: NonZeroUsize,
}

impl ProgressEvent {
    /// Returns the reporting operation.
    pub const fn operation(self) -> ExecutionOperation {
        self.operation
    }

    /// Returns the current deterministic work boundary.
    pub const fn stage(self) -> ExecutionStage {
        self.stage
    }

    /// Returns completed work units; values never decrease within an operation.
    #[must_use]
    pub const fn completed(self) -> usize {
        self.completed
    }

    /// Returns the checked maximum work-unit budget for this operation.
    ///
    /// The final completed count can be smaller when optional work, such as
    /// iterative refinement, stops early.
    #[must_use]
    pub const fn total(self) -> usize {
        self.total
    }

    /// Returns whether deterministic execution was requested.
    #[must_use]
    pub const fn deterministic(self) -> bool {
        self.deterministic
    }

    /// Returns the effective worker-thread count.
    ///
    /// The current dense implementation is serial, so successful operations
    /// report one. A larger explicit request is rejected before work begins.
    #[must_use]
    pub const fn thread_count(self) -> NonZeroUsize {
        self.thread_count
    }
}

/// Synchronous receiver for deterministic progress events.
///
/// `GeoRBF` invokes this callback without holding a core lock. Implementations
/// may request cancellation through a [`CancellationToken`], but must return
/// promptly and must not panic. Events are immutable and may be copied by the
/// sink for asynchronous presentation by an adapter. Cancellation requested
/// while handling [`ExecutionStage::Completed`] is post-completion and affects
/// only later operations that reuse the token.
pub trait ProgressSink: Send + Sync {
    /// Observes one progress event.
    fn on_progress(&self, event: ProgressEvent);
}

/// Borrowed optional controls for one synchronous core operation.
#[derive(Clone, Copy, Default)]
#[must_use]
pub struct ExecutionControl<'a> {
    cancellation: Option<&'a CancellationToken>,
    progress: Option<&'a dyn ProgressSink>,
}

impl<'a> ExecutionControl<'a> {
    /// Creates controls from independently optional cancellation and progress inputs.
    pub const fn new(
        cancellation: Option<&'a CancellationToken>,
        progress: Option<&'a dyn ProgressSink>,
    ) -> Self {
        Self {
            cancellation,
            progress,
        }
    }

    /// Creates controls using only a cancellation token.
    pub const fn with_cancellation(cancellation: &'a CancellationToken) -> Self {
        Self::new(Some(cancellation), None)
    }

    /// Creates controls using only a progress sink.
    pub const fn with_progress(progress: &'a dyn ProgressSink) -> Self {
        Self::new(None, Some(progress))
    }

    /// Borrows the configured cancellation token, if any.
    #[must_use]
    pub const fn cancellation(self) -> Option<&'a CancellationToken> {
        self.cancellation
    }

    /// Borrows the configured progress sink, if any.
    #[must_use]
    pub const fn progress(self) -> Option<&'a dyn ProgressSink> {
        self.progress
    }
}

impl fmt::Debug for ExecutionControl<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ExecutionControl")
            .field("has_cancellation", &self.cancellation.is_some())
            .field("has_progress", &self.progress.is_some())
            .finish()
    }
}

/// Structured execution-policy or cancellation failure.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub enum ExecutionError {
    /// The caller requested cancellation at a deterministic work boundary.
    Cancelled {
        /// Operation that was cancelled.
        operation: ExecutionOperation,
        /// Last boundary reached by the operation.
        stage: ExecutionStage,
    },
    /// The current serial implementation cannot honor the requested worker count.
    UnsupportedThreadCount {
        /// Operation whose policy was rejected.
        operation: ExecutionOperation,
        /// Explicit requested worker count.
        requested: NonZeroUsize,
        /// Largest worker count currently implemented for the operation.
        maximum: NonZeroUsize,
    },
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cancelled { operation, stage } => {
                write!(formatter, "{operation} cancelled at {stage:?}")
            }
            Self::UnsupportedThreadCount {
                operation,
                requested,
                maximum,
            } => write!(
                formatter,
                "{operation} requested {requested} worker threads but supports at most {maximum}"
            ),
        }
    }
}

impl Error for ExecutionError {}

pub(crate) struct ProgressTracker<'a> {
    control: ExecutionControl<'a>,
    operation: ExecutionOperation,
    deterministic: bool,
    total: usize,
    completed: usize,
}

impl<'a> ProgressTracker<'a> {
    pub(crate) fn try_new(
        control: ExecutionControl<'a>,
        operation: ExecutionOperation,
        options: ExecutionOptions,
        total: usize,
    ) -> Result<Self, ExecutionError> {
        if let Some(requested) = options.thread_count()
            && requested > SERIAL_THREAD_COUNT
        {
            return Err(ExecutionError::UnsupportedThreadCount {
                operation,
                requested,
                maximum: SERIAL_THREAD_COUNT,
            });
        }
        let tracker = Self {
            control,
            operation,
            deterministic: options.deterministic(),
            total,
            completed: 0,
        };
        tracker.check_cancelled(ExecutionStage::Started)?;
        tracker.report(ExecutionStage::Started);
        tracker.check_cancelled(ExecutionStage::Started)?;
        Ok(tracker)
    }

    pub(crate) fn advance(&mut self, stage: ExecutionStage) -> Result<(), ExecutionError> {
        self.check_cancelled(stage)?;
        self.completed = self.completed.saturating_add(1).min(self.total);
        self.report(stage);
        self.check_cancelled(stage)
    }

    pub(crate) fn finish_work<T, E>(
        &mut self,
        stage: ExecutionStage,
        result: Result<T, E>,
    ) -> Result<T, E>
    where
        E: From<ExecutionError>,
    {
        let value = self.observe_result(stage, result)?;
        self.advance(stage).map_err(E::from)?;
        Ok(value)
    }

    pub(crate) fn observe_result<T, E>(
        &self,
        stage: ExecutionStage,
        result: Result<T, E>,
    ) -> Result<T, E>
    where
        E: From<ExecutionError>,
    {
        self.check_cancelled(stage).map_err(E::from)?;
        result
    }

    pub(crate) fn complete(self) -> Result<(), ExecutionError> {
        self.check_cancelled(ExecutionStage::Completed)?;
        self.report(ExecutionStage::Completed);
        Ok(())
    }

    fn check_cancelled(&self, stage: ExecutionStage) -> Result<(), ExecutionError> {
        if self
            .control
            .cancellation()
            .is_some_and(CancellationToken::is_cancelled)
        {
            return Err(ExecutionError::Cancelled {
                operation: self.operation,
                stage,
            });
        }
        Ok(())
    }

    fn report(&self, stage: ExecutionStage) {
        if let Some(sink) = self.control.progress() {
            sink.on_progress(ProgressEvent {
                operation: self.operation,
                stage,
                completed: self.completed,
                total: self.total,
                deterministic: self.deterministic,
                thread_count: SERIAL_THREAD_COUNT,
            });
        }
    }
}
