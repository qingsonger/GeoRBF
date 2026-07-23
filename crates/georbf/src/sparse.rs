//! Compact-support sparse field assembly and checked positive-definite solving.
//!
//! The public storage and diagnostics in this module are GeoRBF-owned. `rstar`
//! and `faer` remain private implementation details.

use std::error::Error;
use std::fmt;
use std::mem::size_of;
use std::num::NonZeroUsize;

use faer::prelude::Solve;
use faer::sparse::{SparseColMat, SymbolicSparseColMat};
use faer::{Col, Side};
use rstar::primitives::GeomWithData;
use rstar::{ParentNode, RTree, RTreeNode};

use crate::Point;
use crate::anisotropy::{AnisotropyError, GlobalAnisotropy};
use crate::dimension::{Dim, SupportedDimension};
use crate::execution::{
    ExecutionControl, ExecutionError, ExecutionOperation, ExecutionStage, ProgressTracker,
};
use crate::field::{
    FieldAssemblyError, FieldLinearizationError, FieldProblem, observation_at,
    validate_capabilities,
};
use crate::functional::CenterRepresenter;
use crate::kernel::Wendland;
use crate::kernel_calculus::{KernelCalculusError, RadialSeparation};
use crate::model::{KernelDefinition, KernelDefinitionEvaluationError};
use crate::problem_ir::{
    AffineExpression, AffineTerm, CanonicalEquality, CanonicalLinearBound, CanonicalProblem,
    CanonicalSecondOrderCone, CanonicalSoftObjective, ExecutionOptions, VariableBlock,
};
use crate::solver::ExactDotAccumulator;

const BACKEND_NAME: &str = "faer";
const BACKEND_VERSION: &str = "0.24.4";
const ORDERING_NAME: &str = "amd";
const RESIDUAL_TOLERANCE_FACTOR: f64 = 128.0;

/// Private sparse factorization selected by ADR-0012.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub enum SparseFactorization {
    /// Faer sparse positive-definite `LL^T` with AMD ordering.
    FaerLlt,
}

/// Explicit compact sparse fitting policy.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub struct SparseFitOptions {
    factorization: SparseFactorization,
    memory_limit_bytes: NonZeroUsize,
}

impl SparseFitOptions {
    /// Constructs an explicit sparse factorization and memory policy.
    pub const fn new(factorization: SparseFactorization, memory_limit_bytes: NonZeroUsize) -> Self {
        Self {
            factorization,
            memory_limit_bytes,
        }
    }

    /// Returns the requested sparse factorization.
    pub const fn factorization(self) -> SparseFactorization {
        self.factorization
    }

    /// Returns the caller's nonzero memory limit.
    #[must_use]
    pub const fn memory_limit_bytes(self) -> NonZeroUsize {
        self.memory_limit_bytes
    }
}

/// Storage role used by sparse allocation diagnostics.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SparseStorage {
    /// Indexed functional atom locations.
    IndexItems,
    /// Candidate representer pairs.
    NeighborPairs,
    /// Symmetric sparse entries.
    Entries,
    /// Canonical row offsets.
    RowOffsets,
    /// Canonical affine terms.
    AffineTerms,
    /// Canonical variable blocks.
    VariableBlocks,
    /// CSC column pointers.
    ColumnPointers,
    /// CSC row indices.
    RowIndices,
    /// CSC numeric values.
    Values,
    /// Equality right-hand side.
    RightHandSide,
    /// Backend symbolic indices.
    BackendIndices,
    /// Backend numeric values.
    BackendValues,
    /// Solution vector.
    Solution,
    /// Exact residual accumulators.
    ResidualAccumulators,
    /// Residual row sums.
    ResidualRowSums,
    /// Local evaluation center indices.
    EvaluationCenters,
}

/// Failure while querying one immutable compact-support neighborhood.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub enum CompactNeighborhoodError<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    /// A query-to-center separation was not representable.
    Calculus(KernelCalculusError),
    /// An anisotropy-transformed separation was not representable.
    Anisotropy(AnisotropyError<D>),
    /// A retained center or term index was inconsistent.
    InvalidIndex {
        /// Center index.
        center: usize,
        /// Term index within the center.
        term: usize,
    },
    /// Checked collection growth failed.
    AllocationFailed {
        /// Storage role.
        storage: SparseStorage,
        /// Requested minimum entries.
        requested: usize,
    },
}

impl<const D: usize> fmt::Display for CompactNeighborhoodError<D>
where
    Dim<D>: SupportedDimension,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Calculus(source) => source.fmt(formatter),
            Self::Anisotropy(source) => source.fmt(formatter),
            Self::InvalidIndex { center, term } => {
                write!(
                    formatter,
                    "compact index refers to missing center {center} term {term}"
                )
            }
            Self::AllocationFailed { storage, requested } => write!(
                formatter,
                "could not reserve {requested} entries for sparse storage {storage:?}"
            ),
        }
    }
}

impl<const D: usize> Error for CompactNeighborhoodError<D>
where
    Dim<D>: SupportedDimension,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Calculus(source) => Some(source),
            Self::Anisotropy(source) => Some(source),
            Self::InvalidIndex { .. } | Self::AllocationFailed { .. } => None,
        }
    }
}

/// Deterministic support-neighborhood evidence.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct CompactNeighborhoodDiagnostics {
    /// Number of indexed atomic center locations.
    pub indexed_terms: usize,
    /// Conservative Euclidean candidate-query radius.
    pub candidate_radius: f64,
    /// Raw atom hits returned by the index before exact support filtering.
    pub candidate_term_hits: usize,
    /// Sorted unique upper-triangle representer pairs inside exact support.
    pub supported_pairs: usize,
    /// Centers with no off-diagonal supported representer.
    pub isolated_centers: usize,
    /// Smallest supported representer count in any matrix row.
    pub minimum_row_neighbors: usize,
    /// Largest supported representer count in any matrix row.
    pub maximum_row_neighbors: usize,
}

/// Checked retained and peak logical-memory evidence for sparse assembly.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct SparseAssemblyMemoryDiagnostics {
    /// Retained immutable neighborhood-index payload.
    pub retained_index_bytes: usize,
    /// Retained canonical equality payload, including allocation capacities.
    pub retained_canonical_bytes: usize,
    /// Retained CSC allocation capacities.
    pub retained_matrix_bytes: usize,
    /// Retained right-hand-side allocation capacity.
    pub retained_rhs_bytes: usize,
    /// Temporary accepted-pair allocation capacity at the assembly peak.
    pub temporary_neighbor_pairs_bytes: usize,
    /// Temporary reflected-entry allocation capacity at the assembly peak.
    pub temporary_entries_bytes: usize,
    /// Temporary canonical row-offset allocation capacity at the assembly peak.
    pub temporary_row_offsets_bytes: usize,
    /// Temporary support-coverage allocation capacity at the assembly peak.
    pub temporary_row_neighbors_bytes: usize,
    /// Temporary bulk-load item payload during index construction.
    pub temporary_index_items_bytes: usize,
    /// Conservative canonical payload checked before canonical allocation.
    pub canonicalization_payload_upper_bound_bytes: usize,
    /// Sum of the four retained logical components.
    pub estimated_retained_bytes: usize,
    /// Index plus temporary bulk-load items.
    pub index_construction_peak_bytes: usize,
    /// Simultaneously live index, pairs, entries, row buffers, and canonical payload.
    pub canonicalization_peak_bytes: usize,
    /// Simultaneously live retained storage and assembly temporaries.
    pub storage_materialization_peak_bytes: usize,
    /// Maximum of every explicitly checked assembly-stage sum.
    pub estimated_peak_bytes: usize,
}

/// Sparse field-assembly evidence.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct SparseFieldAssemblyDiagnostics {
    /// Square system dimension.
    pub system_dimension: usize,
    /// Exact support-neighbor kernel actions evaluated.
    pub kernel_entry_evaluations: usize,
    /// Stored symmetric nonzeros.
    pub stored_nonzeros: usize,
    /// Stored nonzeros divided by the dense square entry count.
    pub density: f64,
    /// Largest absolute stored value.
    pub maximum_absolute_entry: f64,
    /// Checked conservative logical sparse-payload estimate.
    pub estimated_storage_bytes: usize,
    /// Checked retained and assembly-peak logical memory evidence.
    pub memory: SparseAssemblyMemoryDiagnostics,
    /// Effective explicit memory limit.
    pub memory_limit_bytes: usize,
    /// Neighborhood and coverage evidence.
    pub neighborhood: CompactNeighborhoodDiagnostics,
}

