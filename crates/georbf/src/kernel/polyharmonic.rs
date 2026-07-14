//! Concrete polyharmonic and dimension-specific surface-spline kernels.
//!
//! [`PolyharmonicSpline`] uses an integer radial power and supports D=1, D=2,
//! and D=3. [`SurfaceSpline`] selects the dimension-dependent power
//! `p = 2m - D` for Sobolev order `m`. Unsupported dimensions fail at compile
//! time:
//!
//! ```compile_fail
//! use georbf::SurfaceSpline;
//!
//! fn unsupported(_: SurfaceSpline<0>) {}
//! ```
//!
//! ```compile_fail
//! use georbf::SurfaceSpline;
//!
//! fn unsupported(_: SurfaceSpline<4>) {}
//! ```

use std::error::Error;
use std::fmt;

use crate::dimension::{Dim, SupportedDimension};
use crate::kernel_calculus::{
    KernelCalculusError, RadialDerivativeOrder, RadialExpansionCoefficients, RadialJet,
    RadialSeparation,
};

use super::{
    CpdOrder, KernelDefiniteness, KernelDerivativeCapabilities, KernelDerivativeOrder,
    KernelDimensions, KernelMetadata, KernelSupport,
};

const NO_PARAMETERS: &[super::KernelParameterDefinition<'static>] = &[];

/// Integer-power polyharmonic spline with CPD-positive sign normalization.
///
/// For power `p >= 1`, let `s_p = (-1)^(floor(p / 2) + 1)`. The radial
/// formula is
///
/// ```text
/// phi_p(0) = 0,
/// phi_p(r) = s_p r^p              for positive r and odd p,
/// phi_p(r) = s_p r^p log(r)       for positive r and even p.
/// ```
///
/// The family is conditionally positive definite of order
/// `floor(p / 2) + 1` and has no scalar or shape parameter. Power is a
/// discrete family selector, not a configured physical parameter. For even
/// powers, `log(r)` uses the active coordinate representation's unit radius as
/// its fixed reference rather than introducing a tunable scale.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub struct PolyharmonicSpline {
    power: u16,
}

impl PolyharmonicSpline {
    /// Constructs the integer-power family member.
    ///
    /// # Errors
    ///
    /// Returns [`PolyharmonicSplineConstructionError::ZeroPower`] when
    /// `power` is zero.
    pub const fn try_new(power: u16) -> Result<Self, PolyharmonicSplineConstructionError> {
        if power == 0 {
            Err(PolyharmonicSplineConstructionError::ZeroPower)
        } else {
            Ok(Self { power })
        }
    }

    /// Returns the positive integer radial power.
    #[must_use]
    pub const fn power(self) -> u16 {
        self.power
    }

    /// Returns CPD order `floor(power / 2) + 1`.
    pub const fn cpd_order(self) -> CpdOrder {
        CpdOrder((self.power / 2 + 1) as usize)
    }

