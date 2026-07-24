//! Deterministic one-dimensional level-point extraction.
//!
//! [`FittedField<1>`] is evaluated only through its retained analytic value
//! and original-coordinate gradient. The extractor scans an explicit finite
//! resolution, refines value and derivative sign brackets by bisection, and
//! reports both isolated points and numerically degenerate level intervals.
//! It does not refit the field, use finite differences, or claim that a finite
//! scan proves the absence of arbitrarily oscillatory roots.
//!
//! ```
//! use std::error::Error;
//! use std::num::NonZeroU32;
//!
//! use georbf::{FittedField, LevelPointRequest, LevelPointSettings};
//!
//! # fn extract(model: &FittedField<1>) -> Result<(), Box<dyn Error>> {
//! let settings = LevelPointSettings::try_new(
//!     NonZeroU32::new(32).ok_or("scan intervals")?,
//!     NonZeroU32::new(64).ok_or("refinement iterations")?,
//!     1.0e-10,
//!     1.0e-9,
//!     1.0e-10,
//! )?;
//! let request = LevelPointRequest::try_new(0.0, -10.0, 10.0, settings)?;
//! let report = model.try_level_points(&request)?;
//! for point in report.points() {
//!     assert!(point.point().components()[0].is_finite());
//! }
//! # Ok(())
//! # }
//! ```

use std::error::Error;
use std::fmt;
use std::num::NonZeroU32;

use crate::execution::{
    ExecutionControl, ExecutionError, ExecutionOperation, ExecutionStage, ProgressTracker,
};
use crate::geometry::Point;
use crate::kernel::KernelDerivativeCapability;
use crate::model::{FittedField, FittedFieldEvaluationError, FittedFieldOutput};
use crate::problem_ir::ExecutionOptions;

mod isoline;

pub use isoline::{
    IsolineAmbiguityDecider, IsolineAmbiguousCell, IsolineBoundaryEndpoint, IsolineBoundarySide,
    IsolineCellPairing, IsolineDiagnostics, IsolineDomainAxis, IsolineError, IsolineMethod,
    IsolinePolyline, IsolineReport, IsolineRequest, IsolineRequestError, IsolineSettings,
    IsolineSettingsError, IsolineStorage, IsolineTolerance, IsolineVertex,
};

/// Explicit scan and bracket-refinement policy for one-dimensional level points.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct LevelPointSettings {
    scan_intervals: NonZeroU32,
    refinement_iterations: NonZeroU32,
    value_tolerance: f64,
    coordinate_tolerance: f64,
    derivative_tolerance: f64,
}

impl LevelPointSettings {
    /// Constructs a finite deterministic scan and refinement policy.
    ///
    /// Each requested scan interval is split once at its midpoint, so the
    /// extractor examines twice `scan_intervals` contiguous segments.
    ///
    /// # Errors
    ///
    /// Every tolerance must be positive and finite.
    pub fn try_new(
        scan_intervals: NonZeroU32,
        refinement_iterations: NonZeroU32,
        value_tolerance: f64,
        coordinate_tolerance: f64,
        derivative_tolerance: f64,
    ) -> Result<Self, LevelPointSettingsError> {
        validate_tolerance(LevelPointTolerance::Value, value_tolerance)?;
        validate_tolerance(LevelPointTolerance::Coordinate, coordinate_tolerance)?;
        validate_tolerance(LevelPointTolerance::Derivative, derivative_tolerance)?;
        Ok(Self {
            scan_intervals,
            refinement_iterations,
            value_tolerance,
            coordinate_tolerance,
            derivative_tolerance,
        })
    }

    /// Returns the caller-requested uniform interval count.
    #[must_use]
    pub const fn scan_intervals(self) -> NonZeroU32 {
        self.scan_intervals
    }

    /// Returns the maximum bisection iterations for each detected bracket.
    #[must_use]
    pub const fn refinement_iterations(self) -> NonZeroU32 {
        self.refinement_iterations
    }

    /// Returns the absolute fitted-value residual tolerance.
    #[must_use]
    pub const fn value_tolerance(self) -> f64 {
        self.value_tolerance
    }

    /// Returns the absolute original-coordinate bracket-width tolerance.
    #[must_use]
    pub const fn coordinate_tolerance(self) -> f64 {
        self.coordinate_tolerance
    }

    /// Returns the absolute original-coordinate derivative tolerance.
    #[must_use]
    pub const fn derivative_tolerance(self) -> f64 {
        self.derivative_tolerance
    }
}

/// Tolerance field rejected while constructing level-point settings.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LevelPointTolerance {
    /// Fitted-value residual tolerance.
    Value,
    /// Original-coordinate bracket-width tolerance.
    Coordinate,
    /// Original-coordinate derivative tolerance.
    Derivative,
}

impl fmt::Display for LevelPointTolerance {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Value => "value",
            Self::Coordinate => "coordinate",
            Self::Derivative => "derivative",
        })
    }
}

