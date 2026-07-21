//! Sign-invariant weighted orientation-tensor estimation.
//!
//! The estimator supports only the core spatial dimensions and estimates
//! principal axes and relative axis ratios, never an absolute correlation
//! length. Axial observations contribute normalized outer products, so
//! reversing any input direction leaves every result unchanged.
//!
//! ```compile_fail
//! use georbf::OrientationTensorEstimator;
//!
//! fn unsupported(_: Option<OrientationTensorEstimator<4>>) {}
//! ```

use std::cmp::Ordering;
use std::error::Error;
use std::fmt;

use nalgebra::{DMatrix, linalg::SymmetricEigen};

use crate::dimension::{Dim, SupportedDimension};
use crate::geometry::UnitDirection;

const EIGENSPACE_RESOLUTION_FACTOR: f64 = 64.0;
const INFLUENCE_ROUNDOFF_FACTOR: f64 = 64.0;

/// One validated axial direction and its nonnegative estimation weight.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct OrientationTensorSample<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    direction: UnitDirection<D>,
    weight: f64,
}

impl<const D: usize> OrientationTensorSample<D>
where
    Dim<D>: SupportedDimension,
{
    /// Constructs a weighted axial sample.
    ///
    /// # Errors
    ///
    /// Returns a structured error when `weight` is non-finite or negative.
    pub fn try_new(
        direction: UnitDirection<D>,
        weight: f64,
    ) -> Result<Self, OrientationTensorError> {
        if !weight.is_finite() {
            return Err(OrientationTensorError::NonFiniteWeight { index: 0, weight });
        }
        if weight < 0.0 {
            return Err(OrientationTensorError::NegativeWeight { index: 0, weight });
        }
        Ok(Self { direction, weight })
    }

    /// Returns the axial unit direction.
    pub const fn direction(&self) -> UnitDirection<D> {
        self.direction
    }

    /// Returns the finite nonnegative weight.
    #[must_use]
    pub const fn weight(&self) -> f64 {
        self.weight
    }
}

/// Principal-axis length ratios ordered with the tensor eigenvalues.
///
/// Ratios are nonincreasing, every value is at least one, and the last value
/// is exactly one. Thus the representation has no arbitrary common scale.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct PrincipalAxisRatios<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    values: [f64; D],
}

impl<const D: usize> PrincipalAxisRatios<D>
where
    Dim<D>: SupportedDimension,
{
    /// Validates normalized, nonincreasing principal-axis ratios.
    ///
    /// # Errors
    ///
    /// Rejects non-finite values, values below one, increasing order, a final
    /// value other than one, or a ratio range whose normalized square is not
    /// representable. The constructor never sorts or rescales caller input.
    pub fn try_new(values: [f64; D]) -> Result<Self, OrientationTensorError> {
        for (axis, value) in values.iter().copied().enumerate() {
            if !value.is_finite() {
                return Err(OrientationTensorError::NonFiniteAxisRatio { axis, value });
            }
            if value < 1.0 {
                return Err(OrientationTensorError::AxisRatioBelowOne { axis, value });
            }
            if axis > 0 && values[axis - 1] < value {
                return Err(OrientationTensorError::UnorderedAxisRatios {
                    first: axis - 1,
                    second: axis,
                });
            }
        }
        if values[D - 1].to_bits() != 1.0_f64.to_bits() {
            return Err(OrientationTensorError::UnnormalizedAxisRatios {
                last: values[D - 1],
            });
        }
        let maximum = values[0];
        for (axis, value) in values.iter().copied().enumerate() {
            let scaled = value / maximum;
            if scaled != 0.0 && scaled * scaled == 0.0 {
                return Err(OrientationTensorError::NonRepresentableRatioSquare {
                    axis,
                    value,
                    maximum,
                });
            }
        }
        Ok(Self { values })
    }

    /// Borrows the normalized ratios in descending-eigenvalue order.
    #[must_use]
    pub const fn values(&self) -> &[f64; D] {
        &self.values
    }

    /// Returns the largest represented axis ratio.
    #[must_use]
    pub const fn maximum(&self) -> f64 {
        self.values[0]
    }
}

/// Whether ratios were supplied directly or selected by cross-validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AxisRatioSelectionKind {
    /// A caller supplied the selected ratios directly.
    Fixed,
    /// Deterministic leave-one-out cross-validation selected a candidate.
    LeaveOneOut,
}

/// One deterministic cross-validation score.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct AxisRatioCandidateScore<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    ratios: PrincipalAxisRatios<D>,
    score: f64,
}

