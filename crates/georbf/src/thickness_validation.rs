//! Post-fit sampled geometric thickness validation.
//!
//! This module measures separation only along the fitted-field normal at each
//! selected location. It does not prove a global minimum distance, mutate a
//! field problem, solve, or refit. Optional proposed [`LocalNormalThickness`]
//! values remain explicit inputs for a later user-visible refit.
//!
//! Validation is available only in one, two, or three dimensions:
//!
//! ```compile_fail
//! use georbf::SampledThicknessRequest;
//!
//! fn unsupported(_: Option<SampledThicknessRequest<4>>) {}
//! ```
//!
//! A request makes every search limit and geometric claim explicit:
//!
//! ```
//! use std::num::{NonZeroU32, NonZeroUsize};
//! use georbf::{
//!     LevelId, ObservationId, Point, SampledThicknessLocation,
//!     SampledThicknessRequest, SampledThicknessSettings, SemanticProvenance,
//!     SourceLocation, ThicknessDiagnosticKind,
//! };
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let provenance = SemanticProvenance::try_new(
//!     ObservationId::new(7),
//!     SourceLocation::try_new("samples.csv".to_owned(), NonZeroUsize::MIN)?,
//!     "m".to_owned(),
//!     "thickness.samples[0]".to_owned(),
//!     None,
//! )?;
//! let settings = SampledThicknessSettings::try_new(
//!     25.0,
//!     NonZeroU32::new(64).ok_or("search steps")?,
//!     NonZeroU32::new(48).ok_or("refinement iterations")?,
//!     1.0e-10,
//!     1.0e-6,
//!     1.0e-12,
//! )?;
//! let request = SampledThicknessRequest::try_new(
//!     LevelId::new(10),
//!     100.0,
//!     LevelId::new(20),
//!     120.0,
//!     5.0,
//!     vec![SampledThicknessLocation::new(Point::try_new([0.0, 0.0])?, provenance)],
//!     vec![0.0, 0.5, 0.95, 1.0],
//!     true,
//!     settings,
//! )?;
//! assert_eq!(
//!     request.diagnostics().kind(),
//!     ThicknessDiagnosticKind::SampledGeometricValidation
//! );
//! # Ok(())
//! # }
//! # example().unwrap();
//! ```

use std::error::Error;
use std::fmt;
use std::num::NonZeroU32;
use std::sync::Arc;

use crate::LevelId;
use crate::dimension::{Dim, SupportedDimension};
use crate::execution::{
    ExecutionControl, ExecutionError, ExecutionOperation, ExecutionStage, ProgressTracker,
};
use crate::geometry::{GeometryError, Point, UnitDirection};
use crate::model::{FittedField, FittedFieldEvaluationError, FittedFieldOutput};
use crate::problem_ir::{ExecutionOptions, ProblemIrError, SemanticProvenance};
use crate::thickness::{LocalNormalThickness, LocalNormalThicknessError, ThicknessDiagnostics};

/// One selected original-coordinate validation location and its provenance.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct SampledThicknessLocation<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    point: Point<D>,
    provenance: Arc<SemanticProvenance>,
}

impl<const D: usize> SampledThicknessLocation<D>
where
    Dim<D>: SupportedDimension,
{
    /// Constructs one selected location.
    pub fn new(point: Point<D>, provenance: SemanticProvenance) -> Self {
        Self {
            point,
            provenance: Arc::new(provenance),
        }
    }

    /// Returns the original-coordinate point.
    pub const fn point(&self) -> Point<D> {
        self.point
    }

    /// Borrows complete caller provenance.
    pub fn provenance(&self) -> &SemanticProvenance {
        &self.provenance
    }
}

/// Explicit deterministic search and refinement limits.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct SampledThicknessSettings {
    maximum_search_distance: f64,
    search_steps: NonZeroU32,
    refinement_iterations: NonZeroU32,
    value_tolerance: f64,
    distance_tolerance: f64,
    minimum_gradient_norm: f64,
}

impl SampledThicknessSettings {
    /// Constructs explicit search and refinement settings.
    ///
    /// # Errors
    ///
    /// Every scalar must be positive and finite.
    pub fn try_new(
        maximum_search_distance: f64,
        search_steps: NonZeroU32,
        refinement_iterations: NonZeroU32,
        value_tolerance: f64,
        distance_tolerance: f64,
        minimum_gradient_norm: f64,
    ) -> Result<Self, SampledThicknessSettingsError> {
        validate_positive_setting("maximum_search_distance", maximum_search_distance)?;
        validate_positive_setting("value_tolerance", value_tolerance)?;
        validate_positive_setting("distance_tolerance", distance_tolerance)?;
        validate_positive_setting("minimum_gradient_norm", minimum_gradient_norm)?;
        Ok(Self {
            maximum_search_distance,
            search_steps,
            refinement_iterations,
            value_tolerance,
            distance_tolerance,
            minimum_gradient_norm,
        })
    }

    /// Returns the maximum distance searched on each side of a sample.
    #[must_use]
    pub const fn maximum_search_distance(self) -> f64 {
        self.maximum_search_distance
    }

    /// Returns the uniform bracketing step count on each side.
    #[must_use]
    pub const fn search_steps(self) -> NonZeroU32 {
        self.search_steps
    }

    /// Returns the maximum bisection iteration count per bracket.
    #[must_use]
    pub const fn refinement_iterations(self) -> NonZeroU32 {
        self.refinement_iterations
    }

    /// Returns the absolute scalar residual tolerance.
    #[must_use]
    pub const fn value_tolerance(self) -> f64 {
        self.value_tolerance
    }

    /// Returns the absolute original-coordinate distance tolerance.
    #[must_use]
    pub const fn distance_tolerance(self) -> f64 {
        self.distance_tolerance
    }

    /// Returns the minimum usable original-coordinate gradient norm.
    #[must_use]
    pub const fn minimum_gradient_norm(self) -> f64 {
        self.minimum_gradient_norm
    }
}

/// Complete request for one adjacent-level sampled validation pass.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct SampledThicknessRequest<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    lower: LevelId,
    lower_value: f64,
    upper: LevelId,
    upper_value: f64,
    minimum_thickness: f64,
    locations: Vec<SampledThicknessLocation<D>>,
    quantiles: Vec<f64>,
    propose_constraints: bool,
    settings: SampledThicknessSettings,
}

