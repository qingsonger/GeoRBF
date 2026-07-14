//! Compactly supported Wendland radial kernels.
//!
//! With `q = r / support_radius` and `t = max(1 - q, 0)`, the catalog is
//!
//! ```text
//! C2: t^4 (1 + 4 q)
//! C4: t^6 (1 + 6 q + 35 q^2 / 3)
//! C6: t^8 (1 + 8 q + 25 q^2 + 32 q^3).
//! ```
//!
//! Each member is normalized to one at the center and uses the exact zero
//! extension for `r >= support_radius`. These are the dimension-three
//! Wendland functions and are strictly positive definite in D=1, D=2, and
//! D=3. Unsupported dimensions fail at compile time:
//!
//! ```compile_fail
//! use georbf::{RadialSeparation, Wendland, WendlandSmoothness};
//!
//! fn unsupported(kernel: Wendland, separation: RadialSeparation<0>) {
//!     let _ = kernel.radial_jet(separation);
//! }
//! ```
//!
//! ```compile_fail
//! use georbf::{RadialSeparation, Wendland};
//!
//! fn unsupported(kernel: Wendland, separation: RadialSeparation<4>) {
//!     let _ = kernel.radial_jet(separation);
//! }
//! ```

use std::error::Error;
use std::fmt;

use crate::dimension::{Dim, SupportedDimension};
use crate::kernel::{
    KernelDefiniteness, KernelDerivativeCapabilities, KernelDerivativeOrder, KernelDimensions,
    KernelMetadata, KernelParameterConstraint, KernelParameterDefinition, KernelParameterUnit,
    KernelParameterValueError, KernelSupport,
};
use crate::kernel_calculus::{
    KernelCalculusError, RadialDerivativeOrder, RadialExpansionCoefficients, RadialJet,
    RadialSeparation,
};

const SUPPORT_RADIUS_PARAMETERS: &[KernelParameterDefinition<'static>] =
    &[KernelParameterDefinition {
        name: "support_radius",
        unit: KernelParameterUnit::CoordinateLength,
        constraint: KernelParameterConstraint::Positive,
        description: "Positive compact-support radius in the active coordinate length unit",
    }];

/// Smoothness member of the dimension-three Wendland catalog.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum WendlandSmoothness {
    /// The `C2` member `(1-q)^4_+ (1+4q)`.
    C2,
    /// The normalized `C4` member `(1-q)^6_+ (1+6q+35q^2/3)`.
    C4,
    /// The `C6` member `(1-q)^8_+ (1+8q+25q^2+32q^3)`.
    C6,
}

impl WendlandSmoothness {
    const fn metadata_name(self) -> &'static str {
        match self {
            Self::C2 => "wendland_c2",
            Self::C4 => "wendland_c4",
            Self::C6 => "wendland_c6",
        }
    }

    const fn center_order(self) -> KernelDerivativeOrder {
        match self {
            Self::C2 => KernelDerivativeOrder::Second,
            Self::C4 | Self::C6 => KernelDerivativeOrder::Third,
        }
    }
}

/// A normalized, compactly supported Wendland kernel.
///
/// The configured support radius has the same physical length unit as radial
/// coordinates. Values and derivatives through the declared capability are
/// identically zero at and beyond that radius.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct Wendland {
    smoothness: WendlandSmoothness,
    scale: SupportScale,
}

impl Wendland {
    /// Constructs one of the supported Wendland members.
    ///
    /// # Errors
    ///
    /// Returns a structured error for a non-positive or non-finite support
    /// radius, or when its reciprocal derivative scale through order three is
    /// not representable.
    pub fn try_new(
        smoothness: WendlandSmoothness,
        support_radius: f64,
    ) -> Result<Self, WendlandConstructionError> {
        Ok(Self {
            smoothness,
            scale: SupportScale::try_new(support_radius)?,
        })
    }

    /// Returns the selected smoothness member.
    #[must_use]
    pub const fn smoothness(self) -> WendlandSmoothness {
        self.smoothness
    }

    /// Returns the positive compact-support radius.
    #[must_use]
    pub const fn support_radius(self) -> f64 {
        self.scale.support_radius
    }

