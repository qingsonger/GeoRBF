//! Deterministic selection over explicit, bounded parameter candidates.
//!
//! Tuning is deliberately separate from semantic problem construction and
//! fitting. The caller supplies the candidate set and implements
//! [`TuningEvaluator`] with the fitting workflow appropriate to its data. The
//! core owns deterministic fold construction, scoring, validation, tie
//! breaking, and diagnostics. It never changes a hard constraint, generates
//! unrequested regularization, skips a failed candidate, or falls back to a
//! different criterion.

use std::error::Error;
use std::fmt;
use std::num::NonZeroUsize;

use crate::dimension::{Dim, SupportedDimension};
use crate::geometry::Point;

const PARAMETER_COUNT: usize = 5;

/// One tunable scalar in the v1 parameter search space.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub enum TuningParameter {
    /// Positive kernel correlation length.
    Length,
    /// Positive compact-support radius.
    SupportRadius,
    /// Nonnegative explicit diagonal regularization.
    Regularization,
    /// Principal-to-transverse ratio, constrained to at least one.
    AxisRatio,
    /// Positive local-control influence radius.
    InfluenceRadius,
}

impl TuningParameter {
    const ALL: [Self; PARAMETER_COUNT] = [
        Self::Length,
        Self::SupportRadius,
        Self::Regularization,
        Self::AxisRatio,
        Self::InfluenceRadius,
    ];

    const fn index(self) -> usize {
        match self {
            Self::Length => 0,
            Self::SupportRadius => 1,
            Self::Regularization => 2,
            Self::AxisRatio => 3,
            Self::InfluenceRadius => 4,
        }
    }

    const fn minimum_allowed(self) -> f64 {
        match self {
            Self::Regularization => 0.0,
            Self::AxisRatio => 1.0,
            Self::Length | Self::SupportRadius | Self::InfluenceRadius => f64::MIN_POSITIVE,
        }
    }
}

impl fmt::Display for TuningParameter {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Length => "length",
            Self::SupportRadius => "support radius",
            Self::Regularization => "regularization",
            Self::AxisRatio => "axis ratio",
            Self::InfluenceRadius => "influence radius",
        })
    }
}

/// One finite inclusive candidate bound.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct TuningRange {
    minimum: f64,
    maximum: f64,
}

impl TuningRange {
    /// Returns the inclusive minimum.
    #[must_use]
    pub const fn minimum(self) -> f64 {
        self.minimum
    }

    /// Returns the inclusive maximum.
    #[must_use]
    pub const fn maximum(self) -> f64 {
        self.maximum
    }

    const fn contains(self, value: f64) -> bool {
        value >= self.minimum && value <= self.maximum
    }
}

/// Inclusive bounds for every parameter active in a candidate set.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct TuningBounds {
    ranges: [Option<TuningRange>; PARAMETER_COUNT],
}

impl TuningBounds {
    /// Constructs and validates the active inclusive bounds.
    ///
    /// Each tuple is `(minimum, maximum)`. Length, support, and influence
    /// bounds must be strictly positive; regularization may start at zero; and
    /// axis ratios must be at least one. At least one parameter must be active.
    ///
    /// # Errors
    ///
    /// Returns [`TuningError::NoActiveParameters`] for an empty space, or a
    /// structured finite, physical-domain, or ordering error for one range.
    pub fn try_new(
        length: Option<(f64, f64)>,
        support_radius: Option<(f64, f64)>,
        regularization: Option<(f64, f64)>,
        axis_ratio: Option<(f64, f64)>,
        influence_radius: Option<(f64, f64)>,
    ) -> Result<Self, TuningError> {
        let supplied = [
            length,
            support_radius,
            regularization,
            axis_ratio,
            influence_radius,
        ];
        if supplied.iter().all(Option::is_none) {
            return Err(TuningError::NoActiveParameters);
        }
        let mut ranges = [None; PARAMETER_COUNT];
        for (parameter, endpoints) in TuningParameter::ALL.into_iter().zip(supplied) {
            if let Some((minimum, maximum)) = endpoints {
                validate_parameter_value(parameter, minimum, ParameterValueRole::BoundMinimum)?;
                validate_parameter_value(parameter, maximum, ParameterValueRole::BoundMaximum)?;
                if minimum > maximum {
                    return Err(TuningError::ReversedBound {
                        parameter,
                        minimum,
                        maximum,
                    });
                }
                ranges[parameter.index()] = Some(TuningRange { minimum, maximum });
            }
        }
        Ok(Self { ranges })
    }

    /// Returns the bound for one parameter, or `None` when it is inactive.
    #[must_use]
    pub const fn range(&self, parameter: TuningParameter) -> Option<TuningRange> {
        self.ranges[parameter.index()]
    }
}

/// One immutable point in the bounded tuning search space.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct TuningParameters {
    values: [Option<f64>; PARAMETER_COUNT],
}

impl TuningParameters {
    /// Constructs and validates one parameter candidate.
    ///
    /// # Errors
    ///
    /// Returns a structured error when no parameter is active or a supplied
    /// value is nonfinite or outside its physical domain.
    pub fn try_new(
        length: Option<f64>,
        support_radius: Option<f64>,
        regularization: Option<f64>,
        axis_ratio: Option<f64>,
        influence_radius: Option<f64>,
    ) -> Result<Self, TuningError> {
        let values = [
            length,
            support_radius,
            regularization,
            axis_ratio,
            influence_radius,
        ];
        if values.iter().all(Option::is_none) {
            return Err(TuningError::NoActiveParameters);
        }
        for (parameter, value) in TuningParameter::ALL.into_iter().zip(values) {
            if let Some(value) = value {
                validate_parameter_value(parameter, value, ParameterValueRole::Candidate)?;
            }
        }
        Ok(Self { values })
    }

