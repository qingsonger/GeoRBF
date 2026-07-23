//! Renderer- and format-neutral exports for local anisotropy diagnostics.
//!
//! The owned records in this module are an in-memory Rust API, not a versioned
//! persistence schema. They preserve the caller order and explicit policy
//! evidence already validated by [`crate::try_compile_local_trend_controls`].
//! Export never changes controls, refits a field, or adjusts an SPD metric.
//!
//! ```compile_fail
//! use georbf::AnisotropyDiagnosticExport;
//!
//! fn unsupported(_: Option<AnisotropyDiagnosticExport<4>>) {}
//! ```

use std::error::Error;
use std::fmt;

use crate::dimension::{Dim, SupportedDimension};
use crate::geometry::Point;
use crate::local_trend::{LocalTrendCoverage, LocalTrendEvaluationError, SmoothRegion};
use crate::project::FieldId;
use crate::trend_controls::{
    CompiledTrendControls, ResolvedTrendDirection, ResolvedTrendDirectionSource,
    ResolvedTrendOrientation,
};

/// Honest resolved orientation for one exported local control.
///
/// A spheroid has no unique transverse basis, so only its unique principal
/// axis and transverse length are exported. An ellipsoid retains every
/// caller-ordered resolved axis and paired length.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnisotropyControlOrientation<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    /// One principal axis and a shared orthogonal-complement length.
    Spheroidal {
        /// Resolved principal direction with provenance and confidence.
        principal_axis: ResolvedTrendDirection<D>,
        /// Correlation length along the principal axis.
        axial_length: f64,
        /// Correlation length throughout the orthogonal complement.
        transverse_length: f64,
    },
    /// Fully caller-resolved ellipsoidal axes and lengths.
    Ellipsoidal {
        /// Caller-ordered resolved axes.
        principal_axes: [ResolvedTrendDirection<D>; D],
        /// Correlation lengths paired with `principal_axes`.
        axis_lengths: [f64; D],
        /// Caller-selected absolute orthogonality tolerance.
        orthogonality_tolerance: f64,
    },
}

impl<const D: usize> From<ResolvedTrendOrientation<D>> for AnisotropyControlOrientation<D>
where
    Dim<D>: SupportedDimension,
{
    fn from(value: ResolvedTrendOrientation<D>) -> Self {
        match value {
            ResolvedTrendOrientation::Spheroidal {
                principal_axis,
                axial_length,
                transverse_length,
            } => Self::Spheroidal {
                principal_axis,
                axial_length,
                transverse_length,
            },
            ResolvedTrendOrientation::Ellipsoidal {
                principal_axes,
                axis_lengths,
                orthogonality_tolerance,
            } => Self::Ellipsoidal {
                principal_axes,
                axis_lengths,
                orthogonality_tolerance,
            },
        }
    }
}

/// One deterministic control row in an anisotropy diagnostic export.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct AnisotropyControlRecord<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    control_index: usize,
    component_index: usize,
    position: Point<D>,
    orientation: AnisotropyControlOrientation<D>,
    strength: f64,
    influence_radius: f64,
    region: Option<SmoothRegion<D>>,
    condition_number: f64,
    direction_jump_from_previous_radians: Option<f64>,
    direction_jump_exceeds_policy: bool,
}