impl<const D: usize> SampledThicknessRequest<D>
where
    Dim<D>: SupportedDimension,
{
    /// Constructs one validation request in original coordinate units.
    ///
    /// Quantiles use probabilities in `[0, 1]`. The report applies the
    /// deterministic type-7 convention: rank `q * (n - 1)` with linear
    /// interpolation between adjacent sorted distances.
    ///
    /// # Errors
    ///
    /// Rejects equal levels, non-finite or non-increasing level values, a
    /// non-positive minimum thickness, no locations or quantiles, or an
    /// invalid quantile probability.
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        lower: LevelId,
        lower_value: f64,
        upper: LevelId,
        upper_value: f64,
        minimum_thickness: f64,
        locations: Vec<SampledThicknessLocation<D>>,
        quantiles: Vec<f64>,
        propose_constraints: bool,
        settings: SampledThicknessSettings,
    ) -> Result<Self, SampledThicknessValidationError<D>> {
        if lower == upper {
            return Err(SampledThicknessValidationError::EqualLevels { level_id: lower });
        }
        if !lower_value.is_finite() || !upper_value.is_finite() || lower_value >= upper_value {
            return Err(SampledThicknessValidationError::InvalidLevelValues {
                lower_value,
                upper_value,
            });
        }
        if !minimum_thickness.is_finite() || minimum_thickness <= 0.0 {
            return Err(SampledThicknessValidationError::InvalidMinimumThickness {
                minimum_thickness,
            });
        }
        if locations.is_empty() {
            return Err(SampledThicknessValidationError::EmptyLocations);
        }
        if quantiles.is_empty() {
            return Err(SampledThicknessValidationError::EmptyQuantiles);
        }
        for (index, probability) in quantiles.iter().copied().enumerate() {
            if !probability.is_finite() || !(0.0..=1.0).contains(&probability) {
                return Err(SampledThicknessValidationError::InvalidQuantile {
                    index,
                    probability,
                });
            }
        }
        Ok(Self {
            lower,
            lower_value,
            upper,
            upper_value,
            minimum_thickness,
            locations,
            quantiles,
            propose_constraints,
            settings,
        })
    }

    /// Returns the lower level identifier.
    pub const fn lower(&self) -> LevelId {
        self.lower
    }

    /// Returns the lower fitted scalar value.
    #[must_use]
    pub const fn lower_value(&self) -> f64 {
        self.lower_value
    }

    /// Returns the upper level identifier.
    pub const fn upper(&self) -> LevelId {
        self.upper
    }

    /// Returns the upper fitted scalar value.
    #[must_use]
    pub const fn upper_value(&self) -> f64 {
        self.upper_value
    }

    /// Returns the required minimum geometric thickness.
    #[must_use]
    pub const fn minimum_thickness(&self) -> f64 {
        self.minimum_thickness
    }

    /// Borrows selected locations in deterministic input order.
    pub fn locations(&self) -> &[SampledThicknessLocation<D>] {
        &self.locations
    }

    /// Borrows requested probabilities in caller order.
    #[must_use]
    pub fn quantiles(&self) -> &[f64] {
        &self.quantiles
    }

    /// Returns whether violating locations should produce proposed constraints.
    #[must_use]
    pub const fn proposes_constraints(&self) -> bool {
        self.propose_constraints
    }

    /// Returns explicit search and refinement settings.
    pub const fn settings(&self) -> SampledThicknessSettings {
        self.settings
    }

    /// Labels this operation as sampled geometric evidence.
    pub const fn diagnostics(&self) -> ThicknessDiagnostics {
        ThicknessDiagnostics::SAMPLED_GEOMETRIC
    }
}

/// Why an endpoint intersection was unavailable.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ThicknessIntersectionFailure {
    /// No sign-changing bracket or tolerance hit occurred within the search radius.
    NotFound,
    /// A bracket was found but the explicit refinement limit was exhausted.
    RefinementLimitReached {
        /// Width of the remaining distance bracket.
        distance_interval: f64,
        /// Absolute residual at the last midpoint.
        value_residual: f64,
    },
}

/// Structured reason one selected location produced no distance measurement.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SampledThicknessFailureReason {
    /// The fitted gradient could not define a stable local normal.
    GradientBelowTolerance {
        /// Scale-safe Euclidean norm of the fitted gradient.
        gradient_norm: f64,
        /// Required minimum norm.
        minimum_gradient_norm: f64,
    },
    /// At least one requested endpoint intersection failed.
    Intersections {
        /// Lower-side failure, or `None` when the lower intersection succeeded.
        lower: Option<ThicknessIntersectionFailure>,
        /// Upper-side failure, or `None` when the upper intersection succeeded.
        upper: Option<ThicknessIntersectionFailure>,
    },
}

/// One selected location that produced no complete measurement.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct SampledThicknessFailure<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    sample_index: usize,
    point: Point<D>,
    reason: SampledThicknessFailureReason,
    provenance: Arc<SemanticProvenance>,
}

impl<const D: usize> SampledThicknessFailure<D>
where
    Dim<D>: SupportedDimension,
{
    /// Returns the stable zero-based input location index.
    #[must_use]
    pub const fn sample_index(&self) -> usize {
        self.sample_index
    }

    /// Returns the selected original-coordinate point.
    pub const fn point(&self) -> Point<D> {
        self.point
    }

    /// Returns the complete failure reason.
    #[must_use]
    pub const fn reason(&self) -> SampledThicknessFailureReason {
        self.reason
    }

    /// Borrows preserved caller provenance.
    pub fn provenance(&self) -> &SemanticProvenance {
        &self.provenance
    }
}

/// One complete lower-to-upper measurement along a selected local normal.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct SampledThicknessMeasurement<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    sample_index: usize,
    sample_point: Point<D>,
    normal: UnitDirection<D>,
    lower_intersection: Point<D>,
    upper_intersection: Point<D>,
    distance: f64,
    provenance: Arc<SemanticProvenance>,
}

impl<const D: usize> SampledThicknessMeasurement<D>
where
    Dim<D>: SupportedDimension,
{
    /// Returns the stable zero-based input location index.
    #[must_use]
    pub const fn sample_index(&self) -> usize {
        self.sample_index
    }

    /// Returns the selected point at which the normal was evaluated.
    pub const fn sample_point(&self) -> Point<D> {
        self.sample_point
    }

    /// Returns the oriented fitted-field normal toward increasing values.
    pub const fn normal(&self) -> UnitDirection<D> {
        self.normal
    }

    /// Returns the refined lower-level intersection.
    pub const fn lower_intersection(&self) -> Point<D> {
        self.lower_intersection
    }

    /// Returns the refined upper-level intersection.
    pub const fn upper_intersection(&self) -> Point<D> {
        self.upper_intersection
    }

    /// Returns the measured original-coordinate distance along the normal line.
    #[must_use]
    pub const fn distance(&self) -> f64 {
        self.distance
    }

    /// Borrows preserved caller provenance.
    pub fn provenance(&self) -> &SemanticProvenance {
        &self.provenance
    }
}

