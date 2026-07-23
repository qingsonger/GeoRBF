//! Compilation of regional and reference-field controls into SPD local trends.
//!
//! This layer resolves geological control inputs once, constructs only fixed
//! [`crate::GlobalAnisotropy`] metrics, and lowers them into the positive-
//! definite [`crate::LocalTrendMixture`] primitive. It never installs an
//! arbitrary location-dependent point-pair metric and never mutates or refits
//! a referenced scalar field.
//!
//! ```compile_fail
//! use georbf::CompiledTrendControls;
//!
//! fn unsupported(_: Option<CompiledTrendControls<4>>) {}
//! ```

use std::error::Error;
use std::fmt;

use crate::anisotropy::{AnisotropyConditionPolicy, AnisotropyError, GlobalAnisotropy};
use crate::dimension::{Dim, SupportedDimension};
use crate::geometry::{Point, UnitDirection};
use crate::local_trend::{
    LocalTrendComponent, LocalTrendConstructionError, LocalTrendMixture, OperationalDomain,
    SmoothRegion, SmoothSpatialWeight,
};
use crate::model::{FittedFieldEvaluationError, KernelDefinition};
use crate::project::{FieldId, GeoProject};

/// One caller-supplied direction or one direction sampled from a project field.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TrendDirectionSource<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    /// An already validated explicit unit direction.
    Explicit(UnitDirection<D>),
    /// The normalized Cartesian gradient of this immutable project field.
    ReferenceFieldGradient(FieldId),
}

/// Spheroidal or fully ellipsoidal fixed orientation requested by a control.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TrendControlOrientation<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    /// One principal axis and axial/transverse correlation lengths.
    Spheroidal {
        /// Principal direction or reference-gradient source.
        principal_axis: TrendDirectionSource<D>,
        /// Positive correlation length along the principal direction.
        axial_length: f64,
        /// Positive correlation length in the orthogonal complement.
        transverse_length: f64,
    },
    /// Ordered orthogonal axes and their correlation lengths.
    Ellipsoidal {
        /// Caller-ordered axis sources. The compiler does not orthogonalize them.
        principal_axes: [TrendDirectionSource<D>; D],
        /// Positive length paired with each ordered axis.
        axis_lengths: [f64; D],
        /// Explicit absolute dot-product tolerance in `[0, 1)`.
        orthogonality_tolerance: f64,
    },
}

/// One local geological control before fixed-metric compilation.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct LocalTrendControl<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    location: Point<D>,
    kernel: KernelDefinition<D>,
    orientation: TrendControlOrientation<D>,
    influence_radius: f64,
    strength: f64,
    region: Option<SmoothRegion<D>>,
}

impl<const D: usize> LocalTrendControl<D>
where
    Dim<D>: SupportedDimension,
{
    /// Retains one ordered control. Validation occurs during compilation so
    /// errors can identify the control index.
    pub const fn new(
        location: Point<D>,
        kernel: KernelDefinition<D>,
        orientation: TrendControlOrientation<D>,
        influence_radius: f64,
        strength: f64,
        region: Option<SmoothRegion<D>>,
    ) -> Self {
        Self {
            location,
            kernel,
            orientation,
            influence_radius,
            strength,
            region,
        }
    }

    /// Returns the control location in the active original-coordinate convention.
    pub const fn location(self) -> Point<D> {
        self.location
    }

    /// Returns the fixed SPD-kernel candidate.
    pub const fn kernel(self) -> KernelDefinition<D> {
        self.kernel
    }

    /// Returns the requested orientation.
    #[must_use]
    pub const fn orientation(self) -> TrendControlOrientation<D> {
        self.orientation
    }

    /// Returns the Gaussian influence radius.
    #[must_use]
    pub const fn influence_radius(self) -> f64 {
        self.influence_radius
    }

    /// Returns the signed nonzero component strength.
    #[must_use]
    pub const fn strength(self) -> f64 {
        self.strength
    }

    /// Returns the optional compact C2 region.
    #[must_use]
    pub const fn region(self) -> Option<SmoothRegion<D>> {
        self.region
    }
}