impl<const D: usize> AnisotropyControlRecord<D>
where
    Dim<D>: SupportedDimension,
{
    /// Returns the zero-based caller control index.
    #[must_use]
    pub const fn control_index(self) -> usize {
        self.control_index
    }

    /// Returns the corresponding local-mixture component index.
    #[must_use]
    pub const fn component_index(self) -> usize {
        self.component_index
    }

    /// Returns the control position in the active coordinate convention.
    pub const fn position(self) -> Point<D> {
        self.position
    }

    /// Returns the resolved spheroidal or ellipsoidal orientation.
    #[must_use]
    pub const fn orientation(self) -> AnisotropyControlOrientation<D> {
        self.orientation
    }

    /// Returns the signed spatial-weight amplitude.
    #[must_use]
    pub const fn strength(self) -> f64 {
        self.strength
    }

    /// Returns the Gaussian influence radius.
    #[must_use]
    pub const fn influence_radius(self) -> f64 {
        self.influence_radius
    }

    /// Returns the optional compact C2 support region.
    #[must_use]
    pub const fn region(self) -> Option<SmoothRegion<D>> {
        self.region
    }

    /// Returns the fixed-metric Euclidean condition number.
    #[must_use]
    pub const fn condition_number(self) -> f64 {
        self.condition_number
    }

    /// Returns the sign-invariant primary-axis jump from the preceding control.
    #[must_use]
    pub const fn direction_jump_from_previous_radians(self) -> Option<f64> {
        self.direction_jump_from_previous_radians
    }

    /// Reports whether the adjacent jump exceeds explicit compilation policy.
    #[must_use]
    pub const fn direction_jump_exceeds_policy(self) -> bool {
        self.direction_jump_exceeds_policy
    }
}

/// Strict-background evidence exported independently of local control rows.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct AnisotropyBackgroundRecord {
    component_index: usize,
    weight_magnitude: f64,
    minimum_weight: f64,
    policy_ratio: f64,
    condition_number: f64,
}

impl AnisotropyBackgroundRecord {
    /// Returns the strict-background mixture component index.
    #[must_use]
    pub const fn component_index(self) -> usize {
        self.component_index
    }

    /// Returns the absolute everywhere-constant background weight.
    #[must_use]
    pub const fn weight_magnitude(self) -> f64 {
        self.weight_magnitude
    }

    /// Returns the caller's explicit positive background minimum.
    #[must_use]
    pub const fn minimum_weight(self) -> f64 {
        self.minimum_weight
    }

    /// Returns `weight_magnitude / minimum_weight`.
    #[must_use]
    pub const fn policy_ratio(self) -> f64 {
        self.policy_ratio
    }

    /// Returns the background component's fixed-metric condition number.
    #[must_use]
    pub const fn condition_number(self) -> f64 {
        self.condition_number
    }
}

/// Pointwise signed component weights and aggregate coverage evidence.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct AnisotropySampleRecord<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    position: Point<D>,
    component_weights: Vec<f64>,
    coverage: LocalTrendCoverage,
}

impl<const D: usize> AnisotropySampleRecord<D>
where
    Dim<D>: SupportedDimension,
{
    /// Returns the sampled position.
    pub const fn position(&self) -> Point<D> {
        self.position
    }

    /// Borrows signed spatial weights in local-mixture component order.
    #[must_use]
    pub fn component_weights(&self) -> &[f64] {
        &self.component_weights
    }

    /// Returns aggregate squared-weight coverage and domain membership.
    pub const fn coverage(&self) -> LocalTrendCoverage {
        self.coverage
    }
}

/// One source-aware low-confidence reference direction and its support region.
///
/// `None` for [`Self::region`] means the Gaussian has no compact region; the
/// export does not invent a finite box for that unbounded support.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct LowConfidenceTrendRegion<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    control_index: usize,
    axis_index: usize,
    position: Point<D>,
    region: Option<SmoothRegion<D>>,
    field_id: FieldId,
    gradient_norm: f64,
}

impl<const D: usize> LowConfidenceTrendRegion<D>
where
    Dim<D>: SupportedDimension,
{
    /// Returns the zero-based caller control index.
    #[must_use]
    pub const fn control_index(self) -> usize {
        self.control_index
    }

    /// Returns the zero-based orientation-axis index.
    #[must_use]
    pub const fn axis_index(self) -> usize {
        self.axis_index
    }

    /// Returns the control position where the reference gradient was sampled.
    pub const fn position(self) -> Point<D> {
        self.position
    }

    /// Returns the optional compact C2 support region.
    #[must_use]
    pub const fn region(self) -> Option<SmoothRegion<D>> {
        self.region
    }

    /// Returns the referenced immutable project field identifier.
    pub const fn field_id(self) -> FieldId {
        self.field_id
    }

    /// Returns the original Cartesian gradient norm before normalization.
    #[must_use]
    pub const fn gradient_norm(self) -> f64 {
        self.gradient_norm
    }
}

