//! Positive-definite local kernel mixtures with complete query product rules.
//!
//! A local trend is represented without a location-dependent point-pair
//! metric:
//!
//! ```text
//! k(x, y) = sum_r b_r(x) b_r(y) k_r(x, y).
//! ```
//!
//! Every `k_r` is a fixed-metric strictly positive-definite kernel. One
//! explicitly identified background has a finite nonzero constant weight, so
//! its diagonal congruence is invertible for every finite point set. The
//! operational-domain lower bound is explicit policy, not hidden jitter.
//!
//! ```compile_fail
//! use georbf::LocalTrendMixture;
//!
//! fn unsupported(_: Option<LocalTrendMixture<4>>) {}
//! ```

use std::error::Error;
use std::fmt;

use crate::anisotropy::GlobalAnisotropy;
use crate::dimension::{Dim, SupportedDimension};
use crate::geometry::Point;
use crate::kernel::{KernelDefiniteness, KernelDerivativeCapability, KernelDerivativeOrder};
use crate::kernel_calculus::KernelArgument;
use crate::model::{KernelDefinition, KernelDefinitionEvaluationError};

/// Closed finite axis-aligned domain on which background conditioning policy applies.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct OperationalDomain<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    minimum: Point<D>,
    maximum: Point<D>,
}

/// A closed axis-aligned control region with an analytic C2 interior taper.
///
/// The gate is exactly zero outside the region and on its boundary. Along
/// each axis it rises with the quintic smootherstep polynomial over
/// `transition_width`, remains one across the interior plateau, and falls
/// symmetrically at the opposite boundary. Value, gradient, and Hessian are
/// therefore continuous through every face, edge, and corner.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct SmoothRegion<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    minimum: Point<D>,
    maximum: Point<D>,
    transition_width: f64,
    inverse_transition_width: f64,
    inverse_transition_width_squared: f64,
}

impl<const D: usize> SmoothRegion<D>
where
    Dim<D>: SupportedDimension,
{
    /// Constructs a finite nondegenerate region with a non-overlapping C2 taper.
    ///
    /// The transition width must be positive, have representable first and
    /// second derivative scales, and be no greater than half the extent of
    /// every region axis.
    ///
    /// # Errors
    ///
    /// Returns a structured extent, width, or derivative-scale error.
    pub fn try_new(
        minimum: Point<D>,
        maximum: Point<D>,
        transition_width: f64,
    ) -> Result<Self, LocalTrendConstructionError<D>> {
        if !transition_width.is_finite() {
            return Err(
                LocalTrendConstructionError::NonFiniteRegionTransitionWidth {
                    width: transition_width,
                },
            );
        }
        if transition_width <= 0.0 {
            return Err(
                LocalTrendConstructionError::NonPositiveRegionTransitionWidth {
                    width: transition_width,
                },
            );
        }
        for axis in 0..D {
            let minimum_value = minimum.components()[axis];
            let maximum_value = maximum.components()[axis];
            if minimum_value >= maximum_value {
                return Err(LocalTrendConstructionError::NonPositiveRegionExtent {
                    axis,
                    minimum: minimum_value,
                    maximum: maximum_value,
                });
            }
            let half_extent = 0.5 * maximum_value - 0.5 * minimum_value;
            if transition_width > half_extent {
                return Err(LocalTrendConstructionError::RegionTransitionTooWide {
                    axis,
                    width: transition_width,
                    maximum_width: half_extent,
                });
            }
        }
        let inverse_transition_width = transition_width.recip();
        let inverse_transition_width_squared = inverse_transition_width * inverse_transition_width;
        if !inverse_transition_width.is_finite()
            || inverse_transition_width == 0.0
            || !inverse_transition_width_squared.is_finite()
            || inverse_transition_width_squared == 0.0
            || !(60.0 * inverse_transition_width_squared).is_finite()
        {
            return Err(
                LocalTrendConstructionError::NonRepresentableRegionTransitionWidth {
                    width: transition_width,
                },
            );
        }
        Ok(Self {
            minimum,
            maximum,
            transition_width,
            inverse_transition_width,
            inverse_transition_width_squared,
        })
    }

    /// Returns the inclusive minimum corner.
    pub const fn minimum(self) -> Point<D> {
        self.minimum
    }

    /// Returns the inclusive maximum corner.
    pub const fn maximum(self) -> Point<D> {
        self.maximum
    }

    /// Returns the C2 transition width applied at every face.
    #[must_use]
    pub const fn transition_width(self) -> f64 {
        self.transition_width
    }

    /// Reports whether the point lies in the closed geometric region.
    #[must_use]
    pub fn contains(self, point: Point<D>) -> bool {
        (0..D).all(|axis| {
            point.components()[axis] >= self.minimum.components()[axis]
                && point.components()[axis] <= self.maximum.components()[axis]
        })
    }

    fn try_jet(
        self,
        point: Point<D>,
        demanded: KernelDerivativeOrder,
    ) -> Result<WeightJet<D>, LocalTrendEvaluationError<D>> {
        let mut factors = [AxisGate::default(); D];
        for (axis, factor) in factors.iter_mut().enumerate() {
            *factor = axis_region_gate(
                point.components()[axis],
                self.minimum.components()[axis],
                self.maximum.components()[axis],
                self.inverse_transition_width,
                self.inverse_transition_width_squared,
                demanded,
            );
        }

        let value = factors.iter().map(|factor| factor.value).product::<f64>();
        let gradient = if demanded >= KernelDerivativeOrder::First {
            std::array::from_fn(|axis| {
                factors[axis].first
                    * factors
                        .iter()
                        .enumerate()
                        .filter(|(other, _)| *other != axis)
                        .map(|(_, factor)| factor.value)
                        .product::<f64>()
            })
        } else {
            [0.0; D]
        };
        let hessian = if demanded >= KernelDerivativeOrder::Second {
            std::array::from_fn(|row| {
                std::array::from_fn(|column| {
                    if row == column {
                        factors[row].second
                            * factors
                                .iter()
                                .enumerate()
                                .filter(|(other, _)| *other != row)
                                .map(|(_, factor)| factor.value)
                                .product::<f64>()
                    } else {
                        factors[row].first
                            * factors[column].first
                            * factors
                                .iter()
                                .enumerate()
                                .filter(|(other, _)| *other != row && *other != column)
                                .map(|(_, factor)| factor.value)
                                .product::<f64>()
                    }
                })
            })
        } else {
            [[0.0; D]; D]
        };
        if !value.is_finite()
            || gradient.iter().any(|entry| !entry.is_finite())
            || hessian.iter().flatten().any(|entry| !entry.is_finite())
        {
            return Err(LocalTrendEvaluationError::NonFiniteWeightDerivative {
                component: usize::MAX,
                quantity: LocalTrendQuantity::Value,
            });
        }
        Ok(WeightJet {
            value,
            gradient,
            hessian,
        })
    }
}