/// Explicit fixed background retained as component zero of compiled mixtures.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct LocalTrendBackground<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    kernel: KernelDefinition<D>,
    anisotropy: GlobalAnisotropy<D>,
    weight: SmoothSpatialWeight<D>,
}

impl<const D: usize> LocalTrendBackground<D>
where
    Dim<D>: SupportedDimension,
{
    /// Retains the fixed background inputs. The final mixture constructor
    /// enforces the constant, everywhere-nonzero, strictly-PD policy.
    pub const fn new(
        kernel: KernelDefinition<D>,
        anisotropy: GlobalAnisotropy<D>,
        weight: SmoothSpatialWeight<D>,
    ) -> Self {
        Self {
            kernel,
            anisotropy,
            weight,
        }
    }
}

/// Explicit numerical and diagnostic policy for control compilation.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct TrendControlPolicy {
    anisotropy_condition: AnisotropyConditionPolicy,
    minimum_reference_gradient_norm: f64,
    low_confidence_reference_gradient_norm: f64,
    maximum_direction_jump_radians: f64,
}

impl TrendControlPolicy {
    /// Constructs policy without hidden condition, confidence, or jump thresholds.
    ///
    /// # Errors
    ///
    /// Returns a structured error for a non-finite or out-of-range threshold.
    pub fn try_new(
        anisotropy_condition: AnisotropyConditionPolicy,
        minimum_reference_gradient_norm: f64,
        low_confidence_reference_gradient_norm: f64,
        maximum_direction_jump_radians: f64,
    ) -> Result<Self, TrendControlPolicyError> {
        if !minimum_reference_gradient_norm.is_finite() {
            return Err(TrendControlPolicyError::NonFiniteMinimumGradient {
                value: minimum_reference_gradient_norm,
            });
        }
        if minimum_reference_gradient_norm <= 0.0 {
            return Err(TrendControlPolicyError::NonPositiveMinimumGradient {
                value: minimum_reference_gradient_norm,
            });
        }
        if !low_confidence_reference_gradient_norm.is_finite() {
            return Err(TrendControlPolicyError::NonFiniteLowConfidenceGradient {
                value: low_confidence_reference_gradient_norm,
            });
        }
        if low_confidence_reference_gradient_norm < minimum_reference_gradient_norm {
            return Err(TrendControlPolicyError::LowConfidenceBelowMinimum {
                minimum: minimum_reference_gradient_norm,
                low_confidence: low_confidence_reference_gradient_norm,
            });
        }
        if !maximum_direction_jump_radians.is_finite() {
            return Err(TrendControlPolicyError::NonFiniteMaximumJump {
                value: maximum_direction_jump_radians,
            });
        }
        if !(0.0..=std::f64::consts::FRAC_PI_2).contains(&maximum_direction_jump_radians) {
            return Err(TrendControlPolicyError::MaximumJumpOutOfRange {
                value: maximum_direction_jump_radians,
            });
        }
        Ok(Self {
            anisotropy_condition,
            minimum_reference_gradient_norm,
            low_confidence_reference_gradient_norm,
            maximum_direction_jump_radians,
        })
    }

    /// Returns the fixed-metric condition-number policy.
    #[must_use]
    pub const fn anisotropy_condition(self) -> AnisotropyConditionPolicy {
        self.anisotropy_condition
    }

    /// Returns the strict reference-gradient normalization lower bound.
    #[must_use]
    pub const fn minimum_reference_gradient_norm(self) -> f64 {
        self.minimum_reference_gradient_norm
    }

    /// Returns the inclusive low-confidence diagnostic threshold.
    #[must_use]
    pub const fn low_confidence_reference_gradient_norm(self) -> f64 {
        self.low_confidence_reference_gradient_norm
    }