/// Invalid level-point search settings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LevelPointSettingsError {
    /// A tolerance was zero, negative, infinite, or NaN.
    InvalidTolerance {
        /// Rejected tolerance field.
        tolerance: LevelPointTolerance,
        /// Rejected scalar.
        value: f64,
    },
}

impl fmt::Display for LevelPointSettingsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTolerance { tolerance, value } => {
                write!(
                    formatter,
                    "level-point {tolerance} tolerance must be positive and finite, got {value}"
                )
            }
        }
    }
}

impl Error for LevelPointSettingsError {}

/// One finite target-level search over an original-coordinate interval.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct LevelPointRequest {
    level: f64,
    lower: f64,
    upper: f64,
    settings: LevelPointSettings,
}

impl LevelPointRequest {
    /// Constructs a one-dimensional level-point request.
    ///
    /// # Errors
    ///
    /// The level and bounds must be finite, `lower < upper`, and the
    /// original-coordinate span must be representable as a finite `f64`.
    pub fn try_new(
        level: f64,
        lower: f64,
        upper: f64,
        settings: LevelPointSettings,
    ) -> Result<Self, LevelPointRequestError> {
        if !level.is_finite() {
            return Err(LevelPointRequestError::NonFiniteLevel { level });
        }
        if !lower.is_finite() {
            return Err(LevelPointRequestError::NonFiniteBound {
                bound: LevelPointBound::Lower,
                value: lower,
            });
        }
        if !upper.is_finite() {
            return Err(LevelPointRequestError::NonFiniteBound {
                bound: LevelPointBound::Upper,
                value: upper,
            });
        }
        if lower >= upper {
            return Err(LevelPointRequestError::InvalidDomain { lower, upper });
        }
        if !(upper - lower).is_finite() {
            return Err(LevelPointRequestError::UnrepresentableDomainSpan { lower, upper });
        }
        Ok(Self {
            level,
            lower,
            upper,
            settings,
        })
    }

    /// Returns the requested fitted scalar value.
    #[must_use]
    pub const fn level(self) -> f64 {
        self.level
    }

    /// Returns the inclusive lower original-coordinate bound.
    #[must_use]
    pub const fn lower(self) -> f64 {
        self.lower
    }

    /// Returns the inclusive upper original-coordinate bound.
    #[must_use]
    pub const fn upper(self) -> f64 {
        self.upper
    }

    /// Returns the explicit scan and refinement settings.
    pub const fn settings(self) -> LevelPointSettings {
        self.settings
    }
}

/// Original-coordinate bound rejected by a request.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LevelPointBound {
    /// Lower bound.
    Lower,
    /// Upper bound.
    Upper,
}

impl fmt::Display for LevelPointBound {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Lower => "lower",
            Self::Upper => "upper",
        })
    }
}

/// Invalid one-dimensional target level or domain.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LevelPointRequestError {
    /// Target level was infinite or NaN.
    NonFiniteLevel {
        /// Rejected level.
        level: f64,
    },
    /// A domain bound was infinite or NaN.
    NonFiniteBound {
        /// Rejected bound.
        bound: LevelPointBound,
        /// Rejected scalar.
        value: f64,
    },
    /// Bounds were empty or reversed.
    InvalidDomain {
        /// Lower bound.
        lower: f64,
        /// Upper bound.
        upper: f64,
    },
    /// Subtracting the finite bounds produced an unrepresentable span.
    UnrepresentableDomainSpan {
        /// Lower bound.
        lower: f64,
        /// Upper bound.
        upper: f64,
    },
}

impl fmt::Display for LevelPointRequestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteLevel { level } => {
                write!(formatter, "level-point target must be finite, got {level}")
            }
            Self::NonFiniteBound { bound, value } => {
                write!(
                    formatter,
                    "level-point {bound} bound must be finite, got {value}"
                )
            }
            Self::InvalidDomain { lower, upper } => write!(
                formatter,
                "level-point domain must satisfy lower < upper, got [{lower}, {upper}]"
            ),
            Self::UnrepresentableDomainSpan { lower, upper } => write!(
                formatter,
                "level-point domain span is not representable for [{lower}, {upper}]"
            ),
        }
    }
}

impl Error for LevelPointRequestError {}

/// Classification of one isolated returned level point.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LevelPointKind {
    /// The point is exactly one of the requested domain boundaries.
    Boundary,
    /// A value-sign-change bracket located the point with non-stationary slope.
    Crossing,
    /// Analytic derivative evidence classifies the point as stationary.
    Stationary,
}

/// One isolated level point in original coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct LevelPoint {
    point: Point<1>,
    value: f64,
    residual: f64,
    derivative: f64,
    kind: LevelPointKind,
}

impl LevelPoint {
    /// Returns the original-coordinate point.
    pub const fn point(self) -> Point<1> {
        self.point
    }