/// Immutable GeoRBF-owned sorted-unique CSC matrix.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct SparseFieldMatrix {
    dimension: usize,
    column_pointers: Vec<usize>,
    row_indices: Vec<usize>,
    values: Vec<f64>,
}

impl SparseFieldMatrix {
    /// Returns the equal row and column count.
    #[must_use]
    pub const fn dimension(&self) -> usize {
        self.dimension
    }

    /// Borrows the `dimension + 1` monotone CSC column pointers.
    #[must_use]
    pub fn column_pointers(&self) -> &[usize] {
        &self.column_pointers
    }

    /// Borrows sorted unique row indices within every column.
    #[must_use]
    pub fn row_indices(&self) -> &[usize] {
        &self.row_indices
    }

    /// Borrows finite values aligned with [`Self::row_indices`].
    #[must_use]
    pub fn values(&self) -> &[f64] {
        &self.values
    }

    /// Returns one stored value, or exact zero for an in-range structural zero.
    #[must_use]
    pub fn get(&self, row: usize, column: usize) -> Option<f64> {
        if row >= self.dimension || column >= self.dimension {
            return None;
        }
        let range = self.column_pointers[column]..self.column_pointers[column + 1];
        match self.row_indices[range.clone()].binary_search(&row) {
            Ok(offset) => Some(self.values[range.start + offset]),
            Err(_) => Some(0.0),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct SupportItem {
    center: usize,
    term: usize,
}

type IndexedAtom = GeomWithData<[f64; 3], SupportItem>;

#[derive(Clone)]
pub(crate) struct CompactNeighborhood<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    tree: RTree<IndexedAtom>,
    support_radius: f64,
    candidate_radius: f64,
    candidate_radius_squared: f64,
    indexed_terms: usize,
}

impl<const D: usize> fmt::Debug for CompactNeighborhood<D>
where
    Dim<D>: SupportedDimension,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CompactNeighborhood")
            .field("support_radius", &self.support_radius)
            .field("candidate_radius", &self.candidate_radius)
            .field("candidate_radius_squared", &self.candidate_radius_squared)
            .field("indexed_terms", &self.indexed_terms)
            .finish_non_exhaustive()
    }
}

impl<const D: usize> PartialEq for CompactNeighborhood<D>
where
    Dim<D>: SupportedDimension,
{
    fn eq(&self, other: &Self) -> bool {
        self.support_radius == other.support_radius
            && self.candidate_radius == other.candidate_radius
            && self.candidate_radius_squared == other.candidate_radius_squared
            && self.indexed_terms == other.indexed_terms
    }
}

impl<const D: usize> CompactNeighborhood<D>
where
    Dim<D>: SupportedDimension,
{
    fn try_new(
        centers: &[CenterRepresenter<D>],
        support_radius: f64,
        anisotropy: Option<&GlobalAnisotropy<D>>,
        memory_limit_bytes: usize,
    ) -> Result<Self, SparseFieldAssemblyError<D>> {
        let indexed_terms = centers.iter().try_fold(0_usize, |total, center| {
            total
                .checked_add(center.expression().terms().len())
                .ok_or(SparseFieldAssemblyError::CountOverflow)
        })?;
        enforce_assembly_limit(
            index_construction_peak_bytes(indexed_terms)?,
            memory_limit_bytes,
        )?;
        let mut items = try_with_capacity(
            indexed_terms,
            SparseStorage::IndexItems,
            assembly_allocation::<D>,
        )?;
        for (center_index, center) in centers.iter().enumerate() {
            for (term_index, term) in center.expression().terms().iter().enumerate() {
                items.push(GeomWithData::new(
                    lift_point(term.atom().point()),
                    SupportItem {
                        center: center_index,
                        term: term_index,
                    },
                ));
            }
        }

        let candidate_radius = candidate_radius(support_radius, anisotropy)?;
        let candidate_radius_squared = (candidate_radius * candidate_radius).next_up();
        if !candidate_radius_squared.is_finite() || candidate_radius_squared == 0.0 {
            return Err(SparseFieldAssemblyError::UnrepresentableCandidateRadius {
                support_radius,
            });
        }
        Ok(Self {
            tree: RTree::bulk_load(items),
            support_radius,
            candidate_radius,
            candidate_radius_squared,
            indexed_terms,
        })
    }

    pub(crate) fn try_center_indices_into(
        &self,
        query: Point<D>,
        centers: &[CenterRepresenter<D>],
        anisotropy: Option<&GlobalAnisotropy<D>>,
        selected: &mut Vec<usize>,
    ) -> Result<(), CompactNeighborhoodError<D>> {
        selected.clear();
        for neighbor in self
            .tree
            .locate_within_distance(lift_point(query), self.candidate_radius_squared)
        {
            let item = neighbor.data;
            let term = centers
                .get(item.center)
                .and_then(|center| center.expression().terms().get(item.term))
                .ok_or(CompactNeighborhoodError::InvalidIndex {
                    center: item.center,
                    term: item.term,
                })?;
            if exact_radius(query, term.atom().point(), anisotropy).map_err(
                |error| match error {
                    SeparationError::Calculus(source) => CompactNeighborhoodError::Calculus(source),
                    SeparationError::Anisotropy(source) => {
                        CompactNeighborhoodError::Anisotropy(source)
                    }
                },
            )? < self.support_radius
            {
                if selected.len() == selected.capacity() {
                    let requested = selected.len().saturating_add(1);
                    selected.try_reserve(1).map_err(|_| {
                        CompactNeighborhoodError::AllocationFailed {
                            storage: SparseStorage::EvaluationCenters,
                            requested,
                        }
                    })?;
                }
                selected.push(item.center);
            }
        }
        selected.sort_unstable();
        selected.dedup();
        Ok(())
    }
}

/// Immutable canonical and sparse forms of one compact field problem.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct SparseFieldSystem<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    canonical: CanonicalProblem,
    execution: ExecutionOptions,
    options: SparseFitOptions,
    matrix: SparseFieldMatrix,
    rhs: Vec<f64>,
    center_count: usize,
    diagnostics: SparseFieldAssemblyDiagnostics,
    neighborhood: CompactNeighborhood<D>,
}

impl<const D: usize> SparseFieldSystem<D>
where
    Dim<D>: SupportedDimension,
{
    /// Borrows canonical observation equalities.
    pub const fn canonical_problem(&self) -> &CanonicalProblem {
        &self.canonical
    }

    /// Borrows the sorted-unique symmetric CSC matrix.
    pub const fn matrix(&self) -> &SparseFieldMatrix {
        &self.matrix
    }

    /// Borrows the equality right-hand side.
    #[must_use]
    pub fn rhs(&self) -> &[f64] {
        &self.rhs
    }

    /// Returns the number of center-weight variables.
    #[must_use]
    pub const fn center_count(&self) -> usize {
        self.center_count
    }

    /// Returns retained assembly and coverage evidence.
    pub const fn diagnostics(&self) -> SparseFieldAssemblyDiagnostics {
        self.diagnostics
    }

    /// Returns retained explicit sparse policy.
    pub const fn options(&self) -> SparseFitOptions {
        self.options
    }

    pub(crate) fn into_model_parts(
        self,
    ) -> (SparseFieldAssemblyDiagnostics, CompactNeighborhood<D>) {
        (self.diagnostics, self.neighborhood)
    }
}