    /// Returns the inclusive sign-invariant adjacent-direction jump limit.
    #[must_use]
    pub const fn maximum_direction_jump_radians(self) -> f64 {
        self.maximum_direction_jump_radians
    }
}

/// Invalid explicit compiler policy.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TrendControlPolicyError {
    /// The minimum reference-gradient norm is not finite.
    NonFiniteMinimumGradient {
        /// Rejected value.
        value: f64,
    },
    /// The minimum reference-gradient norm is not positive.
    NonPositiveMinimumGradient {
        /// Rejected value.
        value: f64,
    },
    /// The low-confidence threshold is not finite.
    NonFiniteLowConfidenceGradient {
        /// Rejected value.
        value: f64,
    },
    /// The low-confidence threshold is below the rejection threshold.
    LowConfidenceBelowMinimum {
        /// Rejection threshold.
        minimum: f64,
        /// Rejected diagnostic threshold.
        low_confidence: f64,
    },
    /// The maximum direction jump is not finite.
    NonFiniteMaximumJump {
        /// Rejected radians.
        value: f64,
    },
    /// The sign-invariant jump threshold is outside `[0, pi/2]`.
    MaximumJumpOutOfRange {
        /// Rejected radians.
        value: f64,
    },
}

impl fmt::Display for TrendControlPolicyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteMinimumGradient { value } => {
                write!(
                    formatter,
                    "minimum reference-gradient norm must be finite, got {value}"
                )
            }
            Self::NonPositiveMinimumGradient { value } => {
                write!(
                    formatter,
                    "minimum reference-gradient norm must be positive, got {value}"
                )
            }
            Self::NonFiniteLowConfidenceGradient { value } => write!(
                formatter,
                "low-confidence reference-gradient norm must be finite, got {value}"
            ),
            Self::LowConfidenceBelowMinimum {
                minimum,
                low_confidence,
            } => write!(
                formatter,
                "low-confidence gradient threshold {low_confidence} is below minimum {minimum}"
            ),
            Self::NonFiniteMaximumJump { value } => {
                write!(
                    formatter,
                    "maximum direction jump must be finite, got {value}"
                )
            }
            Self::MaximumJumpOutOfRange { value } => write!(
                formatter,
                "maximum sign-invariant direction jump must lie in [0, pi/2], got {value}"
            ),
        }
    }
}

impl Error for TrendControlPolicyError {}

/// Provenance and confidence attached to one resolved unit direction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ResolvedTrendDirectionSource {
    /// Caller supplied an explicit unit direction.
    Explicit,
    /// Direction was obtained by normalizing one fitted-field gradient.
    ReferenceFieldGradient {
        /// Stable referenced project field identifier.
        field_id: FieldId,
        /// Original finite Cartesian gradient norm before normalization.
        gradient_norm: f64,
        /// Whether the norm is no greater than the explicit confidence threshold.
        low_confidence: bool,
    },
}

/// One resolved unit direction and its immutable provenance.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct ResolvedTrendDirection<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    direction: UnitDirection<D>,
    source: ResolvedTrendDirectionSource,
}

impl<const D: usize> ResolvedTrendDirection<D>
where
    Dim<D>: SupportedDimension,
{
    /// Returns the resolved unit direction.
    pub const fn direction(self) -> UnitDirection<D> {
        self.direction
    }

    /// Returns direction provenance and reference-gradient confidence.
    #[must_use]
    pub const fn source(self) -> ResolvedTrendDirectionSource {
        self.source
    }
}

/// Resolved fixed orientation retained in compiler diagnostics.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ResolvedTrendOrientation<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    /// Resolved spheroid.
    Spheroidal {
        /// Resolved principal direction and provenance.
        principal_axis: ResolvedTrendDirection<D>,
        /// Principal-axis correlation length.
        axial_length: f64,
        /// Orthogonal-complement correlation length.
        transverse_length: f64,
    },
    /// Resolved caller-ordered ellipsoid.
    Ellipsoidal {
        /// Resolved ordered directions and provenance.
        principal_axes: [ResolvedTrendDirection<D>; D],
        /// Correlation lengths paired with the axes.
        axis_lengths: [f64; D],
        /// Caller-selected orthogonality tolerance.
        orthogonality_tolerance: f64,
    },
}