    /// Returns one active parameter value.
    #[must_use]
    pub const fn value(&self, parameter: TuningParameter) -> Option<f64> {
        self.values[parameter.index()]
    }

    /// Returns the positive kernel correlation length.
    #[must_use]
    pub const fn length(&self) -> Option<f64> {
        self.value(TuningParameter::Length)
    }

    /// Returns the positive compact-support radius.
    #[must_use]
    pub const fn support_radius(&self) -> Option<f64> {
        self.value(TuningParameter::SupportRadius)
    }

    /// Returns the nonnegative explicit regularization amount.
    #[must_use]
    pub const fn regularization(&self) -> Option<f64> {
        self.value(TuningParameter::Regularization)
    }

    /// Returns the principal-to-transverse axis ratio.
    #[must_use]
    pub const fn axis_ratio(&self) -> Option<f64> {
        self.value(TuningParameter::AxisRatio)
    }

    /// Returns the positive local-control influence radius.
    #[must_use]
    pub const fn influence_radius(&self) -> Option<f64> {
        self.value(TuningParameter::InfluenceRadius)
    }
}

/// The requested deterministic selection criterion.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub enum TuningCriterion {
    /// Select one explicit candidate without evaluating alternatives.
    Fixed,
    /// Match active length-like parameters to the median nearest-neighbor distance.
    DistanceHeuristic,
    /// Minimize weighted held-out squared error over deterministic folds.
    CrossValidation,
    /// Minimize `RSS / (observation_count - effective_degrees_of_freedom)^2`.
    GeneralizedCrossValidation,
    /// Minimize the worst squared power-function value.
    PowerFunction,
}

/// One explicit deterministic tuning request.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub enum TuningStrategy {
    /// Return exactly one supplied candidate index.
    Fixed {
        /// Candidate to return without evaluation.
        candidate_index: usize,
    },
    /// Use the median nearest-neighbor distance and seeded exact-score ties.
    DistanceHeuristic {
        /// Reproducible exact-score tie-breaking seed.
        seed: u64,
    },
    /// Use deterministic shuffled round-robin validation folds.
    CrossValidation {
        /// Required nonzero validation-fold count.
        folds: NonZeroUsize,
        /// Reproducible fold-order and exact-score tie-breaking seed.
        seed: u64,
    },
    /// Use generalized-cross-validation evidence from the evaluator.
    GeneralizedCrossValidation {
        /// Reproducible exact-score tie-breaking seed.
        seed: u64,
    },
    /// Use worst-case squared power evidence from the evaluator.
    PowerFunction {
        /// Reproducible exact-score tie-breaking seed.
        seed: u64,
    },
}

impl TuningStrategy {
    /// Returns the requested criterion.
    pub const fn criterion(self) -> TuningCriterion {
        match self {
            Self::Fixed { .. } => TuningCriterion::Fixed,
            Self::DistanceHeuristic { .. } => TuningCriterion::DistanceHeuristic,
            Self::CrossValidation { .. } => TuningCriterion::CrossValidation,
            Self::GeneralizedCrossValidation { .. } => TuningCriterion::GeneralizedCrossValidation,
            Self::PowerFunction { .. } => TuningCriterion::PowerFunction,
        }
    }

    /// Returns the explicit seed for a search criterion.
    #[must_use]
    pub const fn seed(self) -> Option<u64> {
        match self {
            Self::Fixed { .. } => None,
            Self::DistanceHeuristic { seed }
            | Self::CrossValidation { seed, .. }
            | Self::GeneralizedCrossValidation { seed }
            | Self::PowerFunction { seed } => Some(seed),
        }
    }
}

/// One deterministic validation fold.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct TuningFold {
    index: usize,
    validation_indices: Vec<usize>,
}

impl TuningFold {
    /// Returns the stable zero-based fold index.
    #[must_use]
    pub const fn index(&self) -> usize {
        self.index
    }

    /// Borrows validation indices in deterministic seeded order.
    #[must_use]
    pub fn validation_indices(&self) -> &[usize] {
        &self.validation_indices
    }
}

/// Weighted held-out loss returned for one candidate and fold.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct CrossValidationEvidence {
    /// Sum of nonnegative weighted squared errors.
    pub weighted_squared_error: f64,
    /// Strictly positive sum of validation weights.
    pub weight: f64,
}

/// Generalized-cross-validation evidence for one candidate.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct GeneralizedCrossValidationEvidence {
    /// Nonnegative fitted residual sum of squares.
    pub residual_sum_squares: f64,
    /// Positive number of fitted observations represented by the residual.
    pub observation_count: usize,
    /// Nonnegative effective degrees of freedom, strictly below the count.
    pub effective_degrees_of_freedom: f64,
}

/// Power-function evidence for one candidate.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct PowerFunctionEvidence {
    /// Maximum nonnegative squared power over the caller's review samples.
    pub maximum_squared_power: f64,
    /// Positive number of reviewed samples.
    pub sample_count: usize,
}

/// Stable evaluator failure retained without interpreting backend details.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub struct TuningEvaluationFailure {
    operation: &'static str,
}

impl TuningEvaluationFailure {
    /// Creates a failure with a stable caller-owned operation label.
    pub const fn new(operation: &'static str) -> Self {
        Self { operation }
    }

    /// Returns the caller-owned operation label.
    #[must_use]
    pub const fn operation(self) -> &'static str {
        self.operation
    }
}

impl fmt::Display for TuningEvaluationFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "tuning evaluation failed during {}",
            self.operation
        )
    }
}

impl Error for TuningEvaluationFailure {}

/// Caller-supplied field-fitting and validation operations.
///
/// Only the method corresponding to the requested strategy is called. A
/// failure rejects the complete search; `GeoRBF` never skips the candidate or
/// changes criteria.
pub trait TuningEvaluator {
    /// Fits the training complement and returns held-out evidence for one fold.
    ///
    /// # Errors
    ///
    /// Returns a stable failure when the caller's fitting or validation
    /// operation cannot produce evidence for this exact candidate and fold.
    fn cross_validation(
        &mut self,
        candidate: &TuningParameters,
        fold: &TuningFold,
    ) -> Result<CrossValidationEvidence, TuningEvaluationFailure>;