/// One deterministic requested quantile.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct SampledThicknessQuantile {
    probability: f64,
    distance: Option<f64>,
}

impl SampledThicknessQuantile {
    /// Returns the requested probability.
    #[must_use]
    pub const fn probability(self) -> f64 {
        self.probability
    }

    /// Returns the interpolated distance, or `None` when no sample succeeded.
    #[must_use]
    pub const fn distance(self) -> Option<f64> {
        self.distance
    }
}

/// One measured location below the requested minimum thickness.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct SampledThicknessViolation<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    sample_index: usize,
    point: Point<D>,
    measured_distance: f64,
    minimum_thickness: f64,
    provenance: Arc<SemanticProvenance>,
}

impl<const D: usize> SampledThicknessViolation<D>
where
    Dim<D>: SupportedDimension,
{
    /// Returns the stable zero-based input location index.
    #[must_use]
    pub const fn sample_index(&self) -> usize {
        self.sample_index
    }

    /// Returns the violation location.
    pub const fn point(&self) -> Point<D> {
        self.point
    }

    /// Returns the measured distance.
    #[must_use]
    pub const fn measured_distance(&self) -> f64 {
        self.measured_distance
    }

    /// Returns the requested minimum thickness.
    #[must_use]
    pub const fn minimum_thickness(&self) -> f64 {
        self.minimum_thickness
    }

    /// Borrows preserved caller provenance.
    pub fn provenance(&self) -> &SemanticProvenance {
        &self.provenance
    }
}

/// Complete immutable validation evidence.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct SampledThicknessReport<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    measurements: Vec<SampledThicknessMeasurement<D>>,
    failures: Vec<SampledThicknessFailure<D>>,
    minimum: Option<f64>,
    quantiles: Vec<SampledThicknessQuantile>,
    violations: Vec<SampledThicknessViolation<D>>,
    proposed_constraints: Vec<LocalNormalThickness<D>>,
}

impl<const D: usize> SampledThicknessReport<D>
where
    Dim<D>: SupportedDimension,
{
    /// Borrows successful measurements in input order.
    pub fn measurements(&self) -> &[SampledThicknessMeasurement<D>] {
        &self.measurements
    }

    /// Borrows failures in input order.
    pub fn failures(&self) -> &[SampledThicknessFailure<D>] {
        &self.failures
    }

    /// Returns the smallest successful distance, or `None` when none succeeded.
    #[must_use]
    pub const fn minimum(&self) -> Option<f64> {
        self.minimum
    }

    /// Borrows requested quantiles in caller order.
    pub fn quantiles(&self) -> &[SampledThicknessQuantile] {
        &self.quantiles
    }

    /// Borrows all measured violations in input order.
    pub fn violations(&self) -> &[SampledThicknessViolation<D>] {
        &self.violations
    }

    /// Borrows proposed local constraints for an explicit later refit.
    pub fn proposed_constraints(&self) -> &[LocalNormalThickness<D>] {
        &self.proposed_constraints
    }

    /// Labels this report as sampled geometric evidence.
    pub const fn diagnostics(&self) -> ThicknessDiagnostics {
        ThicknessDiagnostics::SAMPLED_GEOMETRIC
    }
}

impl<const D: usize> FittedField<D>
where
    Dim<D>: SupportedDimension,
{
    /// Validates sampled adjacent-level separation along fitted local normals.
    ///
    /// The lower level is searched along the negative fitted gradient and the
    /// upper level along the positive fitted gradient. Search uses uniform
    /// deterministic bracketing followed by bisection. The fitted field is not
    /// mutated and no constraint is inserted or solved.
    ///
    /// # Errors
    ///
    /// Returns structured allocation, fitted evaluation, coordinate
    /// representation, distance, or proposed-constraint construction errors.
    pub fn try_validate_sampled_thickness(
        &self,
        request: &SampledThicknessRequest<D>,
    ) -> Result<SampledThicknessReport<D>, SampledThicknessValidationError<D>> {
        self.try_validate_sampled_thickness_with_control(
            request,
            ExecutionOptions::default(),
            ExecutionControl::default(),
        )
    }

    /// Validates sampled separation with explicit execution metadata and caller controls.
    ///
    /// The current validation implementation is serial and rejects an explicit
    /// thread count greater than one before preparing evaluation storage or
    /// evaluating the fitted field. Progress events preserve the caller's
    /// determinism choice and report one effective worker.
    ///
    /// Cancellation is checked before work, after reusable evaluation storage is
    /// prepared, and after every fitted-field evaluation. A cancellation returns
    /// no partial report. The borrowed control is never retained by the model.
    ///
    /// # Errors
    ///
    /// Returns the same failures as [`Self::try_validate_sampled_thickness`], plus
    /// a typed execution error when cancellation is observed.
    pub fn try_validate_sampled_thickness_with_control(
        &self,
        request: &SampledThicknessRequest<D>,
        execution: ExecutionOptions,
        control: ExecutionControl<'_>,
    ) -> Result<SampledThicknessReport<D>, SampledThicknessValidationError<D>> {
        let mut progress = validation_progress(request, execution, control)?;
        let scratch_result = self
            .try_evaluation_scratch(FittedFieldOutput::Gradient)
            .map_err(|source| SampledThicknessValidationError::Preparation {
                source: Box::new(source),
            });
        let mut scratch = progress.observe_result(ExecutionStage::Started, scratch_result)?;
        let report =
            try_validate_with_progress(request, &mut progress, |point, demand| match demand {
                EvaluationDemand::Value => {
                    self.try_value_with_scratch(point, &mut scratch)
                        .map(|value| FieldEvaluation {
                            value,
                            gradient: [0.0; D],
                        })
                }
                EvaluationDemand::ValueGradient => self
                    .try_evaluate_with_scratch(point, &mut scratch)
                    .map(|evaluation| FieldEvaluation {
                        value: evaluation.value(),
                        gradient: evaluation.gradient().into_components(),
                    }),
            })?;
        progress.complete()?;
        Ok(report)
    }
}

#[derive(Clone, Copy)]
struct FieldEvaluation<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    value: f64,
    gradient: [f64; D],
}