/// Structured compact sparse assembly failure.
#[derive(Debug)]
#[must_use]
pub enum SparseFieldAssemblyError<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    /// Caller execution policy or cancellation failed.
    Execution(ExecutionError),
    /// Shared field capability, kernel-action, or canonicalization failed.
    Field(Box<FieldAssemblyError<KernelDefinitionEvaluationError<D>>>),
    /// A support separation was not representable.
    Calculus(KernelCalculusError),
    /// An anisotropy-transformed support separation failed.
    Anisotropy(AnisotropyError<D>),
    /// Checked count or byte arithmetic overflowed.
    CountOverflow,
    /// A conservative anisotropic candidate radius was not representable.
    UnrepresentableCandidateRadius {
        /// Configured compact support radius.
        support_radius: f64,
    },
    /// Sparse payload exceeded the effective explicit memory limit.
    MemoryLimitExceeded {
        /// Checked payload bytes.
        estimated_bytes: usize,
        /// Effective limit.
        limit_bytes: usize,
    },
    /// A checked allocation failed.
    AllocationFailed {
        /// Storage role.
        storage: SparseStorage,
        /// Requested minimum entries.
        requested: usize,
    },
    /// A validated semantic row was unexpectedly unavailable.
    InvalidProblemState {
        /// Missing row.
        row: usize,
    },
    /// Sparse reflection did not remain exactly symmetric.
    NotSymmetric {
        /// Row index.
        row: usize,
        /// Column index.
        column: usize,
    },
}

impl<const D: usize> From<ExecutionError> for SparseFieldAssemblyError<D>
where
    Dim<D>: SupportedDimension,
{
    fn from(value: ExecutionError) -> Self {
        Self::Execution(value)
    }
}

impl<const D: usize> fmt::Display for SparseFieldAssemblyError<D>
where
    Dim<D>: SupportedDimension,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Execution(source) => source.fmt(formatter),
            Self::Field(source) => source.fmt(formatter),
            Self::Calculus(source) => source.fmt(formatter),
            Self::Anisotropy(source) => source.fmt(formatter),
            Self::CountOverflow => formatter.write_str("sparse field count overflowed"),
            Self::UnrepresentableCandidateRadius { support_radius } => write!(
                formatter,
                "support radius {support_radius} has no finite conservative neighborhood radius"
            ),
            Self::MemoryLimitExceeded {
                estimated_bytes,
                limit_bytes,
            } => write!(
                formatter,
                "estimated sparse payload {estimated_bytes} bytes exceeds explicit limit {limit_bytes} bytes"
            ),
            Self::AllocationFailed { storage, requested } => write!(
                formatter,
                "could not reserve {requested} entries for sparse storage {storage:?}"
            ),
            Self::InvalidProblemState { row } => {
                write!(
                    formatter,
                    "validated sparse field is missing equality row {row}"
                )
            }
            Self::NotSymmetric { row, column } => {
                write!(
                    formatter,
                    "sparse field entries ({row}, {column}) are not symmetric"
                )
            }
        }
    }
}

