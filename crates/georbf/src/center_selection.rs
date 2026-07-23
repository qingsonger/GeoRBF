//! Deterministic, rank-safe selection from an explicit candidate basis.
//!
//! Selection is deliberately separate from semantic constraints, field
//! assembly, and fitting. A selection returns stable candidate indices; it
//! never removes observations, relaxes a hard relation, or refits a model.
//! The current atomic capability is explicitly limited to strictly
//! positive-definite kernels; CPD input is rejected before selection because
//! it requires polynomial actions and projected-positive review.

use std::error::Error;
use std::fmt;
use std::num::NonZeroUsize;

use crate::dimension::{Dim, SupportedDimension};
use crate::geometry::Point;
use crate::kernel::{CpdOrder, KernelDefiniteness};
use crate::solver::{
    ConditionPolicy, DenseEqualitySystem, DenseEqualitySystemError, DenseFactorization,
    DenseRankDiagnostics, DenseSolveError, DenseSolveOptions, DenseSolverConfigurationError,
    Regularization,
};

/// The implemented center-selection algorithm.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub enum CenterSelectionKind {
    /// Retain every candidate in input order.
    AllRepresenters,
    /// Retain an explicit ordered set of candidate indices.
    UserProvided,
    /// Greedily maximize distance from the selected set.
    FarthestPoint,
    /// Greedily maximize the current interpolation residual.
    ResidualGreedy,
    /// Greedily maximize the current squared power function.
    PowerGreedy,
}

/// One deterministic center-selection request.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub enum CenterSelectionStrategy {
    /// Retain every candidate in input order.
    AllRepresenters,
    /// Retain the supplied unique indices in the exact supplied order.
    UserProvided(Vec<usize>),
    /// Select `count` candidates by farthest-point traversal.
    FarthestPoint {
        /// Required nonzero selected count.
        count: NonZeroUsize,
        /// Reproducible first-candidate and tie-breaking seed.
        seed: u64,
    },
    /// Select `count` candidates by interpolation-residual magnitude.
    ResidualGreedy {
        /// Required nonzero selected count.
        count: NonZeroUsize,
        /// Reproducible tie-breaking seed.
        seed: u64,
    },
    /// Select `count` candidates by squared power.
    PowerGreedy {
        /// Required nonzero selected count.
        count: NonZeroUsize,
        /// Reproducible tie-breaking seed.
        seed: u64,
    },
}

impl CenterSelectionStrategy {
    /// Returns the algorithm classification.
    pub const fn kind(&self) -> CenterSelectionKind {
        match self {
            Self::AllRepresenters => CenterSelectionKind::AllRepresenters,
            Self::UserProvided(_) => CenterSelectionKind::UserProvided,
            Self::FarthestPoint { .. } => CenterSelectionKind::FarthestPoint,
            Self::ResidualGreedy { .. } => CenterSelectionKind::ResidualGreedy,
            Self::PowerGreedy { .. } => CenterSelectionKind::PowerGreedy,
        }
    }

    /// Returns the deterministic seed when the strategy accepts one.
    #[must_use]
    pub const fn seed(&self) -> Option<u64> {
        match self {
            Self::FarthestPoint { seed, .. }
            | Self::ResidualGreedy { seed, .. }
            | Self::PowerGreedy { seed, .. } => Some(*seed),
            Self::AllRepresenters | Self::UserProvided(_) => None,
        }
    }
}

/// Explicit execution policy for one center selection.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct CenterSelectionOptions {
    strategy: CenterSelectionStrategy,
    memory_limit_bytes: NonZeroUsize,
}

impl CenterSelectionOptions {
    /// Constructs a request with an explicit nonzero numerical-review memory limit.
    pub const fn new(strategy: CenterSelectionStrategy, memory_limit_bytes: NonZeroUsize) -> Self {
        Self {
            strategy,
            memory_limit_bytes,
        }
    }