#[derive(Clone, Copy)]
enum EvaluationDemand {
    Value,
    ValueGradient,
}

#[derive(Clone, Copy)]
enum SearchOutcome<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    Found { point: Point<D>, distance: f64 },
    Failed(ThicknessIntersectionFailure),
}

#[allow(clippy::too_many_lines)]
#[cfg(test)]
fn try_validate_with<const D: usize, E>(
    request: &SampledThicknessRequest<D>,
    evaluate: E,
) -> Result<SampledThicknessReport<D>, SampledThicknessValidationError<D>>
where
    Dim<D>: SupportedDimension,
    E: FnMut(
        Point<D>,
        EvaluationDemand,
    ) -> Result<FieldEvaluation<D>, FittedFieldEvaluationError<D>>,
{
    let mut progress = validation_progress(
        request,
        ExecutionOptions::default(),
        ExecutionControl::default(),
    )?;
    let report = try_validate_with_progress(request, &mut progress, evaluate)?;
    progress.complete()?;
    Ok(report)
}

fn validation_progress<'a, const D: usize>(
    request: &SampledThicknessRequest<D>,
    execution: ExecutionOptions,
    control: ExecutionControl<'a>,
) -> Result<ProgressTracker<'a>, SampledThicknessValidationError<D>>
where
    Dim<D>: SupportedDimension,
{
    let search_steps = usize::try_from(request.settings.search_steps.get()).map_err(|_| {
        SampledThicknessValidationError::WorkBudgetOverflow {
            location_count: request.locations.len(),
            search_steps: request.settings.search_steps.get(),
            refinement_iterations: request.settings.refinement_iterations.get(),
        }
    })?;
    let refinement_iterations = usize::try_from(request.settings.refinement_iterations.get())
        .map_err(|_| SampledThicknessValidationError::WorkBudgetOverflow {
            location_count: request.locations.len(),
            search_steps: request.settings.search_steps.get(),
            refinement_iterations: request.settings.refinement_iterations.get(),
        })?;
    let per_location = search_steps
        .checked_add(refinement_iterations)
        .and_then(|per_side| per_side.checked_mul(2))
        .and_then(|both_sides| both_sides.checked_add(1))
        .ok_or(SampledThicknessValidationError::WorkBudgetOverflow {
            location_count: request.locations.len(),
            search_steps: request.settings.search_steps.get(),
            refinement_iterations: request.settings.refinement_iterations.get(),
        })?;
    let total = request.locations.len().checked_mul(per_location).ok_or(
        SampledThicknessValidationError::WorkBudgetOverflow {
            location_count: request.locations.len(),
            search_steps: request.settings.search_steps.get(),
            refinement_iterations: request.settings.refinement_iterations.get(),
        },
    )?;
    Ok(ProgressTracker::try_new(
        control,
        ExecutionOperation::SampledThicknessValidation,
        execution,
        total,
    )?)
}

#[allow(clippy::too_many_lines)]
fn try_validate_with_progress<const D: usize, E>(
    request: &SampledThicknessRequest<D>,
    progress: &mut ProgressTracker<'_>,
    mut evaluate: E,
) -> Result<SampledThicknessReport<D>, SampledThicknessValidationError<D>>
where
    Dim<D>: SupportedDimension,
    E: FnMut(
        Point<D>,
        EvaluationDemand,
    ) -> Result<FieldEvaluation<D>, FittedFieldEvaluationError<D>>,
{
    let count = request.locations.len();
    let mut measurements = try_vec(count, SampledThicknessStorage::Measurements)?;
    let mut failures = try_vec(count, SampledThicknessStorage::Failures)?;
    let mut violations = try_vec(count, SampledThicknessStorage::Violations)?;
    let proposed_count = if request.propose_constraints {
        count
    } else {
        0
    };
    let mut proposed_constraints =
        try_vec(proposed_count, SampledThicknessStorage::ProposedConstraints)?;

    for (sample_index, location) in request.locations.iter().enumerate() {
        let evaluated = evaluate_with_progress(
            sample_index,
            location.point,
            EvaluationDemand::ValueGradient,
            &mut evaluate,
            progress,
        )?;
        let gradient_norm = stable_norm(evaluated.gradient);
        if gradient_norm < request.settings.minimum_gradient_norm {
            failures.push(SampledThicknessFailure {
                sample_index,
                point: location.point,
                reason: SampledThicknessFailureReason::GradientBelowTolerance {
                    gradient_norm,
                    minimum_gradient_norm: request.settings.minimum_gradient_norm,
                },
                provenance: Arc::clone(&location.provenance),
            });
            continue;
        }
        let normal = UnitDirection::try_new(evaluated.gradient).map_err(|source| {
            SampledThicknessValidationError::Geometry {
                sample_index,
                source,
            }
        })?;
        let lower = search_intersection(
            request,
            sample_index,
            location.point,
            evaluated.value,
            normal,
            -1.0,
            request.lower_value,
            &mut evaluate,
            progress,
        )?;
        let upper = search_intersection(
            request,
            sample_index,
            location.point,
            evaluated.value,
            normal,
            1.0,
            request.upper_value,
            &mut evaluate,
            progress,
        )?;
        match (lower, upper) {
            (
                SearchOutcome::Found {
                    point: lower_intersection,
                    distance: lower_distance,
                },
                SearchOutcome::Found {
                    point: upper_intersection,
                    distance: upper_distance,
                },
            ) => {
                let distance = intersection_distance(lower_intersection, upper_intersection);
                if !distance.is_finite() {
                    return Err(SampledThicknessValidationError::DistanceNotRepresentable {
                        sample_index,
                        lower_distance,
                        upper_distance,
                    });
                }
                measurements.push(SampledThicknessMeasurement {
                    sample_index,
                    sample_point: location.point,
                    normal,
                    lower_intersection,
                    upper_intersection,
                    distance,
                    provenance: Arc::clone(&location.provenance),
                });
                if distance < request.minimum_thickness {
                    violations.push(SampledThicknessViolation {
                        sample_index,
                        point: location.point,
                        measured_distance: distance,
                        minimum_thickness: request.minimum_thickness,
                        provenance: Arc::clone(&location.provenance),
                    });
                    if request.propose_constraints {
                        proposed_constraints.push(
                            LocalNormalThickness::try_new(
                                request.lower,
                                request.upper,
                                location.point,
                                request.minimum_thickness,
                                location.provenance.try_clone_for_canonical().map_err(
                                    |source| SampledThicknessValidationError::Provenance {
                                        sample_index,
                                        source,
                                    },
                                )?,
                            )
                            .map_err(SampledThicknessValidationError::ProposedConstraint)?,
                        );
                    }
                }
            }
            (lower, upper) => failures.push(SampledThicknessFailure {
                sample_index,
                point: location.point,
                reason: SampledThicknessFailureReason::Intersections {
                    lower: search_failure(lower),
                    upper: search_failure(upper),
                },
                provenance: Arc::clone(&location.provenance),
            }),
        }
    }

    let mut distances = try_vec(measurements.len(), SampledThicknessStorage::Distances)?;
    distances.extend(
        measurements
            .iter()
            .map(SampledThicknessMeasurement::distance),
    );
    distances.sort_by(f64::total_cmp);
    let minimum = distances.first().copied();
    let mut quantiles = try_vec(request.quantiles.len(), SampledThicknessStorage::Quantiles)?;
    for probability in request.quantiles.iter().copied() {
        quantiles.push(SampledThicknessQuantile {
            probability,
            distance: type_seven_quantile(&distances, probability),
        });
    }

    Ok(SampledThicknessReport {
        measurements,
        failures,
        minimum,
        quantiles,
        violations,
        proposed_constraints,
    })
}