    /// Fits once and returns generalized-cross-validation evidence.
    ///
    /// # Errors
    ///
    /// Returns a stable failure when the caller's fitting or trace operation
    /// cannot produce evidence for this exact candidate.
    fn generalized_cross_validation(
        &mut self,
        candidate: &TuningParameters,
    ) -> Result<GeneralizedCrossValidationEvidence, TuningEvaluationFailure>;

    /// Fits or factors once and returns sampled squared power evidence.
    ///
    /// # Errors
    ///
    /// Returns a stable failure when the caller's power-function review cannot
    /// produce evidence for this exact candidate.
    fn power_function(
        &mut self,
        candidate: &TuningParameters,
    ) -> Result<PowerFunctionEvidence, TuningEvaluationFailure>;
}

/// A no-op evaluator for fixed and distance-heuristic selection.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[must_use]
pub struct NoopTuningEvaluator;

impl TuningEvaluator for NoopTuningEvaluator {
    fn cross_validation(
        &mut self,
        _: &TuningParameters,
        _: &TuningFold,
    ) -> Result<CrossValidationEvidence, TuningEvaluationFailure> {
        Err(TuningEvaluationFailure::new("unavailable cross-validation"))
    }

    fn generalized_cross_validation(
        &mut self,
        _: &TuningParameters,
    ) -> Result<GeneralizedCrossValidationEvidence, TuningEvaluationFailure> {
        Err(TuningEvaluationFailure::new(
            "unavailable generalized cross-validation",
        ))
    }

    fn power_function(
        &mut self,
        _: &TuningParameters,
    ) -> Result<PowerFunctionEvidence, TuningEvaluationFailure> {
        Err(TuningEvaluationFailure::new("unavailable power function"))
    }
}

/// Evidence specific to one evaluated candidate.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub enum TuningScoreEvidence {
    /// The candidate was not evaluated by fixed selection.
    NotEvaluated,
    /// The exact fixed candidate.
    Fixed,
    /// Distance-heuristic score and shared distance statistic.
    DistanceHeuristic {
        /// Median positive nearest-neighbor distance.
        median_nearest_neighbor_distance: f64,
        /// Number of active length-like values included in the score.
        compared_parameter_count: usize,
    },
    /// Deterministic per-fold mean losses in fold order.
    CrossValidation {
        /// Per-fold `weighted_squared_error / weight`.
        fold_losses: Vec<f64>,
    },
    /// Validated generalized-cross-validation evidence.
    GeneralizedCrossValidation(GeneralizedCrossValidationEvidence),
    /// Validated worst-case squared power evidence.
    PowerFunction(PowerFunctionEvidence),
}

/// One candidate's score and supporting evidence.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct TuningCandidateDiagnostics {
    /// Stable input candidate index.
    pub candidate_index: usize,
    /// Comparable nonnegative score, or `None` for an unevaluated fixed alternative.
    pub score: Option<f64>,
    /// Criterion-specific evidence.
    pub evidence: TuningScoreEvidence,
}

/// Complete deterministic tuning diagnostics.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct TuningDiagnostics {
    /// Requested criterion.
    pub criterion: TuningCriterion,
    /// Explicit deterministic seed when the criterion performs a search.
    pub seed: Option<u64>,
    /// Inclusive parameter bounds applied to every candidate.
    pub bounds: TuningBounds,
    /// Validation folds, empty outside cross-validation.
    pub folds: Vec<TuningFold>,
    /// Candidate evidence in stable input order.
    pub candidates: Vec<TuningCandidateDiagnostics>,
    /// Number of candidates exactly tied at the minimum score.
    pub tied_best_count: usize,
}

/// One selected immutable parameter set and complete diagnostics.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct TuningResult {
    selected_index: usize,
    parameters: TuningParameters,
    diagnostics: TuningDiagnostics,
}

impl TuningResult {
    /// Returns the stable selected candidate index.
    #[must_use]
    pub const fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Borrows the selected immutable parameters.
    pub const fn parameters(&self) -> &TuningParameters {
        &self.parameters
    }

    /// Borrows complete selection diagnostics.
    pub const fn diagnostics(&self) -> &TuningDiagnostics {
        &self.diagnostics
    }
}

/// Immutable locations, explicit bounded candidates, and search bounds.
///
/// ```
/// use std::num::NonZeroUsize;
///
/// use georbf::{
///     CrossValidationEvidence, GeneralizedCrossValidationEvidence, Point,
///     PowerFunctionEvidence, TuningBounds, TuningEvaluationFailure, TuningEvaluator,
///     TuningFold, TuningParameters, TuningProblem, TuningStrategy,
/// };
///
/// struct AnalyticEvaluator;
///
/// impl TuningEvaluator for AnalyticEvaluator {
///     fn cross_validation(
///         &mut self,
///         candidate: &TuningParameters,
///         fold: &TuningFold,
///     ) -> Result<CrossValidationEvidence, TuningEvaluationFailure> {
///         let length = candidate.length().ok_or(
///             TuningEvaluationFailure::new("missing length"),
///         )?;
///         let error = (length - 2.0).powi(2);
///         Ok(CrossValidationEvidence {
///             weighted_squared_error: error * fold.validation_indices().len() as f64,
///             weight: fold.validation_indices().len() as f64,
///         })
///     }
///
///     fn generalized_cross_validation(
///         &mut self,
///         _: &TuningParameters,
///     ) -> Result<GeneralizedCrossValidationEvidence, TuningEvaluationFailure> {
///         Err(TuningEvaluationFailure::new("unused GCV"))
///     }
///
///     fn power_function(
///         &mut self,
///         _: &TuningParameters,
///     ) -> Result<PowerFunctionEvidence, TuningEvaluationFailure> {
///         Err(TuningEvaluationFailure::new("unused power function"))
///     }
/// }
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let candidates = [1.0, 2.0, 4.0]
///     .into_iter()
///     .map(|length| TuningParameters::try_new(Some(length), None, None, None, None))
///     .collect::<Result<Vec<_>, _>>()?;
/// let problem = TuningProblem::try_new(
///     vec![Point::try_new([0.0])?, Point::try_new([1.0])?],
///     candidates,
///     TuningBounds::try_new(Some((1.0, 4.0)), None, None, None, None)?,
/// )?;
/// let result = problem.try_tune(
///     TuningStrategy::CrossValidation {
///         folds: NonZeroUsize::new(2).ok_or("folds")?,
///         seed: 7,
///     },
///     &mut AnalyticEvaluator,
/// )?;
/// assert_eq!(result.parameters().length(), Some(2.0));
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct TuningProblem<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    locations: Vec<Point<D>>,
    candidates: Vec<TuningParameters>,
    bounds: TuningBounds,
}