    /// Borrows the requested strategy.
    pub const fn strategy(&self) -> &CenterSelectionStrategy {
        &self.strategy
    }

    /// Returns the explicit numerical-review memory limit.
    #[must_use]
    pub const fn memory_limit_bytes(&self) -> NonZeroUsize {
        self.memory_limit_bytes
    }
}

/// Immutable candidate locations, basis Gram matrix, and target residuals.
///
/// `gram` is row-major with shape `candidate_count * candidate_count`.
/// Residual-greedy interprets `targets` as the initial interpolation residual.
/// All other strategies retain the targets only so one candidate problem can
/// be compared across strategies.
///
/// ```
/// use std::num::NonZeroUsize;
///
/// use georbf::{
///     CenterSelectionOptions, CenterSelectionProblem, CenterSelectionStrategy,
///     KernelDefiniteness, Point,
/// };
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let problem = CenterSelectionProblem::try_from_row_major(
///     KernelDefiniteness::StrictlyPositiveDefinite,
///     vec![
///         Point::try_new([0.0])?,
///         Point::try_new([1.0])?,
///         Point::try_new([3.0])?,
///     ],
///     vec![
///         1.0, 0.25, 0.01,
///         0.25, 1.0, 0.1,
///         0.01, 0.1, 1.0,
///     ],
///     vec![0.5, -2.0, 1.0],
/// )?;
/// let options = CenterSelectionOptions::new(
///     CenterSelectionStrategy::ResidualGreedy {
///         count: NonZeroUsize::new(2).ok_or("count")?,
///         seed: 7,
///     },
///     NonZeroUsize::new(16 * 1024 * 1024).ok_or("memory limit")?,
/// );
/// let selection = problem.try_select(&options)?;
/// assert_eq!(selection.indices()[0], 1);
/// assert_eq!(selection.indices().len(), 2);
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct CenterSelectionProblem<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    locations: Vec<Point<D>>,
    gram: Vec<f64>,
    targets: Vec<f64>,
}