impl<const D: usize> OperationalDomain<D>
where
    Dim<D>: SupportedDimension,
{
    /// Constructs a domain whose minimum does not exceed its maximum on any axis.
    ///
    /// # Errors
    ///
    /// Returns [`LocalTrendConstructionError::ReversedDomainAxis`] for the
    /// first reversed interval.
    pub fn try_new(
        minimum: Point<D>,
        maximum: Point<D>,
    ) -> Result<Self, LocalTrendConstructionError<D>> {
        for axis in 0..D {
            if minimum.components()[axis] > maximum.components()[axis] {
                return Err(LocalTrendConstructionError::ReversedDomainAxis {
                    axis,
                    minimum: minimum.components()[axis],
                    maximum: maximum.components()[axis],
                });
            }
        }
        Ok(Self { minimum, maximum })
    }

    /// Returns the inclusive minimum corner.
    pub const fn minimum(self) -> Point<D> {
        self.minimum
    }

    /// Returns the inclusive maximum corner.
    pub const fn maximum(self) -> Point<D> {
        self.maximum
    }

    /// Reports whether a finite point lies in the closed domain.
    #[must_use]
    pub fn contains(self, point: Point<D>) -> bool {
        (0..D).all(|axis| {
            point.components()[axis] >= self.minimum.components()[axis]
                && point.components()[axis] <= self.maximum.components()[axis]
        })
    }
}

/// A concrete smooth scalar basis used on both arguments of one mixture component.
///
/// Both supported forms are analytic through Hessian order. Gaussian weights may
/// underflow to exact zero far from their centers and therefore cannot serve
/// as the strict background; they remain valid positive-semidefinite diagonal
/// congruence factors for non-background components.
///
/// The representation is intentionally private so every instance preserves
/// the validation and cached-parameter invariants established by
/// [`Self::try_constant`] and [`Self::try_gaussian`].
///
/// ```compile_fail
/// use georbf::SmoothSpatialWeight;
///
/// let _ = SmoothSpatialWeight::<1>::Constant { value: 1.0 };
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct SmoothSpatialWeight<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    kind: SmoothSpatialWeightKind<D>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum SmoothSpatialWeightKind<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    /// A spatially constant finite weight.
    Constant {
        /// Signed constant value.
        value: f64,
    },
    /// An isotropic Gaussian basis `amplitude * exp(-||x-c||^2/(2 radius^2))`.
    Gaussian {
        /// Finite center.
        center: Point<D>,
        /// Signed finite amplitude.
        amplitude: f64,
        /// Positive finite radius.
        radius: f64,
        /// Cached finite reciprocal radius.
        inverse_radius: f64,
        /// Cached finite squared reciprocal radius.
        inverse_radius_squared: f64,
    },
    /// A Gaussian basis multiplied by one compact C2 region gate.
    RegionalGaussian {
        center: Point<D>,
        amplitude: f64,
        radius: f64,
        inverse_radius: f64,
        inverse_radius_squared: f64,
        region: SmoothRegion<D>,
    },
}

impl<const D: usize> SmoothSpatialWeight<D>
where
    Dim<D>: SupportedDimension,
{
    /// Constructs a finite constant weight.
    ///
    /// Zero is permitted for non-background components. The mixture
    /// constructor applies the stronger strict-background invariant.
    ///
    /// # Errors
    ///
    /// Returns a structured error when the value is non-finite or the square
    /// of a nonzero value is zero or non-finite.
    pub fn try_constant(value: f64) -> Result<Self, LocalTrendConstructionError<D>> {
        validate_amplitude(value)?;
        Ok(Self {
            kind: SmoothSpatialWeightKind::Constant { value },
        })
    }

    /// Constructs an analytic Gaussian spatial weight.
    ///
    /// # Errors
    ///
    /// Returns a structured error for a non-finite amplitude, a nonpositive
    /// radius, or a radius whose inverse derivatives are not representable.
    pub fn try_gaussian(
        center: Point<D>,
        amplitude: f64,
        radius: f64,
    ) -> Result<Self, LocalTrendConstructionError<D>> {
        validate_amplitude(amplitude)?;
        let (inverse_radius, inverse_radius_squared) = validate_radius(radius)?;
        Ok(Self {
            kind: SmoothSpatialWeightKind::Gaussian {
                center,
                amplitude,
                radius,
                inverse_radius,
                inverse_radius_squared,
            },
        })
    }

    /// Constructs a Gaussian influence multiplied by a compact C2 region gate.
    ///
    /// # Errors
    ///
    /// Returns a structured amplitude or radius representation error.
    pub fn try_regional_gaussian(
        center: Point<D>,
        amplitude: f64,
        radius: f64,
        region: SmoothRegion<D>,
    ) -> Result<Self, LocalTrendConstructionError<D>> {
        validate_amplitude(amplitude)?;
        let (inverse_radius, inverse_radius_squared) = validate_radius(radius)?;
        Ok(Self {
            kind: SmoothSpatialWeightKind::RegionalGaussian {
                center,
                amplitude,
                radius,
                inverse_radius,
                inverse_radius_squared,
                region,
            },
        })
    }

    /// Returns the constant value, or `None` for a spatially varying basis.
    #[must_use]
    pub const fn constant_value(self) -> Option<f64> {
        match self.kind {
            SmoothSpatialWeightKind::Constant { value } => Some(value),
            SmoothSpatialWeightKind::Gaussian { .. }
            | SmoothSpatialWeightKind::RegionalGaussian { .. } => None,
        }
    }

    /// Returns the Gaussian center, amplitude, and radius when applicable.
    #[must_use]
    pub const fn gaussian_parameters(self) -> Option<(Point<D>, f64, f64)> {
        match self.kind {
            SmoothSpatialWeightKind::Constant { .. } => None,
            SmoothSpatialWeightKind::Gaussian {
                center,
                amplitude,
                radius,
                ..
            }
            | SmoothSpatialWeightKind::RegionalGaussian {
                center,
                amplitude,
                radius,
                ..
            } => Some((center, amplitude, radius)),
        }
    }

    /// Returns the compact region for a regional Gaussian basis.
    #[must_use]
    pub const fn region(self) -> Option<SmoothRegion<D>> {
        match self.kind {
            SmoothSpatialWeightKind::RegionalGaussian { region, .. } => Some(region),
            SmoothSpatialWeightKind::Constant { .. } | SmoothSpatialWeightKind::Gaussian { .. } => {
                None
            }
        }
    }

    fn try_jet(
        self,
        point: Point<D>,
        demanded: KernelDerivativeOrder,
    ) -> Result<WeightJet<D>, LocalTrendEvaluationError<D>> {
        match self.kind {
            SmoothSpatialWeightKind::Constant { value } => Ok(WeightJet {
                value,
                gradient: [0.0; D],
                hessian: [[0.0; D]; D],
            }),
            SmoothSpatialWeightKind::Gaussian {
                center,
                amplitude,
                radius,
                inverse_radius,
                inverse_radius_squared,
            } => gaussian_weight_jet(
                point,
                center,
                amplitude,
                radius,
                inverse_radius,
                inverse_radius_squared,
                demanded,
            ),
            SmoothSpatialWeightKind::RegionalGaussian {
                center,
                amplitude,
                radius,
                inverse_radius,
                inverse_radius_squared,
                region,
            } => regional_gaussian_weight_jet(
                point,
                center,
                amplitude,
                radius,
                inverse_radius,
                inverse_radius_squared,
                region,
                demanded,
            ),
        }
    }
}

/// One fixed strictly-positive-definite anisotropic kernel and its smooth basis.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct LocalTrendComponent<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    kernel: KernelDefinition<D>,
    anisotropy: GlobalAnisotropy<D>,
    weight: SmoothSpatialWeight<D>,
}

impl<const D: usize> LocalTrendComponent<D>
where
    Dim<D>: SupportedDimension,
{
    /// Retains one configured kernel, fixed anisotropy, and smooth spatial weight.
    ///
    /// Positive-definiteness classification is checked by
    /// [`LocalTrendMixture::try_new`] so the error can identify the component index.
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

    /// Returns the configured fixed kernel.
    pub const fn kernel(self) -> KernelDefinition<D> {
        self.kernel
    }

    /// Borrows the fixed anisotropy.
    pub const fn anisotropy(&self) -> &GlobalAnisotropy<D> {
        &self.anisotropy
    }

    /// Returns the smooth spatial weight.
    pub const fn weight(self) -> SmoothSpatialWeight<D> {
        self.weight
    }
}