    /// Returns the analytic fitted value at the returned coordinate.
    #[must_use]
    pub const fn value(self) -> f64 {
        self.value
    }

    /// Returns `value - requested_level`.
    #[must_use]
    pub const fn residual(self) -> f64 {
        self.residual
    }

    /// Returns the analytic derivative in original coordinates.
    #[must_use]
    pub const fn derivative(self) -> f64 {
        self.derivative
    }

    /// Returns the boundary, crossing, or stationary classification.
    #[must_use]
    pub const fn kind(self) -> LevelPointKind {
        self.kind
    }
}

/// One isolated analytic stationary-point candidate.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct StationaryLevelPoint {
    point: Point<1>,
    value: f64,
    residual: f64,
    derivative: f64,
    at_level: bool,
}

impl StationaryLevelPoint {
    /// Returns the original-coordinate stationary candidate.
    pub const fn point(self) -> Point<1> {
        self.point
    }

    /// Returns the analytic fitted value.
    #[must_use]
    pub const fn value(self) -> f64 {
        self.value
    }

    /// Returns `value - requested_level`.
    #[must_use]
    pub const fn residual(self) -> f64 {
        self.residual
    }

    /// Returns the analytic original-coordinate derivative.
    #[must_use]
    pub const fn derivative(self) -> f64 {
        self.derivative
    }

    /// Returns whether the value residual satisfies the requested tolerance.
    #[must_use]
    pub const fn is_at_level(self) -> bool {
        self.at_level
    }
}

/// One numerically continuous level interval at the explicit scan resolution.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct DegenerateLevelInterval {
    lower: f64,
    upper: f64,
    maximum_value_residual: f64,
    maximum_absolute_derivative: f64,
}

impl DegenerateLevelInterval {
    /// Returns the inclusive lower original-coordinate bound.
    #[must_use]
    pub const fn lower(self) -> f64 {
        self.lower
    }

    /// Returns the inclusive upper original-coordinate bound.
    #[must_use]
    pub const fn upper(self) -> f64 {
        self.upper
    }

    /// Returns the largest absolute level residual among the merged scan nodes.
    #[must_use]
    pub const fn maximum_value_residual(self) -> f64 {
        self.maximum_value_residual
    }

    /// Returns the largest absolute derivative among the merged scan nodes.
    #[must_use]
    pub const fn maximum_absolute_derivative(self) -> f64 {
        self.maximum_absolute_derivative
    }
}

/// One detected original-coordinate sign bracket.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct LevelPointInterval {
    lower: f64,
    upper: f64,
}

impl LevelPointInterval {
    /// Returns the lower bracket coordinate.
    #[must_use]
    pub const fn lower(self) -> f64 {
        self.lower
    }

    /// Returns the upper bracket coordinate.
    #[must_use]
    pub const fn upper(self) -> f64 {
        self.upper
    }
}

/// Deterministic scan, bracketing, and evaluation evidence.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct LevelPointDiagnostics {
    requested_scan_intervals: u32,
    examined_segments: usize,
    value_brackets: Vec<LevelPointInterval>,
    stationary_brackets: Vec<LevelPointInterval>,
    evaluations: usize,
}

impl LevelPointDiagnostics {
    /// Returns the caller-requested interval count.
    #[must_use]
    pub const fn requested_scan_intervals(&self) -> u32 {
        self.requested_scan_intervals
    }

    /// Returns the actual segment count after deterministic midpoint splitting.
    #[must_use]
    pub const fn examined_segments(&self) -> usize {
        self.examined_segments
    }

    /// Borrows every fitted-value sign-change interval in scan order.
    pub fn value_brackets(&self) -> &[LevelPointInterval] {
        &self.value_brackets
    }

    /// Borrows every analytic-derivative bracket in scan order.
    ///
    /// Each interval has opposite-sign endpoint derivatives or collapses to
    /// one scan node whose derivative is exactly zero. A merely
    /// tolerance-small derivative remains candidate evidence, not a bracket.
    pub fn stationary_brackets(&self) -> &[LevelPointInterval] {
        &self.stationary_brackets
    }

    /// Returns all fitted-field evaluations, including refinement work.
    #[must_use]
    pub const fn evaluations(&self) -> usize {
        self.evaluations
    }
}

/// Complete isolated and non-isolated evidence for one request.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct LevelPointReport {
    request: LevelPointRequest,
    points: Vec<LevelPoint>,
    stationary_points: Vec<StationaryLevelPoint>,
    degenerate_intervals: Vec<DegenerateLevelInterval>,
    diagnostics: LevelPointDiagnostics,
}

impl LevelPointReport {
    /// Returns the validated request.
    pub const fn request(&self) -> LevelPointRequest {
        self.request
    }