impl<const D: usize> ResolvedTrendOrientation<D>
where
    Dim<D>: SupportedDimension,
{
    fn primary_direction(self) -> UnitDirection<D> {
        match self {
            Self::Spheroidal { principal_axis, .. } => principal_axis.direction,
            Self::Ellipsoidal { principal_axes, .. } => principal_axes[0].direction,
        }
    }
}

/// Deterministic diagnostic record for one compiled control.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct TrendControlDiagnostics<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    location: Point<D>,
    orientation: ResolvedTrendOrientation<D>,
    influence_radius: f64,
    strength: f64,
    region: Option<SmoothRegion<D>>,
    anisotropy_condition_number: f64,
    direction_jump_from_previous_radians: Option<f64>,
    direction_jump_exceeds_policy: bool,
}

impl<const D: usize> TrendControlDiagnostics<D>
where
    Dim<D>: SupportedDimension,
{
    /// Returns the control location.
    pub const fn location(self) -> Point<D> {
        self.location
    }
    /// Returns the resolved fixed orientation and direction provenance.
    #[must_use]
    pub const fn orientation(self) -> ResolvedTrendOrientation<D> {
        self.orientation
    }
    /// Returns the Gaussian influence radius.
    #[must_use]
    pub const fn influence_radius(self) -> f64 {
        self.influence_radius
    }
    /// Returns the signed nonzero strength.
    #[must_use]
    pub const fn strength(self) -> f64 {
        self.strength
    }
    /// Returns the optional compact C2 region.
    #[must_use]
    pub const fn region(self) -> Option<SmoothRegion<D>> {
        self.region
    }
    /// Returns the fixed anisotropy condition number.
    #[must_use]
    pub const fn anisotropy_condition_number(self) -> f64 {
        self.anisotropy_condition_number
    }
    /// Returns the sign-invariant jump from the preceding control.
    #[must_use]
    pub const fn direction_jump_from_previous_radians(self) -> Option<f64> {
        self.direction_jump_from_previous_radians
    }
    /// Reports whether that jump exceeds explicit policy.
    #[must_use]
    pub const fn direction_jump_exceeds_policy(self) -> bool {
        self.direction_jump_exceeds_policy
    }
}

/// Immutable compiler diagnostics in caller control order.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct TrendCompilationDiagnostics<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    controls: Vec<TrendControlDiagnostics<D>>,
    maximum_direction_jump_radians: f64,
    low_confidence_direction_count: usize,
    jump_exceedance_count: usize,
}

impl<const D: usize> TrendCompilationDiagnostics<D>
where
    Dim<D>: SupportedDimension,
{
    /// Borrows per-control records in caller order.
    pub fn controls(&self) -> &[TrendControlDiagnostics<D>] {
        &self.controls
    }
    /// Returns the largest adjacent primary-direction jump.
    #[must_use]
    pub const fn maximum_direction_jump_radians(&self) -> f64 {
        self.maximum_direction_jump_radians
    }
    /// Returns the number of resolved directions at or below confidence policy.
    #[must_use]
    pub const fn low_confidence_direction_count(&self) -> usize {
        self.low_confidence_direction_count
    }
    /// Returns the number of adjacent jumps exceeding policy.
    #[must_use]
    pub const fn jump_exceedance_count(&self) -> usize {
        self.jump_exceedance_count
    }
}

/// Validated local mixture and the control evidence used to build it.
#[derive(Debug)]
#[must_use]
pub struct CompiledTrendControls<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    mixture: LocalTrendMixture<D>,
    diagnostics: TrendCompilationDiagnostics<D>,
}

