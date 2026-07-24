//! Reusable fitted-field batch evaluation and performance diagnostics.
//!
//! Batch evaluation preserves input order and assigns one isolated scratch
//! workspace to each explicitly requested worker. It never configures a
//! process-global thread pool. Logical memory is checked before allocation,
//! and repeated serial batches can reuse both caller-owned output capacity and
//! a [`FittedFieldEvaluationWorkspace`] without allocating per query.

use std::error::Error;
use std::fmt;
use std::io;
use std::mem::size_of;
use std::num::NonZeroUsize;
use std::thread;

use crate::dimension::{Dim, SupportedDimension};
use crate::geometry::Point;
use crate::model::{
    FittedField, FittedFieldAssemblyDiagnostics, FittedFieldEvaluation, FittedFieldEvaluationError,
    FittedFieldEvaluationScratch, FittedFieldOutput,
};
use crate::polynomial::PolynomialSpace;

/// Explicit worker and logical-memory policy for one fitted-field batch.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub struct BatchEvaluationOptions {
    thread_count: NonZeroUsize,
    memory_limit_bytes: NonZeroUsize,
}

impl BatchEvaluationOptions {
    /// Creates a batch policy with no implicit thread or memory choice.
    pub const fn new(thread_count: NonZeroUsize, memory_limit_bytes: NonZeroUsize) -> Self {
        Self {
            thread_count,
            memory_limit_bytes,
        }
    }

    /// Returns the requested maximum worker count.
    #[must_use]
    pub const fn thread_count(self) -> NonZeroUsize {
        self.thread_count
    }

    /// Returns the logical peak-memory limit checked before allocation.
    #[must_use]
    pub const fn memory_limit_bytes(self) -> NonZeroUsize {
        self.memory_limit_bytes
    }
}

/// Checked logical-memory evidence for one value/gradient batch.
///
/// The estimate covers GeoRBF-owned output and scratch payloads. Operating
/// system thread stacks and standard-library thread bookkeeping are outside
/// this portable logical estimate.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub struct BatchEvaluationMemoryDiagnostics {
    /// Number of original-coordinate query points.
    pub query_count: usize,
    /// Number of workers that will receive nonempty query ranges.
    pub worker_count: usize,
    /// Requested output payload bytes.
    pub output_bytes: usize,
    /// Scratch payload bytes requested for one worker.
    pub workspace_bytes_per_worker: usize,
    /// Scratch payload bytes requested across all workers.
    pub workspace_bytes: usize,
    /// Sum of output and worker scratch payloads.
    pub estimated_peak_bytes: usize,
    /// Explicit caller limit enforced before allocation.
    pub memory_limit_bytes: usize,
}

/// Deterministic work and memory evidence for a completed batch.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub struct BatchEvaluationDiagnostics {
    /// Number of evaluated query points.
    pub query_count: usize,
    /// Maximum worker count requested by the caller.
    pub requested_thread_count: NonZeroUsize,
    /// Workers assigned nonempty deterministic contiguous ranges.
    pub worker_count: usize,
    /// Sum of center representers visited across all query points.
    pub center_evaluations: usize,
    /// Checked logical-memory evidence.
    pub memory: BatchEvaluationMemoryDiagnostics,
}

/// Completed value/gradient batch in original input order.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct FittedFieldBatchEvaluation<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    evaluations: Vec<FittedFieldEvaluation<D>>,
    diagnostics: BatchEvaluationDiagnostics,
}

impl<const D: usize> FittedFieldBatchEvaluation<D>
where
    Dim<D>: SupportedDimension,
{
    /// Borrows evaluations in exact input order.
    pub fn evaluations(&self) -> &[FittedFieldEvaluation<D>] {
        &self.evaluations
    }

    /// Returns deterministic work and memory evidence.
    pub const fn diagnostics(&self) -> BatchEvaluationDiagnostics {
        self.diagnostics
    }

    /// Consumes the batch and returns evaluations in exact input order.
    #[must_use]
    pub fn into_evaluations(self) -> Vec<FittedFieldEvaluation<D>> {
        self.evaluations
    }
}

/// Reusable value/gradient scratch for serial fitted-field batches.
#[must_use]
pub struct FittedFieldEvaluationWorkspace<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    sparse_center_capacity: usize,
    polynomial_count: usize,
    estimated_bytes: usize,
    scratch: FittedFieldEvaluationScratch<D>,
}