    /// Borrows isolated level points in increasing original-coordinate order.
    pub fn points(&self) -> &[LevelPoint] {
        &self.points
    }

    /// Borrows isolated stationary candidates in increasing coordinate order.
    pub fn stationary_points(&self) -> &[StationaryLevelPoint] {
        &self.stationary_points
    }

    /// Borrows merged numerically degenerate level intervals.
    pub fn degenerate_intervals(&self) -> &[DegenerateLevelInterval] {
        &self.degenerate_intervals
    }

    /// Returns whether the reported level set contains a non-isolated interval.
    #[must_use]
    pub fn has_non_isolated_level_set(&self) -> bool {
        !self.degenerate_intervals.is_empty()
    }

    /// Borrows deterministic scan and refinement evidence.
    pub const fn diagnostics(&self) -> &LevelPointDiagnostics {
        &self.diagnostics
    }
}

/// Logical allocation owned by level-point extraction.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LevelPointStorage {
    /// Analytic samples at deterministic scan nodes.
    ScanSamples,
    /// Candidate isolated roots before tolerance deduplication.
    RootCandidates,
    /// Candidate isolated stationary points.
    StationaryCandidates,
    /// Returned isolated level points.
    LevelPoints,
    /// Returned stationary-point diagnostics.
    StationaryPoints,
    /// Merged degenerate level intervals.
    DegenerateIntervals,
    /// Fitted-value sign-change brackets.
    ValueBrackets,
    /// Analytic-derivative sign-change brackets.
    StationaryBrackets,
}

impl fmt::Display for LevelPointStorage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ScanSamples => "scan samples",
            Self::RootCandidates => "root candidates",
            Self::StationaryCandidates => "stationary candidates",
            Self::LevelPoints => "level points",
            Self::StationaryPoints => "stationary points",
            Self::DegenerateIntervals => "degenerate intervals",
            Self::ValueBrackets => "value brackets",
            Self::StationaryBrackets => "stationary brackets",
        })
    }
}

/// Quantity whose detected bracket exhausted its explicit iteration limit.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LevelPointRefinement {
    /// Fitted value minus target level.
    Value,
    /// Analytic original-coordinate derivative.
    Derivative,
}

impl fmt::Display for LevelPointRefinement {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Value => "value residual",
            Self::Derivative => "derivative",
        })
    }
}

/// Structured level-point extraction failure.
#[derive(Debug)]
pub enum LevelPointError {
    /// Scan or refinement work arithmetic overflowed.
    WorkBudgetOverflow {
        /// Requested interval count.
        scan_intervals: u32,
        /// Requested iterations per bracket.
        refinement_iterations: u32,
    },
    /// A logical output could not reserve its complete requested capacity.
    AllocationFailed {
        /// Logical storage.
        storage: LevelPointStorage,
        /// Requested element capacity.
        requested: usize,
    },
    /// Analytic gradients are not defined throughout the requested domain.
    UnsupportedGradientCapability {
        /// Reported fitted-field gradient capability.
        capability: KernelDerivativeCapability,
    },
    /// Reusable analytic-gradient evaluation storage could not be prepared.
    Preparation {
        /// Fitted-field failure.
        source: Box<FittedFieldEvaluationError<1>>,
    },
    /// The fitted field could not be evaluated at one original coordinate.
    Evaluation {
        /// Original coordinate.
        coordinate: f64,
        /// Fitted-field failure.
        source: Box<FittedFieldEvaluationError<1>>,
    },
    /// A derived original coordinate could not form a finite geometry point.
    NonFiniteCoordinate {
        /// Rejected derived coordinate.
        coordinate: f64,
    },
    /// Subtracting the finite fitted value and target level was not representable.
    NonFiniteResidual {
        /// Original coordinate.
        coordinate: f64,
        /// Finite fitted value.
        value: f64,
        /// Finite requested level.
        level: f64,
    },
    /// A bracket remained wider than the coordinate tolerance with excessive residual.
    RefinementLimitReached {
        /// Refined quantity.
        quantity: LevelPointRefinement,
        /// Remaining lower coordinate.
        lower: f64,
        /// Remaining upper coordinate.
        upper: f64,
        /// Smallest absolute quantity observed at the last bracket endpoints.
        absolute_residual: f64,
    },
    /// Caller execution policy or cancellation failure.
    Execution(ExecutionError),
}

impl From<ExecutionError> for LevelPointError {
    fn from(source: ExecutionError) -> Self {
        Self::Execution(source)
    }
}