/// Immutable construction diagnostics for a validated local mixture.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct LocalTrendDiagnostics {
    component_count: usize,
    background_index: usize,
    background_weight_magnitude: f64,
    minimum_background_weight: f64,
    background_policy_ratio: f64,
    maximum_anisotropy_condition_number: f64,
}

impl LocalTrendDiagnostics {
    /// Returns the finite component count.
    #[must_use]
    pub const fn component_count(self) -> usize {
        self.component_count
    }

    /// Returns the explicitly identified background component index.
    #[must_use]
    pub const fn background_index(self) -> usize {
        self.background_index
    }

    /// Returns the everywhere-constant absolute background weight.
    #[must_use]
    pub const fn background_weight_magnitude(self) -> f64 {
        self.background_weight_magnitude
    }

    /// Returns the caller's positive conditioning-policy lower bound.
    #[must_use]
    pub const fn minimum_background_weight(self) -> f64 {
        self.minimum_background_weight
    }

    /// Returns `background_weight_magnitude / minimum_background_weight`.
    #[must_use]
    pub const fn background_policy_ratio(self) -> f64 {
        self.background_policy_ratio
    }

    /// Returns the largest fixed-metric Euclidean condition number.
    #[must_use]
    pub const fn maximum_anisotropy_condition_number(self) -> f64 {
        self.maximum_anisotropy_condition_number
    }
}

/// Pointwise aggregate coverage diagnostics for the spatial bases.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct LocalTrendCoverage {
    inside_operational_domain: bool,
    active_components: usize,
    squared_weight_sum: f64,
    background_squared_weight: f64,
}

impl LocalTrendCoverage {
    /// Reports whether the inspected point lies in the declared domain.
    #[must_use]
    pub const fn inside_operational_domain(self) -> bool {
        self.inside_operational_domain
    }

    /// Returns the number of components with a nonzero represented weight.
    #[must_use]
    pub const fn active_components(self) -> usize {
        self.active_components
    }

    /// Returns `sum_r b_r(x)^2`, the diagonal-congruence coverage measure.
    #[must_use]
    pub const fn squared_weight_sum(self) -> f64 {
        self.squared_weight_sum
    }

    /// Returns the strictly positive background contribution `b_bg(x)^2`.
    #[must_use]
    pub const fn background_squared_weight(self) -> f64 {
        self.background_squared_weight
    }
}

/// Demand-bounded value, query gradient, and query Hessian of a local mixture.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct LocalTrendEvaluation<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    available_through: KernelDerivativeOrder,
    value: f64,
    gradient: [f64; D],
    hessian: [[f64; D]; D],
}

impl<const D: usize> LocalTrendEvaluation<D>
where
    Dim<D>: SupportedDimension,
{
    /// Returns the highest derivative order carried by this evaluation.
    #[must_use]
    pub const fn available_through(self) -> KernelDerivativeOrder {
        self.available_through
    }

    /// Returns the mixture value.
    #[must_use]
    pub const fn value(self) -> f64 {
        self.value
    }

    /// Returns the query gradient when demanded.
    #[must_use]
    pub const fn gradient(self) -> Option<[f64; D]> {
        if matches!(
            self.available_through,
            KernelDerivativeOrder::First
                | KernelDerivativeOrder::Second
                | KernelDerivativeOrder::Third
        ) {
            Some(self.gradient)
        } else {
            None
        }
    }

    /// Returns the query Hessian when demanded.
    #[must_use]
    pub const fn hessian(self) -> Option<[[f64; D]; D]> {
        if matches!(
            self.available_through,
            KernelDerivativeOrder::Second | KernelDerivativeOrder::Third
        ) {
            Some(self.hessian)
        } else {
            None
        }
    }
}

/// A finite positive-definite mixture with one strict constant background.
#[derive(Debug)]
#[must_use]
pub struct LocalTrendMixture<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    components: Vec<LocalTrendComponent<D>>,
    background_index: usize,
    operational_domain: OperationalDomain<D>,
    diagnostics: LocalTrendDiagnostics,
}