impl<const D: usize> AxisRatioCandidateScore<D>
where
    Dim<D>: SupportedDimension,
{
    /// Returns the scored candidate.
    pub const fn ratios(&self) -> PrincipalAxisRatios<D> {
        self.ratios
    }

    /// Returns the weighted mean squared share mismatch; lower is better.
    #[must_use]
    pub const fn score(&self) -> f64 {
        self.score
    }
}

/// Leave-one-out tensor influence for one input sample.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct OrientationTensorInfluence {
    sample_index: usize,
    normalized_tensor_change: f64,
}

impl OrientationTensorInfluence {
    /// Returns the original sample index.
    #[must_use]
    pub const fn sample_index(&self) -> usize {
        self.sample_index
    }

    /// Returns `||C-C_-i||_F/sqrt(2)` in `[0,1]` after the documented
    /// dimension-scaled roundoff-bound policy.
    ///
    /// Removing the sole positive-weight sample is defined to have influence
    /// one because no leave-one-out estimate exists.
    #[must_use]
    pub const fn normalized_tensor_change(&self) -> f64 {
        self.normalized_tensor_change
    }
}

/// Immutable diagnostics for an orientation-tensor estimate.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct OrientationTensorDiagnostics<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    normalized_eigenvalue_gaps: Vec<f64>,
    axis_confidence: [f64; D],
    isotropic: bool,
    isotropy_threshold: f64,
    positive_sample_count: usize,
    maximum_weight_fraction: f64,
    selected_maximum_ratio: f64,
    maximum_outlier_influence: f64,
    most_influential_sample: Option<usize>,
    selection_kind: AxisRatioSelectionKind,
}

impl<const D: usize> OrientationTensorDiagnostics<D>
where
    Dim<D>: SupportedDimension,
{
    /// Returns adjacent eigenvalue gaps divided by the eigenvalue sum.
    #[must_use]
    pub fn normalized_eigenvalue_gaps(&self) -> &[f64] {
        &self.normalized_eigenvalue_gaps
    }

    /// Returns each axis' smallest adjacent normalized eigenvalue gap.
    ///
    /// D=1 has confidence one because no competing eigendirection exists.
    #[must_use]
    pub const fn axis_confidence(&self) -> &[f64; D] {
        &self.axis_confidence
    }

    /// Returns whether `(lambda_max-lambda_min)/sum(lambda)` is no greater
    /// than the explicit caller threshold.
    #[must_use]
    pub const fn is_isotropic(&self) -> bool {
        self.isotropic
    }

    /// Returns the caller-selected isotropy threshold in `[0,1]`.
    #[must_use]
    pub const fn isotropy_threshold(&self) -> f64 {
        self.isotropy_threshold
    }

    /// Returns the number of strictly positive-weight samples.
    #[must_use]
    pub const fn positive_sample_count(&self) -> usize {
        self.positive_sample_count
    }

    /// Returns the largest normalized sample weight.
    #[must_use]
    pub const fn maximum_weight_fraction(&self) -> f64 {
        self.maximum_weight_fraction
    }

    /// Returns the selected candidate's largest axis ratio.
    #[must_use]
    pub const fn selected_maximum_ratio(&self) -> f64 {
        self.selected_maximum_ratio
    }

    /// Returns the largest normalized leave-one-out tensor change.
    #[must_use]
    pub const fn maximum_outlier_influence(&self) -> f64 {
        self.maximum_outlier_influence
    }

    /// Returns the first most influential sample, or `None` for empty input.
    #[must_use]
    pub const fn most_influential_sample(&self) -> Option<usize> {
        self.most_influential_sample
    }

    /// Returns how the axis ratios were selected.
    #[must_use]
    pub const fn selection_kind(&self) -> AxisRatioSelectionKind {
        self.selection_kind
    }
}

/// Immutable orientation-tensor result.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct OrientationTensorEstimate<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    tensor: [[f64; D]; D],
    principal_axes: [UnitDirection<D>; D],
    eigenvalues: [f64; D],
    normalized_eigenvalues: [f64; D],
    axis_ratios: PrincipalAxisRatios<D>,
    candidate_scores: Vec<AxisRatioCandidateScore<D>>,
    influences: Vec<OrientationTensorInfluence>,
    diagnostics: OrientationTensorDiagnostics<D>,
}