    /// Returns compact-support SPD metadata and exact derivative capabilities.
    pub const fn metadata(self) -> KernelMetadata<'static> {
        KernelMetadata {
            name: self.smoothness.metadata_name(),
            definiteness: KernelDefiniteness::StrictlyPositiveDefinite,
            dimensions: KernelDimensions {
                flags: [true, true, true],
            },
            derivatives: KernelDerivativeCapabilities {
                away_through: KernelDerivativeOrder::Third,
                center_through: Some(self.smoothness.center_order()),
            },
            support: KernelSupport::Compact {
                radius_parameter: "support_radius",
            },
            parameters: SUPPORT_RADIUS_PARAMETERS,
        }
    }

    /// Evaluates the dimensionless radial value.
    ///
    /// The result is exactly positive zero for `radius >= support_radius`.
    ///
    /// # Errors
    ///
    /// Returns a structured error for an invalid radius or non-representable
    /// result.
    pub fn radial_value(self, radius: f64) -> Result<f64, WendlandEvaluationError> {
        evaluate_derivative(
            self.smoothness,
            self.scale,
            radius,
            KernelDerivativeOrder::Value,
        )
        .map(|value| value.unwrap_or(0.0))
    }

    /// Evaluates one spatially valid radial derivative through third order.
    ///
    /// At the center, `C2` returns `None` for third order because its finite
    /// one-sided radial third derivative is not a Euclidean third derivative.
    /// All supported derivatives are exactly positive zero on the support
    /// boundary and exterior branch.
    ///
    /// # Errors
    ///
    /// Returns a structured error for an invalid radius or non-representable
    /// derivative.
    pub fn radial_derivative(
        self,
        radius: f64,
        order: KernelDerivativeOrder,
    ) -> Result<Option<f64>, WendlandEvaluationError> {
        evaluate_derivative(self.smoothness, self.scale, radius, order)
    }

    /// Builds the complete radial jet consumed by Cartesian calculus.
    ///
    /// # Errors
    ///
    /// `C2` rejects a center jet instead of fabricating its unavailable third
    /// spatial derivative. Other invalid or non-representable radial data also
    /// return structured errors.
    pub fn radial_jet<const D: usize>(
        self,
        separation: RadialSeparation<D>,
    ) -> Result<RadialJet, WendlandEvaluationError>
    where
        Dim<D>: SupportedDimension,
    {
        build_jet::<D>(self.smoothness, self.scale, separation)
    }
}

/// Error returned while constructing a Wendland kernel.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WendlandConstructionError {
    /// The support radius failed its positive finite parameter contract.
    InvalidSupportRadius(KernelParameterValueError),
    /// The reciprocal scale needed through third derivative order overflowed.
    NonRepresentableDerivativeScale {
        /// Rejected positive finite support radius.
        support_radius: f64,
    },
}

impl fmt::Display for WendlandConstructionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSupportRadius(source) => {
                write!(formatter, "invalid support radius: {source}")
            }
            Self::NonRepresentableDerivativeScale { support_radius } => write!(
                formatter,
                "support radius {support_radius} has a non-representable reciprocal derivative scale through order three"
            ),
        }
    }
}

impl Error for WendlandConstructionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidSupportRadius(source) => Some(source),
            Self::NonRepresentableDerivativeScale { .. } => None,
        }
    }
}

impl From<KernelParameterValueError> for WendlandConstructionError {
    fn from(source: KernelParameterValueError) -> Self {
        Self::InvalidSupportRadius(source)
    }
}

/// Error returned by Wendland radial evaluation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WendlandEvaluationError {
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
    /// A complete zero-through-third center jet was requested from `C2`.
    CenterJetUnsupported {
        /// Member whose complete center jet is unavailable.
        smoothness: WendlandSmoothness,
        /// Highest spatial derivative supported at the center.
        maximum_center_order: KernelDerivativeOrder,
    },
    /// Shared radial-jet validation rejected a non-finite analytic result.
    KernelCalculus(KernelCalculusError),
}

impl fmt::Display for WendlandEvaluationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteRadius { radius } => {
                write!(formatter, "Wendland radius must be finite, got {radius}")
            }
            Self::NegativeRadius { radius } => write!(
                formatter,
                "Wendland radius must be nonnegative, got {radius}"
            ),
            Self::CenterJetUnsupported {
                smoothness,
                maximum_center_order,
            } => write!(
                formatter,
                "Wendland {smoothness:?} supports center derivatives only through {maximum_center_order:?}, not a complete third-order center jet"
            ),
            Self::KernelCalculus(source) => {
                write!(formatter, "Wendland radial jet is invalid: {source}")
            }
        }
    }
}

impl Error for WendlandEvaluationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::KernelCalculus(source) => Some(source),
            Self::NonFiniteRadius { .. }
            | Self::NegativeRadius { .. }
            | Self::CenterJetUnsupported { .. } => None,
        }
    }
}