impl<const D: usize> LocalTrendMixture<D>
where
    Dim<D>: SupportedDimension,
{
    /// Validates a complete local mixture.
    ///
    /// The background must use a constant finite nonzero weight. Because its
    /// kernel is strictly positive definite and the constant diagonal factor
    /// is invertible, every distinct finite point set has a strictly positive
    /// Gram matrix. Other components add positive-semidefinite congruence terms.
    ///
    /// # Errors
    ///
    /// Rejects an empty mixture, an invalid background index or lower bound,
    /// any CPD component, a nonconstant/zero/too-small background, or a
    /// non-representable policy diagnostic.
    pub fn try_new(
        components: Vec<LocalTrendComponent<D>>,
        background_index: usize,
        operational_domain: OperationalDomain<D>,
        minimum_background_weight: f64,
    ) -> Result<Self, LocalTrendConstructionError<D>> {
        if components.is_empty() {
            return Err(LocalTrendConstructionError::EmptyMixture);
        }
        if background_index >= components.len() {
            return Err(LocalTrendConstructionError::BackgroundIndexOutOfBounds {
                index: background_index,
                component_count: components.len(),
            });
        }
        if !minimum_background_weight.is_finite() {
            return Err(LocalTrendConstructionError::NonFiniteBackgroundMinimum {
                minimum: minimum_background_weight,
            });
        }
        if minimum_background_weight <= 0.0 {
            return Err(LocalTrendConstructionError::NonPositiveBackgroundMinimum {
                minimum: minimum_background_weight,
            });
        }

        let mut maximum_condition = 1.0_f64;
        for (component, entry) in components.iter().enumerate() {
            if let KernelDefiniteness::ConditionallyPositiveDefinite { order } =
                entry.kernel.metadata().definiteness()
            {
                return Err(
                    LocalTrendConstructionError::ConditionallyPositiveDefiniteComponent {
                        component,
                        order: order.get(),
                    },
                );
            }
            maximum_condition =
                maximum_condition.max(entry.anisotropy.diagnostics().condition_number());
        }

        let background_weight = components[background_index].weight.constant_value().ok_or(
            LocalTrendConstructionError::NonConstantBackground {
                component: background_index,
            },
        )?;
        let background_weight_magnitude = background_weight.abs();
        if background_weight_magnitude == 0.0 {
            return Err(LocalTrendConstructionError::ZeroBackgroundWeight {
                component: background_index,
            });
        }
        if background_weight_magnitude < minimum_background_weight {
            return Err(LocalTrendConstructionError::BackgroundBelowMinimum {
                component: background_index,
                magnitude: background_weight_magnitude,
                minimum: minimum_background_weight,
            });
        }
        let background_policy_ratio = background_weight_magnitude / minimum_background_weight;
        if !background_policy_ratio.is_finite() {
            return Err(LocalTrendConstructionError::NonRepresentableBackgroundPolicyRatio);
        }
        let diagnostics = LocalTrendDiagnostics {
            component_count: components.len(),
            background_index,
            background_weight_magnitude,
            minimum_background_weight,
            background_policy_ratio,
            maximum_anisotropy_condition_number: maximum_condition,
        };
        Ok(Self {
            components,
            background_index,
            operational_domain,
            diagnostics,
        })
    }

    /// Borrows components in deterministic caller order.
    pub fn components(&self) -> &[LocalTrendComponent<D>] {
        &self.components
    }

    /// Returns the background component index.
    #[must_use]
    pub const fn background_index(&self) -> usize {
        self.background_index
    }

    /// Returns the declared operational domain.
    pub const fn operational_domain(&self) -> OperationalDomain<D> {
        self.operational_domain
    }

    /// Returns immutable construction diagnostics.
    pub const fn diagnostics(&self) -> LocalTrendDiagnostics {
        self.diagnostics
    }

    /// Classifies aggregate support for value, query gradient, or query Hessian.
    ///
    /// Third derivatives are not part of this requirement and are reported as unsupported.
    #[must_use]
    pub fn derivative_capability(
        &self,
        order: KernelDerivativeOrder,
    ) -> KernelDerivativeCapability {
        if order == KernelDerivativeOrder::Third {
            return KernelDerivativeCapability::Unsupported;
        }
        let mut aggregate = KernelDerivativeCapability::SupportedEverywhere;
        for component in &self.components {
            match component.kernel.metadata().derivatives().capability(order) {
                KernelDerivativeCapability::Unsupported => {
                    return KernelDerivativeCapability::Unsupported;
                }
                KernelDerivativeCapability::SupportedAwayFromCenters => {
                    aggregate = KernelDerivativeCapability::SupportedAwayFromCenters;
                }
                KernelDerivativeCapability::SupportedEverywhere => {}
            }
        }
        aggregate
    }

    /// Computes allocation-free pointwise weight coverage.
    ///
    /// # Errors
    ///
    /// Returns a structured evaluation error if a weight or the accumulated
    /// squared coverage is not finitely representable.
    pub fn try_coverage(
        &self,
        point: Point<D>,
    ) -> Result<LocalTrendCoverage, LocalTrendEvaluationError<D>> {
        let mut active_components = 0_usize;
        let mut squared_weight_sum = 0.0;
        let mut background_squared_weight = 0.0;
        for (index, component) in self.components.iter().enumerate() {
            let value = component
                .weight
                .try_jet(point, KernelDerivativeOrder::Value)
                .map_err(|source| source.with_component(index))?
                .value;
            if value != 0.0 {
                active_components += 1;
            }
            let squared = finite_product(value, value)
                .ok_or(LocalTrendEvaluationError::NonFiniteCoverage { component: index })?;
            squared_weight_sum = finite_sum(squared_weight_sum, squared)
                .ok_or(LocalTrendEvaluationError::NonFiniteCoverage { component: index })?;
            if index == self.background_index {
                background_squared_weight = squared;
            }
        }
        Ok(LocalTrendCoverage {
            inside_operational_domain: self.operational_domain.contains(point),
            active_components,
            squared_weight_sum,
            background_squared_weight,
        })
    }

    /// Evaluates the mixture through the demanded query derivative order.
    ///
    /// The gradient and Hessian include every product-rule term from `b_r(x)`:
    /// `b_y (grad(b_x) k + b_x grad(k))` and
    /// `b_y (H(b_x) k + grad(b_x) grad(k)^T + grad(k) grad(b_x)^T + b_x H(k))`.
    ///
    /// # Errors
    ///
    /// Rejects third-order demand, unavailable center capability, a component
    /// kernel failure, or a non-finite represented contribution/sum.
    pub fn try_evaluate(
        &self,
        query: Point<D>,
        center: Point<D>,
        demanded: KernelDerivativeOrder,
    ) -> Result<LocalTrendEvaluation<D>, LocalTrendEvaluationError<D>> {
        if demanded == KernelDerivativeOrder::Third {
            return Err(LocalTrendEvaluationError::UnsupportedDerivative { demanded });
        }
        let at_center = query == center;
        let capability = self.derivative_capability(demanded);
        if capability == KernelDerivativeCapability::Unsupported
            || (at_center && capability == KernelDerivativeCapability::SupportedAwayFromCenters)
        {
            return Err(LocalTrendEvaluationError::UnavailableDerivative {
                demanded,
                at_center,
            });
        }

        let mut value = 0.0;
        let mut gradient = [0.0; D];
        let mut hessian = [[0.0; D]; D];
        for (component_index, component) in self.components.iter().enumerate() {
            let query_weight = component
                .weight
                .try_jet(query, demanded)
                .map_err(|source| source.with_component(component_index))?;
            let center_weight = component
                .weight
                .try_jet(center, KernelDerivativeOrder::Value)
                .map_err(|source| source.with_component(component_index))?
                .value;
            let kernel = component
                .kernel
                .try_spatial_jet(query, center, demanded, Some(&component.anisotropy))
                .map_err(|source| LocalTrendEvaluationError::Kernel {
                    component: component_index,
                    source,
                })?;

            let value_term = checked_terms(
                &[query_weight.value, center_weight, kernel.value()],
                component_index,
                LocalTrendQuantity::Value,
            )?;
            value = checked_add(
                value,
                value_term,
                component_index,
                LocalTrendQuantity::Value,
            )?;

            if demanded >= KernelDerivativeOrder::First {
                let kernel_gradient = kernel.first_derivative(KernelArgument::Query);
                for axis in 0..D {
                    let weight_term = checked_terms(
                        &[center_weight, query_weight.gradient[axis], kernel.value()],
                        component_index,
                        LocalTrendQuantity::Gradient { axis },
                    )?;
                    let kernel_term = checked_terms(
                        &[center_weight, query_weight.value, kernel_gradient[axis]],
                        component_index,
                        LocalTrendQuantity::Gradient { axis },
                    )?;
                    let contribution = checked_add(
                        weight_term,
                        kernel_term,
                        component_index,
                        LocalTrendQuantity::Gradient { axis },
                    )?;
                    gradient[axis] = checked_add(
                        gradient[axis],
                        contribution,
                        component_index,
                        LocalTrendQuantity::Gradient { axis },
                    )?;
                }

                if demanded >= KernelDerivativeOrder::Second {
                    let kernel_hessian =
                        kernel.second_derivative([KernelArgument::Query, KernelArgument::Query]);
                    accumulate_component_hessian(
                        &mut hessian,
                        query_weight,
                        center_weight,
                        kernel.value(),
                        kernel_gradient,
                        kernel_hessian,
                        component_index,
                    )?;
                }
            }
        }
        Ok(LocalTrendEvaluation {
            available_through: demanded,
            value,
            gradient,
            hessian,
        })
    }
}

