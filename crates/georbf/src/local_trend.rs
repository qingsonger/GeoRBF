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
    /// The transition width must be positive, keep the exact maximum first and
    /// second smootherstep derivatives representable, and be no greater than
    /// half the extent of every region axis. No loose polynomial-coefficient
    /// bound rejects an otherwise representable C2 taper.
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
        let maximum_second_derivative = (10.0 / 3.0_f64.sqrt()) * inverse_transition_width_squared;
        if !inverse_transition_width.is_finite()
            || inverse_transition_width == 0.0
            || !inverse_transition_width_squared.is_finite()
            || inverse_transition_width_squared == 0.0
            || !maximum_second_derivative.is_finite()
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

    #[cfg(test)]
    fn try_jet(
        self,
        point: Point<D>,
        demanded: KernelDerivativeOrder,
    ) -> Result<WeightJet<D>, LocalTrendEvaluationError<D>> {
        WeightJet::try_from_stable(self.stable_jet(point, demanded), demanded)
    }

    fn stable_jet(self, point: Point<D>, demanded: KernelDerivativeOrder) -> StableWeightJet<D> {
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

        let value = StableFactor::product_many(factors.iter().map(|factor| factor.value));
        let gradient = if demanded >= KernelDerivativeOrder::First {
            std::array::from_fn(|axis| {
                StableFactor::product_many(
                    std::iter::once(factors[axis].first).chain(
                        factors
                            .iter()
                            .enumerate()
                            .filter(|(other, _)| *other != axis)
                            .map(|(_, factor)| factor.value),
                    ),
                )
            })
        } else {
            [StableFactor::ZERO; D]
        };
        let hessian = if demanded >= KernelDerivativeOrder::Second {
            std::array::from_fn(|row| {
                std::array::from_fn(|column| {
                    if row == column {
                        StableFactor::product_many(
                            std::iter::once(factors[row].second).chain(
                                factors
                                    .iter()
                                    .enumerate()
                                    .filter(|(other, _)| *other != row)
                                    .map(|(_, factor)| factor.value),
                            ),
                        )
                    } else {
                        StableFactor::product_many(
                            [factors[row].first, factors[column].first]
                                .into_iter()
                                .chain(
                                    factors
                                        .iter()
                                        .enumerate()
                                        .filter(|(other, _)| *other != row && *other != column)
                                        .map(|(_, factor)| factor.value),
                                ),
                        )
                    }
                })
            })
        } else {
            [[StableFactor::ZERO; D]; D]
        };
        StableWeightJet {
            value,
            gradient,
            hessian,
        }
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
/// underflow to represented zero far from their centers and therefore cannot serve
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
                stable: StableWeightJet {
                    value: StableFactor::from_factors(&[value]),
                    gradient: [StableFactor::ZERO; D],
                    hessian: [[StableFactor::ZERO; D]; D],
                },
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

    fn ensure_derivative_available(
        &self,
        query: Point<D>,
        center: Point<D>,
        demanded: KernelDerivativeOrder,
    ) -> Result<(), LocalTrendEvaluationError<D>> {
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
        Ok(())
    }

    /// Evaluates the mixture through the demanded query derivative order.
    ///
    /// The gradient and Hessian include every product-rule term from `b_r(x)`:
    /// `b_y (grad(b_x) k + b_x grad(k))` and
    /// `b_y (H(b_x) k + grad(b_x) grad(k)^T + grad(k) grad(b_x)^T + b_x H(k))`.
    /// A component whose complete demanded query-weight jet or center weight is
    /// mathematically exactly zero is skipped before its fixed kernel is
    /// evaluated. Gaussian factors that merely underflow individually retain
    /// their logarithmic scale through the complete represented contribution,
    /// including the fixed Gaussian kernel value, gradient, and Hessian. Weight
    /// and fixed-Gaussian derivatives are not required to be individually
    /// representable before the complete mixture term is formed.
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
        self.ensure_derivative_available(query, center, demanded)?;

        let mut value = 0.0;
        let mut gradient = [0.0; D];
        let mut hessian = [[0.0; D]; D];
        for (component_index, component) in self.components.iter().enumerate() {
            let query_weight = component
                .weight
                .try_jet(query, demanded)
                .map_err(|source| source.with_component(component_index))?;
            if query_weight.is_exact_zero() {
                continue;
            }
            let center_weight = component
                .weight
                .try_jet(center, KernelDerivativeOrder::Value)
                .map_err(|source| source.with_component(component_index))?;
            if center_weight.stable.value.exact_zero {
                continue;
            }
            let stable_kernel =
                component_kernel_jet(component, query, center, demanded, component_index)?;

            let value_term = checked_stable_terms(
                &[
                    query_weight.stable.value,
                    center_weight.stable.value,
                    stable_kernel.value,
                ],
                &[],
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
                for (axis, gradient_entry) in gradient.iter_mut().enumerate() {
                    let weight_term = checked_stable_terms(
                        &[
                            center_weight.stable.value,
                            query_weight.stable.gradient[axis],
                            stable_kernel.value,
                        ],
                        &[],
                        component_index,
                        LocalTrendQuantity::Gradient { axis },
                    )?;
                    let kernel_term = checked_stable_terms(
                        &[
                            center_weight.stable.value,
                            query_weight.stable.value,
                            stable_kernel.gradient[axis],
                        ],
                        &[],
                        component_index,
                        LocalTrendQuantity::Gradient { axis },
                    )?;
                    let contribution = checked_add(
                        weight_term,
                        kernel_term,
                        component_index,
                        LocalTrendQuantity::Gradient { axis },
                    )?;
                    *gradient_entry = checked_add(
                        *gradient_entry,
                        contribution,
                        component_index,
                        LocalTrendQuantity::Gradient { axis },
                    )?;
                }

                if demanded >= KernelDerivativeOrder::Second {
                    accumulate_component_hessian(
                        &mut hessian,
                        query_weight,
                        center_weight.stable.value,
                        stable_kernel,
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
    stable: StableWeightJet<D>,
}

impl<const D: usize> WeightJet<D>
where
    Dim<D>: SupportedDimension,
{
    fn try_from_stable(
        stable: StableWeightJet<D>,
        _demanded: KernelDerivativeOrder,
    ) -> Result<Self, LocalTrendEvaluationError<D>> {
        let value = stable.value.represented_value(1.0).ok_or(
            LocalTrendEvaluationError::NonFiniteWeightDerivative {
                component: usize::MAX,
                quantity: LocalTrendQuantity::Value,
            },
        )?;
        Ok(Self { value, stable })
    }

    fn exact_zero(demanded: KernelDerivativeOrder) -> Result<Self, LocalTrendEvaluationError<D>> {
        Self::try_from_stable(StableWeightJet::exact_zero(), demanded)
    }

    fn inexact_zero(demanded: KernelDerivativeOrder) -> Result<Self, LocalTrendEvaluationError<D>> {
        Self::try_from_stable(StableWeightJet::inexact_zero(), demanded)
    }

    fn is_exact_zero(self) -> bool {
        self.stable.value.exact_zero
            && self.stable.gradient.iter().all(|entry| entry.exact_zero)
            && self
                .stable
                .hessian
                .iter()
                .flatten()
                .all(|entry| entry.exact_zero)
    }
}

#[derive(Clone, Copy)]
struct StableWeightJet<const D: usize> {
    value: StableFactor,
    gradient: [StableFactor; D],
    hessian: [[StableFactor; D]; D],
}

#[derive(Clone, Copy)]
struct StableKernelJet<const D: usize> {
    value: StableFactor,
    gradient: [StableFactor; D],
    hessian: [[StableFactor; D]; D],
}

impl<const D: usize> StableWeightJet<D> {
    const fn exact_zero() -> Self {
        Self {
            value: StableFactor::ZERO,
            gradient: [StableFactor::ZERO; D],
            hessian: [[StableFactor::ZERO; D]; D],
        }
    }

    const fn inexact_zero() -> Self {
        Self {
            value: StableFactor::INEXACT_ZERO,
            gradient: [StableFactor::INEXACT_ZERO; D],
            hessian: [[StableFactor::INEXACT_ZERO; D]; D],
        }
    }
}

fn double_sum(left: (f64, f64), right: (f64, f64)) -> (f64, f64) {
    if !left.0.is_finite() || !right.0.is_finite() {
        return (left.0 + right.0, 0.0);
    }
    let sum = left.0 + right.0;
    let right_virtual = sum - left.0;
    let error = (left.0 - (sum - right_virtual)) + (right.0 - right_virtual) + left.1 + right.1;
    let corrected = sum + error;
    (corrected, error - (corrected - sum))
}

fn double_product(left: (f64, f64), right: (f64, f64)) -> (f64, f64) {
    let product = left.0 * right.0;
    if !product.is_finite() || product == 0.0 {
        return (product, 0.0);
    }
    let error =
        left.0.mul_add(right.0, -product) + left.0 * right.1 + left.1 * right.0 + left.1 * right.1;
    if !error.is_finite() {
        return (product, 0.0);
    }
    let corrected = product + error;
    (corrected, error - (corrected - product))
}

fn double_product_f64(left: (f64, f64), right: f64) -> (f64, f64) {
    double_product(left, (right, 0.0))
}

#[derive(Clone, Copy)]
struct StableFactor {
    direct: f64,
    direct_error: f64,
    logarithm: f64,
    negative: bool,
    exact_zero: bool,
}

impl StableFactor {
    const ZERO: Self = Self {
        direct: 0.0,
        direct_error: 0.0,
        logarithm: f64::NEG_INFINITY,
        negative: false,
        exact_zero: true,
    };

    const ONE: Self = Self {
        direct: 1.0,
        direct_error: 0.0,
        logarithm: 0.0,
        negative: false,
        exact_zero: false,
    };

    const INEXACT_ZERO: Self = Self {
        direct: 0.0,
        direct_error: 0.0,
        logarithm: f64::NEG_INFINITY,
        negative: false,
        exact_zero: false,
    };

    fn from_factors(factors: &[f64]) -> Self {
        if factors.contains(&0.0) {
            return Self::ZERO;
        }
        let (direct, direct_error) = factors.iter().copied().fold((1.0, 0.0), double_product_f64);
        let negative = factors.iter().fold(false, |negative, factor| {
            negative ^ factor.is_sign_negative()
        });
        let logarithm = factors.iter().map(|factor| factor.abs().ln()).sum();
        Self {
            direct,
            direct_error,
            logarithm,
            negative,
            exact_zero: false,
        }
    }

    fn from_double(direct: f64, direct_error: f64) -> Self {
        if direct == 0.0 && direct_error == 0.0 {
            return Self::ZERO;
        }
        let represented = direct + direct_error;
        Self {
            direct,
            direct_error,
            logarithm: represented.abs().ln(),
            negative: represented.is_sign_negative(),
            exact_zero: false,
        }
    }

    fn from_gaussian(amplitude: f64, exponent: f64) -> Self {
        if amplitude == 0.0 {
            return Self::ZERO;
        }
        let (direct, direct_error) = double_product_f64((amplitude, 0.0), exponent.exp());
        Self {
            direct,
            direct_error,
            logarithm: amplitude.abs().ln() + exponent,
            negative: amplitude.is_sign_negative(),
            exact_zero: false,
        }
    }

    fn product_many(factors: impl IntoIterator<Item = Self>) -> Self {
        factors.into_iter().fold(Self::ONE, Self::product)
    }

    fn product(self, other: Self) -> Self {
        if self.exact_zero || other.exact_zero {
            return Self::ZERO;
        }
        let (direct, direct_error) = double_product(
            (self.direct, self.direct_error),
            (other.direct, other.direct_error),
        );
        Self {
            direct,
            direct_error,
            logarithm: self.logarithm + other.logarithm,
            negative: self.negative ^ other.negative,
            exact_zero: false,
        }
    }

    fn product_factors(self, factors: &[f64]) -> Self {
        if self.exact_zero || factors.contains(&0.0) {
            return Self::ZERO;
        }
        let (direct, direct_error) = factors
            .iter()
            .copied()
            .fold((self.direct, self.direct_error), double_product_f64);
        Self {
            direct,
            direct_error,
            logarithm: factors
                .iter()
                .fold(self.logarithm, |sum, factor| sum + factor.abs().ln()),
            negative: factors.iter().fold(self.negative, |negative, factor| {
                negative ^ factor.is_sign_negative()
            }),
            exact_zero: false,
        }
    }

    fn negated(mut self) -> Self {
        if !self.exact_zero {
            self.direct = -self.direct;
            self.direct_error = -self.direct_error;
            self.negative = !self.negative;
        }
        self
    }

    fn sum(self, other: Self) -> Self {
        if self.exact_zero {
            return other;
        }
        if other.exact_zero {
            return self;
        }
        let (direct, direct_error) = double_sum(
            (self.direct, self.direct_error),
            (other.direct, other.direct_error),
        );
        if self.negative == other.negative {
            let maximum = self.logarithm.max(other.logarithm);
            let minimum = self.logarithm.min(other.logarithm);
            return Self {
                direct,
                direct_error,
                logarithm: maximum + (minimum - maximum).exp().ln_1p(),
                negative: self.negative,
                exact_zero: false,
            };
        }
        if self.logarithm.total_cmp(&other.logarithm).is_eq() {
            return if direct.is_finite() && (direct != 0.0 || direct_error != 0.0) {
                let represented = direct + direct_error;
                Self {
                    direct,
                    direct_error,
                    logarithm: represented.abs().ln(),
                    negative: represented.is_sign_negative(),
                    exact_zero: false,
                }
            } else {
                Self::INEXACT_ZERO
            };
        }
        let (larger, smaller) = if self.logarithm > other.logarithm {
            (self, other)
        } else {
            (other, self)
        };
        Self {
            direct,
            direct_error,
            logarithm: larger.logarithm + (-(smaller.logarithm - larger.logarithm).exp()).ln_1p(),
            negative: larger.negative,
            exact_zero: false,
        }
    }

    fn represented_value(self, scale: f64) -> Option<f64> {
        if self.exact_zero || scale == 0.0 {
            return Some(0.0);
        }
        if !scale.is_finite() || self.logarithm.is_nan() || self.logarithm == f64::INFINITY {
            return None;
        }
        if self.logarithm == f64::NEG_INFINITY {
            return Some(if scale.is_sign_negative() ^ self.negative {
                -0.0
            } else {
                0.0
            });
        }
        let direct = scale.mul_add(self.direct, scale * self.direct_error);
        if direct.is_finite() && direct != 0.0 {
            return Some(direct);
        }
        let magnitude = (scale.abs().ln() + self.logarithm).exp();
        let negative = scale.is_sign_negative() ^ self.negative;
        let value = if negative { -magnitude } else { magnitude };
        value.is_finite().then_some(value)
    }
}

#[derive(Clone, Copy)]
struct AxisGate {
    value: StableFactor,
    first: StableFactor,
    second: StableFactor,
}

impl Default for AxisGate {
    fn default() -> Self {
        Self {
            value: StableFactor::ZERO,
            first: StableFactor::ZERO,
            second: StableFactor::ZERO,
        }
    }
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
    let value = left.value.product(right.value);
    let first = if demanded >= KernelDerivativeOrder::First {
        left.first
            .product(right.value)
            .sum(left.value.product(right.first).negated())
    } else {
        StableFactor::ZERO
    };
    let second = if demanded >= KernelDerivativeOrder::Second {
        left.second
            .product(right.value)
            .sum(
                StableFactor::from_factors(&[-2.0])
                    .product(left.first)
                    .product(right.first),
            )
            .sum(left.value.product(right.second))
    } else {
        StableFactor::ZERO
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
            value: StableFactor::ONE,
            first: StableFactor::ZERO,
            second: StableFactor::ZERO,
        };
    }
    let parameter_factor = StableFactor::from_factors(&[parameter]);
    let complement = parameter_factor.sum(StableFactor::from_factors(&[-1.0]));
    let polynomial = StableFactor::from_factors(&[6.0, parameter, parameter])
        .sum(StableFactor::from_factors(&[-15.0, parameter]))
        .sum(StableFactor::from_factors(&[10.0]));
    let two_parameter_minus_one =
        StableFactor::from_factors(&[2.0, parameter]).sum(StableFactor::from_factors(&[-1.0]));
    AxisGate {
        value: StableFactor::from_factors(&[parameter, parameter, parameter]).product(polynomial),
        first: StableFactor::from_factors(&[30.0, parameter, inverse_width, parameter])
            .product(complement)
            .product(complement),
        second: StableFactor::from_factors(&[60.0, parameter, inverse_width_squared])
            .product(complement)
            .product(two_parameter_minus_one),
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
    let gate = region.stable_jet(point, demanded);
    if gate.value.exact_zero
        && gate.gradient.iter().all(|entry| entry.exact_zero)
        && gate.hessian.iter().flatten().all(|entry| entry.exact_zero)
    {
        return WeightJet::exact_zero(demanded);
    }
    let Some(gaussian) = gaussian_weight_state(point, center, amplitude, inverse_radius)? else {
        return WeightJet::inexact_zero(demanded);
    };
    let value = gaussian.factor.product(gate.value);
    let gradient = if demanded >= KernelDerivativeOrder::First {
        std::array::from_fn(|axis| {
            gaussian
                .factor
                .product(gate.value)
                .product_factors(&[-inverse_radius_squared])
                .product(StableFactor::from_double(
                    gaussian.displacements[axis],
                    gaussian.displacement_errors[axis],
                ))
                .sum(gaussian.factor.product(gate.gradient[axis]))
        })
    } else {
        [StableFactor::ZERO; D]
    };
    let hessian = if demanded >= KernelDerivativeOrder::Second {
        std::array::from_fn(|row| {
            std::array::from_fn(|column| {
                let gaussian_curvature = if row == column {
                    gaussian.scaled_diagonal_curvature(row, radius, inverse_radius_squared)
                } else {
                    let first_axis = row.min(column);
                    let second_axis = row.max(column);
                    gaussian
                        .displacement(first_axis)
                        .product(gaussian.displacement(second_axis))
                        .product_factors(&[inverse_radius_squared, inverse_radius_squared])
                };
                gaussian
                    .factor
                    .product(gate.value)
                    .product(gaussian_curvature)
                    .sum(
                        gaussian
                            .factor
                            .product(gate.gradient[column])
                            .product_factors(&[-inverse_radius_squared])
                            .product(gaussian.displacement(row)),
                    )
                    .sum(
                        gaussian
                            .factor
                            .product(gate.gradient[row])
                            .product_factors(&[-inverse_radius_squared])
                            .product(gaussian.displacement(column)),
                    )
                    .sum(gaussian.factor.product(gate.hessian[row][column]))
            })
        })
    } else {
        [[StableFactor::ZERO; D]; D]
    };
    WeightJet::try_from_stable(
        StableWeightJet {
            value,
            gradient,
            hessian,
        },
        demanded,
    )
}

#[derive(Clone, Copy)]
struct GaussianWeightState<const D: usize> {
    displacements: [f64; D],
    displacement_errors: [f64; D],
    factor: StableFactor,
}

impl<const D: usize> GaussianWeightState<D> {
    fn displacement(self, axis: usize) -> StableFactor {
        StableFactor::from_double(self.displacements[axis], self.displacement_errors[axis])
    }

    fn scaled_diagonal_curvature(
        self,
        axis: usize,
        radius: f64,
        inverse_radius_squared: f64,
    ) -> StableFactor {
        let displacement = (self.displacements[axis], self.displacement_errors[axis]);
        let lower = double_sum(displacement, (-radius, 0.0));
        let upper = double_sum(displacement, (radius, 0.0));
        StableFactor::from_double(lower.0, lower.1)
            .product_factors(&[inverse_radius_squared])
            .product(
                StableFactor::from_double(upper.0, upper.1)
                    .product_factors(&[inverse_radius_squared]),
            )
    }
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
    let mut displacement_errors = [0.0; D];
    let mut squared_radius = (0.0, 0.0);
    for (axis, displacement) in displacements.iter_mut().enumerate() {
        *displacement = point.components()[axis] - center.components()[axis];
        if !displacement.is_finite() {
            return Err(LocalTrendEvaluationError::NonFiniteWeightDisplacement {
                component: usize::MAX,
                axis,
            });
        }
        displacement_errors[axis] = double_sum(
            (point.components()[axis], 0.0),
            (-center.components()[axis], 0.0),
        )
        .1;
        let scaled = double_product_f64((*displacement, displacement_errors[axis]), inverse_radius);
        let square = double_product(scaled, scaled);
        if !scaled.0.is_finite() || !scaled.1.is_finite() || !square.0.is_finite() {
            return Ok(None);
        }
        squared_radius = double_sum(squared_radius, square);
        if !squared_radius.0.is_finite() || !squared_radius.1.is_finite() {
            return Ok(None);
        }
    }
    let exponent = -0.5 * (squared_radius.0 + squared_radius.1);
    Ok(Some(GaussianWeightState {
        displacements,
        displacement_errors,
        factor: StableFactor::from_gaussian(amplitude, exponent),
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
            return WeightJet::inexact_zero(demanded);
        }
        squared_radius += square;
        if !squared_radius.is_finite() {
            return WeightJet::inexact_zero(demanded);
        }
    }
    let exponent = -0.5 * squared_radius;
    let value = StableFactor::from_gaussian(amplitude, exponent);
    let gradient = if demanded >= KernelDerivativeOrder::First {
        std::array::from_fn(|axis| {
            value.product_factors(&[-inverse_radius_squared, displacements[axis]])
        })
    } else {
        [StableFactor::ZERO; D]
    };
    let hessian = if demanded >= KernelDerivativeOrder::Second {
        gaussian_weight_hessian(value, radius, inverse_radius_squared, displacements)
    } else {
        [[StableFactor::ZERO; D]; D]
    };
    WeightJet::try_from_stable(
        StableWeightJet {
            value,
            gradient,
            hessian,
        },
        demanded,
    )
}

fn gaussian_weight_hessian<const D: usize>(
    value: StableFactor,
    radius: f64,
    inverse_radius_squared: f64,
    displacements: [f64; D],
) -> [[StableFactor; D]; D]
where
    Dim<D>: SupportedDimension,
{
    let mut hessian = [[StableFactor::ZERO; D]; D];
    for (row, hessian_row) in hessian.iter_mut().enumerate() {
        for (column, entry) in hessian_row.iter_mut().enumerate() {
            *entry = if row == column {
                value.product_factors(&[
                    inverse_radius_squared,
                    inverse_radius_squared,
                    displacements[row] - radius,
                    displacements[row] + radius,
                ])
            } else {
                let first_axis = row.min(column);
                let second_axis = row.max(column);
                value.product_factors(&[
                    inverse_radius_squared,
                    inverse_radius_squared,
                    displacements[first_axis],
                    displacements[second_axis],
                ])
            };
        }
    }
    hessian
}

fn finite_product(left: f64, right: f64) -> Option<f64> {
    let product = left * right;
    product.is_finite().then_some(product)
}

fn finite_sum(left: f64, right: f64) -> Option<f64> {
    let sum = left + right;
    sum.is_finite().then_some(sum)
}

fn checked_stable_terms<const D: usize>(
    stable_terms: &[StableFactor],
    represented_terms: &[f64],
    component: usize,
    quantity: LocalTrendQuantity,
) -> Result<f64, LocalTrendEvaluationError<D>>
where
    Dim<D>: SupportedDimension,
{
    if represented_terms.iter().any(|term| !term.is_finite()) {
        return Err(LocalTrendEvaluationError::NonFiniteContribution {
            component,
            quantity,
        });
    }
    StableFactor::product_many(stable_terms.iter().copied().chain(std::iter::once(
        StableFactor::from_factors(represented_terms),
    )))
    .represented_value(1.0)
    .ok_or(LocalTrendEvaluationError::NonFiniteContribution {
        component,
        quantity,
    })
}

fn stable_kernel_jet<const D: usize>(
    kernel: KernelDefinition<D>,
    query: Point<D>,
    center: Point<D>,
    anisotropy: &GlobalAnisotropy<D>,
    demanded: KernelDerivativeOrder,
) -> Result<StableKernelJet<D>, KernelDefinitionEvaluationError<D>>
where
    Dim<D>: SupportedDimension,
{
    if let KernelDefinition::Gaussian(gaussian) = kernel {
        let separation = anisotropy
            .try_transform_separation(query, center)
            .map_err(KernelDefinitionEvaluationError::Anisotropy)?;
        let scaled = separation.radius() / gaussian.length_scale();
        let value = StableFactor::from_gaussian(1.0, -0.5 * scaled * scaled);
        let transformed = match separation.unit_displacement() {
            Some(unit) => std::array::from_fn(|axis| unit[axis] * separation.radius()),
            None => [0.0; D],
        };
        let inverse_length = gaussian.length_scale().recip();
        let inverse_length_squared = inverse_length * inverse_length;
        let scaled_projections = if demanded >= KernelDerivativeOrder::First {
            std::array::from_fn(|axis| {
                (0..D)
                    .map(|row| {
                        StableFactor::from_factors(&[
                            anisotropy.transform()[row][axis],
                            transformed[row],
                            inverse_length_squared,
                        ])
                    })
                    .fold(StableFactor::ZERO, StableFactor::sum)
            })
        } else {
            [StableFactor::ZERO; D]
        };
        let gradient = if demanded >= KernelDerivativeOrder::First {
            scaled_projections.map(|projection| value.product(projection.negated()))
        } else {
            [StableFactor::ZERO; D]
        };
        let hessian = if demanded >= KernelDerivativeOrder::Second {
            std::array::from_fn(|row| {
                std::array::from_fn(|column| {
                    let curvature = (0..D)
                        .map(|axis| {
                            StableFactor::from_factors(&[
                                -inverse_length_squared,
                                anisotropy.transform()[axis][row],
                                anisotropy.transform()[axis][column],
                            ])
                        })
                        .fold(StableFactor::ZERO, StableFactor::sum);
                    value.product(
                        scaled_projections[row]
                            .product(scaled_projections[column])
                            .sum(curvature),
                    )
                })
            })
        } else {
            [[StableFactor::ZERO; D]; D]
        };
        Ok(StableKernelJet {
            value,
            gradient,
            hessian,
        })
    } else {
        let represented = kernel.try_spatial_jet(query, center, demanded, Some(anisotropy))?;
        Ok(StableKernelJet {
            value: StableFactor::from_factors(&[represented.value()]),
            gradient: if demanded >= KernelDerivativeOrder::First {
                represented
                    .first_derivative(KernelArgument::Query)
                    .map(|entry| StableFactor::from_factors(&[entry]))
            } else {
                [StableFactor::ZERO; D]
            },
            hessian: if demanded >= KernelDerivativeOrder::Second {
                represented
                    .second_derivative([KernelArgument::Query, KernelArgument::Query])
                    .map(|row| row.map(|entry| StableFactor::from_factors(&[entry])))
            } else {
                [[StableFactor::ZERO; D]; D]
            },
        })
    }
}

fn component_kernel_jet<const D: usize>(
    component: &LocalTrendComponent<D>,
    query: Point<D>,
    center: Point<D>,
    demanded: KernelDerivativeOrder,
    component_index: usize,
) -> Result<StableKernelJet<D>, LocalTrendEvaluationError<D>>
where
    Dim<D>: SupportedDimension,
{
    stable_kernel_jet(
        component.kernel,
        query,
        center,
        &component.anisotropy,
        demanded,
    )
    .map_err(|source| LocalTrendEvaluationError::Kernel {
        component: component_index,
        source,
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
    center_weight: StableFactor,
    kernel: StableKernelJet<D>,
    component: usize,
) -> Result<(), LocalTrendEvaluationError<D>>
where
    Dim<D>: SupportedDimension,
{
    for (row, total_row) in total.iter_mut().enumerate() {
        for (column, total_entry) in total_row.iter_mut().enumerate() {
            let quantity = LocalTrendQuantity::Hessian { row, column };
            let terms = [
                checked_stable_terms(
                    &[
                        center_weight,
                        query_weight.stable.hessian[row][column],
                        kernel.value,
                    ],
                    &[],
                    component,
                    quantity,
                )?,
                checked_stable_terms(
                    &[
                        center_weight,
                        query_weight.stable.gradient[row],
                        kernel.gradient[column],
                    ],
                    &[],
                    component,
                    quantity,
                )?,
                checked_stable_terms(
                    &[
                        center_weight,
                        query_weight.stable.gradient[column],
                        kernel.gradient[row],
                    ],
                    &[],
                    component,
                    quantity,
                )?,
                checked_stable_terms(
                    &[
                        center_weight,
                        query_weight.stable.value,
                        kernel.hessian[row][column],
                    ],
                    &[],
                    component,
                    quantity,
                )?,
            ];
            let inside = terms
                .into_iter()
                .try_fold(0.0, |sum, term| checked_add(sum, term, component, quantity))?;
            *total_entry = checked_add(*total_entry, inside, component, quantity)?;
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
        let first = gate.first.represented_value(1.0).unwrap_or(f64::NAN);
        assert!(first.is_finite() && first != 0.0);
        assert!((first - expected).abs() <= expected.abs() * 16.0 * f64::EPSILON);
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
        let second = gate.second.represented_value(1.0).unwrap_or(f64::NAN);
        assert!(second.is_finite() && second != 0.0);
        assert!((second - expected).abs() <= expected.abs() * 16.0 * f64::EPSILON);
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
        assert!(jet.stable.gradient.iter().all(|factor| factor.exact_zero));
        assert_eq!(
            jet.stable.hessian[0][0]
                .represented_value(1.0)
                .unwrap_or(f64::NAN)
                .to_bits(),
            0.0_f64.to_bits()
        );
        Ok(())
    }

    #[test]
    fn regional_weight_retains_amplitude_scaled_gate_jet() -> Result<(), Box<dyn Error>> {
        let coordinate = 1.0e-110_f64;
        let amplitude = 1.0e154_f64;
        let region = SmoothRegion::try_new(Point::try_new([0.0])?, Point::try_new([2.0])?, 1.0)?;
        let complement = coordinate - 1.0;
        let polynomial = coordinate * (6.0 * coordinate - 15.0) + 10.0;
        let expected_value = ((amplitude * coordinate) * coordinate) * coordinate * polynomial;
        let expected_first =
            30.0 * ((amplitude * coordinate) * coordinate) * complement * complement;
        let expected_second =
            60.0 * (amplitude * coordinate) * complement * (2.0 * coordinate - 1.0)
                - expected_value;

        let jet = regional_gaussian_weight_jet(
            Point::try_new([coordinate])?,
            Point::try_new([coordinate])?,
            amplitude,
            1.0,
            1.0,
            1.0,
            region,
            KernelDerivativeOrder::Second,
        )?;

        for value in [expected_value, expected_first, expected_second] {
            assert!(value.is_finite() && value != 0.0);
        }
        let actual_first = jet.stable.gradient[0]
            .represented_value(1.0)
            .unwrap_or(f64::NAN);
        let actual_second = jet.stable.hessian[0][0]
            .represented_value(1.0)
            .unwrap_or(f64::NAN);
        assert!(jet.value.is_finite() && jet.value != 0.0);
        assert!(actual_first.is_finite() && actual_first != 0.0);
        assert!(actual_second.is_finite() && actual_second != 0.0);
        assert!((jet.value - expected_value).abs() <= expected_value.abs() * 256.0 * f64::EPSILON);
        assert!(
            (actual_first - expected_first).abs() <= expected_first.abs() * 32.0 * f64::EPSILON
        );
        assert!(
            (actual_second - expected_second).abs() <= expected_second.abs() * 32.0 * f64::EPSILON
        );
        Ok(())
    }

    #[test]
    fn region_accepts_exactly_representable_smootherstep_curvature() -> Result<(), Box<dyn Error>> {
        let width = 5.0e-154_f64;
        let parameter = (3.0 - 3.0_f64.sqrt()) / 6.0;
        let region = SmoothRegion::try_new(Point::try_new([0.0])?, Point::try_new([1.0])?, width)?;
        let jet = region.try_jet(
            Point::try_new([parameter * width])?,
            KernelDerivativeOrder::Second,
        )?;
        let expected = (10.0 / 3.0_f64.sqrt()) * (width.recip() * width.recip());
        let actual = jet.stable.hessian[0][0]
            .represented_value(1.0)
            .unwrap_or(f64::NAN);

        assert!(expected.is_finite() && expected != 0.0);
        assert!(actual.is_finite() && actual != 0.0);
        assert!((actual - expected).abs() <= expected.abs() * 32.0 * f64::EPSILON);
        Ok(())
    }
}