impl fmt::Display for LevelPointError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WorkBudgetOverflow {
                scan_intervals,
                refinement_iterations,
            } => write!(
                formatter,
                "level-point work budget is not representable for {scan_intervals} scan intervals and {refinement_iterations} refinement iterations"
            ),
            Self::AllocationFailed { storage, requested } => write!(
                formatter,
                "level-point {storage} could not reserve {requested} elements"
            ),
            Self::UnsupportedGradientCapability { capability } => write!(
                formatter,
                "level-point extraction requires gradients supported everywhere, got {capability:?}"
            ),
            Self::Preparation { source } => {
                write!(
                    formatter,
                    "level-point evaluation preparation failed: {source}"
                )
            }
            Self::Evaluation { coordinate, source } => write!(
                formatter,
                "level-point fitted-field evaluation failed at {coordinate}: {source}"
            ),
            Self::NonFiniteCoordinate { coordinate } => write!(
                formatter,
                "level-point derived coordinate is not finite: {coordinate}"
            ),
            Self::NonFiniteResidual {
                coordinate,
                value,
                level,
            } => write!(
                formatter,
                "level-point residual is not finite at {coordinate}: {value} - {level}"
            ),
            Self::RefinementLimitReached {
                quantity,
                lower,
                upper,
                absolute_residual,
            } => write!(
                formatter,
                "level-point {quantity} refinement exhausted on [{lower}, {upper}] with absolute residual {absolute_residual}"
            ),
            Self::Execution(source) => source.fmt(formatter),
        }
    }
}

impl Error for LevelPointError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Preparation { source } | Self::Evaluation { source, .. } => Some(source.as_ref()),
            Self::Execution(source) => Some(source),
            Self::WorkBudgetOverflow { .. }
            | Self::AllocationFailed { .. }
            | Self::UnsupportedGradientCapability { .. }
            | Self::NonFiniteCoordinate { .. }
            | Self::NonFiniteResidual { .. }
            | Self::RefinementLimitReached { .. } => None,
        }
    }
}

impl FittedField<1> {
    /// Extracts deterministic isolated and degenerate level-point evidence.
    ///
    /// The field is sampled and refined in its original-coordinate convention.
    /// Analytic gradients locate stationary candidates and tangencies; no
    /// finite-difference derivative or implicit refit is used.
    ///
    /// # Errors
    ///
    /// The field's gradient capability must be
    /// [`KernelDerivativeCapability::SupportedEverywhere`]. Returns structured
    /// capability, preparation, evaluation, allocation, arithmetic, or
    /// refinement failures. No partial report is returned.
    pub fn try_level_points(
        &self,
        request: &LevelPointRequest,
    ) -> Result<LevelPointReport, LevelPointError> {
        self.try_level_points_with_control(
            request,
            ExecutionOptions::default(),
            ExecutionControl::default(),
        )
    }

    /// Extracts level points with explicit serial execution metadata and controls.
    ///
    /// The current algorithm rejects an explicit thread count above one before
    /// preparing fitted-field scratch or evaluating a point. Cancellation is
    /// checked before and after every analytic fitted-field evaluation and
    /// before the successful terminal progress event.
    ///
    /// # Errors
    ///
    /// Returns the same failures as [`Self::try_level_points`], plus structured
    /// execution-policy and cancellation failures.
    pub fn try_level_points_with_control(
        &self,
        request: &LevelPointRequest,
        execution: ExecutionOptions,
        control: ExecutionControl<'_>,
    ) -> Result<LevelPointReport, LevelPointError> {
        let (segment_count, maximum_evaluations) = work_budget(request)?;
        let mut progress = ProgressTracker::try_new(
            control,
            ExecutionOperation::LevelPointExtraction,
            execution,
            maximum_evaluations,
        )?;
        let gradient_capability = self.capabilities().gradient();
        progress.observe_result(
            ExecutionStage::Started,
            if gradient_capability == KernelDerivativeCapability::SupportedEverywhere {
                Ok(())
            } else {
                Err(LevelPointError::UnsupportedGradientCapability {
                    capability: gradient_capability,
                })
            },
        )?;
        let scratch_result = self
            .try_evaluation_scratch(FittedFieldOutput::Gradient)
            .map_err(|source| LevelPointError::Preparation {
                source: Box::new(source),
            });
        let mut scratch = progress.observe_result(ExecutionStage::Started, scratch_result)?;
        let mut evaluation_count = 0_usize;
        let mut report = extract_level_points(request, segment_count, |coordinate| {
            let point = Point::try_new([coordinate])
                .map_err(|_| LevelPointError::NonFiniteCoordinate { coordinate })?;
            let result = self
                .try_evaluate_with_scratch(point, &mut scratch)
                .map(|evaluation| {
                    let value = evaluation.value();
                    let residual = value - request.level;
                    if !residual.is_finite() {
                        return Err(LevelPointError::NonFiniteResidual {
                            coordinate,
                            value,
                            level: request.level,
                        });
                    }
                    Ok(AnalyticSample {
                        coordinate,
                        value,
                        residual,
                        derivative: evaluation.gradient().components()[0],
                    })
                })
                .map_err(|source| LevelPointError::Evaluation {
                    coordinate,
                    source: Box::new(source),
                })
                .and_then(|sample| sample);
            let sample = progress.finish_work(ExecutionStage::LevelPointEvaluation, result)?;
            evaluation_count = evaluation_count.saturating_add(1);
            Ok(sample)
        })?;
        report.diagnostics.evaluations = evaluation_count;
        progress.complete()?;
        Ok(report)
    }
}