impl<const D: usize> Error for SparseFieldAssemblyError<D>
where
    Dim<D>: SupportedDimension,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Execution(source) => Some(source),
            Self::Field(source) => Some(source.as_ref()),
            Self::Calculus(source) => Some(source),
            Self::Anisotropy(source) => Some(source),
            Self::CountOverflow
            | Self::UnrepresentableCandidateRadius { .. }
            | Self::MemoryLimitExceeded { .. }
            | Self::AllocationFailed { .. }
            | Self::InvalidProblemState { .. }
            | Self::NotSymmetric { .. } => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct NeighborPair {
    row: usize,
    column: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct SparseEntry {
    row: usize,
    column: usize,
    value: f64,
}

impl<const D: usize> FieldProblem<D>
where
    Dim<D>: SupportedDimension,
{
    /// Assembles a Wendland field directly into canonical sparse CSC storage.
    ///
    /// Candidate neighborhoods are only a prefilter. Every atomic point pair
    /// is independently checked against the exact isotropic or global-
    /// anisotropic radius before a representer pair is accepted.
    ///
    /// # Errors
    ///
    /// Returns structured execution, neighborhood, kernel, canonicalization,
    /// symmetry, memory, or allocation failures without returning a partial
    /// system.
    pub fn try_assemble_sparse(
        &self,
        kernel: Wendland,
        anisotropy: Option<GlobalAnisotropy<D>>,
        options: SparseFitOptions,
    ) -> Result<SparseFieldSystem<D>, SparseFieldAssemblyError<D>> {
        self.try_assemble_sparse_with_control(
            kernel,
            anisotropy,
            options,
            ExecutionControl::default(),
        )
    }

    /// Assembles compact CSC storage with borrowed cancellation and progress.
    ///
    /// # Errors
    ///
    /// Returns the same failures as [`Self::try_assemble_sparse`], including
    /// cancellation and unsupported explicit thread-count errors.
    #[allow(clippy::too_many_lines)]
    pub fn try_assemble_sparse_with_control(
        &self,
        kernel: Wendland,
        anisotropy: Option<GlobalAnisotropy<D>>,
        options: SparseFitOptions,
        control: ExecutionControl<'_>,
    ) -> Result<SparseFieldSystem<D>, SparseFieldAssemblyError<D>> {
        let centers = self.centers().len();
        let upper_pairs = centers
            .checked_mul(
                centers
                    .checked_add(1)
                    .ok_or(SparseFieldAssemblyError::CountOverflow)?,
            )
            .and_then(|count| count.checked_div(2))
            .ok_or(SparseFieldAssemblyError::CountOverflow)?;
        let total_progress = upper_pairs
            .checked_add(centers)
            .and_then(|count| count.checked_add(4))
            .ok_or(SparseFieldAssemblyError::CountOverflow)?;
        let mut progress = ProgressTracker::try_new(
            control,
            ExecutionOperation::SparseFieldAssembly,
            self.semantic_problem().execution_options(),
            total_progress,
        )?;
        let memory_limit_bytes =
            effective_memory_limit(self.semantic_problem().execution_options(), options);
        let neighborhood = progress.finish_work(
            ExecutionStage::NeighborhoodIndex,
            CompactNeighborhood::try_new(
                self.centers(),
                kernel.support_radius(),
                anisotropy.as_ref(),
                memory_limit_bytes,
            ),
        )?;
        let retained_index_bytes = retained_index_payload_bytes::<D>(neighborhood.indexed_terms)?;
        let mut checked_assembly_peak =
            index_construction_peak_bytes::<D>(neighborhood.indexed_terms)?;
        let initial_pair_bytes = capacity_bytes::<NeighborPair, D>(centers)?;
        observe_assembly_peak(
            &mut checked_assembly_peak,
            &[retained_index_bytes, initial_pair_bytes],
            memory_limit_bytes,
        )?;
        let mut pairs = try_with_capacity(
            centers,
            SparseStorage::NeighborPairs,
            assembly_allocation::<D>,
        )?;
        let mut candidate_term_hits = 0_usize;
        for (row, center) in self.centers().iter().enumerate() {
            for row_term in center.expression().terms() {
                let query = row_term.atom().point();
                for neighbor in neighborhood.tree.locate_within_distance(
                    lift_point(query),
                    neighborhood.candidate_radius_squared,
                ) {
                    candidate_term_hits = candidate_term_hits
                        .checked_add(1)
                        .ok_or(SparseFieldAssemblyError::CountOverflow)?;
                    let item = neighbor.data;
                    if item.center < row {
                        continue;
                    }
                    let column_term = self
                        .centers()
                        .get(item.center)
                        .and_then(|candidate| candidate.expression().terms().get(item.term))
                        .ok_or(SparseFieldAssemblyError::InvalidProblemState {
                            row: item.center,
                        })?;
                    if exact_radius(query, column_term.atom().point(), anisotropy.as_ref())
                        .map_err(|error| match error {
                            SeparationError::Calculus(source) => {
                                SparseFieldAssemblyError::Calculus(source)
                            }
                            SeparationError::Anisotropy(source) => {
                                SparseFieldAssemblyError::Anisotropy(source)
                            }
                        })?
                        < kernel.support_radius()
                    {
                        try_push_limited(
                            &mut pairs,
                            NeighborPair {
                                row,
                                column: item.center,
                            },
                            SparseStorage::NeighborPairs,
                            memory_limit_bytes,
                            retained_index_bytes,
                            &mut checked_assembly_peak,
                        )?;
                    }
                }
            }
            progress.advance(ExecutionStage::NeighborhoodQuery)?;
        }
        pairs.sort_unstable_by_key(|pair| (pair.row, pair.column));
        pairs.dedup_by_key(|pair| (pair.row, pair.column));

        let definition = KernelDefinition::from(kernel);
        let entry_capacity = pairs
            .len()
            .checked_mul(2)
            .ok_or(SparseFieldAssemblyError::CountOverflow)?;
        let pair_capacity_bytes = capacity_bytes::<NeighborPair, D>(pairs.capacity())?;
        let entry_capacity_bytes = capacity_bytes::<SparseEntry, D>(entry_capacity)?;
        let row_neighbor_bytes = capacity_bytes::<usize, D>(centers)?;
        observe_assembly_peak(
            &mut checked_assembly_peak,
            &[
                retained_index_bytes,
                pair_capacity_bytes,
                entry_capacity_bytes,
                row_neighbor_bytes,
            ],
            memory_limit_bytes,
        )?;
        let mut entries = try_with_capacity(
            entry_capacity,
            SparseStorage::Entries,
            assembly_allocation::<D>,
        )?;
        let mut row_neighbors = try_filled(
            centers,
            0_usize,
            SparseStorage::RowOffsets,
            assembly_allocation::<D>,
        )?;
        let mut maximum_absolute_entry = 0.0_f64;
        for pair in &pairs {
            let observation = observation_at(self.semantic_problem(), pair.row)
                .ok_or(SparseFieldAssemblyError::InvalidProblemState { row: pair.row })?;
            let center = self
                .centers()
                .get(pair.column)
                .ok_or(SparseFieldAssemblyError::InvalidProblemState { row: pair.column })?;
            validate_capabilities(
                definition.metadata(),
                pair.row,
                observation,
                pair.column,
                center,
            )
            .map_err(|source| SparseFieldAssemblyError::Field(Box::new(source)))?;
            let value = observation
                .try_apply_kernel(center, |query, center, demanded| {
                    definition.try_assembly_prefix(query, center, demanded, anisotropy.as_ref())
                })
                .map_err(|source| {
                    SparseFieldAssemblyError::Field(Box::new(FieldAssemblyError::KernelAction {
                        observation_index: pair.row,
                        center_index: pair.column,
                        source,
                    }))
                })?;
            row_neighbors[pair.row] = row_neighbors[pair.row]
                .checked_add(1)
                .ok_or(SparseFieldAssemblyError::CountOverflow)?;
            if pair.row != pair.column {
                row_neighbors[pair.column] = row_neighbors[pair.column]
                    .checked_add(1)
                    .ok_or(SparseFieldAssemblyError::CountOverflow)?;
            }
            if value != 0.0 {
                maximum_absolute_entry = maximum_absolute_entry.max(value.abs());
                entries.push(SparseEntry {
                    row: pair.row,
                    column: pair.column,
                    value,
                });
                if pair.row != pair.column {
                    entries.push(SparseEntry {
                        row: pair.column,
                        column: pair.row,
                        value,
                    });
                }
            }
            progress.advance(ExecutionStage::KernelAssembly)?;
        }
        entries.sort_unstable_by_key(|entry| (entry.row, entry.column));
        if entries
            .windows(2)
            .any(|window| (window[0].row, window[0].column) >= (window[1].row, window[1].column))
        {
            return Err(SparseFieldAssemblyError::NotSymmetric { row: 0, column: 0 });
        }

        let row_offset_count = centers
            .checked_add(1)
            .ok_or(SparseFieldAssemblyError::CountOverflow)?;
        let row_offset_bytes = capacity_bytes::<usize, D>(row_offset_count)?;
        observe_assembly_peak(
            &mut checked_assembly_peak,
            &[
                retained_index_bytes,
                pair_capacity_bytes,
                capacity_bytes::<SparseEntry, D>(entries.capacity())?,
                capacity_bytes::<usize, D>(row_neighbors.capacity())?,
                row_offset_bytes,
            ],
            memory_limit_bytes,
        )?;
        let row_offsets = row_offsets(centers, &entries)?;
        let canonical_upper_bound =
            canonical_equality_payload_upper_bound(self, centers, entries.len())?;
        observe_assembly_peak(
            &mut checked_assembly_peak,
            &[
                retained_index_bytes,
                pair_capacity_bytes,
                capacity_bytes::<SparseEntry, D>(entries.capacity())?,
                capacity_bytes::<usize, D>(row_neighbors.capacity())?,
                capacity_bytes::<usize, D>(row_offsets.capacity())?,
                canonical_upper_bound,
            ],
            memory_limit_bytes,
        )?;
        let mut next_row = 0_usize;
        let canonical = self
            .semantic_problem()
            .try_compile(
                [VariableBlock::try_new(
                    "center_weights".to_owned(),
                    NonZeroUsize::new(centers).ok_or(SparseFieldAssemblyError::CountOverflow)?,
                )
                .map_err(|source| {
                    SparseFieldAssemblyError::Field(Box::new(FieldAssemblyError::Ir(source)))
                })?],
                |_, _| {
                    let row = next_row;
                    next_row = next_row.saturating_add(1);
                    if row >= centers {
                        return Err(FieldLinearizationError::MissingPreassembledRow {
                            row,
                            rows: centers,
                        });
                    }
                    let range = row_offsets[row]..row_offsets[row + 1];
                    let mut terms = Vec::new();
                    terms.try_reserve_exact(range.len()).map_err(|_| {
                        FieldLinearizationError::AllocationFailed {
                            requested: range.len(),
                        }
                    })?;
                    for entry in &entries[range] {
                        terms.push(
                            AffineTerm::try_new(entry.column, entry.value)
                                .map_err(FieldLinearizationError::Ir)?,
                        );
                    }
                    AffineExpression::try_new(terms, 0.0).map_err(FieldLinearizationError::Ir)
                },
            )
            .map_err(|source| {
                SparseFieldAssemblyError::Field(Box::new(FieldAssemblyError::Canonicalization(
                    source,
                )))
            })?;
        progress.advance(ExecutionStage::Canonicalization)?;
        let retained_canonical_bytes = canonical
            .equality_payload_capacity_bytes()
            .ok_or(SparseFieldAssemblyError::CountOverflow)?;
        observe_assembly_peak(
            &mut checked_assembly_peak,
            &[
                retained_index_bytes,
                pair_capacity_bytes,
                capacity_bytes::<SparseEntry, D>(entries.capacity())?,
                capacity_bytes::<usize, D>(row_neighbors.capacity())?,
                capacity_bytes::<usize, D>(row_offsets.capacity())?,
                retained_canonical_bytes,
            ],
            memory_limit_bytes,
        )?;

        review_exact_symmetry(centers, &entries)?;
        progress.advance(ExecutionStage::SymmetryReview)?;

        let rhs_target_bytes = capacity_bytes::<f64, D>(centers)?;
        let matrix_target_bytes = matrix_payload_bytes::<D>(centers, entries.len())?;
        observe_assembly_peak(
            &mut checked_assembly_peak,
            &[
                retained_index_bytes,
                pair_capacity_bytes,
                capacity_bytes::<SparseEntry, D>(entries.capacity())?,
                capacity_bytes::<usize, D>(row_neighbors.capacity())?,
                capacity_bytes::<usize, D>(row_offsets.capacity())?,
                retained_canonical_bytes,
                rhs_target_bytes,
                matrix_target_bytes,
            ],
            memory_limit_bytes,
        )?;
        let mut rhs = try_filled(
            centers,
            0.0_f64,
            SparseStorage::RightHandSide,
            assembly_allocation::<D>,
        )?;
        for (row, equality) in canonical.equalities().iter().enumerate() {
            rhs[row] = equality.rhs();
        }
        let temporary_entries_bytes = capacity_bytes::<SparseEntry, D>(entries.capacity())?;
        entries.sort_unstable_by_key(|entry| (entry.column, entry.row));
        let matrix = materialize_csc(centers, entries)?;
        progress.advance(ExecutionStage::SparseStorage)?;

        let stored_nonzeros = matrix.values.len();
        let retained_matrix_bytes = matrix_capacity_bytes::<D>(&matrix)?;
        let retained_rhs_bytes = capacity_bytes::<f64, D>(rhs.capacity())?;
        let temporary_neighbor_pairs_bytes = capacity_bytes::<NeighborPair, D>(pairs.capacity())?;
        let temporary_row_offsets_bytes = capacity_bytes::<usize, D>(row_offsets.capacity())?;
        let temporary_row_neighbors_bytes = capacity_bytes::<usize, D>(row_neighbors.capacity())?;
        let estimated_storage_bytes = checked_sum_assembly::<D>(&[
            retained_index_bytes,
            retained_canonical_bytes,
            retained_matrix_bytes,
            retained_rhs_bytes,
        ])?;
        let storage_materialization_peak_bytes = checked_sum_assembly::<D>(&[
            estimated_storage_bytes,
            temporary_neighbor_pairs_bytes,
            temporary_entries_bytes,
            temporary_row_offsets_bytes,
            temporary_row_neighbors_bytes,
        ])?;
        let temporary_index_items_bytes =
            capacity_bytes::<IndexedAtom, D>(neighborhood.indexed_terms)?;
        let index_construction_peak_bytes =
            checked_sum_assembly::<D>(&[retained_index_bytes, temporary_index_items_bytes])?;
        let canonicalization_peak_bytes = checked_sum_assembly::<D>(&[
            retained_index_bytes,
            temporary_neighbor_pairs_bytes,
            temporary_entries_bytes,
            temporary_row_offsets_bytes,
            temporary_row_neighbors_bytes,
            canonical_upper_bound,
        ])?;
        let estimated_peak_bytes = index_construction_peak_bytes
            .max(canonicalization_peak_bytes)
            .max(storage_materialization_peak_bytes);
        observe_assembly_peak(
            &mut checked_assembly_peak,
            &[estimated_peak_bytes],
            memory_limit_bytes,
        )?;
        debug_assert_eq!(checked_assembly_peak, estimated_peak_bytes);
        let isolated_centers = row_neighbors.iter().filter(|&&count| count <= 1).count();
        let minimum_row_neighbors = row_neighbors.iter().copied().min().unwrap_or(0);
        let maximum_row_neighbors = row_neighbors.iter().copied().max().unwrap_or(0);
        let memory = SparseAssemblyMemoryDiagnostics {
            retained_index_bytes,
            retained_canonical_bytes,
            retained_matrix_bytes,
            retained_rhs_bytes,
            temporary_neighbor_pairs_bytes,
            temporary_entries_bytes,
            temporary_row_offsets_bytes,
            temporary_row_neighbors_bytes,
            temporary_index_items_bytes,
            canonicalization_payload_upper_bound_bytes: canonical_upper_bound,
            estimated_retained_bytes: estimated_storage_bytes,
            index_construction_peak_bytes,
            canonicalization_peak_bytes,
            storage_materialization_peak_bytes,
            estimated_peak_bytes,
        };
        let diagnostics = SparseFieldAssemblyDiagnostics {
            system_dimension: centers,
            kernel_entry_evaluations: pairs.len(),
            stored_nonzeros,
            density: density(centers, stored_nonzeros)?,
            maximum_absolute_entry,
            estimated_storage_bytes,
            memory,
            memory_limit_bytes,
            neighborhood: CompactNeighborhoodDiagnostics {
                indexed_terms: neighborhood.indexed_terms,
                candidate_radius: neighborhood.candidate_radius,
                candidate_term_hits,
                supported_pairs: pairs.len(),
                isolated_centers,
                minimum_row_neighbors,
                maximum_row_neighbors,
            },
        };
        let system = SparseFieldSystem {
            canonical,
            execution: self.semantic_problem().execution_options(),
            options,
            matrix,
            rhs,
            center_count: centers,
            diagnostics,
            neighborhood,
        };
        progress.complete()?;
        Ok(system)
    }
}

/// Original-unit residual evidence from a sparse solution.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct SparseResidualDiagnostics {
    /// Exact-product-and-sum residual infinity norm.
    pub original_infinity: f64,
    /// Normwise backward error in original units.
    pub original_backward_error: f64,
}