    /// Returns static metadata for this family member.
    ///
    /// All three supported dimensions are valid, support is global, and the
    /// parameter slice is empty. Away derivatives are supplied through third
    /// order. Center support is exact through `min(power - 1, 3)`.
    pub const fn metadata(self) -> KernelMetadata<'static> {
        KernelMetadata {
            name: "polyharmonic_spline",
            definiteness: KernelDefiniteness::ConditionallyPositiveDefinite {
                order: self.cpd_order(),
            },
            dimensions: KernelDimensions {
                flags: [true, true, true],
            },
            derivatives: derivative_capabilities(self.power),
            support: KernelSupport::Global,
            parameters: NO_PARAMETERS,
        }
    }

    /// Evaluates one radial derivative through third order.
    ///
    /// At a positive radius every requested derivative is analytic. At the
    /// center the result is `Some(0.0)` exactly when the corresponding spatial
    /// derivative is supported by [`Self::metadata`], and `None` otherwise.
    /// In particular, this does not expose a one-sided radial derivative as a
    /// nonexistent Euclidean center derivative.
    ///
    /// # Errors
    ///
    /// Returns a structured error for a negative or non-finite radius or a
    /// derivative that is not finitely representable.
    pub fn radial_derivative(
        self,
        radius: f64,
        order: KernelDerivativeOrder,
    ) -> Result<Option<f64>, PolyharmonicSplineEvaluationError> {
        validate_radius(radius)?;
        if radius == 0.0 {
            return Ok(center_supports(self.power, order).then_some(0.0));
        }
        evaluate_away_derivative(self.power, radius, order).map(Some)
    }

    /// Evaluates `phi(r)`, including the exact center value zero.
    ///
    /// # Errors
    ///
    /// Returns a structured error for an invalid radius or a value that is not
    /// finitely representable.
    pub fn radial_value(self, radius: f64) -> Result<f64, PolyharmonicSplineEvaluationError> {
        validate_radius(radius)?;
        if radius == 0.0 {
            Ok(0.0)
        } else {
            evaluate_away_derivative(self.power, radius, KernelDerivativeOrder::Value)
        }
    }

    /// Builds the complete radial jet consumed by the shared Cartesian calculus.
    ///
    /// Positive radii use direct analytic derivatives. D=2 and D=3 also use
    /// direct closed forms for `phi'(r)/r` and
    /// `(phi''(r) - phi'(r)/r)/r`; the latter is never reconstructed by
    /// subtracting rounded derivative values. At the center a complete jet is
    /// returned only when the family supports all derivatives through third
    /// order.
    ///
    /// # Errors
    ///
    /// Returns [`PolyharmonicSplineEvaluationError::CenterJetUnsupported`]
    /// rather than fabricating unavailable center derivatives. Non-finite
    /// radial values, coefficients, or derivatives retain the shared
    /// [`KernelCalculusError`] diagnostic as their source.
    pub fn radial_jet<const D: usize>(
        self,
        separation: RadialSeparation<D>,
    ) -> Result<RadialJet, PolyharmonicSplineEvaluationError>
    where
        Dim<D>: SupportedDimension,
    {
        let radius = separation.radius();
        if separation.is_center() {
            if self.power < 4 {
                return Err(PolyharmonicSplineEvaluationError::CenterJetUnsupported {
                    power: self.power,
                    maximum_center_order: center_order(self.power),
                });
            }
            return RadialJet::try_center(0.0, 0.0).map_err(Into::into);
        }

        let value = evaluate_away_derivative(self.power, radius, KernelDerivativeOrder::Value)?;
        let first = evaluate_away_derivative(self.power, radius, KernelDerivativeOrder::First)?;
        let second = evaluate_away_derivative(self.power, radius, KernelDerivativeOrder::Second)?;
        let third = evaluate_away_derivative(self.power, radius, KernelDerivativeOrder::Third)?;
        if D == 1 {
            return RadialJet::try_away(value, first, second, third).map_err(Into::into);
        }

        let (first_over_radius, second_remainder_over_radius) =
            expansion_coefficients(self.power, radius);
        let expansion =
            RadialExpansionCoefficients::try_new(first_over_radius, second_remainder_over_radius)?;
        RadialJet::try_away_with_expansion(value, first, second, third, expansion)
            .map_err(Into::into)
    }
}

/// Dimension-specific surface spline of Sobolev order `m`.
///
/// Construction requires `2m > D` and selects the polyharmonic power
/// `p = 2m - D`. Its metadata supports only the compile-time dimension `D`
/// and declares CPD order `m`, requiring the later complete polynomial side
/// space through total degree `m - 1`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub struct SurfaceSpline<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    order: u16,
    polyharmonic: PolyharmonicSpline,
}

impl<const D: usize> SurfaceSpline<D>
where
    Dim<D>: SupportedDimension,
{
    /// Constructs a dimension-valid surface spline.
    ///
    /// # Errors
    ///
    /// Returns [`SurfaceSplineConstructionError::OrderTooLow`] unless
    /// `2 * order > D`, or [`SurfaceSplineConstructionError::OrderOverflow`]
    /// when doubling the supplied order exceeds `u16`.
    pub const fn try_new(order: u16) -> Result<Self, SurfaceSplineConstructionError> {
        let Some(twice_order) = order.checked_mul(2) else {
            return Err(SurfaceSplineConstructionError::OrderOverflow { order });
        };
        if (twice_order as usize) <= D {
            return Err(SurfaceSplineConstructionError::OrderTooLow {
                order,
                dimension: D,
            });
        }
        let dimension = match D {
            1 => 1_u16,
            2 => 2_u16,
            3 => 3_u16,
            _ => 0_u16,
        };
        let power = twice_order - dimension;
        Ok(Self {
            order,
            polyharmonic: PolyharmonicSpline { power },
        })
    }

    /// Returns the positive Sobolev/surface-spline order `m`.
    #[must_use]
    pub const fn order(self) -> u16 {
        self.order
    }

    /// Returns the derived positive power `2m - D`.
    #[must_use]
    pub const fn power(self) -> u16 {
        self.polyharmonic.power
    }

    /// Returns dimension-specific global-support metadata.
    pub const fn metadata(self) -> KernelMetadata<'static> {
        KernelMetadata {
            name: "surface_spline",
            definiteness: KernelDefiniteness::ConditionallyPositiveDefinite {
                order: CpdOrder(self.order as usize),
            },
            dimensions: KernelDimensions {
                flags: [D == 1, D == 2, D == 3],
            },
            derivatives: derivative_capabilities(self.power()),
            support: KernelSupport::Global,
            parameters: NO_PARAMETERS,
        }
    }

    /// Evaluates one spatially valid radial derivative through third order.
    ///
    /// # Errors
    ///
    /// Returns the same structured diagnostics as
    /// [`PolyharmonicSpline::radial_derivative`].
    pub fn radial_derivative(
        self,
        radius: f64,
        order: KernelDerivativeOrder,
    ) -> Result<Option<f64>, PolyharmonicSplineEvaluationError> {
        self.polyharmonic.radial_derivative(radius, order)
    }

    /// Evaluates the radial value, including the exact center value zero.
    ///
    /// # Errors
    ///
    /// Returns the same structured diagnostics as
    /// [`PolyharmonicSpline::radial_value`].
    pub fn radial_value(self, radius: f64) -> Result<f64, PolyharmonicSplineEvaluationError> {
        self.polyharmonic.radial_value(radius)
    }

    /// Builds the complete radial jet for the matching dimension `D`.
    ///
    /// # Errors
    ///
    /// Returns the same structured diagnostics as
    /// [`PolyharmonicSpline::radial_jet`].
    pub fn radial_jet(
        self,
        separation: RadialSeparation<D>,
    ) -> Result<RadialJet, PolyharmonicSplineEvaluationError> {
        self.polyharmonic.radial_jet(separation)
    }
}