#[derive(Clone, Copy, Debug)]
struct AnalyticSample {
    coordinate: f64,
    value: f64,
    residual: f64,
    derivative: f64,
}

#[derive(Clone, Copy, Debug)]
struct RootCandidate {
    sample: AnalyticSample,
    stationary: bool,
}

#[allow(clippy::too_many_lines)]
fn extract_level_points(
    request: &LevelPointRequest,
    segment_count: usize,
    mut evaluate: impl FnMut(f64) -> Result<AnalyticSample, LevelPointError>,
) -> Result<LevelPointReport, LevelPointError> {
    let node_count = segment_count
        .checked_add(1)
        .ok_or_else(|| work_overflow(request))?;
    let mut samples = try_vec(node_count, LevelPointStorage::ScanSamples)?;
    let segment_count_u32 = u32::try_from(segment_count).map_err(|_| work_overflow(request))?;
    for index in 0..=segment_count_u32 {
        let coordinate = if index == 0 {
            request.lower
        } else if index == segment_count_u32 {
            request.upper
        } else {
            let ratio = f64::from(index) / f64::from(segment_count_u32);
            request.lower.mul_add(1.0 - ratio, request.upper * ratio)
        };
        samples.push(evaluate(coordinate)?);
    }

    let mut degenerate_intervals = try_vec(segment_count, LevelPointStorage::DegenerateIntervals)?;
    detect_degenerate_intervals(request.settings, &samples, &mut degenerate_intervals);

    let stationary_capacity = node_count
        .checked_add(segment_count)
        .ok_or_else(|| work_overflow(request))?;
    let root_capacity = node_count
        .checked_add(segment_count)
        .and_then(|value_candidates| value_candidates.checked_add(stationary_capacity))
        .ok_or_else(|| work_overflow(request))?;
    let mut root_candidates = try_vec(root_capacity, LevelPointStorage::RootCandidates)?;
    let mut stationary_candidates =
        try_vec(stationary_capacity, LevelPointStorage::StationaryCandidates)?;
    let mut value_brackets = try_vec(segment_count, LevelPointStorage::ValueBrackets)?;
    let mut stationary_brackets = try_vec(segment_count, LevelPointStorage::StationaryBrackets)?;

    for (index, sample) in samples.iter().copied().enumerate() {
        if inside_degenerate(sample.coordinate, &degenerate_intervals) {
            continue;
        }
        if sample.residual.abs() <= request.settings.value_tolerance {
            root_candidates.push(RootCandidate {
                sample,
                stationary: sample.derivative.abs() <= request.settings.derivative_tolerance,
            });
        }
        if sample.derivative.abs() <= request.settings.derivative_tolerance
            && isolated_stationary_node(index, &samples, request.settings.derivative_tolerance)
        {
            stationary_candidates.push(sample);
            if let Some(bracket) = stationary_node_bracket(index, &samples) {
                stationary_brackets.push(bracket);
            }
        }
    }

    for window in samples.windows(2) {
        let left = window[0];
        let right = window[1];
        if segment_is_degenerate(request.settings, left, right) {
            continue;
        }
        if opposite_signs(left.residual, right.residual)
            && left.residual.abs() > request.settings.value_tolerance
            && right.residual.abs() > request.settings.value_tolerance
        {
            value_brackets.push(LevelPointInterval {
                lower: left.coordinate,
                upper: right.coordinate,
            });
            root_candidates.push(RootCandidate {
                sample: refine_bracket(
                    request,
                    LevelPointRefinement::Value,
                    left,
                    right,
                    &mut evaluate,
                )?,
                stationary: false,
            });
        }
        if opposite_signs(left.derivative, right.derivative)
            && left.derivative.abs() > request.settings.derivative_tolerance
            && right.derivative.abs() > request.settings.derivative_tolerance
        {
            stationary_brackets.push(LevelPointInterval {
                lower: left.coordinate,
                upper: right.coordinate,
            });
            stationary_candidates.push(refine_bracket(
                request,
                LevelPointRefinement::Derivative,
                left,
                right,
                &mut evaluate,
            )?);
        }
    }

    deduplicate_samples(
        &mut stationary_candidates,
        request.settings.coordinate_tolerance,
    );
    for sample in &stationary_candidates {
        if sample.residual.abs() <= request.settings.value_tolerance {
            root_candidates.push(RootCandidate {
                sample: *sample,
                stationary: true,
            });
        }
    }
    deduplicate_roots(&mut root_candidates, request.settings.coordinate_tolerance);

    let mut points = try_vec(root_candidates.len(), LevelPointStorage::LevelPoints)?;
    for candidate in root_candidates {
        let coordinate = candidate.sample.coordinate;
        let kind = if coordinate.to_bits() == request.lower.to_bits()
            || coordinate.to_bits() == request.upper.to_bits()
        {
            LevelPointKind::Boundary
        } else if candidate.stationary
            || candidate.sample.derivative.abs() <= request.settings.derivative_tolerance
        {
            LevelPointKind::Stationary
        } else {
            LevelPointKind::Crossing
        };
        points.push(LevelPoint {
            point: Point::try_new([coordinate])
                .map_err(|_| LevelPointError::NonFiniteCoordinate { coordinate })?,
            value: candidate.sample.value,
            residual: candidate.sample.residual,
            derivative: candidate.sample.derivative,
            kind,
        });
    }

    let mut stationary_points = try_vec(
        stationary_candidates.len(),
        LevelPointStorage::StationaryPoints,
    )?;
    for sample in stationary_candidates {
        stationary_points.push(StationaryLevelPoint {
            point: Point::try_new([sample.coordinate]).map_err(|_| {
                LevelPointError::NonFiniteCoordinate {
                    coordinate: sample.coordinate,
                }
            })?,
            value: sample.value,
            residual: sample.residual,
            derivative: sample.derivative,
            at_level: sample.residual.abs() <= request.settings.value_tolerance,
        });
    }

    Ok(LevelPointReport {
        request: *request,
        points,
        stationary_points,
        degenerate_intervals,
        diagnostics: LevelPointDiagnostics {
            requested_scan_intervals: request.settings.scan_intervals.get(),
            examined_segments: segment_count,
            value_brackets,
            stationary_brackets,
            evaluations: 0,
        },
    })
}