impl<const D: usize> OrientationTensorEstimate<D>
where
    Dim<D>: SupportedDimension,
{
    /// Borrows the finite symmetric normalized tensor with trace one.
    #[must_use]
    pub const fn tensor(&self) -> &[[f64; D]; D] {
        &self.tensor
    }

    /// Borrows principal axes ordered by nonincreasing eigenvalue.
    pub const fn principal_axes(&self) -> &[UnitDirection<D>; D] {
        &self.principal_axes
    }

    /// Borrows the ordered raw eigensolver values of the normalized tensor.
    #[must_use]
    pub const fn eigenvalues(&self) -> &[f64; D] {
        &self.eigenvalues
    }

    /// Borrows eigenvalues normalized by their finite positive sum.
    #[must_use]
    pub const fn normalized_eigenvalues(&self) -> &[f64; D] {
        &self.normalized_eigenvalues
    }

    /// Returns the fixed or cross-validated principal-axis ratios.
    pub const fn axis_ratios(&self) -> PrincipalAxisRatios<D> {
        self.axis_ratios
    }

    /// Borrows candidate scores in caller order; fixed selection returns none.
    pub fn candidate_scores(&self) -> &[AxisRatioCandidateScore<D>] {
        &self.candidate_scores
    }

    /// Borrows per-sample leave-one-out influence in input order.
    pub fn influences(&self) -> &[OrientationTensorInfluence] {
        &self.influences
    }

    /// Borrows aggregate confidence, isotropy, ratio, and influence diagnostics.
    pub const fn diagnostics(&self) -> &OrientationTensorDiagnostics<D> {
        &self.diagnostics
    }
}

#[derive(Clone, Debug, PartialEq)]
enum RatioSelection<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    Fixed(PrincipalAxisRatios<D>),
    LeaveOneOut {
        candidates: Vec<PrincipalAxisRatios<D>>,
        maximum_ratio: f64,
    },
}

/// Reusable immutable orientation-tensor estimation policy.
///
/// Cross-validation fits principal axes without each held-out positive sample.
/// For candidate ratios `r_j`, expected squared direction shares are
/// `p_j=r_j^2/sum_k(r_k^2)`. The score is the held-out-weighted mean of
/// squared share residuals. Within a training fold's numerically unresolved
/// repeated eigenspace, observed and expected shares are summed before the
/// residual is formed, so the score is independent of the arbitrary basis in
/// that subspace. Lowest score wins; exact score ties choose the
/// lexicographically smaller ratio array, independent of candidate order.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct OrientationTensorEstimator<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    selection: RatioSelection<D>,
    isotropy_threshold: f64,
}