/// Error returned by [`PolyharmonicSpline`] construction.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PolyharmonicSplineConstructionError {
    /// Integer power zero has no member in the implemented family.
    ZeroPower,
}

impl fmt::Display for PolyharmonicSplineConstructionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("polyharmonic spline power must be positive")
    }
}

impl Error for PolyharmonicSplineConstructionError {}

/// Error returned by [`SurfaceSpline`] construction.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SurfaceSplineConstructionError {
    /// The supplied order does not satisfy `2m > D`.
    OrderTooLow {
        /// Rejected Sobolev/surface-spline order.
        order: u16,
        /// Compile-time spatial dimension.
        dimension: usize,
    },
    /// Doubling the supplied order is not representable as `u16`.
    OrderOverflow {
        /// Rejected Sobolev/surface-spline order.
        order: u16,
    },
}

impl fmt::Display for SurfaceSplineConstructionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OrderTooLow { order, dimension } => write!(
                formatter,
                "surface spline order {order} is invalid in D={dimension}; require 2m > D"
            ),
            Self::OrderOverflow { order } => write!(
                formatter,
                "surface spline order {order} is too large to form 2m"
            ),
        }
    }
}

impl Error for SurfaceSplineConstructionError {}

/// Error returned by polyharmonic or surface-spline radial evaluation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PolyharmonicSplineEvaluationError {
    /// Radius is NaN or infinite.
    NonFiniteRadius {
        /// Rejected radius.
        radius: f64,
    },
    /// Radius is finite but negative.
    NegativeRadius {
        /// Rejected radius.
        radius: f64,
    },
    /// A complete zero-through-third center jet was requested from a less
    /// smooth family member.
    CenterJetUnsupported {
        /// Integer polyharmonic power.
        power: u16,
        /// Highest spatial derivative supported at the center.
        maximum_center_order: KernelDerivativeOrder,
    },
    /// Shared radial-jet validation rejected a non-finite derivative or
    /// expansion coefficient.
    KernelCalculus(KernelCalculusError),
}

impl fmt::Display for PolyharmonicSplineEvaluationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteRadius { radius } => {
                write!(
                    formatter,
                    "polyharmonic radius must be finite, got {radius}"
                )
            }
            Self::NegativeRadius { radius } => {
                write!(
                    formatter,
                    "polyharmonic radius must be nonnegative, got {radius}"
                )
            }
            Self::CenterJetUnsupported {
                power,
                maximum_center_order,
            } => write!(
                formatter,
                "polyharmonic power {power} supports center derivatives only through {maximum_center_order:?}"
            ),
            Self::KernelCalculus(source) => source.fmt(formatter),
        }
    }
}

impl Error for PolyharmonicSplineEvaluationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::KernelCalculus(source) => Some(source),
            _ => None,
        }
    }
}

impl From<KernelCalculusError> for PolyharmonicSplineEvaluationError {
    fn from(value: KernelCalculusError) -> Self {
        Self::KernelCalculus(value)
    }
}

const fn derivative_capabilities(power: u16) -> KernelDerivativeCapabilities {
    KernelDerivativeCapabilities {
        away_through: KernelDerivativeOrder::Third,
        center_through: Some(center_order(power)),
    }
}

const fn center_order(power: u16) -> KernelDerivativeOrder {
    match power {
        1 => KernelDerivativeOrder::Value,
        2 => KernelDerivativeOrder::First,
        3 => KernelDerivativeOrder::Second,
        _ => KernelDerivativeOrder::Third,
    }
}