impl<const D: usize> CenterSelectionProblem<D>
where
    Dim<D>: SupportedDimension,
{
    /// Validates and owns one explicit candidate-selection problem.
    ///
    /// # Errors
    ///
    /// Returns a structured error for empty candidates, shape overflow,
    /// length mismatch, nonfinite values, or a matrix that is not exactly
    /// symmetric. This atomic selector supports only kernels declared strictly
    /// positive definite. A conditionally positive-definite declaration is
    /// rejected at this typed boundary because selection would additionally
    /// require complete polynomial actions and projected-positive review.
    /// Positive definiteness is reviewed on the selected principal matrix; it
    /// is never fabricated by jitter or diagonal substitution.
    #[allow(clippy::float_cmp)]
    pub fn try_from_row_major(
        definiteness: KernelDefiniteness,
        locations: Vec<Point<D>>,
        gram: Vec<f64>,
        targets: Vec<f64>,
    ) -> Result<Self, CenterSelectionError> {
        if let KernelDefiniteness::ConditionallyPositiveDefinite { order } = definiteness {
            return Err(CenterSelectionError::ConditionallyPositiveDefiniteUnsupported { order });
        }
        let candidates = locations.len();
        if candidates == 0 {
            return Err(CenterSelectionError::EmptyCandidates);
        }
        let entries = candidates
            .checked_mul(candidates)
            .ok_or(CenterSelectionError::ShapeOverflow { candidates })?;
        if gram.len() != entries {
            return Err(CenterSelectionError::GramLengthMismatch {
                candidates,
                expected: entries,
                actual: gram.len(),
            });
        }
        if targets.len() != candidates {
            return Err(CenterSelectionError::TargetLengthMismatch {
                candidates,
                actual: targets.len(),
            });
        }
        for (index, value) in gram.iter().copied().enumerate() {
            if !value.is_finite() {
                return Err(CenterSelectionError::NonFiniteGram { index, value });
            }
        }
        for (index, value) in targets.iter().copied().enumerate() {
            if !value.is_finite() {
                return Err(CenterSelectionError::NonFiniteTarget { index, value });
            }
        }
        for row in 0..candidates {
            for column in row + 1..candidates {
                let upper = gram[row * candidates + column];
                let lower = gram[column * candidates + row];
                if upper != lower {
                    return Err(CenterSelectionError::GramNotExactlySymmetric {
                        row,
                        column,
                        upper,
                        lower,
                    });
                }
            }
        }
        Ok(Self {
            locations,
            gram,
            targets,
        })
    }

    /// Returns the only definiteness classification supported by this problem.
    #[must_use]
    pub const fn definiteness(&self) -> KernelDefiniteness {
        KernelDefiniteness::StrictlyPositiveDefinite
    }

    /// Returns the number of candidates.
    #[must_use]
    pub fn candidate_count(&self) -> usize {
        self.locations.len()
    }

    /// Borrows candidate locations in stable input order.
    pub fn locations(&self) -> &[Point<D>] {
        &self.locations
    }

    /// Borrows the exact symmetric row-major candidate Gram matrix.
    #[must_use]
    pub fn gram(&self) -> &[f64] {
        &self.gram
    }

    /// Borrows initial target residuals in candidate order.
    #[must_use]
    pub fn targets(&self) -> &[f64] {
        &self.targets
    }

    /// Selects centers without mutating this candidate problem.
    ///
    /// # Errors
    ///
    /// Returns a structured strategy, allocation, arithmetic, basis-rank, or
    /// numerical-review error. Every success has passed the existing
    /// eight-pass RRQR and bounded-SVD rank review plus checked Cholesky on the
    /// selected principal Gram matrix.
    pub fn try_select(
        &self,
        options: &CenterSelectionOptions,
    ) -> Result<CenterSelection, CenterSelectionError> {
        let candidates = self.candidate_count();
        let kind = options.strategy.kind();
        let seed = options.strategy.seed();
        let mut greedy = None;
        let indices = match &options.strategy {
            CenterSelectionStrategy::AllRepresenters => {
                let mut indices = Vec::new();
                try_reserve_exact(&mut indices, candidates, CenterSelectionStorage::Indices)?;
                indices.extend(0..candidates);
                indices
            }
            CenterSelectionStrategy::UserProvided(indices) => {
                validate_user_indices(indices, candidates)?;
                try_copy(indices, CenterSelectionStorage::Indices)?
            }
            CenterSelectionStrategy::FarthestPoint { count, seed } => {
                validate_count(count.get(), candidates)?;
                farthest_point(self.locations(), count.get(), *seed)?
            }
            CenterSelectionStrategy::ResidualGreedy { count, seed } => {
                validate_count(count.get(), candidates)?;
                let outcome = kernel_greedy(
                    &self.gram,
                    &self.targets,
                    candidates,
                    count.get(),
                    *seed,
                    GreedyScore::Residual,
                )?;
                greedy = Some(outcome.diagnostics);
                outcome.indices
            }
            CenterSelectionStrategy::PowerGreedy { count, seed } => {
                validate_count(count.get(), candidates)?;
                let outcome = kernel_greedy(
                    &self.gram,
                    &self.targets,
                    candidates,
                    count.get(),
                    *seed,
                    GreedyScore::Power,
                )?;
                greedy = Some(outcome.diagnostics);
                outcome.indices
            }
        };

        let rank =
            review_selected_basis(&self.gram, candidates, &indices, options.memory_limit_bytes)?;
        Ok(CenterSelection {
            indices,
            diagnostics: CenterSelectionDiagnostics {
                kind,
                candidate_count: candidates,
                selected_count: rank.dimension,
                seed,
                greedy,
                rank,
            },
        })
    }
}