#[allow(clippy::too_many_arguments)]
fn search_intersection<const D: usize, E>(
    request: &SampledThicknessRequest<D>,
    sample_index: usize,
    origin: Point<D>,
    origin_value: f64,
    normal: UnitDirection<D>,
    sign: f64,
    target: f64,
    evaluate: &mut E,
    progress: &mut ProgressTracker<'_>,
) -> Result<SearchOutcome<D>, SampledThicknessValidationError<D>>
where
    Dim<D>: SupportedDimension,
    E: FnMut(
        Point<D>,
        EvaluationDemand,
    ) -> Result<FieldEvaluation<D>, FittedFieldEvaluationError<D>>,
{
    let settings = request.settings;
    let mut previous_distance = 0.0;
    let mut previous_residual = checked_residual(origin_value, target, sample_index)?;
    if previous_residual.abs() <= settings.value_tolerance {
        return Ok(SearchOutcome::Found {
            point: origin,
            distance: 0.0,
        });
    }

    for step in 1..=settings.search_steps.get() {
        let ratio = f64::from(step) / f64::from(settings.search_steps.get());
        let distance = settings.maximum_search_distance * ratio;
        let point = point_along(origin, normal, sign, distance, sample_index)?;
        let evaluated = evaluate_with_progress(
            sample_index,
            point,
            EvaluationDemand::Value,
            evaluate,
            progress,
        )?;
        let residual = checked_residual(evaluated.value, target, sample_index)?;
        if residual.abs() <= settings.value_tolerance {
            return Ok(SearchOutcome::Found { point, distance });
        }
        if residual.is_sign_positive() != previous_residual.is_sign_positive() {
            return refine_intersection(
                settings,
                sample_index,
                origin,
                normal,
                sign,
                target,
                previous_distance,
                previous_residual,
                distance,
                residual,
                evaluate,
                progress,
            );
        }
        previous_distance = distance;
        previous_residual = residual;
    }
    Ok(SearchOutcome::Failed(
        ThicknessIntersectionFailure::NotFound,
    ))
}

#[allow(clippy::too_many_arguments)]
fn refine_intersection<const D: usize, E>(
    settings: SampledThicknessSettings,
    sample_index: usize,
    origin: Point<D>,
    normal: UnitDirection<D>,
    sign: f64,
    target: f64,
    mut left: f64,
    mut left_residual: f64,
    mut right: f64,
    mut right_residual: f64,
    evaluate: &mut E,
    progress: &mut ProgressTracker<'_>,
) -> Result<SearchOutcome<D>, SampledThicknessValidationError<D>>
where
    Dim<D>: SupportedDimension,
    E: FnMut(
        Point<D>,
        EvaluationDemand,
    ) -> Result<FieldEvaluation<D>, FittedFieldEvaluationError<D>>,
{
    let mut last_residual = left_residual.abs().min(right_residual.abs());
    for _ in 0..settings.refinement_iterations.get() {
        let middle = left + (right - left) * 0.5;
        let point = point_along(origin, normal, sign, middle, sample_index)?;
        let evaluated = evaluate_with_progress(
            sample_index,
            point,
            EvaluationDemand::Value,
            evaluate,
            progress,
        )?;
        let residual = checked_residual(evaluated.value, target, sample_index)?;
        last_residual = residual.abs();
        if last_residual <= settings.value_tolerance || right - left <= settings.distance_tolerance
        {
            return Ok(SearchOutcome::Found {
                point,
                distance: middle,
            });
        }
        if residual.is_sign_positive() == left_residual.is_sign_positive() {
            left = middle;
            left_residual = residual;
        } else {
            right = middle;
            right_residual = residual;
        }
    }
    let _ = right_residual;
    Ok(SearchOutcome::Failed(
        ThicknessIntersectionFailure::RefinementLimitReached {
            distance_interval: right - left,
            value_residual: last_residual,
        },
    ))
}

fn evaluate_with_progress<const D: usize, E>(
    sample_index: usize,
    point: Point<D>,
    demand: EvaluationDemand,
    evaluate: &mut E,
    progress: &mut ProgressTracker<'_>,
) -> Result<FieldEvaluation<D>, SampledThicknessValidationError<D>>
where
    Dim<D>: SupportedDimension,
    E: FnMut(
        Point<D>,
        EvaluationDemand,
    ) -> Result<FieldEvaluation<D>, FittedFieldEvaluationError<D>>,
{
    progress.observe_result(
        ExecutionStage::SampledThicknessEvaluation,
        Ok::<(), SampledThicknessValidationError<D>>(()),
    )?;
    let result =
        evaluate(point, demand).map_err(|source| SampledThicknessValidationError::Evaluation {
            sample_index,
            source: Box::new(source),
        });
    progress.finish_work(ExecutionStage::SampledThicknessEvaluation, result)
}

fn intersection_distance<const D: usize>(lower: Point<D>, upper: Point<D>) -> f64
where
    Dim<D>: SupportedDimension,
{
    stable_norm::<D>(std::array::from_fn(|axis| {
        upper.components()[axis] - lower.components()[axis]
    }))
}