impl<const D: usize> TuningProblem<D>
where
    Dim<D>: SupportedDimension,
{
    /// Validates and owns one explicit bounded candidate search.
    ///
    /// # Errors
    ///
    /// Returns a structured error for empty locations or candidates, mismatched
    /// active parameters, out-of-bound values, or exact duplicate candidates.
    pub fn try_new(
        locations: Vec<Point<D>>,
        candidates: Vec<TuningParameters>,
        bounds: TuningBounds,
    ) -> Result<Self, TuningError> {
        if locations.is_empty() {
            return Err(TuningError::EmptyLocations);
        }
        if candidates.is_empty() {
            return Err(TuningError::EmptyCandidates);
        }
        for (candidate_index, candidate) in candidates.iter().enumerate() {
            for parameter in TuningParameter::ALL {
                match (candidate.value(parameter), bounds.range(parameter)) {
                    (Some(value), Some(range)) => {
                        if !range.contains(value) {
                            return Err(TuningError::CandidateOutOfBounds {
                                candidate: candidate_index,
                                parameter,
                                value,
                                minimum: range.minimum,
                                maximum: range.maximum,
                            });
                        }
                    }
                    (Some(_), None) => {
                        return Err(TuningError::CandidateParameterWithoutBound {
                            candidate: candidate_index,
                            parameter,
                        });
                    }
                    (None, Some(_)) => {
                        return Err(TuningError::CandidateMissingBoundedParameter {
                            candidate: candidate_index,
                            parameter,
                        });
                    }
                    (None, None) => {}
                }
            }
            for (first, prior) in candidates[..candidate_index].iter().enumerate() {
                if same_parameters(prior, candidate) {
                    return Err(TuningError::DuplicateCandidate {
                        first,
                        second: candidate_index,
                    });
                }
            }
        }
        Ok(Self {
            locations,
            candidates,
            bounds,
        })
    }

    /// Borrows observation locations in stable input order.
    pub fn locations(&self) -> &[Point<D>] {
        &self.locations
    }

    /// Borrows bounded candidates in stable input order.
    pub fn candidates(&self) -> &[TuningParameters] {
        &self.candidates
    }

    /// Borrows the inclusive search bounds.
    pub const fn bounds(&self) -> &TuningBounds {
        &self.bounds
    }

    /// Selects one candidate using exactly the requested criterion.
    ///
    /// # Errors
    ///
    /// Returns a structured strategy, fold, distance, evaluator, evidence,
    /// arithmetic, or allocation error. Any evaluator failure rejects the
    /// complete search; no candidate is skipped.
    #[allow(clippy::too_many_lines)]
    pub fn try_tune(
        &self,
        strategy: TuningStrategy,
        evaluator: &mut impl TuningEvaluator,
    ) -> Result<TuningResult, TuningError> {
        let criterion = strategy.criterion();
        let seed = strategy.seed();
        let mut folds = Vec::new();
        let mut diagnostics = Vec::new();
        try_reserve_exact(
            &mut diagnostics,
            self.candidates.len(),
            TuningStorage::CandidateDiagnostics,
        )?;

        match strategy {
            TuningStrategy::Fixed { candidate_index } => {
                if candidate_index >= self.candidates.len() {
                    return Err(TuningError::FixedCandidateOutOfBounds {
                        candidate: candidate_index,
                        candidates: self.candidates.len(),
                    });
                }
                for index in 0..self.candidates.len() {
                    diagnostics.push(TuningCandidateDiagnostics {
                        candidate_index: index,
                        score: (index == candidate_index).then_some(0.0),
                        evidence: if index == candidate_index {
                            TuningScoreEvidence::Fixed
                        } else {
                            TuningScoreEvidence::NotEvaluated
                        },
                    });
                }
                self.finish(candidate_index, criterion, seed, folds, diagnostics, 1)
            }
            TuningStrategy::DistanceHeuristic { seed } => {
                let median = median_nearest_neighbor_distance(&self.locations)?;
                for (candidate_index, candidate) in self.candidates.iter().enumerate() {
                    let (score, compared_parameter_count) =
                        distance_heuristic_score(candidate, median, candidate_index)?;
                    diagnostics.push(TuningCandidateDiagnostics {
                        candidate_index,
                        score: Some(score),
                        evidence: TuningScoreEvidence::DistanceHeuristic {
                            median_nearest_neighbor_distance: median,
                            compared_parameter_count,
                        },
                    });
                }
                let (selected, tied) = select_minimum(&diagnostics, seed)?;
                self.finish(selected, criterion, Some(seed), folds, diagnostics, tied)
            }
            TuningStrategy::CrossValidation {
                folds: fold_count,
                seed,
            } => {
                folds = build_folds(self.locations.len(), fold_count, seed)?;
                for (candidate_index, candidate) in self.candidates.iter().enumerate() {
                    let mut fold_losses = Vec::new();
                    try_reserve_exact(&mut fold_losses, folds.len(), TuningStorage::FoldLosses)?;
                    let mut total_error = 0.0;
                    let mut total_weight = 0.0;
                    for fold in &folds {
                        let evidence =
                            evaluator
                                .cross_validation(candidate, fold)
                                .map_err(|source| TuningError::EvaluationFailed {
                                    candidate: candidate_index,
                                    fold: Some(fold.index),
                                    source,
                                })?;
                        validate_cross_validation_evidence(candidate_index, fold.index, evidence)?;
                        let fold_loss = evidence.weighted_squared_error / evidence.weight;
                        if !fold_loss.is_finite() {
                            return Err(TuningError::NonFiniteScore {
                                candidate: candidate_index,
                                criterion,
                            });
                        }
                        fold_losses.push(fold_loss);
                        total_error += evidence.weighted_squared_error;
                        total_weight += evidence.weight;
                        if !total_error.is_finite() || !total_weight.is_finite() {
                            return Err(TuningError::NonFiniteScore {
                                candidate: candidate_index,
                                criterion,
                            });
                        }
                    }
                    let score = total_error / total_weight;
                    diagnostics.push(TuningCandidateDiagnostics {
                        candidate_index,
                        score: Some(score),
                        evidence: TuningScoreEvidence::CrossValidation { fold_losses },
                    });
                }
                let (selected, tied) = select_minimum(&diagnostics, seed)?;
                self.finish(selected, criterion, Some(seed), folds, diagnostics, tied)
            }
            TuningStrategy::GeneralizedCrossValidation { seed } => {
                for (candidate_index, candidate) in self.candidates.iter().enumerate() {
                    let evidence =
                        evaluator
                            .generalized_cross_validation(candidate)
                            .map_err(|source| TuningError::EvaluationFailed {
                                candidate: candidate_index,
                                fold: None,
                                source,
                            })?;
                    let score = generalized_cross_validation_score(candidate_index, evidence)?;
                    diagnostics.push(TuningCandidateDiagnostics {
                        candidate_index,
                        score: Some(score),
                        evidence: TuningScoreEvidence::GeneralizedCrossValidation(evidence),
                    });
                }
                let (selected, tied) = select_minimum(&diagnostics, seed)?;
                self.finish(selected, criterion, Some(seed), folds, diagnostics, tied)
            }
            TuningStrategy::PowerFunction { seed } => {
                for (candidate_index, candidate) in self.candidates.iter().enumerate() {
                    let evidence = evaluator.power_function(candidate).map_err(|source| {
                        TuningError::EvaluationFailed {
                            candidate: candidate_index,
                            fold: None,
                            source,
                        }
                    })?;
                    validate_power_evidence(candidate_index, evidence)?;
                    diagnostics.push(TuningCandidateDiagnostics {
                        candidate_index,
                        score: Some(evidence.maximum_squared_power),
                        evidence: TuningScoreEvidence::PowerFunction(evidence),
                    });
                }
                let (selected, tied) = select_minimum(&diagnostics, seed)?;
                self.finish(selected, criterion, Some(seed), folds, diagnostics, tied)
            }
        }
    }

    fn finish(
        &self,
        selected_index: usize,
        criterion: TuningCriterion,
        seed: Option<u64>,
        folds: Vec<TuningFold>,
        candidates: Vec<TuningCandidateDiagnostics>,
        tied_best_count: usize,
    ) -> Result<TuningResult, TuningError> {
        let parameters = self.candidates.get(selected_index).cloned().ok_or(
            TuningError::InternalSelectionOutOfBounds {
                selected: selected_index,
                candidates: self.candidates.len(),
            },
        )?;
        Ok(TuningResult {
            selected_index,
            parameters,
            diagnostics: TuningDiagnostics {
                criterion,
                seed,
                bounds: self.bounds.clone(),
                folds,
                candidates,
                tied_best_count,
            },
        })
    }
}