impl<const D: usize> CompiledTrendControls<D>
where
    Dim<D>: SupportedDimension,
{
    /// Borrows the strictly positive-definite local mixture.
    pub const fn mixture(&self) -> &LocalTrendMixture<D> {
        &self.mixture
    }
    /// Borrows deterministic compiler diagnostics.
    pub const fn diagnostics(&self) -> &TrendCompilationDiagnostics<D> {
        &self.diagnostics
    }
    /// Consumes the compiler result.
    pub fn into_parts(self) -> (LocalTrendMixture<D>, TrendCompilationDiagnostics<D>) {
        (self.mixture, self.diagnostics)
    }
}

/// Fallible compiler allocation category.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrendControlStorage {
    /// Compiled local-mixture components.
    Components,
    /// Per-control diagnostic records.
    Diagnostics,
}

/// Structured regional/reference-control compilation failure.
#[derive(Debug)]
pub enum TrendControlCompilationError<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    /// No local control was supplied.
    EmptyControls,
    /// One control has exactly zero strength.
    ZeroStrength {
        /// Control index.
        control: usize,
    },
    /// A reference source was supplied without a project.
    MissingReferenceProject {
        /// Control index.
        control: usize,
        /// Axis index within the orientation.
        axis: usize,
        /// Requested field.
        field_id: FieldId,
    },
    /// A project does not contain the requested field.
    UnknownReferenceField {
        /// Control index.
        control: usize,
        /// Axis index within the orientation.
        axis: usize,
        /// Requested field.
        field_id: FieldId,
    },
    /// The immutable fitted field could not evaluate its Cartesian gradient.
    ReferenceEvaluation {
        /// Control index.
        control: usize,
        /// Axis index within the orientation.
        axis: usize,
        /// Requested field.
        field_id: FieldId,
        /// Preserved fitted-field failure.
        source: Box<FittedFieldEvaluationError<D>>,
    },
    /// A finite reference gradient has no finitely representable norm.
    NonRepresentableReferenceGradientNorm {
        /// Control index.
        control: usize,
        /// Axis index within the orientation.
        axis: usize,
        /// Requested field.
        field_id: FieldId,
    },
    /// The reference-gradient norm is below explicit rejection policy.
    ReferenceGradientBelowMinimum {
        /// Control index.
        control: usize,
        /// Axis index within the orientation.
        axis: usize,
        /// Requested field.
        field_id: FieldId,
        /// Computed norm.
        norm: f64,
        /// Required norm.
        minimum: f64,
    },
    /// A non-rejected reference gradient could not form a unit direction.
    InvalidReferenceGradientDirection {
        /// Control index.
        control: usize,
        /// Axis index within the orientation.
        axis: usize,
        /// Requested field.
        field_id: FieldId,
    },
    /// One Gaussian or regional weight failed validation.
    Weight {
        /// Control index.
        control: usize,
        /// Preserved primitive error.
        source: LocalTrendConstructionError<D>,
    },
    /// One fixed spheroidal or ellipsoidal metric failed validation.
    Anisotropy {
        /// Control index.
        control: usize,
        /// Preserved fixed-metric error.
        source: AnisotropyError<D>,
    },
    /// Checked compiler storage allocation failed.
    AllocationFailed {
        /// Intended storage role.
        storage: TrendControlStorage,
        /// Requested entry count.
        requested: usize,
    },
    /// Background plus control count overflowed `usize`.
    ComponentCountOverflow,
    /// The final strict-background mixture rejected compiled components.
    Mixture(LocalTrendConstructionError<D>),
}