fn checked_residual<const D: usize>(
    value: f64,
    target: f64,
    sample_index: usize,
) -> Result<f64, SampledThicknessValidationError<D>>
where
    Dim<D>: SupportedDimension,
{
    let residual = value - target;
    if !residual.is_finite() {
        return Err(SampledThicknessValidationError::ResidualNotRepresentable {
            sample_index,
            value,
            target,
        });
    }
    Ok(residual)
}

fn point_along<const D: usize>(
    origin: Point<D>,
    normal: UnitDirection<D>,
    sign: f64,
    distance: f64,
    sample_index: usize,
) -> Result<Point<D>, SampledThicknessValidationError<D>>
where
    Dim<D>: SupportedDimension,
{
    let components = std::array::from_fn(|axis| {
        origin.components()[axis] + sign * distance * normal.components()[axis]
    });
    Point::try_new(components).map_err(|source| {
        SampledThicknessValidationError::SearchPointNotRepresentable {
            sample_index,
            distance,
            source,
        }
    })
}

const fn search_failure<const D: usize>(
    outcome: SearchOutcome<D>,
) -> Option<ThicknessIntersectionFailure>
where
    Dim<D>: SupportedDimension,
{
    match outcome {
        SearchOutcome::Found { .. } => None,
        SearchOutcome::Failed(failure) => Some(failure),
    }
}

fn stable_norm<const D: usize>(components: [f64; D]) -> f64 {
    let scale = components
        .iter()
        .map(|component| component.abs())
        .fold(0.0_f64, f64::max);
    if scale == 0.0 {
        return 0.0;
    }
    scale
        * components
            .iter()
            .map(|component| (component / scale).powi(2))
            .sum::<f64>()
            .sqrt()
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
fn type_seven_quantile(sorted: &[f64], probability: f64) -> Option<f64> {
    let last = sorted.len().checked_sub(1)?;
    let rank = probability * last as f64;
    let lower = rank.floor() as usize;
    let upper = rank.ceil() as usize;
    let fraction = rank - lower as f64;
    Some(sorted[lower] + fraction * (sorted[upper] - sorted[lower]))
}

fn try_vec<T, const D: usize>(
    capacity: usize,
    storage: SampledThicknessStorage,
) -> Result<Vec<T>, SampledThicknessValidationError<D>>
where
    Dim<D>: SupportedDimension,
{
    let mut values = Vec::new();
    values.try_reserve_exact(capacity).map_err(|_| {
        SampledThicknessValidationError::AllocationFailed {
            storage,
            requested: capacity,
        }
    })?;
    Ok(values)
}

fn validate_positive_setting(
    field: &'static str,
    value: f64,
) -> Result<(), SampledThicknessSettingsError> {
    if !value.is_finite() || value <= 0.0 {
        return Err(SampledThicknessSettingsError { field, value });
    }
    Ok(())
}

/// Invalid scalar in sampled-thickness search settings.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct SampledThicknessSettingsError {
    field: &'static str,
    value: f64,
}

impl SampledThicknessSettingsError {
    /// Returns the stable invalid field name.
    #[must_use]
    pub const fn field(self) -> &'static str {
        self.field
    }

    /// Returns the supplied invalid value.
    #[must_use]
    pub const fn value(self) -> f64 {
        self.value
    }
}

impl fmt::Display for SampledThicknessSettingsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "sampled thickness setting {} must be positive and finite, got {}",
            self.field, self.value
        )
    }
}

impl Error for SampledThicknessSettingsError {}

/// Checked allocation role during validation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SampledThicknessStorage {
    /// Successful measurements.
    Measurements,
    /// Per-location failures.
    Failures,
    /// Measured violations.
    Violations,
    /// Proposed local constraints.
    ProposedConstraints,
    /// Sorted distance scratch.
    Distances,
    /// Quantile outputs.
    Quantiles,
}

/// Structured sampled-thickness validation failure.
#[derive(Debug)]
#[must_use]
pub enum SampledThicknessValidationError<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    /// Endpoint levels are identical.
    EqualLevels {
        /// Repeated level identifier.
        level_id: LevelId,
    },
    /// Endpoint scalar values are non-finite or not strictly increasing.
    InvalidLevelValues {
        /// Supplied lower value.
        lower_value: f64,
        /// Supplied upper value.
        upper_value: f64,
    },
    /// The requested geometric minimum is not positive and finite.
    InvalidMinimumThickness {
        /// Supplied minimum.
        minimum_thickness: f64,
    },
    /// No locations were selected.
    EmptyLocations,
    /// No quantile probabilities were requested.
    EmptyQuantiles,
    /// One probability is non-finite or outside `[0, 1]`.
    InvalidQuantile {
        /// Zero-based probability index.
        index: usize,
        /// Supplied probability.
        probability: f64,
    },
    /// The checked maximum fitted-evaluation budget was not representable.
    WorkBudgetOverflow {
        /// Number of selected locations.
        location_count: usize,
        /// Maximum uniform search steps per side.
        search_steps: u32,
        /// Maximum bisection iterations per side.
        refinement_iterations: u32,
    },
    /// Caller execution control stopped validation.
    Execution(ExecutionError),
    /// Checked temporary allocation failed.
    AllocationFailed {
        /// Storage role.
        storage: SampledThicknessStorage,
        /// Requested capacity.
        requested: usize,
    },
    /// Reusable fitted-field evaluation storage could not be prepared.
    Preparation {
        /// Concrete fitted-field failure.
        source: Box<FittedFieldEvaluationError<D>>,
    },
    /// Fitted-field evaluation failed at one location or search point.
    Evaluation {
        /// Zero-based input location index.
        sample_index: usize,
        /// Concrete fitted-field failure.
        source: Box<FittedFieldEvaluationError<D>>,
    },
    /// Subtracting a finite target from a finite fitted value was non-finite.
    ResidualNotRepresentable {
        /// Zero-based input location index.
        sample_index: usize,
        /// Evaluated fitted value.
        value: f64,
        /// Requested target value.
        target: f64,
    },
    /// Fallible cloning of caller provenance failed for a proposed constraint.
    Provenance {
        /// Zero-based input location index.
        sample_index: usize,
        /// Concrete problem-IR allocation failure.
        source: ProblemIrError,
    },
    /// A fitted gradient could not form a unit normal.
    Geometry {
        /// Zero-based input location index.
        sample_index: usize,
        /// Concrete geometry failure.
        source: GeometryError,
    },
    /// One searched coordinate was not representable.
    SearchPointNotRepresentable {
        /// Zero-based input location index.
        sample_index: usize,
        /// Distance from the selected location.
        distance: f64,
        /// Concrete geometry failure.
        source: GeometryError,
    },
    /// The Euclidean separation of the returned intersections was not representable.
    DistanceNotRepresentable {
        /// Zero-based input location index.
        sample_index: usize,
        /// Nominal lower-side line parameter retained for diagnostic context.
        lower_distance: f64,
        /// Nominal upper-side line parameter retained for diagnostic context.
        upper_distance: f64,
    },
    /// Proposed local-constraint construction failed.
    ProposedConstraint(LocalNormalThicknessError),
}