/// Greedy-update evidence retained by residual- and power-greedy selection.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct CenterGreedyDiagnostics {
    /// Smallest candidate-local threshold `n * epsilon * abs(K_ii)` applied.
    pub minimum_pivot_threshold: f64,
    /// Largest candidate-local threshold `n * epsilon * abs(K_ii)` applied.
    pub maximum_pivot_threshold: f64,
    /// Smallest strictly accepted squared pivot.
    pub minimum_accepted_pivot: f64,
    /// Largest strictly accepted squared pivot.
    pub maximum_accepted_pivot: f64,
}

/// Complete deterministic selection and final numerical-rank evidence.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct CenterSelectionDiagnostics {
    /// Algorithm used.
    pub kind: CenterSelectionKind,
    /// Total candidate count.
    pub candidate_count: usize,
    /// Returned selected count.
    pub selected_count: usize,
    /// Explicit deterministic seed when applicable.
    pub seed: Option<u64>,
    /// Greedy pivot evidence when applicable.
    pub greedy: Option<CenterGreedyDiagnostics>,
    /// Existing scale-aware RRQR/SVD rank evidence for the selected Gram matrix.
    pub rank: DenseRankDiagnostics,
}

/// Stable selected candidate indices plus complete diagnostics.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct CenterSelection {
    indices: Vec<usize>,
    diagnostics: CenterSelectionDiagnostics,
}

impl CenterSelection {
    /// Borrows selected indices in deterministic selection order.
    #[must_use]
    pub fn indices(&self) -> &[usize] {
        &self.indices
    }

    /// Borrows complete selection and basis-rank diagnostics.
    pub const fn diagnostics(&self) -> &CenterSelectionDiagnostics {
        &self.diagnostics
    }
}

/// Fallible storage role used during selection.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub enum CenterSelectionStorage {
    /// Stable selected indices.
    Indices,
    /// Selected-membership flags.
    Membership,
    /// Farthest-point nearest-distance work.
    Distances,
    /// Newton/Cholesky columns.
    NewtonBasis,
    /// Current interpolation residual.
    Residual,
    /// Final selected principal matrix.
    SelectedGram,
    /// Final numerical-review right-hand side.
    ReviewRightHandSide,
}

/// Failure while validating, selecting, or rank-reviewing a center basis.
#[derive(Debug)]
#[non_exhaustive]
pub enum CenterSelectionError {
    /// CPD selection requires polynomial actions and projected-positive review.
    ConditionallyPositiveDefiniteUnsupported {
        /// Declared positive CPD order.
        order: CpdOrder,
    },
    /// No candidate was supplied.
    EmptyCandidates,
    /// Candidate-square shape arithmetic overflowed.
    ShapeOverflow {
        /// Candidate count.
        candidates: usize,
    },
    /// Row-major Gram storage has the wrong length.
    GramLengthMismatch {
        /// Candidate count.
        candidates: usize,
        /// Required entry count.
        expected: usize,
        /// Supplied entry count.
        actual: usize,
    },
    /// Target residuals do not align with candidates.
    TargetLengthMismatch {
        /// Candidate count.
        candidates: usize,
        /// Supplied target count.
        actual: usize,
    },
    /// One Gram entry is NaN or infinite.
    NonFiniteGram {
        /// Row-major entry index.
        index: usize,
        /// Rejected value.
        value: f64,
    },
    /// One target residual is NaN or infinite.
    NonFiniteTarget {
        /// Target index.
        index: usize,
        /// Rejected value.
        value: f64,
    },
    /// Symmetric Gram entries differ exactly.
    GramNotExactlySymmetric {
        /// Upper-triangle row.
        row: usize,
        /// Upper-triangle column.
        column: usize,
        /// Upper-triangle value.
        upper: f64,
        /// Reflected lower-triangle value.
        lower: f64,
    },
    /// A strategy requested more candidates than exist.
    CountExceedsCandidates {
        /// Requested selected count.
        requested: usize,
        /// Available candidate count.
        candidates: usize,
    },
    /// The explicit user selection was empty.
    EmptyUserSelection,
    /// A user-provided index is outside the candidate range.
    UserIndexOutOfBounds {
        /// Position in the user-provided sequence.
        position: usize,
        /// Rejected candidate index.
        index: usize,
        /// Available candidate count.
        candidates: usize,
    },
    /// A user-provided index appears more than once.
    DuplicateUserIndex {
        /// Repeated candidate index.
        index: usize,
        /// Position of its first occurrence.
        first_position: usize,
        /// Position of the repeated occurrence.
        second_position: usize,
    },
    /// A point separation was not finitely representable.
    UnrepresentableDistance {
        /// First candidate index.
        first: usize,
        /// Second candidate index.
        second: usize,
    },
    /// A greedy update produced a nonfinite scalar.
    NonFiniteGreedyUpdate {
        /// Candidate row being updated.
        candidate: usize,
        /// Completed selection step.
        step: usize,
    },
    /// The requested selected count cannot be reached without a deficient pivot.
    InsufficientBasisRank {
        /// Already accepted independent basis count.
        selected: usize,
        /// Requested selected count.
        requested: usize,
        /// Candidate that exposed the deficient pivot.
        candidate: usize,
        /// Computed squared pivot.
        pivot: f64,
        /// Scale-aware strict acceptance threshold.
        threshold: f64,
    },
    /// Owned selector storage could not be reserved.
    AllocationFailed {
        /// Storage role.
        storage: CenterSelectionStorage,
        /// Requested entry count.
        requested: usize,
    },
    /// The selected principal matrix could not be constructed.
    InvalidSelectedBasis(DenseEqualitySystemError),
    /// Internal checked review options were rejected.
    InvalidReviewOptions(DenseSolverConfigurationError),
    /// Existing RRQR/SVD/factorization review rejected the selected basis.
    BasisReview(Box<DenseSolveError>),
}