impl<const D: usize> fmt::Debug for FittedFieldEvaluationWorkspace<D>
where
    Dim<D>: SupportedDimension,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FittedFieldEvaluationWorkspace")
            .field("sparse_center_capacity", &self.sparse_center_capacity)
            .field("polynomial_count", &self.polynomial_count)
            .field("estimated_bytes", &self.estimated_bytes)
            .finish_non_exhaustive()
    }
}

impl<const D: usize> FittedFieldEvaluationWorkspace<D>
where
    Dim<D>: SupportedDimension,
{
    /// Returns the checked requested scratch payload size.
    #[must_use]
    pub const fn estimated_bytes(&self) -> usize {
        self.estimated_bytes
    }
}

/// Storage role used by fallible batch allocation diagnostics.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BatchEvaluationStorage {
    /// Per-worker reusable workspaces.
    Workspaces,
    /// Ordered batch output values.
    Evaluations,
    /// Scoped worker handles.
    WorkerHandles,
}

/// Structured fitted-field batch failure.
#[derive(Debug)]
#[must_use]
pub enum BatchEvaluationError<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    /// Checked byte or work-count arithmetic overflowed.
    MemoryEstimateOverflow,
    /// The pre-allocation logical peak exceeds the explicit caller limit.
    MemoryLimitExceeded {
        /// Checked logical peak.
        estimated_peak_bytes: usize,
        /// Explicit caller limit.
        limit_bytes: usize,
    },
    /// Owned batch storage could not be reserved.
    AllocationFailed {
        /// Affected storage role.
        storage: BatchEvaluationStorage,
        /// Requested entry count.
        requested: usize,
    },
    /// A caller-owned workspace has incompatible scratch dimensions.
    IncompatibleWorkspace {
        /// Sparse center capacity required by the model.
        expected_sparse_center_capacity: usize,
        /// Sparse center capacity retained by the workspace.
        actual_sparse_center_capacity: usize,
        /// Polynomial term count required by the model.
        expected_polynomial_count: usize,
        /// Polynomial term count retained by the workspace.
        actual_polynomial_count: usize,
    },
    /// One worker workspace could not be constructed.
    Workspace {
        /// Stable worker index.
        worker: usize,
        /// Underlying fitted-field allocation failure.
        source: FittedFieldEvaluationError<D>,
    },
    /// One query could not be evaluated.
    Evaluation {
        /// Stable input query index.
        query: usize,
        /// Underlying fitted-field evaluation failure.
        source: FittedFieldEvaluationError<D>,
    },
    /// The standard library could not create a scoped worker.
    WorkerSpawn {
        /// Stable worker index.
        worker: usize,
        /// Operating-system thread creation failure.
        source: io::Error,
    },
    /// A worker panicked instead of returning a structured result.
    WorkerPanicked {
        /// Stable worker index.
        worker: usize,
    },
}

impl<const D: usize> fmt::Display for BatchEvaluationError<D>
where
    Dim<D>: SupportedDimension,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MemoryEstimateOverflow => {
                formatter.write_str("fitted-field batch memory estimate overflowed")
            }
            Self::MemoryLimitExceeded {
                estimated_peak_bytes,
                limit_bytes,
            } => write!(
                formatter,
                "fitted-field batch estimated peak {estimated_peak_bytes} bytes exceeds limit {limit_bytes} bytes"
            ),
            Self::AllocationFailed { storage, requested } => write!(
                formatter,
                "could not reserve {requested} entries for fitted-field batch {storage:?}"
            ),
            Self::IncompatibleWorkspace {
                expected_sparse_center_capacity,
                actual_sparse_center_capacity,
                expected_polynomial_count,
                actual_polynomial_count,
            } => write!(
                formatter,
                "batch workspace mismatch: sparse center capacity {actual_sparse_center_capacity} (expected {expected_sparse_center_capacity}), polynomial count {actual_polynomial_count} (expected {expected_polynomial_count})"
            ),
            Self::Workspace { worker, source } => {
                write!(
                    formatter,
                    "batch worker {worker} workspace failed: {source}"
                )
            }
            Self::Evaluation { query, source } => {
                write!(formatter, "batch query {query} failed: {source}")
            }
            Self::WorkerSpawn { worker, source } => {
                write!(formatter, "could not spawn batch worker {worker}: {source}")
            }
            Self::WorkerPanicked { worker } => {
                write!(formatter, "batch worker {worker} panicked")
            }
        }
    }
}