/// Construction failure for a local positive-definite mixture.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LocalTrendConstructionError<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    /// An operational-domain axis is reversed.
    ReversedDomainAxis {
        /// Zero-based axis.
        axis: usize,
        /// Rejected minimum.
        minimum: f64,
        /// Rejected maximum.
        maximum: f64,
    },
    /// A compact region axis has no positive interior extent.
    NonPositiveRegionExtent {
        /// Zero-based axis.
        axis: usize,
        /// Rejected minimum.
        minimum: f64,
        /// Rejected maximum.
        maximum: f64,
    },
    /// A compact-region transition width is not finite.
    NonFiniteRegionTransitionWidth {
        /// Rejected width.
        width: f64,
    },
    /// A compact-region transition width is not positive.
    NonPositiveRegionTransitionWidth {
        /// Rejected width.
        width: f64,
    },
    /// A transition leaves no untapered interior along one region axis.
    RegionTransitionTooWide {
        /// Zero-based axis.
        axis: usize,
        /// Rejected width.
        width: f64,
        /// Half of the axis extent.
        maximum_width: f64,
    },
    /// Region-gate first or second derivative scales are not representable.
    NonRepresentableRegionTransitionWidth {
        /// Rejected width.
        width: f64,
    },
    /// A weight amplitude is not finite.
    NonFiniteWeightAmplitude {
        /// Rejected amplitude.
        amplitude: f64,
    },
    /// Squaring a finite nonzero amplitude produces zero or a non-finite value.
    NonRepresentableWeightAmplitudeSquare {
        /// Rejected amplitude.
        amplitude: f64,
    },
    /// A Gaussian weight radius is not finite.
    NonFiniteWeightRadius {
        /// Rejected radius.
        radius: f64,
    },
    /// A Gaussian weight radius is not positive.
    NonPositiveWeightRadius {
        /// Rejected radius.
        radius: f64,
    },
    /// Gaussian inverse-radius derivatives are not representable.
    NonRepresentableWeightRadius {
        /// Rejected radius.
        radius: f64,
    },
    /// No component was supplied.
    EmptyMixture,
    /// The background index does not select a component.
    BackgroundIndexOutOfBounds {
        /// Rejected index.
        index: usize,
        /// Available component count.
        component_count: usize,
    },
    /// A component kernel is CPD rather than SPD.
    ConditionallyPositiveDefiniteComponent {
        /// Component index.
        component: usize,
        /// Positive CPD order.
        order: usize,
    },
    /// The background policy lower bound is not finite.
    NonFiniteBackgroundMinimum {
        /// Rejected lower bound.
        minimum: f64,
    },
    /// The background policy lower bound is not positive.
    NonPositiveBackgroundMinimum {
        /// Rejected lower bound.
        minimum: f64,
    },
    /// The selected background is not constant everywhere.
    NonConstantBackground {
        /// Component index.
        component: usize,
    },
    /// The selected constant background vanishes everywhere.
    ZeroBackgroundWeight {
        /// Component index.
        component: usize,
    },
    /// The strict background fails explicit conditioning policy.
    BackgroundBelowMinimum {
        /// Component index.
        component: usize,
        /// Absolute background weight.
        magnitude: f64,
        /// Required minimum.
        minimum: f64,
    },
    /// The policy margin ratio is not finitely representable.
    NonRepresentableBackgroundPolicyRatio,
}

impl<const D: usize> fmt::Display for LocalTrendConstructionError<D>
where
    Dim<D>: SupportedDimension,
{
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReversedDomainAxis {
                axis,
                minimum,
                maximum,
            } => write!(
                formatter,
                "operational-domain axis {axis} has minimum {minimum} greater than maximum {maximum}"
            ),
            Self::NonPositiveRegionExtent {
                axis,
                minimum,
                maximum,
            } => write!(
                formatter,
                "control-region axis {axis} must have positive extent, got [{minimum}, {maximum}]"
            ),
            Self::NonFiniteRegionTransitionWidth { width } => write!(
                formatter,
                "control-region transition width must be finite, got {width}"
            ),
            Self::NonPositiveRegionTransitionWidth { width } => write!(
                formatter,
                "control-region transition width must be positive, got {width}"
            ),
            Self::RegionTransitionTooWide {
                axis,
                width,
                maximum_width,
            } => write!(
                formatter,
                "control-region transition width {width} exceeds half-extent {maximum_width} on axis {axis}"
            ),
            Self::NonRepresentableRegionTransitionWidth { width } => write!(
                formatter,
                "control-region transition width {width} has non-representable derivative scales"
            ),
            Self::NonFiniteWeightAmplitude { amplitude } => {
                write!(
                    formatter,
                    "spatial-weight amplitude must be finite, got {amplitude}"
                )
            }
            Self::NonRepresentableWeightAmplitudeSquare { amplitude } => write!(
                formatter,
                "spatial-weight amplitude {amplitude} has a zero or non-finite represented square"
            ),
            Self::NonFiniteWeightRadius { radius } => {
                write!(
                    formatter,
                    "spatial-weight radius must be finite, got {radius}"
                )
            }
            Self::NonPositiveWeightRadius { radius } => {
                write!(
                    formatter,
                    "spatial-weight radius must be positive, got {radius}"
                )
            }
            Self::NonRepresentableWeightRadius { radius } => write!(
                formatter,
                "spatial-weight radius {radius} has non-representable inverse derivatives"
            ),
            Self::EmptyMixture => formatter.write_str("local trend mixture requires a component"),
            Self::BackgroundIndexOutOfBounds {
                index,
                component_count,
            } => write!(
                formatter,
                "background index {index} is outside {component_count} components"
            ),
            Self::ConditionallyPositiveDefiniteComponent { component, order } => write!(
                formatter,
                "local trend component {component} is conditionally positive definite of order {order}"
            ),
            Self::NonFiniteBackgroundMinimum { minimum } => write!(
                formatter,
                "minimum background weight must be finite, got {minimum}"
            ),
            Self::NonPositiveBackgroundMinimum { minimum } => write!(
                formatter,
                "minimum background weight must be positive, got {minimum}"
            ),
            Self::NonConstantBackground { component } => write!(
                formatter,
                "background component {component} must have a constant everywhere-nonzero weight"
            ),
            Self::ZeroBackgroundWeight { component } => {
                write!(
                    formatter,
                    "background component {component} has zero weight"
                )
            }
            Self::BackgroundBelowMinimum {
                component,
                magnitude,
                minimum,
            } => write!(
                formatter,
                "background component {component} weight magnitude {magnitude} is below policy minimum {minimum}"
            ),
            Self::NonRepresentableBackgroundPolicyRatio => formatter
                .write_str("background weight divided by its policy minimum is not representable"),
        }
    }
}

impl<const D: usize> Error for LocalTrendConstructionError<D> where Dim<D>: SupportedDimension {}

/// Output component whose represented arithmetic failed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LocalTrendQuantity {
    /// Mixture value.
    Value,
    /// Query-gradient component.
    Gradient {
        /// Axis.
        axis: usize,
    },
    /// Query-Hessian component.
    Hessian {
        /// Row.
        row: usize,
        /// Column.
        column: usize,
    },
}

/// Evaluation failure for a local positive-definite mixture.
#[derive(Clone, Debug, PartialEq)]
pub enum LocalTrendEvaluationError<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    /// The local-mixture API does not supply this derivative order.
    UnsupportedDerivative {
        /// Rejected demand.
        demanded: KernelDerivativeOrder,
    },
    /// Aggregate metadata does not permit this location/demand pair.
    UnavailableDerivative {
        /// Rejected demand.
        demanded: KernelDerivativeOrder,
        /// Whether query and center coincide.
        at_center: bool,
    },
    /// A Gaussian-weight displacement is not representable.
    NonFiniteWeightDisplacement {
        /// Component index, or `usize::MAX` before component context is attached.
        component: usize,
        /// Axis.
        axis: usize,
    },
    /// A Gaussian weight derivative is not representable.
    NonFiniteWeightDerivative {
        /// Component index, or `usize::MAX` before component context is attached.
        component: usize,
        /// Derivative quantity.
        quantity: LocalTrendQuantity,
    },
    /// A fixed kernel/anisotropy evaluation failed.
    Kernel {
        /// Component index.
        component: usize,
        /// Underlying evaluation error.
        source: KernelDefinitionEvaluationError<D>,
    },
    /// A represented product-rule contribution or accumulation is not finite.
    NonFiniteContribution {
        /// Component index.
        component: usize,
        /// Output quantity.
        quantity: LocalTrendQuantity,
    },
    /// Pointwise squared-weight coverage is not finite.
    NonFiniteCoverage {
        /// Component index.
        component: usize,
    },
}