/// Aggregate policy evidence for one diagnostic export.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct AnisotropyDiagnosticSummary {
    maximum_anisotropy_condition_number: f64,
    maximum_direction_jump_radians: f64,
    low_confidence_direction_count: usize,
    jump_exceedance_count: usize,
}

impl AnisotropyDiagnosticSummary {
    /// Returns the maximum fixed-metric condition number in the mixture.
    #[must_use]
    pub const fn maximum_anisotropy_condition_number(self) -> f64 {
        self.maximum_anisotropy_condition_number
    }

    /// Returns the maximum sign-invariant adjacent primary-axis jump.
    #[must_use]
    pub const fn maximum_direction_jump_radians(self) -> f64 {
        self.maximum_direction_jump_radians
    }

    /// Returns the number of low-confidence resolved reference directions.
    #[must_use]
    pub const fn low_confidence_direction_count(self) -> usize {
        self.low_confidence_direction_count
    }

    /// Returns the number of adjacent jumps exceeding explicit policy.
    #[must_use]
    pub const fn jump_exceedance_count(self) -> usize {
        self.jump_exceedance_count
    }
}

/// Complete owned renderer-neutral anisotropy diagnostics.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct AnisotropyDiagnosticExport<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    controls: Vec<AnisotropyControlRecord<D>>,
    background: AnisotropyBackgroundRecord,
    samples: Vec<AnisotropySampleRecord<D>>,
    low_confidence_regions: Vec<LowConfidenceTrendRegion<D>>,
    summary: AnisotropyDiagnosticSummary,
}

impl<const D: usize> AnisotropyDiagnosticExport<D>
where
    Dim<D>: SupportedDimension,
{
    /// Borrows control records in caller order.
    pub fn controls(&self) -> &[AnisotropyControlRecord<D>] {
        &self.controls
    }

    /// Returns strict-background policy and condition evidence.
    pub const fn background(&self) -> AnisotropyBackgroundRecord {
        self.background
    }

    /// Borrows point samples in caller order.
    pub fn samples(&self) -> &[AnisotropySampleRecord<D>] {
        &self.samples
    }

    /// Borrows low-confidence reference regions in control/axis order.
    pub fn low_confidence_regions(&self) -> &[LowConfidenceTrendRegion<D>] {
        &self.low_confidence_regions
    }

    /// Returns aggregate condition, confidence, and jump evidence.
    pub const fn summary(&self) -> AnisotropyDiagnosticSummary {
        self.summary
    }
}

/// Fallible owned storage category used by diagnostic export.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnisotropyDiagnosticStorage {
    /// Per-control records.
    Controls,
    /// Point-sample records.
    Samples,
    /// Signed component weights for one point sample.
    ComponentWeights {
        /// Zero-based sample index.
        sample: usize,
    },
    /// Source-aware low-confidence region records.
    LowConfidenceRegions,
}

/// Structured anisotropy diagnostic export failure.
#[derive(Clone, Debug, PartialEq)]
pub enum AnisotropyDiagnosticExportError<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    /// Compiled control and mixture component counts disagree.
    InconsistentComponentCount {
        /// Number of compiler diagnostic control rows.
        controls: usize,
        /// Number of local-mixture components, including the background.
        components: usize,
    },
    /// Observed and retained low-confidence counts disagree.
    InconsistentLowConfidenceCount {
        /// Count retained by compilation diagnostics.
        reported: usize,
        /// Count observed while exporting source-aware region rows.
        observed: usize,
    },
    /// Owned output storage could not be reserved.
    AllocationFailed {
        /// Output category that could not be reserved.
        storage: AnisotropyDiagnosticStorage,
        /// Requested element capacity.
        requested: usize,
    },
    /// Spatial-weight or aggregate-coverage evaluation failed.
    Evaluation {
        /// Zero-based sample index.
        sample: usize,
        /// Underlying local-mixture evaluation error.
        source: LocalTrendEvaluationError<D>,
    },
}

