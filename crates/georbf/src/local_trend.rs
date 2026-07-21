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

    /// Returns the constant value, or `None` for a spatially varying basis.
    #[must_use]
    pub const fn constant_value(self) -> Option<f64> {
        match self.kind {
            SmoothSpatialWeightKind::Constant { value } => Some(value),
            SmoothSpatialWeightKind::Gaussian { .. } => None,
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
            } => Some((center, amplitude, radius)),
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