impl<const D: usize> LocalTrendEvaluationError<D>
where
    Dim<D>: SupportedDimension,
{
    fn with_component(self, component: usize) -> Self {
        match self {
            Self::NonFiniteWeightDisplacement { axis, .. } => {
                Self::NonFiniteWeightDisplacement { component, axis }
            }
            Self::NonFiniteWeightDerivative { quantity, .. } => Self::NonFiniteWeightDerivative {
                component,
                quantity,
            },
            other => other,
        }
    }
}

impl<const D: usize> fmt::Display for LocalTrendEvaluationError<D>
where
    Dim<D>: SupportedDimension,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedDerivative { demanded } => {
                write!(
                    formatter,
                    "local trend does not supply {demanded:?} derivatives"
                )
            }
            Self::UnavailableDerivative {
                demanded,
                at_center,
            } => write!(
                formatter,
                "local trend {demanded:?} derivative is unavailable at_center={at_center}"
            ),
            Self::NonFiniteWeightDisplacement { component, axis } => write!(
                formatter,
                "local trend component {component} weight displacement is not finite on axis {axis}"
            ),
            Self::NonFiniteWeightDerivative {
                component,
                quantity,
            } => write!(
                formatter,
                "local trend component {component} weight derivative {quantity:?} is not finite"
            ),
            Self::Kernel { component, source } => {
                write!(
                    formatter,
                    "local trend component {component} kernel failed: {source}"
                )
            }
            Self::NonFiniteContribution {
                component,
                quantity,
            } => write!(
                formatter,
                "local trend component {component} contribution {quantity:?} is not finite"
            ),
            Self::NonFiniteCoverage { component } => write!(
                formatter,
                "local trend squared-weight coverage is not finite at component {component}"
            ),
        }
    }
}