impl<const D: usize> fmt::Display for SampledThicknessValidationError<D>
where
    Dim<D>: SupportedDimension,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EqualLevels { level_id } => {
                write!(formatter, "sampled thickness levels are both {level_id:?}")
            }
            Self::InvalidLevelValues {
                lower_value,
                upper_value,
            } => write!(
                formatter,
                "sampled thickness values must be finite and increasing, got {lower_value} and {upper_value}"
            ),
            Self::InvalidMinimumThickness { minimum_thickness } => write!(
                formatter,
                "sampled minimum thickness must be positive and finite, got {minimum_thickness}"
            ),
            Self::EmptyLocations => formatter.write_str("sampled thickness requires locations"),
            Self::EmptyQuantiles => formatter.write_str("sampled thickness requires quantiles"),
            Self::InvalidQuantile { index, probability } => write!(
                formatter,
                "sampled thickness quantile {index} must lie in [0, 1], got {probability}"
            ),
            Self::WorkBudgetOverflow {
                location_count,
                search_steps,
                refinement_iterations,
            } => write!(
                formatter,
                "sampled thickness work budget is not representable for {location_count} locations, {search_steps} search steps, and {refinement_iterations} refinement iterations"
            ),
            Self::Execution(source) => source.fmt(formatter),
            Self::AllocationFailed { storage, requested } => write!(
                formatter,
                "could not reserve {requested} sampled thickness entries for {storage:?}"
            ),
            Self::Preparation { source } => write!(
                formatter,
                "could not prepare sampled thickness fitted-field evaluation: {source}"
            ),
            Self::Evaluation {
                sample_index,
                source,
            } => write!(
                formatter,
                "fitted-field evaluation failed for sampled thickness location {sample_index}: {source}"
            ),
            Self::ResidualNotRepresentable {
                sample_index,
                value,
                target,
            } => write!(
                formatter,
                "sampled thickness residual at location {sample_index} is not representable from value {value} and target {target}"
            ),
            Self::Provenance {
                sample_index,
                source,
            } => write!(
                formatter,
                "could not preserve sampled thickness provenance at location {sample_index}: {source}"
            ),
            Self::Geometry {
                sample_index,
                source,
            } => write!(
                formatter,
                "sampled thickness normal failed at location {sample_index}: {source}"
            ),
            Self::SearchPointNotRepresentable {
                sample_index,
                distance,
                source,
            } => write!(
                formatter,
                "sampled thickness search point at distance {distance} for location {sample_index} is invalid: {source}"
            ),
            Self::DistanceNotRepresentable {
                sample_index,
                lower_distance,
                upper_distance,
            } => write!(
                formatter,
                "sampled thickness Euclidean intersection distance at location {sample_index} is not representable for nominal line parameters {lower_distance} and {upper_distance}"
            ),
            Self::ProposedConstraint(source) => source.fmt(formatter),
        }
    }
}

impl<const D: usize> Error for SampledThicknessValidationError<D>
where
    Dim<D>: SupportedDimension,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Preparation { source } | Self::Evaluation { source, .. } => Some(source.as_ref()),
            Self::Geometry { source, .. } | Self::SearchPointNotRepresentable { source, .. } => {
                Some(source)
            }
            Self::Provenance { source, .. } => Some(source),
            Self::Execution(source) => Some(source),
            Self::ProposedConstraint(source) => Some(source),
            Self::EqualLevels { .. }
            | Self::InvalidLevelValues { .. }
            | Self::InvalidMinimumThickness { .. }
            | Self::EmptyLocations
            | Self::EmptyQuantiles
            | Self::InvalidQuantile { .. }
            | Self::WorkBudgetOverflow { .. }
            | Self::AllocationFailed { .. }
            | Self::ResidualNotRepresentable { .. }
            | Self::DistanceNotRepresentable { .. } => None,
        }
    }
}