impl fmt::Display for CenterSelectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConditionallyPositiveDefiniteUnsupported { order } => write!(
                formatter,
                "center selection supports only strictly positive-definite kernels; conditionally positive-definite order {} requires polynomial actions and projected-positive review",
                order.get()
            ),
            Self::EmptyCandidates => formatter.write_str("center selection requires candidates"),
            Self::ShapeOverflow { candidates } => write!(
                formatter,
                "center-selection Gram shape overflows for {candidates} candidates"
            ),
            Self::GramLengthMismatch {
                candidates,
                expected,
                actual,
            } => write!(
                formatter,
                "{candidates}-candidate Gram matrix requires {expected} entries, got {actual}"
            ),
            Self::TargetLengthMismatch { candidates, actual } => write!(
                formatter,
                "{candidates} center candidates require {candidates} targets, got {actual}"
            ),
            Self::NonFiniteGram { index, value } => {
                write!(formatter, "center Gram entry {index} is nonfinite: {value}")
            }
            Self::NonFiniteTarget { index, value } => {
                write!(formatter, "center target {index} is nonfinite: {value}")
            }
            Self::GramNotExactlySymmetric {
                row,
                column,
                upper,
                lower,
            } => write!(
                formatter,
                "center Gram entries ({row}, {column})={upper} and ({column}, {row})={lower} differ"
            ),
            Self::CountExceedsCandidates {
                requested,
                candidates,
            } => write!(
                formatter,
                "center selection requested {requested} of {candidates} candidates"
            ),
            Self::EmptyUserSelection => {
                formatter.write_str("user-provided center selection must be nonempty")
            }
            Self::UserIndexOutOfBounds {
                position,
                index,
                candidates,
            } => write!(
                formatter,
                "user center index {index} at position {position} is outside {candidates} candidates"
            ),
            Self::DuplicateUserIndex {
                index,
                first_position,
                second_position,
            } => write!(
                formatter,
                "user center index {index} repeats at positions {first_position} and {second_position}"
            ),
            Self::UnrepresentableDistance { first, second } => write!(
                formatter,
                "distance between center candidates {first} and {second} is not representable"
            ),
            Self::NonFiniteGreedyUpdate { candidate, step } => write!(
                formatter,
                "center greedy update for candidate {candidate} after step {step} is nonfinite"
            ),
            Self::InsufficientBasisRank {
                selected,
                requested,
                candidate,
                pivot,
                threshold,
            } => write!(
                formatter,
                "center basis cannot reach rank {requested}: candidate {candidate} gives pivot {pivot} at rank {selected}, threshold {threshold}"
            ),
            Self::AllocationFailed { storage, requested } => write!(
                formatter,
                "could not reserve {requested} entries for center-selection storage {storage:?}"
            ),
            Self::InvalidSelectedBasis(source) => source.fmt(formatter),
            Self::InvalidReviewOptions(source) => source.fmt(formatter),
            Self::BasisReview(source) => source.fmt(formatter),
        }
    }
}