/// Fallible storage role used during deterministic tuning.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub enum TuningStorage {
    /// Per-candidate diagnostics.
    CandidateDiagnostics,
    /// Pairwise nearest-neighbor work.
    NearestDistances,
    /// Seeded observation ordering.
    FoldOrder,
    /// Fold containers.
    Folds,
    /// Validation indices within one fold.
    FoldValidationIndices,
    /// Per-candidate validation losses.
    FoldLosses,
}

/// Failure while validating or selecting a bounded parameter candidate.
#[derive(Debug)]
#[non_exhaustive]
pub enum TuningError {
    /// A bound or candidate did not activate any parameter.
    NoActiveParameters,
    /// One scalar was NaN or infinite.
    NonFiniteParameter {
        /// Affected parameter.
        parameter: TuningParameter,
        /// Role of the rejected scalar.
        role: ParameterValueRole,
        /// Rejected scalar.
        value: f64,
    },
    /// One scalar was outside the parameter's physical domain.
    InvalidParameterDomain {
        /// Affected parameter.
        parameter: TuningParameter,
        /// Role of the rejected scalar.
        role: ParameterValueRole,
        /// Smallest allowed value.
        minimum_allowed: f64,
        /// Rejected scalar.
        value: f64,
    },
    /// An inclusive bound has its minimum after its maximum.
    ReversedBound {
        /// Affected parameter.
        parameter: TuningParameter,
        /// Rejected minimum.
        minimum: f64,
        /// Rejected maximum.
        maximum: f64,
    },
    /// No observation location was supplied.
    EmptyLocations,
    /// No parameter candidate was supplied.
    EmptyCandidates,
    /// A candidate supplies a parameter that has no bound.
    CandidateParameterWithoutBound {
        /// Candidate index.
        candidate: usize,
        /// Unbounded parameter.
        parameter: TuningParameter,
    },
    /// A candidate omits a parameter active in the bounds.
    CandidateMissingBoundedParameter {
        /// Candidate index.
        candidate: usize,
        /// Missing parameter.
        parameter: TuningParameter,
    },
    /// One candidate scalar is outside its inclusive bound.
    CandidateOutOfBounds {
        /// Candidate index.
        candidate: usize,
        /// Affected parameter.
        parameter: TuningParameter,
        /// Rejected value.
        value: f64,
        /// Inclusive minimum.
        minimum: f64,
        /// Inclusive maximum.
        maximum: f64,
    },
    /// Two candidates are exactly identical.
    DuplicateCandidate {
        /// First candidate index.
        first: usize,
        /// Repeated candidate index.
        second: usize,
    },
    /// Fixed selection requested a nonexistent candidate.
    FixedCandidateOutOfBounds {
        /// Requested index.
        candidate: usize,
        /// Available count.
        candidates: usize,
    },
    /// The distance heuristic requires at least two locations.
    InsufficientLocationsForDistanceHeuristic {
        /// Available location count.
        locations: usize,
    },
    /// One location has no positive finite nearest-neighbor distance.
    InvalidNearestNeighborDistance {
        /// Location index.
        location: usize,
        /// Rejected nearest distance.
        distance: f64,
    },
    /// A candidate has no active length-like parameter for distance scoring.
    NoDistanceParameter {
        /// Candidate index.
        candidate: usize,
    },
    /// More validation folds were requested than observations.
    FoldCountExceedsObservations {
        /// Requested fold count.
        folds: usize,
        /// Available observation count.
        observations: usize,
    },
    /// Caller-supplied candidate evaluation failed.
    EvaluationFailed {
        /// Candidate index.
        candidate: usize,
        /// Fold index for cross-validation.
        fold: Option<usize>,
        /// Caller failure.
        source: TuningEvaluationFailure,
    },
    /// Cross-validation evidence was invalid.
    InvalidCrossValidationEvidence {
        /// Candidate index.
        candidate: usize,
        /// Fold index.
        fold: usize,
        /// Rejected weighted squared error.
        weighted_squared_error: f64,
        /// Rejected weight.
        weight: f64,
    },
    /// Generalized-cross-validation evidence was invalid.
    InvalidGeneralizedCrossValidationEvidence {
        /// Candidate index.
        candidate: usize,
        /// Rejected residual sum of squares.
        residual_sum_squares: f64,
        /// Rejected observation count.
        observation_count: usize,
        /// Rejected effective degrees of freedom.
        effective_degrees_of_freedom: f64,
    },
    /// Power-function evidence was invalid.
    InvalidPowerFunctionEvidence {
        /// Candidate index.
        candidate: usize,
        /// Rejected maximum squared power.
        maximum_squared_power: f64,
        /// Rejected sample count.
        sample_count: usize,
    },
    /// Criterion scoring produced NaN or infinity.
    NonFiniteScore {
        /// Candidate index.
        candidate: usize,
        /// Active criterion.
        criterion: TuningCriterion,
    },
    /// Owned tuning storage could not be reserved.
    AllocationFailed {
        /// Storage role.
        storage: TuningStorage,
        /// Requested entry count.
        requested: usize,
    },
    /// Internal minimum selection had no scored candidate.
    NoScoredCandidates,
    /// Internal selected index was not present in the immutable candidates.
    InternalSelectionOutOfBounds {
        /// Selected index.
        selected: usize,
        /// Candidate count.
        candidates: usize,
    },
}