const fn center_supports(power: u16, order: KernelDerivativeOrder) -> bool {
    match order {
        KernelDerivativeOrder::Value => true,
        KernelDerivativeOrder::First => power >= 2,
        KernelDerivativeOrder::Second => power >= 3,
        KernelDerivativeOrder::Third => power >= 4,
    }
}

fn validate_radius(radius: f64) -> Result<(), PolyharmonicSplineEvaluationError> {
    if !radius.is_finite() {
        Err(PolyharmonicSplineEvaluationError::NonFiniteRadius { radius })
    } else if radius < 0.0 {
        Err(PolyharmonicSplineEvaluationError::NegativeRadius { radius })
    } else {
        Ok(())
    }
}

fn evaluate_away_derivative(
    power: u16,
    radius: f64,
    order: KernelDerivativeOrder,
) -> Result<f64, PolyharmonicSplineEvaluationError> {
    let p = f64::from(power);
    let sign = cpd_sign(power);
    let value = if power % 2 == 1 {
        let coefficient = match order {
            KernelDerivativeOrder::Value => 1.0,
            KernelDerivativeOrder::First => p,
            KernelDerivativeOrder::Second if power < 2 => 0.0,
            KernelDerivativeOrder::Second => p * (p - 1.0),
            KernelDerivativeOrder::Third if power < 3 => 0.0,
            KernelDerivativeOrder::Third => p * (p - 1.0) * (p - 2.0),
        };
        scaled_integer_power(
            radius,
            i32::from(power) - derivative_index(order),
            sign * coefficient,
        )
    } else {
        let log_radius = radius.ln();
        let bracket = match order {
            KernelDerivativeOrder::Value => log_radius,
            KernelDerivativeOrder::First => p.mul_add(log_radius, 1.0),
            KernelDerivativeOrder::Second => (p * (p - 1.0)).mul_add(log_radius, 2.0 * p - 1.0),
            KernelDerivativeOrder::Third => {
                (p * (p - 1.0) * (p - 2.0)).mul_add(log_radius, 3.0 * p * p - 6.0 * p + 2.0)
            }
        };
        scaled_integer_power(
            radius,
            i32::from(power) - derivative_index(order),
            sign * bracket,
        )
    };
    if value.is_finite() {
        Ok(value)
    } else {
        Err(KernelCalculusError::NonFiniteRadialDerivative {
            order: radial_order(order),
            value,
        }
        .into())
    }
}

fn expansion_coefficients(power: u16, radius: f64) -> (f64, f64) {
    let p = f64::from(power);
    let sign = cpd_sign(power);
    if power % 2 == 1 {
        (
            scaled_integer_power(radius, i32::from(power) - 2, sign * p),
            scaled_integer_power(radius, i32::from(power) - 3, sign * p * (p - 2.0)),
        )
    } else {
        let log_radius = radius.ln();
        (
            scaled_integer_power(
                radius,
                i32::from(power) - 2,
                sign * p.mul_add(log_radius, 1.0),
            ),
            scaled_integer_power(
                radius,
                i32::from(power) - 3,
                sign * (p * (p - 2.0)).mul_add(log_radius, 2.0 * (p - 1.0)),
            ),
        )
    }
}

fn scaled_integer_power(radius: f64, exponent: i32, factor: f64) -> f64 {
    if factor == 0.0 {
        return factor;
    }
    let direct = radius.powi(exponent) * factor;
    if direct != 0.0 && direct.is_finite() {
        return direct;
    }
    if !factor.is_finite() {
        return direct;
    }

    // The bare radial power can underflow before a large derivative
    // coefficient brings the final product back into the representable
    // subnormal range. Re-evaluate only this extreme path in the log domain.
    let log_magnitude = f64::from(exponent).mul_add(radius.ln(), factor.abs().ln());
    factor.signum() * log_magnitude.exp()
}

const fn cpd_sign(power: u16) -> f64 {
    if (power / 2 + 1).is_multiple_of(2) {
        1.0
    } else {
        -1.0
    }
}

const fn derivative_index(order: KernelDerivativeOrder) -> i32 {
    match order {
        KernelDerivativeOrder::Value => 0,
        KernelDerivativeOrder::First => 1,
        KernelDerivativeOrder::Second => 2,
        KernelDerivativeOrder::Third => 3,
    }
}

const fn radial_order(order: KernelDerivativeOrder) -> RadialDerivativeOrder {
    match order {
        KernelDerivativeOrder::Value => RadialDerivativeOrder::Value,
        KernelDerivativeOrder::First => RadialDerivativeOrder::First,
        KernelDerivativeOrder::Second => RadialDerivativeOrder::Second,
        KernelDerivativeOrder::Third => RadialDerivativeOrder::Third,
    }
}