impl<const D: usize> Error for LocalTrendEvaluationError<D>
where
    Dim<D>: SupportedDimension,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Kernel { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
struct WeightJet<const D: usize> {
    value: f64,
    gradient: [f64; D],
    hessian: [[f64; D]; D],
}

#[derive(Clone, Copy, Default)]
struct AxisGate {
    value: f64,
    first: f64,
    second: f64,
}

fn axis_region_gate(
    coordinate: f64,
    minimum: f64,
    maximum: f64,
    inverse_width: f64,
    inverse_width_squared: f64,
    demanded: KernelDerivativeOrder,
) -> AxisGate {
    if coordinate <= minimum || coordinate >= maximum {
        return AxisGate::default();
    }
    let left = scaled_smootherstep_jet(
        (coordinate - minimum) * inverse_width,
        inverse_width,
        inverse_width_squared,
    );
    let right = scaled_smootherstep_jet(
        (maximum - coordinate) * inverse_width,
        inverse_width,
        inverse_width_squared,
    );
    let value = left.value * right.value;
    let first = if demanded >= KernelDerivativeOrder::First {
        left.first * right.value - left.value * right.first
    } else {
        0.0
    };
    let second = if demanded >= KernelDerivativeOrder::Second {
        left.second * right.value - 2.0 * left.first * right.first + left.value * right.second
    } else {
        0.0
    };
    AxisGate {
        value,
        first,
        second,
    }
}

fn scaled_smootherstep_jet(
    parameter: f64,
    inverse_width: f64,
    inverse_width_squared: f64,
) -> AxisGate {
    if parameter <= 0.0 {
        return AxisGate::default();
    }
    if parameter >= 1.0 {
        return AxisGate {
            value: 1.0,
            first: 0.0,
            second: 0.0,
        };
    }
    let complement = parameter - 1.0;
    AxisGate {
        value: parameter * parameter * parameter * (parameter * (6.0 * parameter - 15.0) + 10.0),
        first: 30.0 * (parameter * inverse_width) * parameter * complement * complement,
        second: 60.0 * parameter * complement * (2.0 * parameter - 1.0) * inverse_width_squared,
    }
}

fn validate_amplitude<const D: usize>(amplitude: f64) -> Result<(), LocalTrendConstructionError<D>>
where
    Dim<D>: SupportedDimension,
{
    if !amplitude.is_finite() {
        return Err(LocalTrendConstructionError::NonFiniteWeightAmplitude { amplitude });
    }
    let squared = amplitude * amplitude;
    if !squared.is_finite() || (amplitude != 0.0 && squared == 0.0) {
        return Err(
            LocalTrendConstructionError::NonRepresentableWeightAmplitudeSquare { amplitude },
        );
    }
    Ok(())
}

fn validate_radius<const D: usize>(
    radius: f64,
) -> Result<(f64, f64), LocalTrendConstructionError<D>>
where
    Dim<D>: SupportedDimension,
{
    if !radius.is_finite() {
        return Err(LocalTrendConstructionError::NonFiniteWeightRadius { radius });
    }
    if radius <= 0.0 {
        return Err(LocalTrendConstructionError::NonPositiveWeightRadius { radius });
    }
    let inverse_radius = radius.recip();
    let inverse_radius_squared = inverse_radius * inverse_radius;
    if !inverse_radius.is_finite()
        || inverse_radius == 0.0
        || !inverse_radius_squared.is_finite()
        || inverse_radius_squared == 0.0
    {
        return Err(LocalTrendConstructionError::NonRepresentableWeightRadius { radius });
    }
    Ok((inverse_radius, inverse_radius_squared))
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
fn regional_gaussian_weight_jet<const D: usize>(
    point: Point<D>,
    center: Point<D>,
    amplitude: f64,
    radius: f64,
    inverse_radius: f64,
    inverse_radius_squared: f64,
    region: SmoothRegion<D>,
    demanded: KernelDerivativeOrder,
) -> Result<WeightJet<D>, LocalTrendEvaluationError<D>>
where
    Dim<D>: SupportedDimension,
{
    let gate = region.try_jet(point, demanded)?;
    if gate.value == 0.0
        && gate.gradient.iter().all(|entry| *entry == 0.0)
        && gate.hessian.iter().flatten().all(|entry| *entry == 0.0)
    {
        return Ok(WeightJet {
            value: 0.0,
            gradient: [0.0; D],
            hessian: [[0.0; D]; D],
        });
    }
    let Some(gaussian) = gaussian_weight_state(point, center, amplitude, inverse_radius)? else {
        return Ok(WeightJet {
            value: 0.0,
            gradient: [0.0; D],
            hessian: [[0.0; D]; D],
        });
    };
    let value =
        stable_gaussian_product(gaussian.value, amplitude, gaussian.exponent, &[gate.value])
            .ok_or(LocalTrendEvaluationError::NonFiniteWeightDerivative {
                component: usize::MAX,
                quantity: LocalTrendQuantity::Value,
            })?;
    let mut gradient = [0.0; D];
    let mut hessian = [[0.0; D]; D];
    if demanded >= KernelDerivativeOrder::First {
        for (axis, output) in gradient.iter_mut().enumerate() {
            let first = stable_gaussian_product(
                gaussian.value,
                amplitude,
                gaussian.exponent,
                &[
                    -inverse_radius_squared,
                    gaussian.displacements[axis],
                    gate.value,
                ],
            )
            .ok_or(LocalTrendEvaluationError::NonFiniteWeightDerivative {
                component: usize::MAX,
                quantity: LocalTrendQuantity::Gradient { axis },
            })?;
            let second = stable_gaussian_product(
                gaussian.value,
                amplitude,
                gaussian.exponent,
                &[gate.gradient[axis]],
            )
            .ok_or(LocalTrendEvaluationError::NonFiniteWeightDerivative {
                component: usize::MAX,
                quantity: LocalTrendQuantity::Gradient { axis },
            })?;
            *output = finite_sum(first, second).ok_or(
                LocalTrendEvaluationError::NonFiniteWeightDerivative {
                    component: usize::MAX,
                    quantity: LocalTrendQuantity::Gradient { axis },
                },
            )?;
        }
    }
    if demanded >= KernelDerivativeOrder::Second {
        for (row, hessian_row) in hessian.iter_mut().enumerate() {
            for (column, output) in hessian_row.iter_mut().enumerate() {
                let quantity = LocalTrendQuantity::Hessian { row, column };
                let gaussian_hessian = if row == column {
                    stable_gaussian_product(
                        gaussian.value,
                        amplitude,
                        gaussian.exponent,
                        &[
                            inverse_radius_squared,
                            inverse_radius_squared,
                            gaussian.displacements[row] - radius,
                            gaussian.displacements[row] + radius,
                            gate.value,
                        ],
                    )
                } else {
                    let first_axis = row.min(column);
                    let second_axis = row.max(column);
                    stable_gaussian_product(
                        gaussian.value,
                        amplitude,
                        gaussian.exponent,
                        &[
                            inverse_radius_squared,
                            inverse_radius_squared,
                            gaussian.displacements[first_axis],
                            gaussian.displacements[second_axis],
                            gate.value,
                        ],
                    )
                };
                let terms = [
                    gaussian_hessian,
                    stable_gaussian_product(
                        gaussian.value,
                        amplitude,
                        gaussian.exponent,
                        &[
                            -inverse_radius_squared,
                            gaussian.displacements[row],
                            gate.gradient[column],
                        ],
                    ),
                    stable_gaussian_product(
                        gaussian.value,
                        amplitude,
                        gaussian.exponent,
                        &[
                            -inverse_radius_squared,
                            gaussian.displacements[column],
                            gate.gradient[row],
                        ],
                    ),
                    stable_gaussian_product(
                        gaussian.value,
                        amplitude,
                        gaussian.exponent,
                        &[gate.hessian[row][column]],
                    ),
                ];
                let mut sum = 0.0;
                for term in terms {
                    sum = finite_sum(
                        sum,
                        term.ok_or(LocalTrendEvaluationError::NonFiniteWeightDerivative {
                            component: usize::MAX,
                            quantity,
                        })?,
                    )
                    .ok_or(
                        LocalTrendEvaluationError::NonFiniteWeightDerivative {
                            component: usize::MAX,
                            quantity,
                        },
                    )?;
                }
                *output = sum;
            }
        }
    }
    Ok(WeightJet {
        value,
        gradient,
        hessian,
    })
}

#[derive(Clone, Copy)]
struct GaussianWeightState<const D: usize> {
    displacements: [f64; D],
    exponent: f64,
    value: f64,
}

fn gaussian_weight_state<const D: usize>(
    point: Point<D>,
    center: Point<D>,
    amplitude: f64,
    inverse_radius: f64,
) -> Result<Option<GaussianWeightState<D>>, LocalTrendEvaluationError<D>>
where
    Dim<D>: SupportedDimension,
{
    let mut displacements = [0.0; D];
    let mut squared_radius = 0.0;
    for (axis, displacement) in displacements.iter_mut().enumerate() {
        *displacement = point.components()[axis] - center.components()[axis];
        if !displacement.is_finite() {
            return Err(LocalTrendEvaluationError::NonFiniteWeightDisplacement {
                component: usize::MAX,
                axis,
            });
        }
        let scaled = *displacement * inverse_radius;
        let square = scaled * scaled;
        if !scaled.is_finite() || !square.is_finite() {
            return Ok(None);
        }
        squared_radius += square;
        if !squared_radius.is_finite() {
            return Ok(None);
        }
    }
    let exponent = -0.5 * squared_radius;
    let value = stable_gaussian_value(amplitude, exponent).ok_or(
        LocalTrendEvaluationError::NonFiniteWeightDerivative {
            component: usize::MAX,
            quantity: LocalTrendQuantity::Value,
        },
    )?;
    Ok(Some(GaussianWeightState {
        displacements,
        exponent,
        value,
    }))
}

fn gaussian_weight_jet<const D: usize>(
    point: Point<D>,
    center: Point<D>,
    amplitude: f64,
    radius: f64,
    inverse_radius: f64,
    inverse_radius_squared: f64,
    demanded: KernelDerivativeOrder,
) -> Result<WeightJet<D>, LocalTrendEvaluationError<D>>
where
    Dim<D>: SupportedDimension,
{
    let mut displacements = [0.0; D];
    let mut scaled = [0.0; D];
    let mut squared_radius = 0.0;
    for (axis, output) in scaled.iter_mut().enumerate() {
        let displacement = point.components()[axis] - center.components()[axis];
        if !displacement.is_finite() {
            return Err(LocalTrendEvaluationError::NonFiniteWeightDisplacement {
                component: usize::MAX,
                axis,
            });
        }
        displacements[axis] = displacement;
        *output = displacement * inverse_radius;
        let square = *output * *output;
        if !output.is_finite() || !square.is_finite() {
            return Ok(WeightJet {
                value: 0.0,
                gradient: [0.0; D],
                hessian: [[0.0; D]; D],
            });
        }
        squared_radius += square;
        if !squared_radius.is_finite() {
            return Ok(WeightJet {
                value: 0.0,
                gradient: [0.0; D],
                hessian: [[0.0; D]; D],
            });
        }
    }
    let exponent = -0.5 * squared_radius;
    let value = stable_gaussian_value(amplitude, exponent).ok_or(
        LocalTrendEvaluationError::NonFiniteWeightDerivative {
            component: usize::MAX,
            quantity: LocalTrendQuantity::Value,
        },
    )?;
    let mut gradient = [0.0; D];
    let hessian = [[0.0; D]; D];
    if demanded == KernelDerivativeOrder::Value {
        return Ok(WeightJet {
            value,
            gradient,
            hessian,
        });
    }
    for axis in 0..D {
        gradient[axis] = stable_gaussian_product(
            value,
            amplitude,
            exponent,
            &[-inverse_radius_squared, displacements[axis]],
        )
        .ok_or(LocalTrendEvaluationError::NonFiniteWeightDerivative {
            component: usize::MAX,
            quantity: LocalTrendQuantity::Gradient { axis },
        })?;
    }
    if demanded == KernelDerivativeOrder::First {
        return Ok(WeightJet {
            value,
            gradient,
            hessian,
        });
    }
    let hessian = gaussian_weight_hessian(
        value,
        amplitude,
        exponent,
        radius,
        inverse_radius_squared,
        displacements,
    )?;
    Ok(WeightJet {
        value,
        gradient,
        hessian,
    })
}

fn gaussian_weight_hessian<const D: usize>(
    value: f64,
    amplitude: f64,
    exponent: f64,
    radius: f64,
    inverse_radius_squared: f64,
    displacements: [f64; D],
) -> Result<[[f64; D]; D], LocalTrendEvaluationError<D>>
where
    Dim<D>: SupportedDimension,
{
    let mut hessian = [[0.0; D]; D];
    for (row, hessian_row) in hessian.iter_mut().enumerate() {
        for (column, entry) in hessian_row.iter_mut().enumerate() {
            *entry = if row == column {
                stable_gaussian_product(
                    value,
                    amplitude,
                    exponent,
                    &[
                        inverse_radius_squared,
                        inverse_radius_squared,
                        displacements[row] - radius,
                        displacements[row] + radius,
                    ],
                )
            } else {
                let first_axis = row.min(column);
                let second_axis = row.max(column);
                stable_gaussian_product(
                    value,
                    amplitude,
                    exponent,
                    &[
                        inverse_radius_squared,
                        inverse_radius_squared,
                        displacements[first_axis],
                        displacements[second_axis],
                    ],
                )
            }
            .ok_or(LocalTrendEvaluationError::NonFiniteWeightDerivative {
                component: usize::MAX,
                quantity: LocalTrendQuantity::Hessian { row, column },
            })?;
        }
    }
    Ok(hessian)
}

#[inline]
fn stable_gaussian_value(amplitude: f64, exponent: f64) -> Option<f64> {
    if amplitude == 0.0 {
        return Some(0.0);
    }
    if !amplitude.is_finite() || !exponent.is_finite() {
        return None;
    }
    let value = amplitude * exponent.exp();
    if value.is_normal() {
        return Some(value);
    }

    let logarithm = amplitude.abs().ln() + exponent;
    let magnitude = logarithm.exp();
    let value = if amplitude.is_sign_negative() {
        -magnitude
    } else {
        magnitude
    };
    value.is_finite().then_some(value)
}

#[inline]
fn stable_gaussian_product(
    value: f64,
    amplitude: f64,
    exponent: f64,
    factors: &[f64],
) -> Option<f64> {
    if amplitude == 0.0 || factors.contains(&0.0) {
        return Some(0.0);
    }
    if !amplitude.is_finite()
        || !exponent.is_finite()
        || factors.iter().any(|factor| !factor.is_finite())
    {
        return None;
    }
    let (product, direct_is_normal) = factors.iter().fold(
        (value, value.is_normal()),
        |(product, all_normal), factor| {
            let product = product * factor;
            (product, all_normal && product.is_normal())
        },
    );
    if direct_is_normal {
        return Some(product);
    }

    let negative = factors
        .iter()
        .fold(amplitude.is_sign_negative(), |negative, factor| {
            negative ^ factor.is_sign_negative()
        });
    let logarithm = factors
        .iter()
        .fold(amplitude.abs().ln() + exponent, |logarithm, factor| {
            logarithm + factor.abs().ln()
        });
    let magnitude = logarithm.exp();
    let value = if negative { -magnitude } else { magnitude };
    value.is_finite().then_some(value)
}

fn finite_product(left: f64, right: f64) -> Option<f64> {
    let product = left * right;
    product.is_finite().then_some(product)
}

fn finite_sum(left: f64, right: f64) -> Option<f64> {
    let sum = left + right;
    sum.is_finite().then_some(sum)
}

fn checked_terms<const D: usize>(
    terms: &[f64],
    component: usize,
    quantity: LocalTrendQuantity,
) -> Result<f64, LocalTrendEvaluationError<D>>
where
    Dim<D>: SupportedDimension,
{
    terms.iter().try_fold(1.0, |product, term| {
        finite_product(product, *term).ok_or(LocalTrendEvaluationError::NonFiniteContribution {
            component,
            quantity,
        })
    })
}

fn checked_add<const D: usize>(
    left: f64,
    right: f64,
    component: usize,
    quantity: LocalTrendQuantity,
) -> Result<f64, LocalTrendEvaluationError<D>>
where
    Dim<D>: SupportedDimension,
{
    finite_sum(left, right).ok_or(LocalTrendEvaluationError::NonFiniteContribution {
        component,
        quantity,
    })
}

fn accumulate_component_hessian<const D: usize>(
    total: &mut [[f64; D]; D],
    query_weight: WeightJet<D>,
    center_weight: f64,
    kernel_value: f64,
    kernel_gradient: [f64; D],
    kernel_hessian: [[f64; D]; D],
    component: usize,
) -> Result<(), LocalTrendEvaluationError<D>>
where
    Dim<D>: SupportedDimension,
{
    for row in 0..D {
        for column in 0..D {
            let quantity = LocalTrendQuantity::Hessian { row, column };
            let terms = [
                checked_terms(
                    &[query_weight.hessian[row][column], kernel_value],
                    component,
                    quantity,
                )?,
                checked_terms(
                    &[query_weight.gradient[row], kernel_gradient[column]],
                    component,
                    quantity,
                )?,
                checked_terms(
                    &[kernel_gradient[row], query_weight.gradient[column]],
                    component,
                    quantity,
                )?,
                checked_terms(
                    &[query_weight.value, kernel_hessian[row][column]],
                    component,
                    quantity,
                )?,
            ];
            let inside = terms
                .into_iter()
                .try_fold(0.0, |sum, term| checked_add(sum, term, component, quantity))?;
            let contribution = checked_terms(&[center_weight, inside], component, quantity)?;
            total[row][column] =
                checked_add(total[row][column], contribution, component, quantity)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn narrow_region_gate_preserves_representable_first_derivative() {
        let width = 1.0e-153_f64;
        let inverse_width = width.recip();
        let coordinate = 1.0e-323;
        let parameter = coordinate * inverse_width;
        let complement = 1.0 - parameter;
        let expected = 30.0 * (parameter * inverse_width) * parameter * complement * complement;

        let gate = axis_region_gate(
            coordinate,
            0.0,
            1.0,
            inverse_width,
            inverse_width * inverse_width,
            KernelDerivativeOrder::First,
        );

        assert!(expected.is_finite() && expected != 0.0);
        assert!(gate.first.is_finite() && gate.first != 0.0);
        assert!((gate.first - expected).abs() <= expected.abs() * 16.0 * f64::EPSILON);
    }

    #[test]
    fn narrow_region_gate_preserves_factored_second_derivative_near_join() {
        let width = 1.0e-153_f64;
        let inverse_width = width.recip();
        let coordinate = f64::from_bits(width.to_bits() - 1);
        let parameter = coordinate * inverse_width;
        let expected = 60.0
            * parameter
            * (parameter - 1.0)
            * (2.0 * parameter - 1.0)
            * (inverse_width * inverse_width);

        let gate = axis_region_gate(
            coordinate,
            0.0,
            1.0,
            inverse_width,
            inverse_width * inverse_width,
            KernelDerivativeOrder::Second,
        );

        assert!(expected.is_finite() && expected != 0.0);
        assert!(gate.second.is_finite() && gate.second != 0.0);
        assert!((gate.second - expected).abs() <= expected.abs() * 16.0 * f64::EPSILON);
    }

    #[test]
    fn regional_weight_is_zero_outside_support_before_gaussian_displacement()
    -> Result<(), Box<dyn std::error::Error>> {
        let region = SmoothRegion::try_new(Point::try_new([-1.0])?, Point::try_new([1.0])?, 0.25)?;
        let jet = regional_gaussian_weight_jet(
            Point::try_new([f64::MAX])?,
            Point::try_new([-f64::MAX])?,
            1.0,
            1.0,
            1.0,
            1.0,
            region,
            KernelDerivativeOrder::Second,
        )?;

        assert_eq!(jet.value.to_bits(), 0.0_f64.to_bits());
        assert_eq!(jet.gradient.map(f64::to_bits), [0.0_f64.to_bits()]);
        assert_eq!(
            jet.hessian.map(|row| row.map(f64::to_bits)),
            [[0.0_f64.to_bits()]]
        );
        Ok(())
    }
}