impl From<KernelCalculusError> for WendlandEvaluationError {
    fn from(source: KernelCalculusError) -> Self {
        Self::KernelCalculus(source)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct SupportScale {
    support_radius: f64,
    inverse: f64,
    inverse_squared: f64,
    inverse_cubed: f64,
    log_inverse: f64,
}

impl SupportScale {
    fn try_new(support_radius: f64) -> Result<Self, WendlandConstructionError> {
        SUPPORT_RADIUS_PARAMETERS[0].validate_value(support_radius)?;
        let inverse = support_radius.recip();
        let inverse_squared = inverse * inverse;
        let inverse_cubed = inverse_squared * inverse;
        if !inverse.is_finite() || !inverse_squared.is_finite() || !inverse_cubed.is_finite() {
            return Err(WendlandConstructionError::NonRepresentableDerivativeScale {
                support_radius,
            });
        }
        Ok(Self {
            support_radius,
            inverse,
            inverse_squared,
            inverse_cubed,
            log_inverse: inverse.ln(),
        })
    }

    const fn power(self, order: u8) -> f64 {
        match order {
            0 => 1.0,
            1 => self.inverse,
            2 => self.inverse_squared,
            3 => self.inverse_cubed,
            _ => 0.0,
        }
    }
}

#[derive(Clone, Copy)]
struct AwayTerms {
    derivatives: [f64; 4],
    first_over_radius: f64,
    second_remainder_over_radius: f64,
}

impl AwayTerms {
    const ZERO: Self = Self {
        derivatives: [0.0; 4],
        first_over_radius: 0.0,
        second_remainder_over_radius: 0.0,
    };
}

fn validate_radius(radius: f64) -> Result<(), WendlandEvaluationError> {
    if !radius.is_finite() {
        Err(WendlandEvaluationError::NonFiniteRadius { radius })
    } else if radius < 0.0 {
        Err(WendlandEvaluationError::NegativeRadius { radius })
    } else {
        Ok(())
    }
}

fn evaluate_derivative(
    smoothness: WendlandSmoothness,
    scale: SupportScale,
    radius: f64,
    order: KernelDerivativeOrder,
) -> Result<Option<f64>, WendlandEvaluationError> {
    validate_radius(radius)?;
    if radius == 0.0 {
        return Ok(center_derivative(smoothness, scale, order));
    }
    let value = away_terms(smoothness, scale, radius).derivatives[order as usize];
    validate_derivative(order, value)?;
    Ok(Some(value))
}

fn build_jet<const D: usize>(
    smoothness: WendlandSmoothness,
    scale: SupportScale,
    separation: RadialSeparation<D>,
) -> Result<RadialJet, WendlandEvaluationError>
where
    Dim<D>: SupportedDimension,
{
    if separation.is_center() {
        let maximum_center_order = smoothness.center_order();
        if maximum_center_order != KernelDerivativeOrder::Third {
            return Err(WendlandEvaluationError::CenterJetUnsupported {
                smoothness,
                maximum_center_order,
            });
        }
        let second =
            center_derivative(smoothness, scale, KernelDerivativeOrder::Second).unwrap_or(0.0);
        return RadialJet::try_center(1.0, second).map_err(Into::into);
    }

    let terms = away_terms(smoothness, scale, separation.radius());
    if D == 1 {
        return RadialJet::try_away(
            terms.derivatives[0],
            terms.derivatives[1],
            terms.derivatives[2],
            terms.derivatives[3],
        )
        .map_err(Into::into);
    }
    let expansion = RadialExpansionCoefficients::try_new(
        terms.first_over_radius,
        terms.second_remainder_over_radius,
    )?;
    RadialJet::try_away_with_expansion(
        terms.derivatives[0],
        terms.derivatives[1],
        terms.derivatives[2],
        terms.derivatives[3],
        expansion,
    )
    .map_err(Into::into)
}

fn center_derivative(
    smoothness: WendlandSmoothness,
    scale: SupportScale,
    order: KernelDerivativeOrder,
) -> Option<f64> {
    match order {
        KernelDerivativeOrder::Value => Some(1.0),
        KernelDerivativeOrder::First => Some(0.0),
        KernelDerivativeOrder::Second => Some(scaled_component(
            match smoothness {
                WendlandSmoothness::C2 => -20.0,
                WendlandSmoothness::C4 => -56.0 / 3.0,
                WendlandSmoothness::C6 => -22.0,
            },
            1.0,
            0,
            scale,
            2,
        )),
        KernelDerivativeOrder::Third => match smoothness {
            WendlandSmoothness::C2 => None,
            WendlandSmoothness::C4 | WendlandSmoothness::C6 => Some(0.0),
        },
    }
}

fn away_terms(smoothness: WendlandSmoothness, scale: SupportScale, radius: f64) -> AwayTerms {
    if radius >= scale.support_radius {
        return AwayTerms::ZERO;
    }
    let q = radius * scale.inverse;
    let t = if q <= 0.5 {
        1.0 - q
    } else {
        (scale.support_radius - radius) * scale.inverse
    };
    match smoothness {
        WendlandSmoothness::C2 => c2_terms(q, t, scale),
        WendlandSmoothness::C4 => c4_terms(q, t, scale),
        WendlandSmoothness::C6 => c6_terms(q, t, scale),
    }
}

fn c2_terms(q: f64, t: f64, scale: SupportScale) -> AwayTerms {
    AwayTerms {
        derivatives: [
            scaled_component(1.0 + 4.0 * q, t, 4, scale, 0),
            scaled_component(-20.0 * q, t, 3, scale, 1),
            scaled_component(20.0 * (4.0 * q - 1.0), t, 2, scale, 2),
            scaled_component(120.0 * (1.0 - 2.0 * q), t, 1, scale, 3),
        ],
        first_over_radius: scaled_component(-20.0, t, 3, scale, 2),
        second_remainder_over_radius: scaled_component(60.0, t, 2, scale, 3),
    }
}

fn c4_terms(q: f64, t: f64, scale: SupportScale) -> AwayTerms {
    AwayTerms {
        derivatives: [
            scaled_component(1.0 + q * (6.0 + q * (35.0 / 3.0)), t, 6, scale, 0),
            scaled_component(-(56.0 / 3.0) * q * (1.0 + 5.0 * q), t, 5, scale, 1),
            scaled_component(
                -(56.0 / 3.0) * (1.0 + 4.0 * q - 35.0 * q * q),
                t,
                4,
                scale,
                2,
            ),
            scaled_component(560.0 * q * (3.0 - 7.0 * q), t, 3, scale, 3),
        ],
        first_over_radius: scaled_component(-(56.0 / 3.0) * (1.0 + 5.0 * q), t, 5, scale, 2),
        second_remainder_over_radius: scaled_component(560.0 * q, t, 4, scale, 3),
    }
}

fn c6_terms(q: f64, t: f64, scale: SupportScale) -> AwayTerms {
    AwayTerms {
        derivatives: [
            scaled_component(1.0 + q * (8.0 + q * (25.0 + 32.0 * q)), t, 8, scale, 0),
            scaled_component(-22.0 * q * (1.0 + q * (7.0 + 16.0 * q)), t, 7, scale, 1),
            scaled_component(
                -22.0 * (1.0 + q * (6.0 + q * (-15.0 - 160.0 * q))),
                t,
                6,
                scale,
                2,
            ),
            scaled_component(1584.0 * q * (1.0 + q * (5.0 - 20.0 * q)), t, 5, scale, 3),
        ],
        first_over_radius: scaled_component(-22.0 * (1.0 + q * (7.0 + 16.0 * q)), t, 7, scale, 2),
        second_remainder_over_radius: scaled_component(528.0 * q * (1.0 + 6.0 * q), t, 6, scale, 3),
    }
}

fn scaled_component(
    polynomial: f64,
    t: f64,
    boundary_power: i32,
    scale: SupportScale,
    scale_power: u8,
) -> f64 {
    if polynomial == 0.0 || t == 0.0 {
        return 0.0;
    }
    let direct = polynomial * t.powi(boundary_power) * scale.power(scale_power);
    if direct != 0.0 && direct.is_finite() {
        return direct;
    }
    signed_log_value(
        polynomial.abs().ln()
            + f64::from(boundary_power) * t.ln()
            + f64::from(scale_power) * scale.log_inverse,
        polynomial.signum(),
    )
}

fn signed_log_value(log_magnitude: f64, sign: f64) -> f64 {
    sign * log_magnitude.exp()
}

fn validate_derivative(
    order: KernelDerivativeOrder,
    value: f64,
) -> Result<(), WendlandEvaluationError> {
    if value.is_finite() {
        return Ok(());
    }
    let order = match order {
        KernelDerivativeOrder::Value => RadialDerivativeOrder::Value,
        KernelDerivativeOrder::First => RadialDerivativeOrder::First,
        KernelDerivativeOrder::Second => RadialDerivativeOrder::Second,
        KernelDerivativeOrder::Third => RadialDerivativeOrder::Third,
    };
    Err(KernelCalculusError::NonFiniteRadialDerivative { order, value }.into())
}