/// Checked retained and peak logical-memory evidence for sparse solving.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct SparseSolveMemoryDiagnostics {
    /// Complete retained sparse system while the borrowed solve is active.
    pub retained_system_bytes: usize,
    /// Backend-owned checked CSC copy.
    pub backend_matrix_bytes: usize,
    /// Conservative dense lower-triangle factorization payload.
    pub factorization_bytes: usize,
    /// Right-hand sides, solutions, residual work, and exact accumulators.
    pub working_vector_bytes: usize,
    /// Sum of every simultaneously live solve component.
    pub estimated_peak_bytes: usize,
}

/// Complete evidence for one accepted sparse solution.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct SparseSolveDiagnostics {
    /// Conservative checked peak payload including dense worst-case fill.
    pub estimated_peak_memory_bytes: usize,
    /// Checked retained-system and solve-work memory breakdown.
    pub memory: SparseSolveMemoryDiagnostics,
    /// Effective explicit memory limit.
    pub memory_limit_bytes: usize,
    /// Requested factorization.
    pub requested_factorization: SparseFactorization,
    /// Actual factorization; always identical because fallback is forbidden.
    pub actual_factorization: SparseFactorization,
    /// Private backend name.
    pub backend: &'static str,
    /// Exact pinned backend version.
    pub backend_version: &'static str,
    /// Fill-reducing symbolic ordering.
    pub ordering: &'static str,
    /// Input stored nonzeros.
    pub stored_nonzeros: usize,
    /// Accepted original-unit residual evidence.
    pub residual: SparseResidualDiagnostics,
    /// Dimension-derived backward-error tolerance.
    pub residual_tolerance: f64,
}

/// Immutable accepted sparse solution.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct SparseSolution {
    values: Vec<f64>,
    diagnostics: SparseSolveDiagnostics,
}

impl SparseSolution {
    /// Borrows solution values in center order.
    #[must_use]
    pub fn values(&self) -> &[f64] {
        &self.values
    }

    /// Borrows complete sparse solve evidence.
    pub const fn diagnostics(&self) -> &SparseSolveDiagnostics {
        &self.diagnostics
    }

    pub(crate) fn into_parts(self) -> (Vec<f64>, SparseSolveDiagnostics) {
        (self.values, self.diagnostics)
    }
}

/// Structured sparse factorization, solve, memory, or residual failure.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub enum SparseSolveError {
    /// Caller execution policy or cancellation failed.
    Execution(ExecutionError),
    /// Checked peak-memory arithmetic overflowed.
    MemoryEstimateOverflow {
        /// Matrix dimension.
        dimension: usize,
    },
    /// Conservative peak payload exceeded the effective limit.
    MemoryLimitExceeded {
        /// Estimated peak bytes.
        estimated_peak_bytes: usize,
        /// Effective limit.
        limit_bytes: usize,
    },
    /// A checked allocation failed.
    AllocationFailed {
        /// Storage role.
        storage: SparseStorage,
        /// Requested entries.
        requested: usize,
    },
    /// The checked sparse LLT backend rejected the system.
    FactorizationRejected,
    /// The backend returned a nonfinite coefficient.
    NonFiniteSolution {
        /// Solution index.
        index: usize,
        /// Rejected value.
        value: f64,
    },
    /// Exact original-unit residual accumulation was not representable.
    UnrepresentableOriginalResidual {
        /// Residual row.
        row: usize,
    },
    /// Matrix or vector norm accumulation was not representable.
    UnrepresentableNorm,
    /// Original-unit backward error exceeded the fixed tolerance.
    ResidualRejected {
        /// Rejected evidence.
        residual: SparseResidualDiagnostics,
        /// Fixed tolerance.
        tolerance: f64,
    },
}

impl From<ExecutionError> for SparseSolveError {
    fn from(value: ExecutionError) -> Self {
        Self::Execution(value)
    }
}

impl fmt::Display for SparseSolveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Execution(source) => source.fmt(formatter),
            Self::MemoryEstimateOverflow { dimension } => write!(
                formatter,
                "sparse peak-memory estimate overflowed for dimension {dimension}"
            ),
            Self::MemoryLimitExceeded {
                estimated_peak_bytes,
                limit_bytes,
            } => write!(
                formatter,
                "estimated sparse peak payload {estimated_peak_bytes} bytes exceeds explicit limit {limit_bytes} bytes"
            ),
            Self::AllocationFailed { storage, requested } => write!(
                formatter,
                "could not reserve {requested} entries for sparse solve storage {storage:?}"
            ),
            Self::FactorizationRejected => {
                formatter.write_str("checked faer sparse LLT rejected the system")
            }
            Self::NonFiniteSolution { index, value } => {
                write!(
                    formatter,
                    "sparse solution entry {index} is nonfinite: {value}"
                )
            }
            Self::UnrepresentableOriginalResidual { row } => write!(
                formatter,
                "sparse original-unit residual row {row} is not representable"
            ),
            Self::UnrepresentableNorm => {
                formatter.write_str("sparse original-unit norm is not representable")
            }
            Self::ResidualRejected {
                residual,
                tolerance,
            } => write!(
                formatter,
                "sparse original-unit backward error {} exceeds tolerance {tolerance}",
                residual.original_backward_error
            ),
        }
    }
}