impl<const D: usize> fmt::Display for AnisotropyDiagnosticExportError<D>
where
    Dim<D>: SupportedDimension,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InconsistentComponentCount {
                controls,
                components,
            } => write!(
                formatter,
                "compiled anisotropy diagnostics contain {controls} controls but {components} mixture components"
            ),
            Self::InconsistentLowConfidenceCount { reported, observed } => write!(
                formatter,
                "compiled anisotropy diagnostics report {reported} low-confidence directions but export observed {observed}"
            ),
            Self::AllocationFailed { storage, requested } => write!(
                formatter,
                "could not reserve {requested} entries for anisotropy diagnostic {storage:?}"
            ),
            Self::Evaluation { sample, source } => {
                write!(
                    formatter,
                    "anisotropy diagnostic sample {sample} failed: {source}"
                )
            }
        }
    }
}

impl<const D: usize> Error for AnisotropyDiagnosticExportError<D>
where
    Dim<D>: SupportedDimension,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Evaluation { source, .. } => Some(source),
            Self::InconsistentComponentCount { .. }
            | Self::InconsistentLowConfidenceCount { .. }
            | Self::AllocationFailed { .. } => None,
        }
    }
}

/// Exports deterministic local anisotropy diagnostics at requested positions.
///
/// Control records, low-confidence regions, point samples, and component
/// weights preserve their respective caller or mixture orders. Point weights
/// are signed; coverage retains the sum of their squares and the strict
/// background contribution separately.
///
/// # Errors
///
/// Returns a structured error if owned storage cannot be reserved, an internal
/// compiled-result invariant is inconsistent, or a requested point's weight or
/// aggregate coverage is not finitely representable.
#[allow(clippy::too_many_lines)]
pub fn try_export_anisotropy_diagnostics<const D: usize>(
    compiled: &CompiledTrendControls<D>,
    sample_positions: &[Point<D>],
) -> Result<AnisotropyDiagnosticExport<D>, AnisotropyDiagnosticExportError<D>>
where
    Dim<D>: SupportedDimension,
{
    let mixture = compiled.mixture();
    let compilation = compiled.diagnostics();
    let control_count = compilation.controls().len();
    let component_count = mixture.components().len();
    if control_count.checked_add(1) != Some(component_count) {
        return Err(
            AnisotropyDiagnosticExportError::InconsistentComponentCount {
                controls: control_count,
                components: component_count,
            },
        );
    }

    let mut controls = Vec::new();
    controls.try_reserve_exact(control_count).map_err(|_| {
        AnisotropyDiagnosticExportError::AllocationFailed {
            storage: AnisotropyDiagnosticStorage::Controls,
            requested: control_count,
        }
    })?;
    let reported_low_confidence = compilation.low_confidence_direction_count();
    let mut low_confidence_regions = Vec::new();
    low_confidence_regions
        .try_reserve_exact(reported_low_confidence)
        .map_err(|_| AnisotropyDiagnosticExportError::AllocationFailed {
            storage: AnisotropyDiagnosticStorage::LowConfidenceRegions,
            requested: reported_low_confidence,
        })?;

    for (control_index, diagnostic) in compilation.controls().iter().copied().enumerate() {
        let orientation = diagnostic.orientation();
        collect_low_confidence_regions(
            &mut low_confidence_regions,
            control_index,
            diagnostic.location(),
            diagnostic.region(),
            orientation,
        );
        controls.push(AnisotropyControlRecord {
            control_index,
            component_index: control_index + 1,
            position: diagnostic.location(),
            orientation: orientation.into(),
            strength: diagnostic.strength(),
            influence_radius: diagnostic.influence_radius(),
            region: diagnostic.region(),
            condition_number: diagnostic.anisotropy_condition_number(),
            direction_jump_from_previous_radians: diagnostic.direction_jump_from_previous_radians(),
            direction_jump_exceeds_policy: diagnostic.direction_jump_exceeds_policy(),
        });
    }
    if low_confidence_regions.len() != reported_low_confidence {
        return Err(
            AnisotropyDiagnosticExportError::InconsistentLowConfidenceCount {
                reported: reported_low_confidence,
                observed: low_confidence_regions.len(),
            },
        );
    }

    let mut samples = Vec::new();
    samples
        .try_reserve_exact(sample_positions.len())
        .map_err(|_| AnisotropyDiagnosticExportError::AllocationFailed {
            storage: AnisotropyDiagnosticStorage::Samples,
            requested: sample_positions.len(),
        })?;
    for (sample, position) in sample_positions.iter().copied().enumerate() {
        let mut component_weights = Vec::new();
        component_weights
            .try_reserve_exact(component_count)
            .map_err(|_| AnisotropyDiagnosticExportError::AllocationFailed {
                storage: AnisotropyDiagnosticStorage::ComponentWeights { sample },
                requested: component_count,
            })?;
        for (component, entry) in mixture.components().iter().enumerate() {
            let weight = entry
                .weight()
                .try_diagnostic_value(position, component)
                .map_err(|source| AnisotropyDiagnosticExportError::Evaluation { sample, source })?;
            component_weights.push(weight);
        }
        let coverage = mixture
            .try_coverage(position)
            .map_err(|source| AnisotropyDiagnosticExportError::Evaluation { sample, source })?;
        samples.push(AnisotropySampleRecord {
            position,
            component_weights,
            coverage,
        });
    }

    let mixture_diagnostics = mixture.diagnostics();
    let background_index = mixture.background_index();
    let background_component = mixture.components().get(background_index).ok_or(
        AnisotropyDiagnosticExportError::InconsistentComponentCount {
            controls: control_count,
            components: component_count,
        },
    )?;
    Ok(AnisotropyDiagnosticExport {
        controls,
        background: AnisotropyBackgroundRecord {
            component_index: background_index,
            weight_magnitude: mixture_diagnostics.background_weight_magnitude(),
            minimum_weight: mixture_diagnostics.minimum_background_weight(),
            policy_ratio: mixture_diagnostics.background_policy_ratio(),
            condition_number: background_component
                .anisotropy()
                .diagnostics()
                .condition_number(),
        },
        samples,
        low_confidence_regions,
        summary: AnisotropyDiagnosticSummary {
            maximum_anisotropy_condition_number: mixture_diagnostics
                .maximum_anisotropy_condition_number(),
            maximum_direction_jump_radians: compilation.maximum_direction_jump_radians(),
            low_confidence_direction_count: reported_low_confidence,
            jump_exceedance_count: compilation.jump_exceedance_count(),
        },
    })
}