impl<const D: usize> OrientationTensorEstimator<D>
where
    Dim<D>: SupportedDimension,
{
    /// Constructs an estimator with caller-supplied fixed ratios.
    ///
    /// # Errors
    ///
    /// Rejects a non-finite isotropy threshold or a value outside `[0,1]`.
    pub fn try_fixed(
        ratios: PrincipalAxisRatios<D>,
        isotropy_threshold: f64,
    ) -> Result<Self, OrientationTensorError> {
        validate_isotropy_threshold(isotropy_threshold)?;
        Ok(Self {
            selection: RatioSelection::Fixed(ratios),
            isotropy_threshold,
        })
    }

    /// Constructs a bounded deterministic leave-one-out candidate search.
    ///
    /// # Errors
    ///
    /// Rejects an invalid maximum, no candidates, duplicates, or a candidate
    /// whose largest ratio exceeds the explicit maximum. No candidates are
    /// generated, sorted, clipped, or repaired.
    pub fn try_cross_validated(
        candidates: Vec<PrincipalAxisRatios<D>>,
        maximum_ratio: f64,
        isotropy_threshold: f64,
    ) -> Result<Self, OrientationTensorError> {
        validate_isotropy_threshold(isotropy_threshold)?;
        if !maximum_ratio.is_finite() || maximum_ratio < 1.0 {
            return Err(OrientationTensorError::InvalidMaximumAxisRatio {
                value: maximum_ratio,
            });
        }
        if candidates.is_empty() {
            return Err(OrientationTensorError::EmptyCandidateSet);
        }
        for (index, candidate) in candidates.iter().enumerate() {
            if candidate.maximum() > maximum_ratio {
                return Err(OrientationTensorError::CandidateExceedsMaximum {
                    index,
                    candidate: candidate.maximum(),
                    maximum: maximum_ratio,
                });
            }
            for (earlier, previous) in candidates[..index].iter().enumerate() {
                if previous == candidate {
                    return Err(OrientationTensorError::DuplicateCandidate {
                        first: earlier,
                        second: index,
                    });
                }
            }
        }
        Ok(Self {
            selection: RatioSelection::LeaveOneOut {
                candidates,
                maximum_ratio,
            },
            isotropy_threshold,
        })
    }

    /// Estimates the sign-invariant tensor, axes, ratios, and diagnostics.
    ///
    /// # Errors
    ///
    /// Rejects empty or all-zero-weight samples, invalid sample weights (even
    /// if a future representation source bypasses construction), insufficient
    /// positive samples for leave-one-out selection, or non-finite numerical
    /// results. No eigenvalue clipping or hidden regularization is applied.
    pub fn try_estimate(
        &self,
        samples: &[OrientationTensorSample<D>],
    ) -> Result<OrientationTensorEstimate<D>, OrientationTensorError> {
        validate_samples(samples)?;
        let tensor = normalized_tensor(samples, None)?;
        let decomposition = decompose_tensor(tensor)?;
        let positive_sample_count = samples.iter().filter(|sample| sample.weight > 0.0).count();
        let normalized_weights = normalized_weights(samples, None)?;
        let maximum_weight_fraction = normalized_weights.iter().copied().fold(0.0_f64, f64::max);

        let (axis_ratios, candidate_scores, selection_kind) = match &self.selection {
            RatioSelection::Fixed(ratios) => (*ratios, Vec::new(), AxisRatioSelectionKind::Fixed),
            RatioSelection::LeaveOneOut { candidates, .. } => {
                if positive_sample_count < 2 {
                    return Err(OrientationTensorError::InsufficientPositiveSamples {
                        positive: positive_sample_count,
                    });
                }
                let scores = cross_validation_scores(samples, &normalized_weights, candidates)?;
                let selected = scores
                    .iter()
                    .min_by(|left, right| compare_candidate_scores(left, right))
                    .ok_or(OrientationTensorError::EmptyCandidateSet)?
                    .ratios;
                (selected, scores, AxisRatioSelectionKind::LeaveOneOut)
            }
        };

        let influences = leave_one_out_influences(samples, tensor, positive_sample_count)?;
        let (most_influential_sample, maximum_outlier_influence) = influences
            .iter()
            .map(|influence| (influence.sample_index, influence.normalized_tensor_change))
            .max_by(|left, right| {
                left.1
                    .total_cmp(&right.1)
                    .then_with(|| right.0.cmp(&left.0))
            })
            .map_or((None, 0.0), |(index, value)| (Some(index), value));

        let mut normalized_eigenvalue_gaps = Vec::with_capacity(D.saturating_sub(1));
        for axis in 0..D.saturating_sub(1) {
            normalized_eigenvalue_gaps.push(
                decomposition.normalized_eigenvalues[axis]
                    - decomposition.normalized_eigenvalues[axis + 1],
            );
        }
        let axis_confidence = std::array::from_fn(|axis| {
            if D == 1 {
                1.0
            } else if axis == 0 {
                normalized_eigenvalue_gaps[0]
            } else if axis == D - 1 {
                normalized_eigenvalue_gaps[D - 2]
            } else {
                normalized_eigenvalue_gaps[axis - 1].min(normalized_eigenvalue_gaps[axis])
            }
        });
        let isotropy_measure =
            decomposition.normalized_eigenvalues[0] - decomposition.normalized_eigenvalues[D - 1];
        let diagnostics = OrientationTensorDiagnostics {
            normalized_eigenvalue_gaps,
            axis_confidence,
            isotropic: isotropy_measure <= self.isotropy_threshold,
            isotropy_threshold: self.isotropy_threshold,
            positive_sample_count,
            maximum_weight_fraction,
            selected_maximum_ratio: axis_ratios.maximum(),
            maximum_outlier_influence,
            most_influential_sample,
            selection_kind,
        };

        Ok(OrientationTensorEstimate {
            tensor,
            principal_axes: decomposition.axes,
            eigenvalues: decomposition.eigenvalues,
            normalized_eigenvalues: decomposition.normalized_eigenvalues,
            axis_ratios,
            candidate_scores,
            influences,
            diagnostics,
        })
    }

    /// Returns the explicit isotropy threshold.
    #[must_use]
    pub const fn isotropy_threshold(&self) -> f64 {
        self.isotropy_threshold
    }

    /// Returns the selection kind configured for this estimator.
    #[must_use]
    pub const fn selection_kind(&self) -> AxisRatioSelectionKind {
        match self.selection {
            RatioSelection::Fixed(_) => AxisRatioSelectionKind::Fixed,
            RatioSelection::LeaveOneOut { .. } => AxisRatioSelectionKind::LeaveOneOut,
        }
    }

    /// Returns the explicit candidate maximum for cross-validation.
    #[must_use]
    pub const fn maximum_axis_ratio(&self) -> Option<f64> {
        match self.selection {
            RatioSelection::Fixed(_) => None,
            RatioSelection::LeaveOneOut { maximum_ratio, .. } => Some(maximum_ratio),
        }
    }
}