impl Error for SparseSolveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Execution(source) => Some(source),
            _ => None,
        }
    }
}

/// Solves one compact sparse field through checked faer LLT.
///
/// # Errors
///
/// Returns structured execution, memory, factorization, finite-solution, or
/// original-unit residual failures.
pub fn try_solve_sparse_field<const D: usize>(
    system: &SparseFieldSystem<D>,
) -> Result<SparseSolution, SparseSolveError>
where
    Dim<D>: SupportedDimension,
{
    try_solve_sparse_field_with_control(system, ExecutionControl::default())
}

/// Solves one compact sparse field with borrowed cancellation and progress.
///
/// # Errors
///
/// Returns the same failures as [`try_solve_sparse_field`], including
/// cancellation and unsupported explicit thread-count failures.
#[allow(clippy::too_many_lines)]
pub fn try_solve_sparse_field_with_control<const D: usize>(
    system: &SparseFieldSystem<D>,
    control: ExecutionControl<'_>,
) -> Result<SparseSolution, SparseSolveError>
where
    Dim<D>: SupportedDimension,
{
    let mut progress = ProgressTracker::try_new(
        control,
        ExecutionOperation::SparseSolve,
        system.execution,
        4,
    )?;
    let dimension = system.matrix.dimension;
    let limit = effective_memory_limit(system.execution, system.options);
    let memory = estimate_sparse_peak_bytes(system)?;
    let estimated_peak_memory_bytes = memory.estimated_peak_bytes;
    if estimated_peak_memory_bytes > limit {
        return Err(SparseSolveError::MemoryLimitExceeded {
            estimated_peak_bytes: estimated_peak_memory_bytes,
            limit_bytes: limit,
        });
    }
    progress.advance(ExecutionStage::MemoryReview)?;

    let column_pointers = try_copy_usize(
        &system.matrix.column_pointers,
        SparseStorage::BackendIndices,
    )?;
    let row_indices = try_copy_usize(&system.matrix.row_indices, SparseStorage::BackendIndices)?;
    let values = try_copy_f64(&system.matrix.values, SparseStorage::BackendValues)?;
    let symbolic = SymbolicSparseColMat::<usize>::new_checked(
        dimension,
        dimension,
        column_pointers,
        None,
        row_indices,
    );
    let matrix = SparseColMat::new(symbolic, values);
    let factor = progress.finish_work(
        ExecutionStage::Factorization,
        matrix
            .sp_cholesky(Side::Lower)
            .map_err(|_| SparseSolveError::FactorizationRejected),
    )?;
    let right_hand_side = Col::from_fn(dimension, |row| system.rhs[row]);
    let backend_solution = progress.finish_work(
        ExecutionStage::BackendSolve,
        Ok::<_, SparseSolveError>(factor.solve(&right_hand_side)),
    )?;
    let mut values = try_with_capacity(dimension, SparseStorage::Solution, solve_allocation)?;
    for index in 0..dimension {
        let value = backend_solution[index];
        if !value.is_finite() {
            return Err(SparseSolveError::NonFiniteSolution { index, value });
        }
        values.push(value);
    }

    let residual = sparse_residual(&system.matrix, &system.rhs, &values)?;
    let dimension_u32 = u32::try_from(dimension)
        .map_err(|_| SparseSolveError::MemoryEstimateOverflow { dimension })?;
    let residual_tolerance = RESIDUAL_TOLERANCE_FACTOR * f64::from(dimension_u32) * f64::EPSILON;
    if residual.original_backward_error > residual_tolerance {
        return Err(SparseSolveError::ResidualRejected {
            residual,
            tolerance: residual_tolerance,
        });
    }
    progress.advance(ExecutionStage::ResidualReview)?;
    let diagnostics = SparseSolveDiagnostics {
        estimated_peak_memory_bytes,
        memory,
        memory_limit_bytes: limit,
        requested_factorization: system.options.factorization,
        actual_factorization: system.options.factorization,
        backend: BACKEND_NAME,
        backend_version: BACKEND_VERSION,
        ordering: ORDERING_NAME,
        stored_nonzeros: system.matrix.values.len(),
        residual,
        residual_tolerance,
    };
    progress.complete()?;
    Ok(SparseSolution {
        values,
        diagnostics,
    })
}

#[derive(Debug)]
enum SeparationError<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    Calculus(KernelCalculusError),
    Anisotropy(AnisotropyError<D>),
}

fn exact_radius<const D: usize>(
    query: Point<D>,
    center: Point<D>,
    anisotropy: Option<&GlobalAnisotropy<D>>,
) -> Result<f64, SeparationError<D>>
where
    Dim<D>: SupportedDimension,
{
    match anisotropy {
        Some(anisotropy) => anisotropy
            .try_transform_separation(query, center)
            .map(|separation| separation.radius())
            .map_err(SeparationError::Anisotropy),
        None => RadialSeparation::try_new(query, center)
            .map(|separation| separation.radius())
            .map_err(SeparationError::Calculus),
    }
}

fn candidate_radius<const D: usize>(
    support_radius: f64,
    anisotropy: Option<&GlobalAnisotropy<D>>,
) -> Result<f64, SparseFieldAssemblyError<D>>
where
    Dim<D>: SupportedDimension,
{
    let factor = anisotropy.map_or(1.0, |anisotropy| {
        let maximum = anisotropy
            .inverse_transform()
            .iter()
            .flatten()
            .copied()
            .map(f64::abs)
            .fold(0.0_f64, f64::max);
        (maximum * dimension_as_f64::<D>()).next_up()
    });
    let radius = (support_radius * factor).next_up();
    if radius.is_finite() && radius > 0.0 {
        Ok(radius)
    } else {
        Err(SparseFieldAssemblyError::UnrepresentableCandidateRadius { support_radius })
    }
}

fn lift_point<const D: usize>(point: Point<D>) -> [f64; 3]
where
    Dim<D>: SupportedDimension,
{
    std::array::from_fn(|axis| point.components().get(axis).copied().unwrap_or(0.0))
}

const fn dimension_as_f64<const D: usize>() -> f64 {
    match D {
        1 => 1.0,
        2 => 2.0,
        3 => 3.0,
        _ => 0.0,
    }
}

fn effective_memory_limit(execution: ExecutionOptions, options: SparseFitOptions) -> usize {
    execution
        .memory_limit_bytes()
        .map_or(options.memory_limit_bytes.get(), |execution_limit| {
            execution_limit.get().min(options.memory_limit_bytes.get())
        })
}

fn enforce_assembly_limit<const D: usize>(
    estimated_bytes: usize,
    limit_bytes: usize,
) -> Result<(), SparseFieldAssemblyError<D>>
where
    Dim<D>: SupportedDimension,
{
    if estimated_bytes > limit_bytes {
        Err(SparseFieldAssemblyError::MemoryLimitExceeded {
            estimated_bytes,
            limit_bytes,
        })
    } else {
        Ok(())
    }
}

fn assembly_allocation<const D: usize>(
    storage: SparseStorage,
    requested: usize,
) -> SparseFieldAssemblyError<D>
where
    Dim<D>: SupportedDimension,
{
    SparseFieldAssemblyError::AllocationFailed { storage, requested }
}

const fn solve_allocation(storage: SparseStorage, requested: usize) -> SparseSolveError {
    SparseSolveError::AllocationFailed { storage, requested }
}

fn try_with_capacity<T, E>(
    requested: usize,
    storage: SparseStorage,
    allocation_error: impl FnOnce(SparseStorage, usize) -> E,
) -> Result<Vec<T>, E> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(requested)
        .map_err(|_| allocation_error(storage, requested))?;
    Ok(values)
}