impl<const D: usize> Error for BatchEvaluationError<D>
where
    Dim<D>: SupportedDimension,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Workspace { source, .. } | Self::Evaluation { source, .. } => Some(source),
            Self::WorkerSpawn { source, .. } => Some(source),
            Self::MemoryEstimateOverflow
            | Self::MemoryLimitExceeded { .. }
            | Self::AllocationFailed { .. }
            | Self::IncompatibleWorkspace { .. }
            | Self::WorkerPanicked { .. } => None,
        }
    }
}

impl<const D: usize> FittedField<D>
where
    Dim<D>: SupportedDimension,
{
    /// Allocates one reusable serial value/gradient evaluation workspace.
    ///
    /// Sparse models reserve enough center-index capacity for every retained
    /// center, so exact support filtering cannot allocate inside a query.
    ///
    /// # Errors
    ///
    /// Returns a structured fitted-field allocation failure.
    pub fn try_evaluation_workspace(
        &self,
    ) -> Result<FittedFieldEvaluationWorkspace<D>, FittedFieldEvaluationError<D>> {
        let (sparse_center_capacity, polynomial_count) = self.workspace_shape();
        let estimated_bytes = workspace_bytes::<D>(sparse_center_capacity, polynomial_count)
            .ok_or(FittedFieldEvaluationError::AllocationFailed {
                storage: crate::model::FittedFieldStorage::NeighborhoodCenters,
                requested: sparse_center_capacity,
            })?;
        let scratch = self.try_evaluation_scratch(FittedFieldOutput::Gradient)?;
        Ok(FittedFieldEvaluationWorkspace {
            sparse_center_capacity,
            polynomial_count,
            estimated_bytes,
            scratch,
        })
    }

    /// Evaluates a serial value/gradient batch into reusable caller storage.
    ///
    /// The output is cleared on every failure, so no partial batch is
    /// returned. After output capacity and workspace are established, the
    /// method performs no allocation per query.
    ///
    /// # Errors
    ///
    /// Returns an incompatible-workspace, output-allocation, count-overflow,
    /// or indexed fitted-field evaluation failure.
    pub fn try_evaluate_batch_into(
        &self,
        points: &[Point<D>],
        workspace: &mut FittedFieldEvaluationWorkspace<D>,
        output: &mut Vec<FittedFieldEvaluation<D>>,
    ) -> Result<usize, BatchEvaluationError<D>> {
        let (expected_sparse, expected_polynomial) = self.workspace_shape();
        if workspace.sparse_center_capacity != expected_sparse
            || workspace.polynomial_count != expected_polynomial
        {
            return Err(BatchEvaluationError::IncompatibleWorkspace {
                expected_sparse_center_capacity: expected_sparse,
                actual_sparse_center_capacity: workspace.sparse_center_capacity,
                expected_polynomial_count: expected_polynomial,
                actual_polynomial_count: workspace.polynomial_count,
            });
        }
        output.clear();
        output.try_reserve_exact(points.len()).map_err(|_| {
            BatchEvaluationError::AllocationFailed {
                storage: BatchEvaluationStorage::Evaluations,
                requested: points.len(),
            }
        })?;
        let mut center_evaluations = 0_usize;
        for (query, point) in points.iter().copied().enumerate() {
            let evaluation = match self.try_evaluate_with_scratch(point, &mut workspace.scratch) {
                Ok(evaluation) => evaluation,
                Err(source) => {
                    output.clear();
                    return Err(BatchEvaluationError::Evaluation { query, source });
                }
            };
            center_evaluations = center_evaluations
                .checked_add(evaluation.center_evaluations())
                .ok_or(BatchEvaluationError::MemoryEstimateOverflow)?;
            output.push(evaluation);
        }
        Ok(center_evaluations)
    }

    /// Returns the checked pre-allocation logical-memory plan for a batch.
    ///
    /// Worker count is `min(query_count, requested_threads)`; an empty batch
    /// uses no worker workspace.
    ///
    /// # Errors
    ///
    /// Returns [`BatchEvaluationError::MemoryEstimateOverflow`] when any byte
    /// product or sum cannot be represented by `usize`.
    pub fn try_batch_memory_diagnostics(
        &self,
        query_count: usize,
        options: BatchEvaluationOptions,
    ) -> Result<BatchEvaluationMemoryDiagnostics, BatchEvaluationError<D>> {
        let worker_count = query_count.min(options.thread_count().get());
        let (sparse_center_capacity, polynomial_count) = self.workspace_shape();
        let workspace_bytes_per_worker =
            workspace_bytes::<D>(sparse_center_capacity, polynomial_count)
                .ok_or(BatchEvaluationError::MemoryEstimateOverflow)?;
        let workspace_bytes = workspace_bytes_per_worker
            .checked_mul(worker_count)
            .ok_or(BatchEvaluationError::MemoryEstimateOverflow)?;
        let output_bytes = size_of::<FittedFieldEvaluation<D>>()
            .checked_mul(query_count)
            .ok_or(BatchEvaluationError::MemoryEstimateOverflow)?;
        let estimated_peak_bytes = output_bytes
            .checked_add(workspace_bytes)
            .ok_or(BatchEvaluationError::MemoryEstimateOverflow)?;
        Ok(BatchEvaluationMemoryDiagnostics {
            query_count,
            worker_count,
            output_bytes,
            workspace_bytes_per_worker,
            workspace_bytes,
            estimated_peak_bytes,
            memory_limit_bytes: options.memory_limit_bytes().get(),
        })
    }

    /// Evaluates a value/gradient batch with explicit workers and memory limit.
    ///
    /// Query ranges are contiguous and deterministic. Every successful worker
    /// writes only its assigned output range, and returned values retain exact
    /// input order. No global thread pool or hidden thread-count adjustment is
    /// used.
    ///
    /// # Errors
    ///
    /// Returns a pre-allocation memory, allocation, workspace, indexed query,
    /// worker-spawn, or worker-panic failure without a partial batch.
    #[allow(clippy::too_many_lines)]
    pub fn try_evaluate_batch(
        &self,
        points: &[Point<D>],
        options: BatchEvaluationOptions,
    ) -> Result<FittedFieldBatchEvaluation<D>, BatchEvaluationError<D>> {
        let memory = self.try_batch_memory_diagnostics(points.len(), options)?;
        if memory.estimated_peak_bytes > memory.memory_limit_bytes {
            return Err(BatchEvaluationError::MemoryLimitExceeded {
                estimated_peak_bytes: memory.estimated_peak_bytes,
                limit_bytes: memory.memory_limit_bytes,
            });
        }
        if points.is_empty() {
            return Ok(FittedFieldBatchEvaluation {
                evaluations: Vec::new(),
                diagnostics: BatchEvaluationDiagnostics {
                    query_count: 0,
                    requested_thread_count: options.thread_count(),
                    worker_count: 0,
                    center_evaluations: 0,
                    memory,
                },
            });
        }

        let mut workspaces = Vec::new();
        workspaces
            .try_reserve_exact(memory.worker_count)
            .map_err(|_| BatchEvaluationError::AllocationFailed {
                storage: BatchEvaluationStorage::Workspaces,
                requested: memory.worker_count,
            })?;
        for worker in 0..memory.worker_count {
            workspaces.push(
                self.try_evaluation_workspace()
                    .map_err(|source| BatchEvaluationError::Workspace { worker, source })?,
            );
        }

        let first = self
            .try_evaluate_with_scratch(points[0], &mut workspaces[0].scratch)
            .map_err(|source| BatchEvaluationError::Evaluation { query: 0, source })?;
        let mut evaluations = Vec::new();
        evaluations.try_reserve_exact(points.len()).map_err(|_| {
            BatchEvaluationError::AllocationFailed {
                storage: BatchEvaluationStorage::Evaluations,
                requested: points.len(),
            }
        })?;
        evaluations.resize(points.len(), first);
        let first_center_evaluations = first.center_evaluations();

        let additional_center_evaluations = if memory.worker_count == 1 {
            let mut total = 0_usize;
            for (query, point) in points.iter().copied().enumerate().skip(1) {
                let evaluation = self
                    .try_evaluate_with_scratch(point, &mut workspaces[0].scratch)
                    .map_err(|source| BatchEvaluationError::Evaluation { query, source })?;
                total = total
                    .checked_add(evaluation.center_evaluations())
                    .ok_or(BatchEvaluationError::MemoryEstimateOverflow)?;
                evaluations[query] = evaluation;
            }
            total
        } else {
            thread::scope(|scope| {
                let mut handles = Vec::new();
                handles
                    .try_reserve_exact(memory.worker_count)
                    .map_err(|_| BatchEvaluationError::AllocationFailed {
                        storage: BatchEvaluationStorage::WorkerHandles,
                        requested: memory.worker_count,
                    })?;
                let base = points.len() / memory.worker_count;
                let remainder = points.len() % memory.worker_count;
                let mut query_start = 0_usize;
                let mut remaining_output = evaluations.as_mut_slice();
                let mut failure = None;
                for (worker, mut workspace) in workspaces.into_iter().enumerate() {
                    let count = base + usize::from(worker < remainder);
                    let (worker_output, tail) = remaining_output.split_at_mut(count);
                    remaining_output = tail;
                    let worker_start = query_start;
                    let worker_points = &points[worker_start..worker_start + count];
                    query_start += count;
                    let skip = usize::from(worker_start == 0);
                    let spawned = thread::Builder::new().spawn_scoped(scope, move || {
                        let mut center_evaluations = 0_usize;
                        for local in skip..worker_points.len() {
                            let evaluation = self
                                .try_evaluate_with_scratch(
                                    worker_points[local],
                                    &mut workspace.scratch,
                                )
                                .map_err(|source| BatchEvaluationError::Evaluation {
                                    query: worker_start + local,
                                    source,
                                })?;
                            center_evaluations = center_evaluations
                                .checked_add(evaluation.center_evaluations())
                                .ok_or(BatchEvaluationError::MemoryEstimateOverflow)?;
                            worker_output[local] = evaluation;
                        }
                        Ok::<usize, BatchEvaluationError<D>>(center_evaluations)
                    });
                    match spawned {
                        Ok(handle) => handles.push((worker, handle)),
                        Err(source) => {
                            failure = Some(BatchEvaluationError::WorkerSpawn { worker, source });
                            break;
                        }
                    }
                }
                let mut total = 0_usize;
                for (worker, handle) in handles {
                    match handle.join() {
                        Ok(Ok(center_evaluations)) => {
                            if failure.is_none() {
                                match total.checked_add(center_evaluations) {
                                    Some(next) => total = next,
                                    None => {
                                        failure =
                                            Some(BatchEvaluationError::MemoryEstimateOverflow);
                                    }
                                }
                            }
                        }
                        Ok(Err(error)) => {
                            if failure.is_none() {
                                failure = Some(error);
                            }
                        }
                        Err(_) => {
                            if failure.is_none() {
                                failure = Some(BatchEvaluationError::WorkerPanicked { worker });
                            }
                        }
                    }
                }
                if let Some(error) = failure {
                    return Err(error);
                }
                Ok::<usize, BatchEvaluationError<D>>(total)
            })?
        };
        let center_evaluations = first_center_evaluations
            .checked_add(additional_center_evaluations)
            .ok_or(BatchEvaluationError::MemoryEstimateOverflow)?;

        Ok(FittedFieldBatchEvaluation {
            evaluations,
            diagnostics: BatchEvaluationDiagnostics {
                query_count: points.len(),
                requested_thread_count: options.thread_count(),
                worker_count: memory.worker_count,
                center_evaluations,
                memory,
            },
        })
    }

    fn workspace_shape(&self) -> (usize, usize) {
        let sparse_center_capacity = match self.diagnostics().assembly() {
            FittedFieldAssemblyDiagnostics::Dense(_) => 0,
            FittedFieldAssemblyDiagnostics::Sparse(_) => self.centers().len(),
        };
        let polynomial_count = self
            .polynomial_space()
            .map_or(0, PolynomialSpace::term_count);
        (sparse_center_capacity, polynomial_count)
    }
}

fn workspace_bytes<const D: usize>(
    sparse_center_capacity: usize,
    polynomial_count: usize,
) -> Option<usize>
where
    Dim<D>: SupportedDimension,
{
    let center_bytes = size_of::<usize>().checked_mul(sparse_center_capacity)?;
    let value_bytes = size_of::<f64>().checked_mul(polynomial_count)?;
    let gradient_bytes = size_of::<[f64; D]>().checked_mul(polynomial_count)?;
    center_bytes
        .checked_add(value_bytes)?
        .checked_add(gradient_bytes)
}