/// Role of one validated parameter scalar.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub enum ParameterValueRole {
    /// Candidate value.
    Candidate,
    /// Inclusive bound minimum.
    BoundMinimum,
    /// Inclusive bound maximum.
    BoundMaximum,
}

impl fmt::Display for ParameterValueRole {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Candidate => "candidate",
            Self::BoundMinimum => "bound minimum",
            Self::BoundMaximum => "bound maximum",
        })
    }
}

impl fmt::Display for TuningError {
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoActiveParameters => {
                formatter.write_str("parameter tuning requires at least one active parameter")
            }
            Self::NonFiniteParameter {
                parameter,
                role,
                value,
            } => write!(formatter, "{parameter} {role} must be finite, got {value}"),
            Self::InvalidParameterDomain {
                parameter,
                role,
                minimum_allowed,
                value,
            } => write!(
                formatter,
                "{parameter} {role} must be at least {minimum_allowed}, got {value}"
            ),
            Self::ReversedBound {
                parameter,
                minimum,
                maximum,
            } => write!(
                formatter,
                "{parameter} bound minimum {minimum} exceeds maximum {maximum}"
            ),
            Self::EmptyLocations => {
                formatter.write_str("parameter tuning requires observation locations")
            }
            Self::EmptyCandidates => formatter.write_str("parameter tuning requires candidates"),
            Self::CandidateParameterWithoutBound {
                candidate,
                parameter,
            } => write!(
                formatter,
                "candidate {candidate} supplies unbounded {parameter}"
            ),
            Self::CandidateMissingBoundedParameter {
                candidate,
                parameter,
            } => write!(formatter, "candidate {candidate} omits bounded {parameter}"),
            Self::CandidateOutOfBounds {
                candidate,
                parameter,
                value,
                minimum,
                maximum,
            } => write!(
                formatter,
                "candidate {candidate} {parameter} {value} is outside inclusive bound [{minimum}, {maximum}]"
            ),
            Self::DuplicateCandidate { first, second } => {
                write!(formatter, "candidate {second} duplicates candidate {first}")
            }
            Self::FixedCandidateOutOfBounds {
                candidate,
                candidates,
            } => write!(
                formatter,
                "fixed candidate {candidate} is outside {candidates} candidates"
            ),
            Self::InsufficientLocationsForDistanceHeuristic { locations } => write!(
                formatter,
                "distance heuristic requires at least two locations, got {locations}"
            ),
            Self::InvalidNearestNeighborDistance { location, distance } => write!(
                formatter,
                "location {location} has invalid nearest-neighbor distance {distance}"
            ),
            Self::NoDistanceParameter { candidate } => write!(
                formatter,
                "candidate {candidate} has no length, support radius, or influence radius for distance scoring"
            ),
            Self::FoldCountExceedsObservations {
                folds,
                observations,
            } => write!(
                formatter,
                "{folds} validation folds exceed {observations} observations"
            ),
            Self::EvaluationFailed {
                candidate,
                fold,
                source,
            } => {
                if let Some(fold) = fold {
                    write!(
                        formatter,
                        "candidate {candidate} fold {fold} evaluation failed: {source}"
                    )
                } else {
                    write!(
                        formatter,
                        "candidate {candidate} evaluation failed: {source}"
                    )
                }
            }
            Self::InvalidCrossValidationEvidence {
                candidate,
                fold,
                weighted_squared_error,
                weight,
            } => write!(
                formatter,
                "candidate {candidate} fold {fold} has invalid weighted squared error {weighted_squared_error} or weight {weight}"
            ),
            Self::InvalidGeneralizedCrossValidationEvidence {
                candidate,
                residual_sum_squares,
                observation_count,
                effective_degrees_of_freedom,
            } => write!(
                formatter,
                "candidate {candidate} has invalid GCV evidence RSS={residual_sum_squares}, observations={observation_count}, effective_dof={effective_degrees_of_freedom}"
            ),
            Self::InvalidPowerFunctionEvidence {
                candidate,
                maximum_squared_power,
                sample_count,
            } => write!(
                formatter,
                "candidate {candidate} has invalid maximum squared power {maximum_squared_power} or sample count {sample_count}"
            ),
            Self::NonFiniteScore {
                candidate,
                criterion,
            } => write!(
                formatter,
                "candidate {candidate} produced a nonfinite {criterion:?} score"
            ),
            Self::AllocationFailed { storage, requested } => write!(
                formatter,
                "could not reserve {requested} entries for tuning {storage:?}"
            ),
            Self::NoScoredCandidates => {
                formatter.write_str("parameter tuning produced no scored candidate")
            }
            Self::InternalSelectionOutOfBounds {
                selected,
                candidates,
            } => write!(
                formatter,
                "internal selected index {selected} is outside {candidates} candidates"
            ),
        }
    }
}