fn try_filled<T: Clone, E>(
    requested: usize,
    value: T,
    storage: SparseStorage,
    allocation_error: impl FnOnce(SparseStorage, usize) -> E,
) -> Result<Vec<T>, E> {
    let mut values = try_with_capacity(requested, storage, allocation_error)?;
    values.resize(requested, value);
    Ok(values)
}

fn try_push_limited<const D: usize, T>(
    values: &mut Vec<T>,
    value: T,
    storage: SparseStorage,
    memory_limit_bytes: usize,
    fixed_live_bytes: usize,
    checked_peak_bytes: &mut usize,
) -> Result<(), SparseFieldAssemblyError<D>>
where
    Dim<D>: SupportedDimension,
{
    let requested = values
        .len()
        .checked_add(1)
        .ok_or(SparseFieldAssemblyError::CountOverflow)?;
    if values.len() == values.capacity() {
        let target_capacity = values
            .capacity()
            .max(1)
            .checked_mul(2)
            .map(|capacity| capacity.max(requested))
            .ok_or(SparseFieldAssemblyError::CountOverflow)?;
        observe_assembly_peak(
            checked_peak_bytes,
            &[fixed_live_bytes, capacity_bytes::<T, D>(target_capacity)?],
            memory_limit_bytes,
        )?;
        values
            .try_reserve_exact(target_capacity - values.len())
            .map_err(|_| SparseFieldAssemblyError::AllocationFailed { storage, requested })?;
    }
    observe_assembly_peak(
        checked_peak_bytes,
        &[fixed_live_bytes, capacity_bytes::<T, D>(values.capacity())?],
        memory_limit_bytes,
    )?;
    values.push(value);
    Ok(())
}

fn capacity_bytes<T, const D: usize>(capacity: usize) -> Result<usize, SparseFieldAssemblyError<D>>
where
    Dim<D>: SupportedDimension,
{
    capacity
        .checked_mul(size_of::<T>())
        .ok_or(SparseFieldAssemblyError::CountOverflow)
}

fn checked_sum_assembly<const D: usize>(
    components: &[usize],
) -> Result<usize, SparseFieldAssemblyError<D>>
where
    Dim<D>: SupportedDimension,
{
    components.iter().try_fold(0_usize, |total, component| {
        total
            .checked_add(*component)
            .ok_or(SparseFieldAssemblyError::CountOverflow)
    })
}

fn observe_assembly_peak<const D: usize>(
    checked_peak_bytes: &mut usize,
    components: &[usize],
    memory_limit_bytes: usize,
) -> Result<(), SparseFieldAssemblyError<D>>
where
    Dim<D>: SupportedDimension,
{
    let live_bytes = checked_sum_assembly::<D>(components)?;
    enforce_assembly_limit(live_bytes, memory_limit_bytes)?;
    *checked_peak_bytes = (*checked_peak_bytes).max(live_bytes);
    Ok(())
}

fn canonical_equality_payload_upper_bound<const D: usize>(
    problem: &FieldProblem<D>,
    dimension: usize,
    nonzeros: usize,
) -> Result<usize, SparseFieldAssemblyError<D>>
where
    Dim<D>: SupportedDimension,
{
    let relation_capacity = dimension
        .checked_next_power_of_two()
        .ok_or(SparseFieldAssemblyError::CountOverflow)?
        .max(4);
    let relation_item_bytes = size_of::<CanonicalEquality>()
        .checked_add(size_of::<CanonicalLinearBound>())
        .and_then(|bytes| bytes.checked_add(size_of::<CanonicalSecondOrderCone>()))
        .and_then(|bytes| bytes.checked_add(size_of::<CanonicalSoftObjective>()))
        .ok_or(SparseFieldAssemblyError::CountOverflow)?;
    let mut bytes = relation_capacity
        .checked_mul(relation_item_bytes)
        .and_then(|part| {
            nonzeros
                .checked_mul(size_of::<AffineTerm>())
                .and_then(|terms| part.checked_add(terms))
        })
        .and_then(|total| {
            size_of::<VariableBlock>()
                .checked_mul(2)
                .and_then(|blocks| total.checked_add(blocks))
        })
        .and_then(|total| total.checked_add(size_of::<usize>()))
        .and_then(|total| total.checked_add("center_weights".len()))
        .and_then(|total| {
            dimension
                .checked_mul(2)
                .and_then(|count| count.checked_mul(size_of::<f64>()))
                .and_then(|scaling| total.checked_add(scaling))
        })
        .ok_or(SparseFieldAssemblyError::CountOverflow)?;
    for constraint in problem.semantic_problem().constraints() {
        let provenance = constraint.provenance();
        bytes = bytes
            .checked_add(provenance.source().path().len())
            .and_then(|total| total.checked_add(provenance.original_units().len()))
            .and_then(|total| total.checked_add(provenance.field_path().len()))
            .and_then(|total| total.checked_add(provenance.constraint_group().map_or(0, str::len)))
            .ok_or(SparseFieldAssemblyError::CountOverflow)?;
    }
    Ok(bytes)
}

fn row_offsets<const D: usize>(
    dimension: usize,
    entries: &[SparseEntry],
) -> Result<Vec<usize>, SparseFieldAssemblyError<D>>
where
    Dim<D>: SupportedDimension,
{
    let mut offsets = try_filled(
        dimension
            .checked_add(1)
            .ok_or(SparseFieldAssemblyError::CountOverflow)?,
        0_usize,
        SparseStorage::RowOffsets,
        assembly_allocation::<D>,
    )?;
    for entry in entries {
        offsets[entry.row + 1] = offsets[entry.row + 1]
            .checked_add(1)
            .ok_or(SparseFieldAssemblyError::CountOverflow)?;
    }
    for row in 0..dimension {
        offsets[row + 1] = offsets[row + 1]
            .checked_add(offsets[row])
            .ok_or(SparseFieldAssemblyError::CountOverflow)?;
    }
    Ok(offsets)
}

fn review_exact_symmetry<const D: usize>(
    dimension: usize,
    entries: &[SparseEntry],
) -> Result<(), SparseFieldAssemblyError<D>>
where
    Dim<D>: SupportedDimension,
{
    for entry in entries {
        if entry.row >= dimension || entry.column >= dimension || !entry.value.is_finite() {
            return Err(SparseFieldAssemblyError::NotSymmetric {
                row: entry.row,
                column: entry.column,
            });
        }
        let reverse = entries.binary_search_by_key(&(entry.column, entry.row), |candidate| {
            (candidate.row, candidate.column)
        });
        if reverse
            .ok()
            .and_then(|index| entries.get(index))
            .is_none_or(|candidate| candidate.value.to_bits() != entry.value.to_bits())
        {
            return Err(SparseFieldAssemblyError::NotSymmetric {
                row: entry.row,
                column: entry.column,
            });
        }
    }
    Ok(())
}

fn materialize_csc<const D: usize>(
    dimension: usize,
    entries: Vec<SparseEntry>,
) -> Result<SparseFieldMatrix, SparseFieldAssemblyError<D>>
where
    Dim<D>: SupportedDimension,
{
    let pointer_count = dimension
        .checked_add(1)
        .ok_or(SparseFieldAssemblyError::CountOverflow)?;
    let mut column_pointers = try_filled(
        pointer_count,
        0_usize,
        SparseStorage::ColumnPointers,
        assembly_allocation::<D>,
    )?;
    let mut row_indices = try_with_capacity(
        entries.len(),
        SparseStorage::RowIndices,
        assembly_allocation::<D>,
    )?;
    let mut values = try_with_capacity(
        entries.len(),
        SparseStorage::Values,
        assembly_allocation::<D>,
    )?;
    let mut previous = None;
    for entry in entries {
        if previous.is_some_and(|key| key >= (entry.column, entry.row)) {
            return Err(SparseFieldAssemblyError::NotSymmetric {
                row: entry.row,
                column: entry.column,
            });
        }
        column_pointers[entry.column + 1] = column_pointers[entry.column + 1]
            .checked_add(1)
            .ok_or(SparseFieldAssemblyError::CountOverflow)?;
        row_indices.push(entry.row);
        values.push(entry.value);
        previous = Some((entry.column, entry.row));
    }
    for column in 0..dimension {
        column_pointers[column + 1] = column_pointers[column + 1]
            .checked_add(column_pointers[column])
            .ok_or(SparseFieldAssemblyError::CountOverflow)?;
    }
    Ok(SparseFieldMatrix {
        dimension,
        column_pointers,
        row_indices,
        values,
    })
}