/// Structured orientation-tensor construction or estimation failure.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum OrientationTensorError {
    /// A sample weight is NaN or infinite.
    NonFiniteWeight {
        /// Sample index, or zero during individual sample construction.
        index: usize,
        /// Rejected value.
        weight: f64,
    },
    /// A sample weight is negative.
    NegativeWeight {
        /// Sample index, or zero during individual sample construction.
        index: usize,
        /// Rejected value.
        weight: f64,
    },
    /// No samples were supplied.
    EmptySamples,
    /// Every sample has zero weight.
    NoPositiveWeight,
    /// An axis ratio is NaN or infinite.
    NonFiniteAxisRatio {
        /// Principal-axis index.
        axis: usize,
        /// Rejected value.
        value: f64,
    },
    /// An axis ratio is below one.
    AxisRatioBelowOne {
        /// Principal-axis index.
        axis: usize,
        /// Rejected value.
        value: f64,
    },
    /// Ratios increase instead of following descending eigenvalue order.
    UnorderedAxisRatios {
        /// Earlier axis index.
        first: usize,
        /// Later axis index.
        second: usize,
    },
    /// The final ratio is not exactly one.
    UnnormalizedAxisRatios {
        /// Rejected last ratio.
        last: f64,
    },
    /// A squared ratio share underflows after normalization by the maximum.
    NonRepresentableRatioSquare {
        /// Principal-axis index.
        axis: usize,
        /// Rejected axis ratio.
        value: f64,
        /// Largest candidate ratio.
        maximum: f64,
    },
    /// The isotropy threshold is non-finite or outside `[0,1]`.
    InvalidIsotropyThreshold {
        /// Rejected value.
        value: f64,
    },
    /// The candidate maximum is non-finite or below one.
    InvalidMaximumAxisRatio {
        /// Rejected value.
        value: f64,
    },
    /// Cross-validation received no candidates.
    EmptyCandidateSet,
    /// A candidate exceeds the explicit maximum.
    CandidateExceedsMaximum {
        /// Candidate index.
        index: usize,
        /// Candidate's largest ratio.
        candidate: f64,
        /// Explicit search maximum.
        maximum: f64,
    },
    /// Two exactly equal normalized candidates were supplied.
    DuplicateCandidate {
        /// Earlier candidate index.
        first: usize,
        /// Later candidate index.
        second: usize,
    },
    /// Leave-one-out selection needs at least two positive-weight samples.
    InsufficientPositiveSamples {
        /// Available strictly positive weights.
        positive: usize,
    },
    /// A finite-input arithmetic or eigendecomposition result was not finite.
    NonFiniteNumericalResult {
        /// Stable operation label.
        operation: &'static str,
    },
    /// The bounded symmetric eigendecomposition did not converge.
    EigendecompositionDidNotConverge {
        /// Fixed recorded backend iteration limit.
        maximum_iterations: usize,
    },
    /// The symmetric eigensolver returned a negative eigenvalue; no clipping
    /// was applied.
    NegativeEigenvalue {
        /// Eigenvalue index after descending sort.
        axis: usize,
        /// Rejected value.
        value: f64,
    },
    /// A finite computed influence exceeded one by more than the documented
    /// floating-point roundoff tolerance.
    InfluenceOutsideRoundoffTolerance {
        /// Original sample index.
        sample_index: usize,
        /// Rejected computed influence.
        value: f64,
        /// Explicit dimension-scaled tolerance above one.
        tolerance: f64,
    },
}

impl fmt::Display for OrientationTensorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteWeight { index, weight } => {
                write!(formatter, "sample weight {index} is not finite: {weight}")
            }
            Self::NegativeWeight { index, weight } => {
                write!(formatter, "sample weight {index} is negative: {weight}")
            }
            Self::EmptySamples => {
                formatter.write_str("orientation tensor needs at least one sample")
            }
            Self::NoPositiveWeight => {
                formatter.write_str("orientation tensor needs a positive weight")
            }
            Self::NonFiniteAxisRatio { axis, value } => {
                write!(formatter, "axis ratio {axis} is not finite: {value}")
            }
            Self::AxisRatioBelowOne { axis, value } => {
                write!(formatter, "axis ratio {axis} is below one: {value}")
            }
            Self::UnorderedAxisRatios { first, second } => write!(
                formatter,
                "axis ratios {first} and {second} are not nonincreasing"
            ),
            Self::UnnormalizedAxisRatios { last } => {
                write!(formatter, "last axis ratio must be exactly one, got {last}")
            }
            Self::NonRepresentableRatioSquare {
                axis,
                value,
                maximum,
            } => write!(
                formatter,
                "axis ratio {axis}={value} has no represented squared share relative to {maximum}"
            ),
            Self::InvalidIsotropyThreshold { value } => {
                write!(
                    formatter,
                    "isotropy threshold must be finite in [0,1], got {value}"
                )
            }
            Self::InvalidMaximumAxisRatio { value } => {
                write!(
                    formatter,
                    "maximum axis ratio must be finite and at least one, got {value}"
                )
            }
            Self::EmptyCandidateSet => formatter.write_str("axis-ratio candidate set is empty"),
            Self::CandidateExceedsMaximum {
                index,
                candidate,
                maximum,
            } => write!(
                formatter,
                "axis-ratio candidate {index} maximum {candidate} exceeds {maximum}"
            ),
            Self::DuplicateCandidate { first, second } => {
                write!(
                    formatter,
                    "axis-ratio candidates {first} and {second} are duplicates"
                )
            }
            Self::InsufficientPositiveSamples { positive } => write!(
                formatter,
                "leave-one-out selection needs at least two positive samples, got {positive}"
            ),
            Self::NonFiniteNumericalResult { operation } => {
                write!(
                    formatter,
                    "orientation-tensor operation was not finite: {operation}"
                )
            }
            Self::EigendecompositionDidNotConverge { maximum_iterations } => write!(
                formatter,
                "orientation-tensor eigendecomposition exceeded {maximum_iterations} iterations"
            ),
            Self::NegativeEigenvalue { axis, value } => write!(
                formatter,
                "orientation tensor eigenvalue {axis} is negative ({value}); no clipping applied"
            ),
            Self::InfluenceOutsideRoundoffTolerance {
                sample_index,
                value,
                tolerance,
            } => write!(
                formatter,
                "sample {sample_index} influence {value} exceeds one beyond roundoff tolerance {tolerance}"
            ),
        }
    }
}