impl Error for TuningError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::EvaluationFailed { source, .. } => Some(source),
            _ => None,
        }
    }
}

fn validate_parameter_value(
    parameter: TuningParameter,
    value: f64,
    role: ParameterValueRole,
) -> Result<(), TuningError> {
    if !value.is_finite() {
        return Err(TuningError::NonFiniteParameter {
            parameter,
            role,
            value,
        });
    }
    let minimum_allowed = parameter.minimum_allowed();
    if value < minimum_allowed {
        return Err(TuningError::InvalidParameterDomain {
            parameter,
            role,
            minimum_allowed,
            value,
        });
    }
    Ok(())
}

fn same_parameters(first: &TuningParameters, second: &TuningParameters) -> bool {
    TuningParameter::ALL.into_iter().all(|parameter| {
        match (first.value(parameter), second.value(parameter)) {
            (Some(first), Some(second)) => {
                canonical_parameter_bits(first) == canonical_parameter_bits(second)
            }
            (None, None) => true,
            (Some(_), None) | (None, Some(_)) => false,
        }
    })
}

fn canonical_parameter_bits(value: f64) -> u64 {
    let bits = value.to_bits();
    if bits << 1 == 0 { 0 } else { bits }
}

fn median_nearest_neighbor_distance<const D: usize>(
    locations: &[Point<D>],
) -> Result<f64, TuningError>
where
    Dim<D>: SupportedDimension,
{
    if locations.len() < 2 {
        return Err(TuningError::InsufficientLocationsForDistanceHeuristic {
            locations: locations.len(),
        });
    }
    let mut nearest = Vec::new();
    try_reserve_exact(
        &mut nearest,
        locations.len(),
        TuningStorage::NearestDistances,
    )?;
    for (index, location) in locations.iter().enumerate() {
        let mut minimum = f64::INFINITY;
        for (other_index, other) in locations.iter().enumerate() {
            if index != other_index {
                let mut distance = 0.0_f64;
                for axis in 0..D {
                    distance =
                        distance.hypot(location.components()[axis] - other.components()[axis]);
                }
                minimum = minimum.min(distance);
            }
        }
        if !minimum.is_finite() || minimum <= 0.0 {
            return Err(TuningError::InvalidNearestNeighborDistance {
                location: index,
                distance: minimum,
            });
        }
        nearest.push(minimum);
    }
    nearest.sort_by(f64::total_cmp);
    let middle = nearest.len() / 2;
    let median = if nearest.len() % 2 == 0 {
        0.5 * nearest[middle - 1] + 0.5 * nearest[middle]
    } else {
        nearest[middle]
    };
    if !median.is_finite() || median <= 0.0 {
        return Err(TuningError::InvalidNearestNeighborDistance {
            location: middle,
            distance: median,
        });
    }
    Ok(median)
}

fn distance_heuristic_score(
    candidate: &TuningParameters,
    target: f64,
    candidate_index: usize,
) -> Result<(f64, usize), TuningError> {
    let mut score = 0.0;
    let mut count = 0;
    for parameter in [
        TuningParameter::Length,
        TuningParameter::SupportRadius,
        TuningParameter::InfluenceRadius,
    ] {
        if let Some(value) = candidate.value(parameter) {
            let residual = (value / target).ln();
            score += residual * residual;
            count += 1;
        }
    }
    if count == 0 {
        return Err(TuningError::NoDistanceParameter {
            candidate: candidate_index,
        });
    }
    score /= f64::from(
        u32::try_from(count).map_err(|_| TuningError::NonFiniteScore {
            candidate: candidate_index,
            criterion: TuningCriterion::DistanceHeuristic,
        })?,
    );
    if !score.is_finite() {
        return Err(TuningError::NonFiniteScore {
            candidate: candidate_index,
            criterion: TuningCriterion::DistanceHeuristic,
        });
    }
    Ok((score, count))
}