impl Error for CenterSelectionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidSelectedBasis(source) => Some(source),
            Self::InvalidReviewOptions(source) => Some(source),
            Self::BasisReview(source) => Some(source.as_ref()),
            _ => None,
        }
    }
}

fn validate_count(requested: usize, candidates: usize) -> Result<(), CenterSelectionError> {
    if requested > candidates {
        return Err(CenterSelectionError::CountExceedsCandidates {
            requested,
            candidates,
        });
    }
    Ok(())
}

fn validate_user_indices(indices: &[usize], candidates: usize) -> Result<(), CenterSelectionError> {
    if indices.is_empty() {
        return Err(CenterSelectionError::EmptyUserSelection);
    }
    for (position, &index) in indices.iter().enumerate() {
        if index >= candidates {
            return Err(CenterSelectionError::UserIndexOutOfBounds {
                position,
                index,
                candidates,
            });
        }
        if let Some(first_position) = indices[..position]
            .iter()
            .position(|&previous| previous == index)
        {
            return Err(CenterSelectionError::DuplicateUserIndex {
                index,
                first_position,
                second_position: position,
            });
        }
    }
    Ok(())
}

fn farthest_point<const D: usize>(
    locations: &[Point<D>],
    count: usize,
    seed: u64,
) -> Result<Vec<usize>, CenterSelectionError>
where
    Dim<D>: SupportedDimension,
{
    let candidates = locations.len();
    let mut selected = Vec::new();
    try_reserve_exact(&mut selected, count, CenterSelectionStorage::Indices)?;
    let mut membership = Vec::new();
    try_reserve_exact(
        &mut membership,
        candidates,
        CenterSelectionStorage::Membership,
    )?;
    membership.resize(candidates, false);
    let mut nearest = Vec::new();
    try_reserve_exact(&mut nearest, candidates, CenterSelectionStorage::Distances)?;
    nearest.resize(candidates, f64::INFINITY);

    let first = usize::try_from(seed % u64::try_from(candidates).unwrap_or(u64::MAX)).unwrap_or(0)
        % candidates;
    for step in 0..count {
        let chosen = if step == 0 {
            first
        } else {
            best_candidate(candidates, &membership, seed, |index| nearest[index]).ok_or(
                CenterSelectionError::CountExceedsCandidates {
                    requested: step + 1,
                    candidates,
                },
            )?
        };
        selected.push(chosen);
        membership[chosen] = true;
        for candidate in 0..candidates {
            if membership[candidate] {
                nearest[candidate] = 0.0;
                continue;
            }
            let distance = stable_distance(locations[candidate], locations[chosen]).ok_or(
                CenterSelectionError::UnrepresentableDistance {
                    first: candidate,
                    second: chosen,
                },
            )?;
            nearest[candidate] = nearest[candidate].min(distance);
        }
    }
    Ok(selected)
}

fn stable_distance<const D: usize>(left: Point<D>, right: Point<D>) -> Option<f64>
where
    Dim<D>: SupportedDimension,
{
    let mut norm = 0.0_f64;
    for axis in 0..D {
        let difference = left.components()[axis] - right.components()[axis];
        if !difference.is_finite() {
            return None;
        }
        norm = norm.hypot(difference);
    }
    norm.is_finite().then_some(norm)
}