impl<const D: usize> fmt::Display for TrendControlCompilationError<D>
where
    Dim<D>: SupportedDimension,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyControls => {
                formatter.write_str("trend compilation requires a local control")
            }
            Self::ZeroStrength { control } => {
                write!(formatter, "trend control {control} has zero strength")
            }
            Self::MissingReferenceProject {
                control,
                axis,
                field_id,
            } => write!(
                formatter,
                "trend control {control} axis {axis} references field {} without a project",
                field_id.identifier()
            ),
            Self::UnknownReferenceField {
                control,
                axis,
                field_id,
            } => write!(
                formatter,
                "trend control {control} axis {axis} references unknown field {}",
                field_id.identifier()
            ),
            Self::ReferenceEvaluation {
                control,
                axis,
                field_id,
                source,
            } => write!(
                formatter,
                "trend control {control} axis {axis} could not evaluate reference field {}: {source}",
                field_id.identifier()
            ),
            Self::NonRepresentableReferenceGradientNorm {
                control,
                axis,
                field_id,
            } => write!(
                formatter,
                "trend control {control} axis {axis} reference field {} has a non-representable gradient norm",
                field_id.identifier()
            ),
            Self::ReferenceGradientBelowMinimum {
                control,
                axis,
                field_id,
                norm,
                minimum,
            } => write!(
                formatter,
                "trend control {control} axis {axis} reference field {} gradient norm {norm} is below minimum {minimum}",
                field_id.identifier()
            ),
            Self::InvalidReferenceGradientDirection {
                control,
                axis,
                field_id,
            } => write!(
                formatter,
                "trend control {control} axis {axis} reference field {} gradient could not be normalized",
                field_id.identifier()
            ),
            Self::Weight { control, source } => write!(
                formatter,
                "trend control {control} weight is invalid: {source}"
            ),
            Self::Anisotropy { control, source } => write!(
                formatter,
                "trend control {control} anisotropy is invalid: {source}"
            ),
            Self::AllocationFailed { storage, requested } => write!(
                formatter,
                "could not allocate {requested} trend-control {storage:?} entries"
            ),
            Self::ComponentCountOverflow => {
                formatter.write_str("trend-control component count overflowed usize")
            }
            Self::Mixture(source) => write!(formatter, "compiled local trend is invalid: {source}"),
        }
    }
}