fn build_folds(
    observations: usize,
    fold_count: NonZeroUsize,
    seed: u64,
) -> Result<Vec<TuningFold>, TuningError> {
    let fold_count = fold_count.get();
    if fold_count > observations {
        return Err(TuningError::FoldCountExceedsObservations {
            folds: fold_count,
            observations,
        });
    }
    let mut order = Vec::new();
    try_reserve_exact(&mut order, observations, TuningStorage::FoldOrder)?;
    order.extend(0..observations);
    order.sort_by_key(|&index| (tie_key(seed, index), index));

    let mut folds = Vec::new();
    try_reserve_exact(&mut folds, fold_count, TuningStorage::Folds)?;
    for index in 0..fold_count {
        let capacity = observations / fold_count + usize::from(index < observations % fold_count);
        let mut validation_indices = Vec::new();
        try_reserve_exact(
            &mut validation_indices,
            capacity,
            TuningStorage::FoldValidationIndices,
        )?;
        folds.push(TuningFold {
            index,
            validation_indices,
        });
    }
    for (position, observation) in order.into_iter().enumerate() {
        folds[position % fold_count]
            .validation_indices
            .push(observation);
    }
    Ok(folds)
}

fn validate_cross_validation_evidence(
    candidate: usize,
    fold: usize,
    evidence: CrossValidationEvidence,
) -> Result<(), TuningError> {
    if !evidence.weighted_squared_error.is_finite()
        || evidence.weighted_squared_error < 0.0
        || !evidence.weight.is_finite()
        || evidence.weight <= 0.0
    {
        return Err(TuningError::InvalidCrossValidationEvidence {
            candidate,
            fold,
            weighted_squared_error: evidence.weighted_squared_error,
            weight: evidence.weight,
        });
    }
    Ok(())
}

fn generalized_cross_validation_score(
    candidate: usize,
    evidence: GeneralizedCrossValidationEvidence,
) -> Result<f64, TuningError> {
    let count = u32::try_from(evidence.observation_count).map(f64::from);
    if !evidence.residual_sum_squares.is_finite()
        || evidence.residual_sum_squares < 0.0
        || evidence.observation_count == 0
        || count.is_err()
        || !evidence.effective_degrees_of_freedom.is_finite()
        || evidence.effective_degrees_of_freedom < 0.0
    {
        return Err(TuningError::InvalidGeneralizedCrossValidationEvidence {
            candidate,
            residual_sum_squares: evidence.residual_sum_squares,
            observation_count: evidence.observation_count,
            effective_degrees_of_freedom: evidence.effective_degrees_of_freedom,
        });
    }
    let count = count.map_err(|_| TuningError::InvalidGeneralizedCrossValidationEvidence {
        candidate,
        residual_sum_squares: evidence.residual_sum_squares,
        observation_count: evidence.observation_count,
        effective_degrees_of_freedom: evidence.effective_degrees_of_freedom,
    })?;
    if evidence.effective_degrees_of_freedom >= count {
        return Err(TuningError::InvalidGeneralizedCrossValidationEvidence {
            candidate,
            residual_sum_squares: evidence.residual_sum_squares,
            observation_count: evidence.observation_count,
            effective_degrees_of_freedom: evidence.effective_degrees_of_freedom,
        });
    }
    let denominator = count - evidence.effective_degrees_of_freedom;
    let score = evidence.residual_sum_squares / (denominator * denominator);
    if !score.is_finite() {
        return Err(TuningError::NonFiniteScore {
            candidate,
            criterion: TuningCriterion::GeneralizedCrossValidation,
        });
    }
    Ok(score)
}

fn validate_power_evidence(
    candidate: usize,
    evidence: PowerFunctionEvidence,
) -> Result<(), TuningError> {
    if !evidence.maximum_squared_power.is_finite()
        || evidence.maximum_squared_power < 0.0
        || evidence.sample_count == 0
    {
        return Err(TuningError::InvalidPowerFunctionEvidence {
            candidate,
            maximum_squared_power: evidence.maximum_squared_power,
            sample_count: evidence.sample_count,
        });
    }
    Ok(())
}

fn select_minimum(
    diagnostics: &[TuningCandidateDiagnostics],
    seed: u64,
) -> Result<(usize, usize), TuningError> {
    let minimum = diagnostics
        .iter()
        .filter_map(|candidate| candidate.score)
        .min_by(f64::total_cmp)
        .ok_or(TuningError::NoScoredCandidates)?;
    let tied = diagnostics
        .iter()
        .filter(|candidate| candidate.score == Some(minimum))
        .count();
    let selected = diagnostics
        .iter()
        .filter(|candidate| candidate.score == Some(minimum))
        .min_by_key(|candidate| {
            (
                tie_key(seed, candidate.candidate_index),
                candidate.candidate_index,
            )
        })
        .map(|candidate| candidate.candidate_index)
        .ok_or(TuningError::NoScoredCandidates)?;
    Ok((selected, tied))
}

fn tie_key(seed: u64, index: usize) -> u64 {
    let index = u64::try_from(index).unwrap_or(u64::MAX);
    splitmix64(seed ^ index.wrapping_mul(0x9e37_79b9_7f4a_7c15))
}

const fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn try_reserve_exact<T>(
    values: &mut Vec<T>,
    requested: usize,
    storage: TuningStorage,
) -> Result<(), TuningError> {
    values
        .try_reserve_exact(requested)
        .map_err(|_| TuningError::AllocationFailed { storage, requested })
}