#[derive(Clone, Copy)]
enum GreedyScore {
    Residual,
    Power,
}

struct GreedyOutcome {
    indices: Vec<usize>,
    diagnostics: CenterGreedyDiagnostics,
}

#[allow(clippy::too_many_lines)]
fn kernel_greedy(
    gram: &[f64],
    targets: &[f64],
    candidates: usize,
    count: usize,
    seed: u64,
    score: GreedyScore,
) -> Result<GreedyOutcome, CenterSelectionError> {
    let candidate_scale = u32::try_from(candidates)
        .map_err(|_| CenterSelectionError::ShapeOverflow { candidates })?;
    let basis_entries = candidates
        .checked_mul(count)
        .ok_or(CenterSelectionError::ShapeOverflow { candidates })?;
    let mut basis = Vec::new();
    try_reserve_exact(
        &mut basis,
        basis_entries,
        CenterSelectionStorage::NewtonBasis,
    )?;
    basis.resize(basis_entries, 0.0);
    let mut residual = try_copy(targets, CenterSelectionStorage::Residual)?;
    let mut membership = Vec::new();
    try_reserve_exact(
        &mut membership,
        candidates,
        CenterSelectionStorage::Membership,
    )?;
    membership.resize(candidates, false);
    let mut indices = Vec::new();
    try_reserve_exact(&mut indices, count, CenterSelectionStorage::Indices)?;
    let mut minimum_pivot_threshold = f64::INFINITY;
    let mut maximum_pivot_threshold = 0.0_f64;
    let mut minimum_accepted_pivot = f64::INFINITY;
    let mut maximum_accepted_pivot = 0.0_f64;

    for step in 0..count {
        let chosen = best_candidate(candidates, &membership, seed, |candidate| match score {
            GreedyScore::Residual => residual[candidate].abs(),
            GreedyScore::Power => schur_diagonal(gram, &basis, candidates, count, candidate, step),
        })
        .ok_or(CenterSelectionError::CountExceedsCandidates {
            requested: step + 1,
            candidates,
        })?;
        let pivot = schur_diagonal(gram, &basis, candidates, count, chosen, step);
        let pivot_threshold =
            f64::from(candidate_scale) * f64::EPSILON * gram[chosen * candidates + chosen].abs();
        if !pivot_threshold.is_finite() {
            return Err(CenterSelectionError::NonFiniteGreedyUpdate {
                candidate: chosen,
                step,
            });
        }
        if !pivot.is_finite() || pivot <= pivot_threshold {
            return Err(CenterSelectionError::InsufficientBasisRank {
                selected: step,
                requested: count,
                candidate: chosen,
                pivot,
                threshold: pivot_threshold,
            });
        }
        let root = pivot.sqrt();
        for candidate in 0..candidates {
            let mut value = gram[candidate * candidates + chosen];
            for column in 0..step {
                value -= basis[candidate * count + column] * basis[chosen * count + column];
            }
            value /= root;
            if !value.is_finite() {
                return Err(CenterSelectionError::NonFiniteGreedyUpdate { candidate, step });
            }
            basis[candidate * count + step] = value;
        }
        if matches!(score, GreedyScore::Residual) {
            let coefficient = residual[chosen] / root;
            if !coefficient.is_finite() {
                return Err(CenterSelectionError::NonFiniteGreedyUpdate {
                    candidate: chosen,
                    step,
                });
            }
            for candidate in 0..candidates {
                residual[candidate] -= coefficient * basis[candidate * count + step];
                if !residual[candidate].is_finite() {
                    return Err(CenterSelectionError::NonFiniteGreedyUpdate { candidate, step });
                }
            }
        }
        membership[chosen] = true;
        indices.push(chosen);
        minimum_pivot_threshold = minimum_pivot_threshold.min(pivot_threshold);
        maximum_pivot_threshold = maximum_pivot_threshold.max(pivot_threshold);
        minimum_accepted_pivot = minimum_accepted_pivot.min(pivot);
        maximum_accepted_pivot = maximum_accepted_pivot.max(pivot);
    }

    Ok(GreedyOutcome {
        indices,
        diagnostics: CenterGreedyDiagnostics {
            minimum_pivot_threshold,
            maximum_pivot_threshold,
            minimum_accepted_pivot,
            maximum_accepted_pivot,
        },
    })
}