fn refine_bracket(
    request: &LevelPointRequest,
    quantity: LevelPointRefinement,
    mut left: AnalyticSample,
    mut right: AnalyticSample,
    evaluate: &mut impl FnMut(f64) -> Result<AnalyticSample, LevelPointError>,
) -> Result<AnalyticSample, LevelPointError> {
    let tolerance = match quantity {
        LevelPointRefinement::Value => request.settings.value_tolerance,
        LevelPointRefinement::Derivative => request.settings.derivative_tolerance,
    };
    for _ in 0..request.settings.refinement_iterations.get() {
        let left_measure = measure(left, quantity);
        if left_measure.abs() <= tolerance {
            return Ok(left);
        }
        let right_measure = measure(right, quantity);
        if right_measure.abs() <= tolerance {
            return Ok(right);
        }
        let middle_coordinate = left.coordinate + (right.coordinate - left.coordinate) * 0.5;
        let middle = evaluate(middle_coordinate)?;
        let middle_measure = measure(middle, quantity);
        if middle_measure.abs() <= tolerance
            || right.coordinate - left.coordinate <= request.settings.coordinate_tolerance
        {
            return Ok(middle);
        }
        if same_sign(middle_measure, left_measure) {
            left = middle;
        } else {
            right = middle;
        }
    }
    Err(LevelPointError::RefinementLimitReached {
        quantity,
        lower: left.coordinate,
        upper: right.coordinate,
        absolute_residual: measure(left, quantity)
            .abs()
            .min(measure(right, quantity).abs()),
    })
}

fn measure(sample: AnalyticSample, quantity: LevelPointRefinement) -> f64 {
    match quantity {
        LevelPointRefinement::Value => sample.residual,
        LevelPointRefinement::Derivative => sample.derivative,
    }
}

fn detect_degenerate_intervals(
    settings: LevelPointSettings,
    samples: &[AnalyticSample],
    intervals: &mut Vec<DegenerateLevelInterval>,
) {
    for window in samples.windows(2) {
        let left = window[0];
        let right = window[1];
        if !segment_is_degenerate(settings, left, right) {
            continue;
        }
        let value_residual = left.residual.abs().max(right.residual.abs());
        let derivative = left.derivative.abs().max(right.derivative.abs());
        if let Some(previous) = intervals.last_mut()
            && previous.upper.to_bits() == left.coordinate.to_bits()
        {
            previous.upper = right.coordinate;
            previous.maximum_value_residual = previous.maximum_value_residual.max(value_residual);
            previous.maximum_absolute_derivative =
                previous.maximum_absolute_derivative.max(derivative);
        } else {
            intervals.push(DegenerateLevelInterval {
                lower: left.coordinate,
                upper: right.coordinate,
                maximum_value_residual: value_residual,
                maximum_absolute_derivative: derivative,
            });
        }
    }
}

fn segment_is_degenerate(
    settings: LevelPointSettings,
    left: AnalyticSample,
    right: AnalyticSample,
) -> bool {
    left.residual.abs() <= settings.value_tolerance
        && right.residual.abs() <= settings.value_tolerance
        && left.derivative.abs() <= settings.derivative_tolerance
        && right.derivative.abs() <= settings.derivative_tolerance
}