impl Error for OrientationTensorError {}

fn validate_isotropy_threshold(value: f64) -> Result<(), OrientationTensorError> {
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        return Err(OrientationTensorError::InvalidIsotropyThreshold { value });
    }
    Ok(())
}

fn validate_samples<const D: usize>(
    samples: &[OrientationTensorSample<D>],
) -> Result<(), OrientationTensorError>
where
    Dim<D>: SupportedDimension,
{
    if samples.is_empty() {
        return Err(OrientationTensorError::EmptySamples);
    }
    let mut positive = false;
    for (index, sample) in samples.iter().enumerate() {
        if !sample.weight.is_finite() {
            return Err(OrientationTensorError::NonFiniteWeight {
                index,
                weight: sample.weight,
            });
        }
        if sample.weight < 0.0 {
            return Err(OrientationTensorError::NegativeWeight {
                index,
                weight: sample.weight,
            });
        }
        positive |= sample.weight > 0.0;
    }
    if !positive {
        return Err(OrientationTensorError::NoPositiveWeight);
    }
    Ok(())
}

fn normalized_weights<const D: usize>(
    samples: &[OrientationTensorSample<D>],
    excluded: Option<usize>,
) -> Result<Vec<f64>, OrientationTensorError>
where
    Dim<D>: SupportedDimension,
{
    let maximum = samples
        .iter()
        .enumerate()
        .filter(|(index, _)| Some(*index) != excluded)
        .map(|(_, sample)| sample.weight)
        .fold(0.0_f64, f64::max);
    if maximum == 0.0 {
        return Err(OrientationTensorError::NoPositiveWeight);
    }
    let mut scaled_sum = 0.0;
    let mut compensation = 0.0;
    for (index, sample) in samples.iter().enumerate() {
        if Some(index) == excluded {
            continue;
        }
        let scaled = sample.weight / maximum;
        let adjusted = scaled - compensation;
        let next = scaled_sum + adjusted;
        compensation = (next - scaled_sum) - adjusted;
        scaled_sum = next;
    }
    if !scaled_sum.is_finite() || scaled_sum <= 0.0 {
        return Err(OrientationTensorError::NonFiniteNumericalResult {
            operation: "weight normalization",
        });
    }
    Ok(samples
        .iter()
        .enumerate()
        .map(|(index, sample)| {
            if Some(index) == excluded {
                0.0
            } else {
                (sample.weight / maximum) / scaled_sum
            }
        })
        .collect())
}

fn normalized_tensor<const D: usize>(
    samples: &[OrientationTensorSample<D>],
    excluded: Option<usize>,
) -> Result<[[f64; D]; D], OrientationTensorError>
where
    Dim<D>: SupportedDimension,
{
    let weights = normalized_weights(samples, excluded)?;
    let mut tensor = [[0.0; D]; D];
    let mut compensation = [[0.0; D]; D];
    for (index, sample) in samples.iter().enumerate() {
        if Some(index) == excluded || weights[index] == 0.0 {
            continue;
        }
        for row in 0..D {
            for column in row..D {
                let term = weights[index]
                    * sample.direction.components()[row]
                    * sample.direction.components()[column];
                let adjusted = term - compensation[row][column];
                let next = tensor[row][column] + adjusted;
                compensation[row][column] = (next - tensor[row][column]) - adjusted;
                tensor[row][column] = next;
                if !next.is_finite() {
                    return Err(OrientationTensorError::NonFiniteNumericalResult {
                        operation: "tensor accumulation",
                    });
                }
                tensor[column][row] = next;
            }
        }
    }
    Ok(tensor)
}