fn matrix_payload_bytes<const D: usize>(
    dimension: usize,
    nonzeros: usize,
) -> Result<usize, SparseFieldAssemblyError<D>>
where
    Dim<D>: SupportedDimension,
{
    dimension
        .checked_add(1)
        .and_then(|count| count.checked_mul(size_of::<usize>()))
        .and_then(|bytes| {
            nonzeros
                .checked_mul(size_of::<usize>() + size_of::<f64>())
                .and_then(|part| bytes.checked_add(part))
        })
        .ok_or(SparseFieldAssemblyError::CountOverflow)
}

fn matrix_capacity_bytes<const D: usize>(
    matrix: &SparseFieldMatrix,
) -> Result<usize, SparseFieldAssemblyError<D>>
where
    Dim<D>: SupportedDimension,
{
    checked_sum_assembly::<D>(&[
        capacity_bytes::<usize, D>(matrix.column_pointers.capacity())?,
        capacity_bytes::<usize, D>(matrix.row_indices.capacity())?,
        capacity_bytes::<f64, D>(matrix.values.capacity())?,
    ])
}

fn retained_index_payload_bytes<const D: usize>(
    indexed_terms: usize,
) -> Result<usize, SparseFieldAssemblyError<D>>
where
    Dim<D>: SupportedDimension,
{
    // A bulk-loaded tree has one leaf per item and fewer than one non-root
    // parent per leaf. Two node slots per item plus the root parent therefore
    // bound the retained logical payload without relying on rstar internals.
    indexed_terms
        .checked_mul(2)
        .and_then(|nodes| nodes.checked_mul(size_of::<RTreeNode<IndexedAtom>>()))
        .and_then(|bytes| bytes.checked_add(size_of::<ParentNode<IndexedAtom>>()))
        .ok_or(SparseFieldAssemblyError::CountOverflow)
}

fn index_construction_peak_bytes<const D: usize>(
    indexed_terms: usize,
) -> Result<usize, SparseFieldAssemblyError<D>>
where
    Dim<D>: SupportedDimension,
{
    retained_index_payload_bytes::<D>(indexed_terms)?
        .checked_add(
            indexed_terms
                .checked_mul(size_of::<IndexedAtom>())
                .ok_or(SparseFieldAssemblyError::CountOverflow)?,
        )
        .ok_or(SparseFieldAssemblyError::CountOverflow)
}

fn density<const D: usize>(
    dimension: usize,
    nonzeros: usize,
) -> Result<f64, SparseFieldAssemblyError<D>>
where
    Dim<D>: SupportedDimension,
{
    let dimension_u32 =
        u32::try_from(dimension).map_err(|_| SparseFieldAssemblyError::CountOverflow)?;
    let nonzeros_u32 =
        u32::try_from(nonzeros).map_err(|_| SparseFieldAssemblyError::CountOverflow)?;
    let dense = f64::from(dimension_u32) * f64::from(dimension_u32);
    Ok(f64::from(nonzeros_u32) / dense)
}

fn estimate_sparse_peak_bytes<const D: usize>(
    system: &SparseFieldSystem<D>,
) -> Result<SparseSolveMemoryDiagnostics, SparseSolveError>
where
    Dim<D>: SupportedDimension,
{
    let dimension = system.matrix.dimension;
    let nonzeros = system.matrix.values.len();
    let overflow = || SparseSolveError::MemoryEstimateOverflow { dimension };
    let backend_matrix_bytes = dimension
        .checked_add(1)
        .and_then(|count| count.checked_mul(size_of::<usize>()))
        .and_then(|bytes| {
            nonzeros
                .checked_mul(size_of::<usize>() + size_of::<f64>())
                .and_then(|part| bytes.checked_add(part))
        })
        .ok_or_else(overflow)?;
    let factorization_bytes = dimension
        .checked_mul(dimension.checked_add(1).ok_or_else(overflow)?)
        .and_then(|count| count.checked_div(2))
        .and_then(|count| count.checked_mul(size_of::<usize>() + size_of::<f64>()))
        .ok_or_else(overflow)?;
    let working_vector_bytes = dimension
        .checked_mul(
            6_usize
                .checked_mul(size_of::<f64>())
                .and_then(|bytes| bytes.checked_add(size_of::<ExactDotAccumulator>()))
                .ok_or_else(overflow)?,
        )
        .ok_or_else(overflow)?;
    let retained_system_bytes = system.diagnostics.estimated_storage_bytes;
    let estimated_peak_bytes = retained_system_bytes
        .checked_add(backend_matrix_bytes)
        .and_then(|bytes| bytes.checked_add(factorization_bytes))
        .and_then(|bytes| bytes.checked_add(working_vector_bytes))
        .ok_or_else(overflow)?;
    Ok(SparseSolveMemoryDiagnostics {
        retained_system_bytes,
        backend_matrix_bytes,
        factorization_bytes,
        working_vector_bytes,
        estimated_peak_bytes,
    })
}

fn try_copy_usize(
    source: &[usize],
    storage: SparseStorage,
) -> Result<Vec<usize>, SparseSolveError> {
    let mut copied = try_with_capacity(source.len(), storage, solve_allocation)?;
    copied.extend_from_slice(source);
    Ok(copied)
}

fn try_copy_f64(source: &[f64], storage: SparseStorage) -> Result<Vec<f64>, SparseSolveError> {
    let mut copied = try_with_capacity(source.len(), storage, solve_allocation)?;
    copied.extend_from_slice(source);
    Ok(copied)
}

fn sparse_residual(
    matrix: &SparseFieldMatrix,
    rhs: &[f64],
    solution: &[f64],
) -> Result<SparseResidualDiagnostics, SparseSolveError> {
    let dimension = matrix.dimension;
    let mut accumulators = try_filled(
        dimension,
        ExactDotAccumulator::default(),
        SparseStorage::ResidualAccumulators,
        solve_allocation,
    )?;
    let mut row_sums = try_filled(
        dimension,
        0.0_f64,
        SparseStorage::ResidualRowSums,
        solve_allocation,
    )?;
    for row in 0..dimension {
        accumulators[row]
            .try_add_product(rhs[row], 1.0)
            .ok_or(SparseSolveError::UnrepresentableOriginalResidual { row })?;
    }
    for (column, solution_value) in solution.iter().copied().enumerate().take(dimension) {
        for position in matrix.column_pointers[column]..matrix.column_pointers[column + 1] {
            let row = matrix.row_indices[position];
            let value = matrix.values[position];
            accumulators[row]
                .try_add_product(-value, solution_value)
                .ok_or(SparseSolveError::UnrepresentableOriginalResidual { row })?;
            row_sums[row] += value.abs();
            if !row_sums[row].is_finite() {
                return Err(SparseSolveError::UnrepresentableNorm);
            }
        }
    }
    let mut residual_infinity = 0.0_f64;
    for (row, accumulator) in accumulators.iter().enumerate() {
        residual_infinity = residual_infinity.max(
            accumulator
                .try_abs_f64()
                .ok_or(SparseSolveError::UnrepresentableOriginalResidual { row })?,
        );
    }
    let matrix_infinity = row_sums.into_iter().fold(0.0_f64, f64::max);
    let solution_infinity = solution
        .iter()
        .copied()
        .map(f64::abs)
        .fold(0.0_f64, f64::max);
    let rhs_infinity = rhs.iter().copied().map(f64::abs).fold(0.0_f64, f64::max);
    let denominator = matrix_infinity.mul_add(solution_infinity, rhs_infinity);
    if !denominator.is_finite() {
        return Err(SparseSolveError::UnrepresentableNorm);
    }
    let original_backward_error = if denominator == 0.0 {
        residual_infinity
    } else {
        residual_infinity / denominator
    };
    if !original_backward_error.is_finite() {
        return Err(SparseSolveError::UnrepresentableNorm);
    }
    Ok(SparseResidualDiagnostics {
        original_infinity: residual_infinity,
        original_backward_error,
    })
}