fn collect_low_confidence_regions<const D: usize>(
    output: &mut Vec<LowConfidenceTrendRegion<D>>,
    control_index: usize,
    position: Point<D>,
    region: Option<SmoothRegion<D>>,
    orientation: ResolvedTrendOrientation<D>,
) where
    Dim<D>: SupportedDimension,
{
    match orientation {
        ResolvedTrendOrientation::Spheroidal { principal_axis, .. } => {
            push_low_confidence_region(output, control_index, 0, position, region, principal_axis);
        }
        ResolvedTrendOrientation::Ellipsoidal { principal_axes, .. } => {
            for (axis_index, axis) in principal_axes.into_iter().enumerate() {
                push_low_confidence_region(
                    output,
                    control_index,
                    axis_index,
                    position,
                    region,
                    axis,
                );
            }
        }
    }
}

fn push_low_confidence_region<const D: usize>(
    output: &mut Vec<LowConfidenceTrendRegion<D>>,
    control_index: usize,
    axis_index: usize,
    position: Point<D>,
    region: Option<SmoothRegion<D>>,
    direction: ResolvedTrendDirection<D>,
) where
    Dim<D>: SupportedDimension,
{
    if let ResolvedTrendDirectionSource::ReferenceFieldGradient {
        field_id,
        gradient_norm,
        low_confidence: true,
    } = direction.source()
    {
        output.push(LowConfidenceTrendRegion {
            control_index,
            axis_index,
            position,
            region,
            field_id,
            gradient_norm,
        });
    }
}