struct TensorDecomposition<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    axes: [UnitDirection<D>; D],
    eigenvalues: [f64; D],
    normalized_eigenvalues: [f64; D],
}

fn decompose_tensor<const D: usize>(
    tensor: [[f64; D]; D],
) -> Result<TensorDecomposition<D>, OrientationTensorError>
where
    Dim<D>: SupportedDimension,
{
    const MAXIMUM_EIGEN_ITERATIONS: usize = 64;
    let matrix = DMatrix::from_fn(D, D, |row, column| tensor[row][column]);
    let decomposition = SymmetricEigen::try_new(matrix, f64::EPSILON, MAXIMUM_EIGEN_ITERATIONS)
        .ok_or(OrientationTensorError::EigendecompositionDidNotConverge {
            maximum_iterations: MAXIMUM_EIGEN_ITERATIONS,
        })?;
    let mut order: Vec<usize> = (0..D).collect();
    order.sort_by(|left, right| {
        decomposition.eigenvalues[*right]
            .total_cmp(&decomposition.eigenvalues[*left])
            .then_with(|| left.cmp(right))
    });

    let mut eigenvalues = [0.0; D];
    let mut axis_components = [[0.0; D]; D];
    for (axis, source) in order.iter().copied().enumerate() {
        let value = decomposition.eigenvalues[source];
        if !value.is_finite() {
            return Err(OrientationTensorError::NonFiniteNumericalResult {
                operation: "symmetric eigendecomposition",
            });
        }
        if value < 0.0 {
            return Err(OrientationTensorError::NegativeEigenvalue { axis, value });
        }
        eigenvalues[axis] = value;
        for (component, output) in axis_components[axis].iter_mut().enumerate() {
            *output = decomposition.eigenvectors[(component, source)];
        }
        canonicalize_axis(&mut axis_components[axis]);
    }
    let eigenvalue_sum = eigenvalues.iter().sum::<f64>();
    if !eigenvalue_sum.is_finite() || eigenvalue_sum <= 0.0 {
        return Err(OrientationTensorError::NonFiniteNumericalResult {
            operation: "eigenvalue normalization",
        });
    }
    let normalized_eigenvalues = std::array::from_fn(|axis| eigenvalues[axis] / eigenvalue_sum);
    let mut axes = Vec::with_capacity(D);
    for components in axis_components {
        axes.push(UnitDirection::try_new(components).map_err(|_| {
            OrientationTensorError::NonFiniteNumericalResult {
                operation: "principal-axis normalization",
            }
        })?);
    }
    let axes: [UnitDirection<D>; D] =
        axes.try_into()
            .map_err(|_| OrientationTensorError::NonFiniteNumericalResult {
                operation: "principal-axis collection",
            })?;
    Ok(TensorDecomposition {
        axes,
        eigenvalues,
        normalized_eigenvalues,
    })
}

fn canonicalize_axis<const D: usize>(axis: &mut [f64; D]) {
    let pivot = axis
        .iter()
        .enumerate()
        .max_by(|left, right| {
            left.1
                .abs()
                .total_cmp(&right.1.abs())
                .then_with(|| right.0.cmp(&left.0))
        })
        .map_or(0, |(index, _)| index);
    if axis[pivot].is_sign_negative() {
        for component in axis.iter_mut() {
            *component = -*component;
        }
    }
    for component in axis.iter_mut() {
        if *component == 0.0 {
            *component = 0.0;
        }
    }
}

fn expected_shares<const D: usize>(ratios: PrincipalAxisRatios<D>) -> [f64; D]
where
    Dim<D>: SupportedDimension,
{
    let maximum = ratios.maximum();
    let squares: [f64; D] = std::array::from_fn(|axis| {
        let scaled = ratios.values[axis] / maximum;
        scaled * scaled
    });
    let sum = squares.iter().sum::<f64>();
    std::array::from_fn(|axis| squares[axis] / sum)
}