fn inside_degenerate(coordinate: f64, intervals: &[DegenerateLevelInterval]) -> bool {
    intervals
        .iter()
        .any(|interval| coordinate >= interval.lower && coordinate <= interval.upper)
}

fn isolated_stationary_node(
    index: usize,
    samples: &[AnalyticSample],
    derivative_tolerance: f64,
) -> bool {
    let left_is_nonzero = index
        .checked_sub(1)
        .and_then(|previous| samples.get(previous))
        .is_some_and(|sample| sample.derivative.abs() > derivative_tolerance);
    let right_is_nonzero = samples
        .get(index.saturating_add(1))
        .is_some_and(|sample| sample.derivative.abs() > derivative_tolerance);
    left_is_nonzero || right_is_nonzero
}

fn stationary_node_bracket(index: usize, samples: &[AnalyticSample]) -> Option<LevelPointInterval> {
    let sample = samples[index];
    let neighbors = index
        .checked_sub(1)
        .and_then(|previous| samples.get(previous))
        .zip(samples.get(index.saturating_add(1)));
    if let Some((left, right)) = neighbors
        && opposite_signs(left.derivative, right.derivative)
    {
        return Some(LevelPointInterval {
            lower: left.coordinate,
            upper: right.coordinate,
        });
    }
    (sample.derivative == 0.0).then_some(LevelPointInterval {
        lower: sample.coordinate,
        upper: sample.coordinate,
    })
}

fn deduplicate_samples(samples: &mut Vec<AnalyticSample>, coordinate_tolerance: f64) {
    samples.sort_by(|left, right| left.coordinate.total_cmp(&right.coordinate));
    samples.dedup_by(|later, earlier| {
        if later.coordinate - earlier.coordinate > coordinate_tolerance {
            return false;
        }
        if better_sample(*later, *earlier) {
            *earlier = *later;
        }
        true
    });
}

fn deduplicate_roots(roots: &mut Vec<RootCandidate>, coordinate_tolerance: f64) {
    roots.sort_by(|left, right| left.sample.coordinate.total_cmp(&right.sample.coordinate));
    roots.dedup_by(|later, earlier| {
        if later.sample.coordinate - earlier.sample.coordinate > coordinate_tolerance {
            return false;
        }
        let stationary = earlier.stationary || later.stationary;
        if better_sample(later.sample, earlier.sample) {
            earlier.sample = later.sample;
        }
        earlier.stationary = stationary;
        true
    });
}

fn better_sample(candidate: AnalyticSample, retained: AnalyticSample) -> bool {
    candidate
        .residual
        .abs()
        .total_cmp(&retained.residual.abs())
        .then_with(|| {
            candidate
                .derivative
                .abs()
                .total_cmp(&retained.derivative.abs())
        })
        .is_lt()
}

fn same_sign(left: f64, right: f64) -> bool {
    left.is_sign_positive() == right.is_sign_positive()
}

fn opposite_signs(left: f64, right: f64) -> bool {
    !same_sign(left, right)
}

fn work_budget(request: &LevelPointRequest) -> Result<(usize, usize), LevelPointError> {
    let segment_count_u32 = request
        .settings
        .scan_intervals
        .get()
        .checked_mul(2)
        .ok_or_else(|| work_overflow(request))?;
    let segment_count = usize::try_from(segment_count_u32).map_err(|_| work_overflow(request))?;
    let node_count = segment_count
        .checked_add(1)
        .ok_or_else(|| work_overflow(request))?;
    let iterations = usize::try_from(request.settings.refinement_iterations.get())
        .map_err(|_| work_overflow(request))?;
    let refinements = segment_count
        .checked_mul(2)
        .and_then(|brackets| brackets.checked_mul(iterations))
        .ok_or_else(|| work_overflow(request))?;
    let maximum_evaluations = node_count
        .checked_add(refinements)
        .ok_or_else(|| work_overflow(request))?;
    Ok((segment_count, maximum_evaluations))
}

fn work_overflow(request: &LevelPointRequest) -> LevelPointError {
    LevelPointError::WorkBudgetOverflow {
        scan_intervals: request.settings.scan_intervals.get(),
        refinement_iterations: request.settings.refinement_iterations.get(),
    }
}

fn try_vec<T>(requested: usize, storage: LevelPointStorage) -> Result<Vec<T>, LevelPointError> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(requested)
        .map_err(|_| LevelPointError::AllocationFailed { storage, requested })?;
    Ok(values)
}

fn validate_tolerance(
    tolerance: LevelPointTolerance,
    value: f64,
) -> Result<(), LevelPointSettingsError> {
    if !value.is_finite() || value <= 0.0 {
        return Err(LevelPointSettingsError::InvalidTolerance { tolerance, value });
    }
    Ok(())
}