impl<const D: usize> From<ExecutionError> for SampledThicknessValidationError<D>
where
    Dim<D>: SupportedDimension,
{
    fn from(source: ExecutionError) -> Self {
        Self::Execution(source)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::num::{NonZeroU32, NonZeroUsize};

    use crate::{
        FittedFieldEvaluationError, LevelId, ObservationId, Point, SemanticProvenance,
        SourceLocation,
    };

    use super::{
        FieldEvaluation, SampledThicknessLocation, SampledThicknessRequest,
        SampledThicknessSettings, stable_norm, try_validate_with, type_seven_quantile,
    };

    fn provenance(identifier: u64) -> Result<SemanticProvenance, Box<dyn Error>> {
        Ok(SemanticProvenance::try_new(
            ObservationId::new(identifier),
            SourceLocation::try_new(
                "analytic-curved-levels.csv".to_owned(),
                NonZeroUsize::new(usize::try_from(identifier)?)
                    .ok_or("identifier must be positive")?,
            )?,
            "m".to_owned(),
            format!("thickness.samples[{identifier}]"),
            Some("independent analytic field".to_owned()),
        )?)
    }

    fn settings() -> Result<SampledThicknessSettings, Box<dyn Error>> {
        Ok(SampledThicknessSettings::try_new(
            2.0,
            NonZeroU32::new(64).ok_or("search steps")?,
            NonZeroU32::new(64).ok_or("refinement iterations")?,
            1.0e-12,
            1.0e-12,
            1.0e-12,
        )?)
    }

    #[test]
    fn stable_norm_handles_extreme_scales() {
        assert!(stable_norm([0.0, -0.0]).abs() <= f64::EPSILON);
        assert!(stable_norm([f64::MAX / 2.0, f64::MAX / 2.0]).is_finite());
        assert!(stable_norm([f64::MIN_POSITIVE, f64::MIN_POSITIVE]) > 0.0);
    }

    #[test]
    fn type_seven_quantile_interpolates_in_requested_order() {
        let values = [1.0, 2.0, 4.0, 8.0];
        assert_eq!(type_seven_quantile(&values, 0.0), Some(1.0));
        assert_eq!(type_seven_quantile(&values, 0.5), Some(3.0));
        assert_eq!(type_seven_quantile(&values, 1.0), Some(8.0));
        assert_eq!(type_seven_quantile(&[], 0.5), None);
    }

    #[test]
    fn nonrepresentable_residual_is_a_structured_error() -> Result<(), Box<dyn Error>> {
        let request = SampledThicknessRequest::try_new(
            LevelId::new(1),
            -f64::MAX,
            LevelId::new(2),
            0.0,
            1.0,
            vec![SampledThicknessLocation::new(
                Point::try_new([0.0])?,
                provenance(9)?,
            )],
            vec![0.5],
            false,
            settings()?,
        )?;
        let result = try_validate_with(
            &request,
            |_, _| -> Result<FieldEvaluation<1>, FittedFieldEvaluationError<1>> {
                Ok(FieldEvaluation {
                    value: f64::MAX,
                    gradient: [1.0],
                })
            },
        );
        assert!(matches!(
            result,
            Err(
                super::SampledThicknessValidationError::ResidualNotRepresentable {
                    sample_index: 0,
                    ..
                }
            )
        ));
        Ok(())
    }

    #[test]
    fn rounded_search_points_use_returned_intersection_distance_without_false_proposals()
    -> Result<(), Box<dyn Error>> {
        let origin = 1.0e16;
        let request = SampledThicknessRequest::try_new(
            LevelId::new(1),
            origin - 2.0,
            LevelId::new(2),
            origin + 2.0,
            3.5,
            vec![SampledThicknessLocation::new(
                Point::try_new([origin])?,
                provenance(11)?,
            )],
            vec![0.5],
            true,
            SampledThicknessSettings::try_new(
                2.0,
                NonZeroU32::new(4).ok_or("search steps")?,
                NonZeroU32::new(4).ok_or("refinement iterations")?,
                1.0e-12,
                1.0e-12,
                1.0e-12,
            )?,
        )?;
        let report = try_validate_with(
            &request,
            |point, _| -> Result<FieldEvaluation<1>, FittedFieldEvaluationError<1>> {
                Ok(FieldEvaluation {
                    value: point.components()[0],
                    gradient: [1.0],
                })
            },
        )?;
        let measurement = &report.measurements()[0];
        let euclidean_distance = (measurement.upper_intersection().components()[0]
            - measurement.lower_intersection().components()[0])
            .abs();

        assert_eq!(euclidean_distance.to_bits(), 4.0_f64.to_bits());
        assert_eq!(
            measurement.distance().to_bits(),
            euclidean_distance.to_bits()
        );
        assert!(report.violations().is_empty());
        assert!(report.proposed_constraints().is_empty());
        Ok(())
    }

    #[test]
    fn curved_level_truth_matches_independent_line_roots_and_quantiles()
    -> Result<(), Box<dyn Error>> {
        let xs = [0.0_f64, 0.25, 1.0];
        let locations = xs
            .into_iter()
            .enumerate()
            .map(|(index, x)| {
                Ok(SampledThicknessLocation::new(
                    Point::try_new([x, -x * x])?,
                    provenance(u64::try_from(index + 1)?)?,
                ))
            })
            .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
        let request = SampledThicknessRequest::try_new(
            LevelId::new(1),
            -1.0,
            LevelId::new(2),
            1.0,
            1.95,
            locations,
            vec![0.0, 0.5, 1.0],
            true,
            settings()?,
        )?;
        let report = try_validate_with(
            &request,
            |point, _| -> Result<FieldEvaluation<2>, FittedFieldEvaluationError<2>> {
                let [x, y] = *point.components();
                Ok(FieldEvaluation {
                    value: x * x + y,
                    gradient: [2.0 * x, 1.0],
                })
            },
        )?;

        assert!(report.failures().is_empty());
        assert_eq!(report.measurements().len(), 3);
        let mut expected = xs.map(|x| {
            if x == 0.0 {
                return 2.0;
            }
            let a = 4.0 * x * x / (4.0 * x * x + 1.0);
            let b = (4.0 * x * x + 1.0).sqrt();
            let lower = (b - (b * b - 4.0 * a).sqrt()) / (2.0 * a);
            let upper = (-b + (b * b + 4.0 * a).sqrt()) / (2.0 * a);
            lower + upper
        });
        expected.sort_by(f64::total_cmp);
        for measurement in report.measurements() {
            let actual = measurement.distance();
            let expected_value = match measurement.sample_index() {
                0 => 2.0,
                1 => expected[1],
                2 => expected[0],
                _ => return Err("unexpected sample index".into()),
            };
            assert!((actual - expected_value).abs() <= 2.0e-10);
        }
        assert_eq!(report.violations().len(), 2);
        assert_eq!(report.proposed_constraints().len(), 2);
        assert!(
            (report.quantiles()[0].distance().ok_or("minimum quantile")? - expected[0]).abs()
                <= 2.0e-10
        );
        assert!((report.quantiles()[1].distance().ok_or("median")? - expected[1]).abs() <= 2.0e-10);
        assert!(
            (report.quantiles()[2].distance().ok_or("maximum quantile")? - expected[2]).abs()
                <= 2.0e-10
        );
        Ok(())
    }

    #[test]
    fn off_grid_tangential_contact_is_reported_as_not_found() -> Result<(), Box<dyn Error>> {
        let request = SampledThicknessRequest::try_new(
            LevelId::new(1),
            -1.0,
            LevelId::new(2),
            1.0,
            1.0,
            vec![SampledThicknessLocation::new(
                Point::try_new([0.5, -0.25])?,
                provenance(10)?,
            )],
            vec![0.5],
            false,
            SampledThicknessSettings::try_new(
                2.0,
                NonZeroU32::new(4).ok_or("search steps")?,
                NonZeroU32::new(64).ok_or("refinement iterations")?,
                1.0e-12,
                1.0e-12,
                1.0e-12,
            )?,
        )?;
        let report = try_validate_with(
            &request,
            |point, _| -> Result<FieldEvaluation<2>, FittedFieldEvaluationError<2>> {
                let [x, y] = *point.components();
                Ok(FieldEvaluation {
                    value: x * x + y,
                    gradient: [2.0 * x, 1.0],
                })
            },
        )?;

        assert!(report.measurements().is_empty());
        assert!(matches!(
            report.failures()[0].reason(),
            super::SampledThicknessFailureReason::Intersections {
                lower: Some(super::ThicknessIntersectionFailure::NotFound),
                upper: None,
            }
        ));
        Ok(())
    }
}