fn schur_diagonal(
    gram: &[f64],
    basis: &[f64],
    candidates: usize,
    columns: usize,
    candidate: usize,
    completed: usize,
) -> f64 {
    let mut value = gram[candidate * candidates + candidate];
    for column in 0..completed {
        value -= basis[candidate * columns + column].powi(2);
    }
    if value < 0.0 && value >= -f64::EPSILON * gram[candidate * candidates + candidate].abs() {
        0.0
    } else {
        value
    }
}

fn best_candidate(
    candidates: usize,
    membership: &[bool],
    seed: u64,
    mut score: impl FnMut(usize) -> f64,
) -> Option<usize> {
    let mut best = None::<(usize, f64, u64)>;
    for (candidate, &is_selected) in membership.iter().enumerate().take(candidates) {
        if is_selected {
            continue;
        }
        let value = score(candidate);
        let tie = splitmix64(seed ^ u64::try_from(candidate).unwrap_or(u64::MAX));
        if best.is_none_or(|(_, best_value, best_tie)| {
            value.total_cmp(&best_value).is_gt()
                || (value.total_cmp(&best_value).is_eq() && tie < best_tie)
        }) {
            best = Some((candidate, value, tie));
        }
    }
    best.map(|(candidate, _, _)| candidate)
}

const fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn review_selected_basis(
    gram: &[f64],
    candidates: usize,
    indices: &[usize],
    memory_limit_bytes: NonZeroUsize,
) -> Result<DenseRankDiagnostics, CenterSelectionError> {
    let selected = indices.len();
    let entries = selected
        .checked_mul(selected)
        .ok_or(CenterSelectionError::ShapeOverflow {
            candidates: selected,
        })?;
    let mut matrix = Vec::new();
    try_reserve_exact(&mut matrix, entries, CenterSelectionStorage::SelectedGram)?;
    for &row in indices {
        for &column in indices {
            matrix.push(gram[row * candidates + column]);
        }
    }
    let mut rhs = Vec::new();
    try_reserve_exact(
        &mut rhs,
        selected,
        CenterSelectionStorage::ReviewRightHandSide,
    )?;
    rhs.resize(selected, 0.0);
    let system = DenseEqualitySystem::try_from_row_major(selected, matrix, rhs)
        .map_err(CenterSelectionError::InvalidSelectedBasis)?;
    let solve_options = DenseSolveOptions::try_new(
        DenseFactorization::Cholesky,
        Regularization::None,
        ConditionPolicy::default(),
        0,
        memory_limit_bytes,
    )
    .map_err(CenterSelectionError::InvalidReviewOptions)?;
    let solution = system
        .try_solve(solve_options)
        .map_err(|source| CenterSelectionError::BasisReview(Box::new(source)))?;
    Ok(solution.diagnostics().original_rank.clone())
}

fn try_reserve_exact<T>(
    values: &mut Vec<T>,
    requested: usize,
    storage: CenterSelectionStorage,
) -> Result<(), CenterSelectionError> {
    values
        .try_reserve_exact(requested)
        .map_err(|_| CenterSelectionError::AllocationFailed { storage, requested })
}

fn try_copy<T: Copy>(
    values: &[T],
    storage: CenterSelectionStorage,
) -> Result<Vec<T>, CenterSelectionError> {
    let mut copy = Vec::new();
    try_reserve_exact(&mut copy, values.len(), storage)?;
    copy.extend_from_slice(values);
    Ok(copy)
}