fn cross_validation_scores<const D: usize>(
    samples: &[OrientationTensorSample<D>],
    normalized_weights: &[f64],
    candidates: &[PrincipalAxisRatios<D>],
) -> Result<Vec<AxisRatioCandidateScore<D>>, OrientationTensorError>
where
    Dim<D>: SupportedDimension,
{
    let expected: Vec<[f64; D]> = candidates.iter().copied().map(expected_shares).collect();
    let mut totals = vec![0.0; candidates.len()];
    for (held_out, sample) in samples.iter().enumerate() {
        if sample.weight == 0.0 {
            continue;
        }
        let training = normalized_tensor(samples, Some(held_out))?;
        let decomposition = decompose_tensor(training)?;
        let observed: [f64; D] = std::array::from_fn(|axis| {
            let dot = sample
                .direction
                .components()
                .iter()
                .zip(decomposition.axes[axis].components())
                .map(|(left, right)| left * right)
                .sum::<f64>();
            dot * dot
        });
        for (candidate, total) in totals.iter_mut().enumerate() {
            let loss = grouped_share_loss(
                &observed,
                &expected[candidate],
                &decomposition.normalized_eigenvalues,
            );
            *total += normalized_weights[held_out] * loss;
            if !total.is_finite() {
                return Err(OrientationTensorError::NonFiniteNumericalResult {
                    operation: "cross-validation score",
                });
            }
        }
    }
    Ok(candidates
        .iter()
        .copied()
        .zip(totals)
        .map(|(ratios, score)| AxisRatioCandidateScore { ratios, score })
        .collect())
}

fn grouped_share_loss<const D: usize>(
    observed: &[f64; D],
    expected: &[f64; D],
    normalized_eigenvalues: &[f64; D],
) -> f64
where
    Dim<D>: SupportedDimension,
{
    let resolution = EIGENSPACE_RESOLUTION_FACTOR * dimension_as_f64::<D>() * f64::EPSILON;
    let mut loss = 0.0;
    let mut group_start = 0;
    let mut accumulated_observed_share = 0.0;
    let mut accumulated_expected_share = 0.0;
    for axis in 0..D {
        let group_ends = axis == D - 1
            || normalized_eigenvalues[axis] - normalized_eigenvalues[axis + 1] > resolution;
        if group_ends {
            let final_group = axis == D - 1;
            let (observed_share, expected_share) = if final_group {
                (
                    1.0 - accumulated_observed_share,
                    1.0 - accumulated_expected_share,
                )
            } else {
                let observed_share = observed[group_start..=axis].iter().sum::<f64>();
                let expected_share = expected[group_start..=axis].iter().sum::<f64>();
                accumulated_observed_share += observed_share;
                accumulated_expected_share += expected_share;
                (observed_share, expected_share)
            };
            let residual = observed_share - expected_share;
            loss += residual * residual;
            group_start = axis + 1;
        }
    }
    loss
}

fn dimension_as_f64<const D: usize>() -> f64
where
    Dim<D>: SupportedDimension,
{
    if D == 1 {
        1.0
    } else if D == 2 {
        2.0
    } else {
        3.0
    }
}

fn compare_candidate_scores<const D: usize>(
    left: &AxisRatioCandidateScore<D>,
    right: &AxisRatioCandidateScore<D>,
) -> Ordering
where
    Dim<D>: SupportedDimension,
{
    left.score.total_cmp(&right.score).then_with(|| {
        for axis in 0..D {
            let ordering = left.ratios.values[axis].total_cmp(&right.ratios.values[axis]);
            if ordering != Ordering::Equal {
                return ordering;
            }
        }
        Ordering::Equal
    })
}

fn leave_one_out_influences<const D: usize>(
    samples: &[OrientationTensorSample<D>],
    full: [[f64; D]; D],
    positive_sample_count: usize,
) -> Result<Vec<OrientationTensorInfluence>, OrientationTensorError>
where
    Dim<D>: SupportedDimension,
{
    let mut influences = Vec::with_capacity(samples.len());
    for (sample_index, sample) in samples.iter().enumerate() {
        let computed_change = if sample.weight == 0.0 {
            0.0
        } else if positive_sample_count == 1 {
            1.0
        } else {
            let reduced = normalized_tensor(samples, Some(sample_index))?;
            let squared_difference = full
                .iter()
                .zip(reduced)
                .flat_map(|(left, right)| left.iter().zip(right))
                .map(|(left, right)| {
                    let difference = left - right;
                    difference * difference
                })
                .sum::<f64>();
            (squared_difference / 2.0).sqrt()
        };
        if !computed_change.is_finite() {
            return Err(OrientationTensorError::NonFiniteNumericalResult {
                operation: "leave-one-out influence",
            });
        }
        let dimension = dimension_as_f64::<D>();
        let tolerance = INFLUENCE_ROUNDOFF_FACTOR * dimension * dimension * f64::EPSILON;
        let normalized_tensor_change = if computed_change <= 1.0 {
            computed_change
        } else if computed_change <= 1.0 + tolerance {
            1.0
        } else {
            return Err(OrientationTensorError::InfluenceOutsideRoundoffTolerance {
                sample_index,
                value: computed_change,
                tolerance,
            });
        };
        influences.push(OrientationTensorInfluence {
            sample_index,
            normalized_tensor_change,
        });
    }
    Ok(influences)
}