impl<const D: usize> Error for TrendControlCompilationError<D>
where
    Dim<D>: SupportedDimension,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ReferenceEvaluation { source, .. } => Some(source.as_ref()),
            Self::Weight { source, .. } | Self::Mixture(source) => Some(source),
            Self::Anisotropy { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// Resolves and compiles ordered controls into the existing strict-background mixture.
///
/// Reference-gradient points are interpreted in each referenced field's own
/// original-coordinate convention. The caller is responsible for supplying
/// controls, regions, and lengths in that same convention; no reprojection or
/// cross-field convention inference occurs.
///
/// # Errors
///
/// Returns a structured reference resolution, gradient evaluation, weight,
/// fixed-anisotropy, allocation, or final-mixture validation error.
#[allow(clippy::too_many_arguments)]
pub fn try_compile_local_trend_controls<const D: usize>(
    background: LocalTrendBackground<D>,
    controls: &[LocalTrendControl<D>],
    project: Option<&GeoProject<D>>,
    operational_domain: OperationalDomain<D>,
    minimum_background_weight: f64,
    policy: TrendControlPolicy,
) -> Result<CompiledTrendControls<D>, TrendControlCompilationError<D>>
where
    Dim<D>: SupportedDimension,
{
    if controls.is_empty() {
        return Err(TrendControlCompilationError::EmptyControls);
    }
    let component_count = controls
        .len()
        .checked_add(1)
        .ok_or(TrendControlCompilationError::ComponentCountOverflow)?;
    let mut components = Vec::new();
    components.try_reserve_exact(component_count).map_err(|_| {
        TrendControlCompilationError::AllocationFailed {
            storage: TrendControlStorage::Components,
            requested: component_count,
        }
    })?;
    let mut diagnostics = Vec::new();
    diagnostics.try_reserve_exact(controls.len()).map_err(|_| {
        TrendControlCompilationError::AllocationFailed {
            storage: TrendControlStorage::Diagnostics,
            requested: controls.len(),
        }
    })?;
    components.push(LocalTrendComponent::new(
        background.kernel,
        background.anisotropy,
        background.weight,
    ));

    let mut previous_direction = None;
    let mut maximum_jump = 0.0_f64;
    let mut low_confidence_direction_count = 0_usize;
    let mut jump_exceedance_count = 0_usize;
    for (control_index, control) in controls.iter().copied().enumerate() {
        if control.strength == 0.0 {
            return Err(TrendControlCompilationError::ZeroStrength {
                control: control_index,
            });
        }
        let (anisotropy, orientation, low_confidence_count) = resolve_orientation(
            control.orientation,
            control.location,
            control_index,
            project,
            policy,
        )?;
        low_confidence_direction_count =
            low_confidence_direction_count.saturating_add(low_confidence_count);
        let weight = match control.region {
            Some(region) => SmoothSpatialWeight::try_regional_gaussian(
                control.location,
                control.strength,
                control.influence_radius,
                region,
            ),
            None => SmoothSpatialWeight::try_gaussian(
                control.location,
                control.strength,
                control.influence_radius,
            ),
        }
        .map_err(|source| TrendControlCompilationError::Weight {
            control: control_index,
            source,
        })?;
        let primary_direction = orientation.primary_direction();
        let jump = previous_direction.map(|previous| direction_jump(previous, primary_direction));
        let exceeds = jump.is_some_and(|value| value > policy.maximum_direction_jump_radians);
        if let Some(value) = jump {
            maximum_jump = maximum_jump.max(value);
        }
        if exceeds {
            jump_exceedance_count = jump_exceedance_count.saturating_add(1);
        }
        diagnostics.push(TrendControlDiagnostics {
            location: control.location,
            orientation,
            influence_radius: control.influence_radius,
            strength: control.strength,
            region: control.region,
            anisotropy_condition_number: anisotropy.diagnostics().condition_number(),
            direction_jump_from_previous_radians: jump,
            direction_jump_exceeds_policy: exceeds,
        });
        components.push(LocalTrendComponent::new(control.kernel, anisotropy, weight));
        previous_direction = Some(primary_direction);
    }

    let mixture =
        LocalTrendMixture::try_new(components, 0, operational_domain, minimum_background_weight)
            .map_err(TrendControlCompilationError::Mixture)?;
    Ok(CompiledTrendControls {
        mixture,
        diagnostics: TrendCompilationDiagnostics {
            controls: diagnostics,
            maximum_direction_jump_radians: maximum_jump,
            low_confidence_direction_count,
            jump_exceedance_count,
        },
    })
}

fn resolve_orientation<const D: usize>(
    orientation: TrendControlOrientation<D>,
    location: Point<D>,
    control: usize,
    project: Option<&GeoProject<D>>,
    policy: TrendControlPolicy,
) -> Result<
    (GlobalAnisotropy<D>, ResolvedTrendOrientation<D>, usize),
    TrendControlCompilationError<D>,
>
where
    Dim<D>: SupportedDimension,
{
    match orientation {
        TrendControlOrientation::Spheroidal {
            principal_axis,
            axial_length,
            transverse_length,
        } => {
            let resolved =
                resolve_direction(principal_axis, location, control, 0, project, policy)?;
            let low_confidence = usize::from(matches!(
                resolved.source,
                ResolvedTrendDirectionSource::ReferenceFieldGradient {
                    low_confidence: true,
                    ..
                }
            ));
            let anisotropy = GlobalAnisotropy::try_spheroidal(
                resolved.direction,
                axial_length,
                transverse_length,
                policy.anisotropy_condition,
            )
            .map_err(|source| TrendControlCompilationError::Anisotropy { control, source })?;
            Ok((
                anisotropy,
                ResolvedTrendOrientation::Spheroidal {
                    principal_axis: resolved,
                    axial_length,
                    transverse_length,
                },
                low_confidence,
            ))
        }
        TrendControlOrientation::Ellipsoidal {
            principal_axes,
            axis_lengths,
            orthogonality_tolerance,
        } => {
            let first =
                resolve_direction(principal_axes[0], location, control, 0, project, policy)?;
            let mut resolved = [first; D];
            for axis in 1..D {
                resolved[axis] = resolve_direction(
                    principal_axes[axis],
                    location,
                    control,
                    axis,
                    project,
                    policy,
                )?;
            }
            let low_confidence = resolved
                .iter()
                .filter(|entry| {
                    matches!(
                        entry.source,
                        ResolvedTrendDirectionSource::ReferenceFieldGradient {
                            low_confidence: true,
                            ..
                        }
                    )
                })
                .count();
            let directions = resolved.map(|entry| entry.direction);
            let anisotropy = GlobalAnisotropy::try_ellipsoidal(
                directions,
                axis_lengths,
                orthogonality_tolerance,
                policy.anisotropy_condition,
            )
            .map_err(|source| TrendControlCompilationError::Anisotropy { control, source })?;
            Ok((
                anisotropy,
                ResolvedTrendOrientation::Ellipsoidal {
                    principal_axes: resolved,
                    axis_lengths,
                    orthogonality_tolerance,
                },
                low_confidence,
            ))
        }
    }
}

fn resolve_direction<const D: usize>(
    source: TrendDirectionSource<D>,
    location: Point<D>,
    control: usize,
    axis: usize,
    project: Option<&GeoProject<D>>,
    policy: TrendControlPolicy,
) -> Result<ResolvedTrendDirection<D>, TrendControlCompilationError<D>>
where
    Dim<D>: SupportedDimension,
{
    match source {
        TrendDirectionSource::Explicit(direction) => Ok(ResolvedTrendDirection {
            direction,
            source: ResolvedTrendDirectionSource::Explicit,
        }),
        TrendDirectionSource::ReferenceFieldGradient(field_id) => {
            let project = project.ok_or(TrendControlCompilationError::MissingReferenceProject {
                control,
                axis,
                field_id,
            })?;
            let field = project.field(field_id).ok_or(
                TrendControlCompilationError::UnknownReferenceField {
                    control,
                    axis,
                    field_id,
                },
            )?;
            let gradient = field.try_gradient(location).map_err(|source| {
                TrendControlCompilationError::ReferenceEvaluation {
                    control,
                    axis,
                    field_id,
                    source: Box::new(source),
                }
            })?;
            let norm = stable_norm(*gradient.components()).ok_or(
                TrendControlCompilationError::NonRepresentableReferenceGradientNorm {
                    control,
                    axis,
                    field_id,
                },
            )?;
            if norm < policy.minimum_reference_gradient_norm {
                return Err(
                    TrendControlCompilationError::ReferenceGradientBelowMinimum {
                        control,
                        axis,
                        field_id,
                        norm,
                        minimum: policy.minimum_reference_gradient_norm,
                    },
                );
            }
            let direction = UnitDirection::try_new(*gradient.components()).map_err(|_| {
                TrendControlCompilationError::InvalidReferenceGradientDirection {
                    control,
                    axis,
                    field_id,
                }
            })?;
            Ok(ResolvedTrendDirection {
                direction,
                source: ResolvedTrendDirectionSource::ReferenceFieldGradient {
                    field_id,
                    gradient_norm: norm,
                    low_confidence: norm <= policy.low_confidence_reference_gradient_norm,
                },
            })
        }
    }
}

fn stable_norm<const D: usize>(components: [f64; D]) -> Option<f64> {
    let scale = components
        .iter()
        .fold(0.0_f64, |current, component| current.max(component.abs()));
    if scale == 0.0 {
        return Some(0.0);
    }
    let scaled_squared = components
        .iter()
        .map(|component| component / scale)
        .map(|component| component * component)
        .sum::<f64>();
    let norm = scale * scaled_squared.sqrt();
    norm.is_finite().then_some(norm)
}

fn direction_jump<const D: usize>(first: UnitDirection<D>, second: UnitDirection<D>) -> f64
where
    Dim<D>: SupportedDimension,
{
    let dot = first
        .components()
        .iter()
        .zip(second.components())
        .map(|(left, right)| left * right)
        .sum::<f64>();
    dot.abs().min(1.0).acos()
}
