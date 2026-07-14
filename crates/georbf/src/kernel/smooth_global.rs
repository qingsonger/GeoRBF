//! Smooth global-support radial kernels with explicit physical length scales.
//!
//! The implemented catalog contains Gaussian, inverse multiquadric, signed
//! multiquadric, and Matérn `1/2`, `3/2`, and `5/2` members. Every constructor
//! accepts one positive `length_scale` in the active coordinate length unit.
//! Arbitrary Matérn smoothness and amplitude parameters are deliberately not
//! part of this requirement.
//!
//! Unsupported dimensions fail at compile time:
//!
//! ```compile_fail
//! use georbf::{Gaussian, RadialSeparation};
//!
//! fn unsupported(kernel: Gaussian, separation: RadialSeparation<0>) {
//!     let _ = kernel.radial_jet(separation);
//! }
//! ```
//!
//! ```compile_fail
//! use georbf::{Matern, RadialSeparation};
//!
//! fn unsupported(kernel: Matern, separation: RadialSeparation<4>) {
//!     let _ = kernel.radial_jet(separation);
//! }
//! ```

use std::error::Error;
use std::fmt;

use crate::dimension::{Dim, SupportedDimension};
use crate::kernel::{
    CpdOrder, KernelDefiniteness, KernelDerivativeCapabilities, KernelDerivativeOrder,
    KernelDimensions, KernelMetadata, KernelParameterConstraint, KernelParameterDefinition,
    KernelParameterUnit, KernelParameterValueError, KernelSupport,
};
use crate::kernel_calculus::{
    KernelCalculusError, RadialDerivativeOrder, RadialExpansionCoefficients, RadialJet,
    RadialSeparation,
};

const LENGTH_SCALE_PARAMETERS: &[KernelParameterDefinition<'static>] =
    &[KernelParameterDefinition {
        name: "length_scale",
        unit: KernelParameterUnit::CoordinateLength,
        constraint: KernelParameterConstraint::Positive,
        description: "Positive radial correlation length in the active coordinate length unit",
    }];

/// Discrete Matérn smoothness members supported by `GeoRBF` v1.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MaternSmoothness {
    /// Exponential Matérn kernel with `nu = 1/2`.
    OneHalf,
    /// Once mean-square differentiable Matérn kernel with `nu = 3/2`.
    ThreeHalves,
    /// Twice mean-square differentiable Matérn kernel with `nu = 5/2`.
    FiveHalves,
}

impl MaternSmoothness {
    const fn scale_multiplier(self) -> f64 {
        match self {
            Self::OneHalf => 1.0,
            Self::ThreeHalves => 1.732_050_807_568_877_2,
            Self::FiveHalves => 2.236_067_977_499_79,
        }
    }

    const fn metadata_name(self) -> &'static str {
        match self {
            Self::OneHalf => "matern_1_2",
            Self::ThreeHalves => "matern_3_2",
            Self::FiveHalves => "matern_5_2",
        }
    }

    const fn center_order(self) -> KernelDerivativeOrder {
        match self {
            Self::OneHalf => KernelDerivativeOrder::Value,
            Self::ThreeHalves => KernelDerivativeOrder::Second,
            Self::FiveHalves => KernelDerivativeOrder::Third,
        }
    }
}

/// Identifies a smooth global-support family in structured diagnostics.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SmoothKernelFamily {
    /// Gaussian/squared-exponential family.
    Gaussian,
    /// Inverse multiquadric family.
    InverseMultiquadric,
    /// CPD-positive signed multiquadric family.
    Multiquadric,
    /// One of the supported Matérn members.
    Matern(MaternSmoothness),
}

/// Strictly positive-definite Gaussian kernel.
///
/// With `q = r / length_scale`, the radial formula is
/// `exp(-q^2 / 2)`. The scaling agrees with the standard Matérn limit.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct Gaussian {
    scale: DerivativeScale,
}

/// Strictly positive-definite inverse multiquadric kernel.
///
/// With `q = r / length_scale`, the radial formula is
/// `(1 + q^2)^(-1/2)`.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct InverseMultiquadric {
    scale: DerivativeScale,
}

/// CPD-positive signed multiquadric kernel.
///
/// With `q = r / length_scale`, the radial formula is
/// `-sqrt(1 + q^2)`. The leading minus sign is intentional: the conventional
/// positive square-root multiquadric is conditionally negative definite on
/// the constant-zero subspace, while `GeoRBF` records positive projected Gram
/// energy and therefore uses CPD order one with the opposite sign.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct Multiquadric {
    scale: DerivativeScale,
}

/// Strictly positive-definite Matérn kernel at a supported half-integer smoothness.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct Matern {
    smoothness: MaternSmoothness,
    scale: DerivativeScale,
}

macro_rules! impl_scaled_kernel {
    ($type:ty, $family:expr, $formula:expr, $name:literal, $definiteness:expr) => {
        impl $type {
            /// Constructs the kernel from a positive physical length scale.
            ///
            /// # Errors
            ///
            /// Returns a structured error for a non-positive or non-finite
            /// scale, or when its reciprocal third-order derivative scale is
            /// not representable.
            pub fn try_new(length_scale: f64) -> Result<Self, SmoothKernelConstructionError> {
                Ok(Self {
                    scale: DerivativeScale::try_new(length_scale, 1.0)?,
                })
            }

            /// Returns the positive physical length scale.
            #[must_use]
            pub const fn length_scale(self) -> f64 {
                self.scale.length_scale
            }

            /// Returns global-support metadata for D=1, D=2, and D=3.
            pub const fn metadata(self) -> KernelMetadata<'static> {
                global_metadata($name, $definiteness, KernelDerivativeOrder::Third)
            }

            /// Evaluates the dimensionless radial value.
            ///
            /// # Errors
            ///
            /// Returns a structured error for an invalid radius or a
            /// non-representable result.
            pub fn radial_value(self, radius: f64) -> Result<f64, SmoothKernelEvaluationError> {
                evaluate_value($family, $formula, self.scale, radius)
            }

            /// Evaluates one spatially valid radial derivative through third order.
            ///
            /// # Errors
            ///
            /// Returns a structured error for an invalid radius or a
            /// non-representable derivative.
            pub fn radial_derivative(
                self,
                radius: f64,
                order: KernelDerivativeOrder,
            ) -> Result<Option<f64>, SmoothKernelEvaluationError> {
                evaluate_derivative($family, $formula, self.scale, radius, order)
            }

            /// Builds the complete radial jet consumed by Cartesian calculus.
            ///
            /// # Errors
            ///
            /// Returns a structured error for invalid or non-representable
            /// radial data.
            pub fn radial_jet<const D: usize>(
                self,
                separation: RadialSeparation<D>,
            ) -> Result<RadialJet, SmoothKernelEvaluationError>
            where
                Dim<D>: SupportedDimension,
            {
                build_jet::<D>($family, $formula, self.scale, separation)
            }
        }
    };
}

impl_scaled_kernel!(
    Gaussian,
    SmoothKernelFamily::Gaussian,
    Formula::Gaussian,
    "gaussian",
    KernelDefiniteness::StrictlyPositiveDefinite
);
impl_scaled_kernel!(
    InverseMultiquadric,
    SmoothKernelFamily::InverseMultiquadric,
    Formula::InverseMultiquadric,
    "inverse_multiquadric",
    KernelDefiniteness::StrictlyPositiveDefinite
);
impl_scaled_kernel!(
    Multiquadric,
    SmoothKernelFamily::Multiquadric,
    Formula::Multiquadric,
    "multiquadric",
    KernelDefiniteness::ConditionallyPositiveDefinite { order: CpdOrder(1) }
);

impl Matern {
    /// Constructs one of the explicitly supported half-integer Matérn members.
    ///
    /// `length_scale` uses the standard `sqrt(2 nu) r / length_scale`
    /// convention.
    ///
    /// # Errors
    ///
    /// Returns a structured error for a non-positive or non-finite scale, or
    /// when the member's reciprocal third-order derivative scale is not
    /// representable.
    pub fn try_new(
        smoothness: MaternSmoothness,
        length_scale: f64,
    ) -> Result<Self, SmoothKernelConstructionError> {
        Ok(Self {
            smoothness,
            scale: DerivativeScale::try_new(length_scale, smoothness.scale_multiplier())?,
        })
    }

    /// Returns the selected half-integer smoothness member.
    #[must_use]
    pub const fn smoothness(self) -> MaternSmoothness {
        self.smoothness
    }

    /// Returns the positive physical length scale.
    #[must_use]
    pub const fn length_scale(self) -> f64 {
        self.scale.length_scale
    }

    /// Returns global-support SPD metadata and the exact center capability.
    pub const fn metadata(self) -> KernelMetadata<'static> {
        global_metadata(
            self.smoothness.metadata_name(),
            KernelDefiniteness::StrictlyPositiveDefinite,
            self.smoothness.center_order(),
        )
    }

    /// Evaluates the dimensionless radial value.
    ///
    /// # Errors
    ///
    /// Returns a structured error for an invalid radius or non-representable result.
    pub fn radial_value(self, radius: f64) -> Result<f64, SmoothKernelEvaluationError> {
        evaluate_value(
            SmoothKernelFamily::Matern(self.smoothness),
            Formula::Matern(self.smoothness),
            self.scale,
            radius,
        )
    }

    /// Evaluates one spatially valid radial derivative through third order.
    ///
    /// At the center, derivatives above the member's Euclidean center
    /// capability return `None` even if a one-sided radial derivative is finite.
    ///
    /// # Errors
    ///
    /// Returns a structured error for an invalid radius or non-representable derivative.
    pub fn radial_derivative(
        self,
        radius: f64,
        order: KernelDerivativeOrder,
    ) -> Result<Option<f64>, SmoothKernelEvaluationError> {
        evaluate_derivative(
            SmoothKernelFamily::Matern(self.smoothness),
            Formula::Matern(self.smoothness),
            self.scale,
            radius,
            order,
        )
    }

    /// Builds a complete radial jet when the selected member supports it.
    ///
    /// # Errors
    ///
    /// Matérn `1/2` and `3/2` reject a complete center jet instead of
    /// fabricating unavailable spatial derivatives.
    pub fn radial_jet<const D: usize>(
        self,
        separation: RadialSeparation<D>,
    ) -> Result<RadialJet, SmoothKernelEvaluationError>
    where
        Dim<D>: SupportedDimension,
    {
        build_jet::<D>(
            SmoothKernelFamily::Matern(self.smoothness),
            Formula::Matern(self.smoothness),
            self.scale,
            separation,
        )
    }
}

/// Error returned while constructing a smooth global-support kernel.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SmoothKernelConstructionError {
    /// The physical length scale failed its positive finite parameter contract.
    InvalidLengthScale(KernelParameterValueError),
    /// The reciprocal scale needed through third derivative order overflowed.
    NonRepresentableDerivativeScale {
        /// Rejected positive finite physical length scale.
        length_scale: f64,
        /// Dimensionless multiplier applied before third-order differentiation.
        multiplier: f64,
    },
}

impl fmt::Display for SmoothKernelConstructionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLengthScale(source) => write!(formatter, "invalid length scale: {source}"),
            Self::NonRepresentableDerivativeScale {
                length_scale,
                multiplier,
            } => write!(
                formatter,
                "length scale {length_scale} with multiplier {multiplier} has a non-representable reciprocal derivative scale through order three"
            ),
        }
    }
}

impl Error for SmoothKernelConstructionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidLengthScale(source) => Some(source),
            Self::NonRepresentableDerivativeScale { .. } => None,
        }
    }
}

impl From<KernelParameterValueError> for SmoothKernelConstructionError {
    fn from(source: KernelParameterValueError) -> Self {
        Self::InvalidLengthScale(source)
    }
}

/// Error returned by smooth global-support radial evaluation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SmoothKernelEvaluationError {
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
    /// A complete zero-through-third center jet was requested from a less smooth member.
    CenterJetUnsupported {
        /// Family whose complete center jet is unavailable.
        family: SmoothKernelFamily,
        /// Highest spatial derivative supported at the center.
        maximum_center_order: KernelDerivativeOrder,
    },
    /// Shared radial-jet validation rejected a non-finite analytic result.
    KernelCalculus(KernelCalculusError),
}

impl fmt::Display for SmoothKernelEvaluationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteRadius { radius } => {
                write!(
                    formatter,
                    "smooth-kernel radius must be finite, got {radius}"
                )
            }
            Self::NegativeRadius { radius } => {
                write!(
                    formatter,
                    "smooth-kernel radius must be nonnegative, got {radius}"
                )
            }
            Self::CenterJetUnsupported {
                family,
                maximum_center_order,
            } => write!(
                formatter,
                "{family:?} supports center derivatives only through {maximum_center_order:?}, not a complete third-order center jet"
            ),
            Self::KernelCalculus(source) => {
                write!(formatter, "smooth-kernel jet is invalid: {source}")
            }
        }
    }
}

impl Error for SmoothKernelEvaluationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::KernelCalculus(source) => Some(source),
            Self::NonFiniteRadius { .. }
            | Self::NegativeRadius { .. }
            | Self::CenterJetUnsupported { .. } => None,
        }
    }
}

impl From<KernelCalculusError> for SmoothKernelEvaluationError {
    fn from(source: KernelCalculusError) -> Self {
        Self::KernelCalculus(source)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct DerivativeScale {
    length_scale: f64,
    inverse: f64,
    inverse_squared: f64,
    inverse_cubed: f64,
    log_inverse: f64,
}

impl DerivativeScale {
    fn try_new(length_scale: f64, multiplier: f64) -> Result<Self, SmoothKernelConstructionError> {
        LENGTH_SCALE_PARAMETERS[0].validate_value(length_scale)?;
        let inverse = multiplier / length_scale;
        let inverse_squared = inverse * inverse;
        let inverse_cubed = inverse_squared * inverse;
        if !inverse.is_finite() || !inverse_squared.is_finite() || !inverse_cubed.is_finite() {
            return Err(
                SmoothKernelConstructionError::NonRepresentableDerivativeScale {
                    length_scale,
                    multiplier,
                },
            );
        }
        Ok(Self {
            length_scale,
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
enum Formula {
    Gaussian,
    InverseMultiquadric,
    Multiquadric,
    Matern(MaternSmoothness),
}

#[derive(Clone, Copy)]
struct AwayTerms {
    derivatives: [f64; 4],
    first_over_radius: f64,
    second_remainder_over_radius: f64,
}

const fn global_metadata(
    name: &'static str,
    definiteness: KernelDefiniteness,
    center_through: KernelDerivativeOrder,
) -> KernelMetadata<'static> {
    KernelMetadata {
        name,
        definiteness,
        dimensions: KernelDimensions {
            flags: [true, true, true],
        },
        derivatives: KernelDerivativeCapabilities {
            away_through: KernelDerivativeOrder::Third,
            center_through: Some(center_through),
        },
        support: KernelSupport::Global,
        parameters: LENGTH_SCALE_PARAMETERS,
    }
}

fn validate_radius(radius: f64) -> Result<(), SmoothKernelEvaluationError> {
    if !radius.is_finite() {
        Err(SmoothKernelEvaluationError::NonFiniteRadius { radius })
    } else if radius < 0.0 {
        Err(SmoothKernelEvaluationError::NegativeRadius { radius })
    } else {
        Ok(())
    }
}

fn evaluate_value(
    family: SmoothKernelFamily,
    formula: Formula,
    scale: DerivativeScale,
    radius: f64,
) -> Result<f64, SmoothKernelEvaluationError> {
    evaluate_derivative(family, formula, scale, radius, KernelDerivativeOrder::Value)
        .map(|value| value.unwrap_or(0.0))
}

fn evaluate_derivative(
    _family: SmoothKernelFamily,
    formula: Formula,
    scale: DerivativeScale,
    radius: f64,
    order: KernelDerivativeOrder,
) -> Result<Option<f64>, SmoothKernelEvaluationError> {
    validate_radius(radius)?;
    if radius == 0.0 {
        return Ok(center_derivative(formula, scale, order));
    }
    let value = away_terms(formula, scale, radius).derivatives[order as usize];
    validate_derivative(order, value)?;
    Ok(Some(value))
}

fn build_jet<const D: usize>(
    family: SmoothKernelFamily,
    formula: Formula,
    scale: DerivativeScale,
    separation: RadialSeparation<D>,
) -> Result<RadialJet, SmoothKernelEvaluationError>
where
    Dim<D>: SupportedDimension,
{
    if separation.is_center() {
        let maximum_center_order = center_order(formula);
        if maximum_center_order != KernelDerivativeOrder::Third {
            return Err(SmoothKernelEvaluationError::CenterJetUnsupported {
                family,
                maximum_center_order,
            });
        }
        let value = center_derivative(formula, scale, KernelDerivativeOrder::Value).unwrap_or(0.0);
        let second =
            center_derivative(formula, scale, KernelDerivativeOrder::Second).unwrap_or(0.0);
        return RadialJet::try_center(value, second).map_err(Into::into);
    }

    let terms = away_terms(formula, scale, separation.radius());
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

const fn center_order(formula: Formula) -> KernelDerivativeOrder {
    match formula {
        Formula::Matern(smoothness) => smoothness.center_order(),
        Formula::Gaussian | Formula::InverseMultiquadric | Formula::Multiquadric => {
            KernelDerivativeOrder::Third
        }
    }
}

fn center_derivative(
    formula: Formula,
    scale: DerivativeScale,
    order: KernelDerivativeOrder,
) -> Option<f64> {
    match formula {
        Formula::Gaussian | Formula::InverseMultiquadric => match order {
            KernelDerivativeOrder::Value => Some(1.0),
            KernelDerivativeOrder::First | KernelDerivativeOrder::Third => Some(0.0),
            KernelDerivativeOrder::Second => Some(-scale.inverse_squared),
        },
        Formula::Multiquadric => match order {
            KernelDerivativeOrder::Value => Some(-1.0),
            KernelDerivativeOrder::First | KernelDerivativeOrder::Third => Some(0.0),
            KernelDerivativeOrder::Second => Some(-scale.inverse_squared),
        },
        Formula::Matern(MaternSmoothness::OneHalf) => match order {
            KernelDerivativeOrder::Value => Some(1.0),
            KernelDerivativeOrder::First
            | KernelDerivativeOrder::Second
            | KernelDerivativeOrder::Third => None,
        },
        Formula::Matern(MaternSmoothness::ThreeHalves) => match order {
            KernelDerivativeOrder::Value => Some(1.0),
            KernelDerivativeOrder::First => Some(0.0),
            KernelDerivativeOrder::Second => Some(-scale.inverse_squared),
            KernelDerivativeOrder::Third => None,
        },
        Formula::Matern(MaternSmoothness::FiveHalves) => match order {
            KernelDerivativeOrder::Value => Some(1.0),
            KernelDerivativeOrder::First | KernelDerivativeOrder::Third => Some(0.0),
            KernelDerivativeOrder::Second => Some(-scale.inverse_squared / 3.0),
        },
    }
}

fn away_terms(formula: Formula, scale: DerivativeScale, radius: f64) -> AwayTerms {
    match formula {
        Formula::Gaussian => gaussian_terms(scale, radius),
        Formula::InverseMultiquadric => inverse_multiquadric_terms(scale, radius),
        Formula::Multiquadric => multiquadric_terms(scale, radius),
        Formula::Matern(smoothness) => matern_terms(smoothness, scale, radius),
    }
}

fn gaussian_terms(scale: DerivativeScale, radius: f64) -> AwayTerms {
    let q = radius * scale.inverse;
    if !q.is_finite() || q >= 64.0 {
        return AwayTerms {
            derivatives: [0.0; 4],
            first_over_radius: -0.0,
            second_remainder_over_radius: 0.0,
        };
    }
    let q_squared = q * q;
    let exponent = -0.5 * q_squared;
    AwayTerms {
        derivatives: [
            scaled_exponential(exponent, 1.0, scale, 0),
            scaled_exponential(exponent, -q, scale, 1),
            scaled_exponential(exponent, q_squared - 1.0, scale, 2),
            scaled_exponential(exponent, q * (3.0 - q_squared), scale, 3),
        ],
        first_over_radius: scaled_exponential(exponent, -1.0, scale, 2),
        second_remainder_over_radius: scaled_exponential(exponent, q, scale, 3),
    }
}

fn inverse_multiquadric_terms(scale: DerivativeScale, radius: f64) -> AwayTerms {
    let state = RationalState::new(scale, radius);
    let u_squared = state.u * state.u;
    AwayTerms {
        derivatives: [
            state.scaled(1.0, 0, 1),
            state.scaled(-state.u, 1, 2),
            state.scaled(3.0 * u_squared - 1.0, 2, 3),
            state.scaled(3.0 * state.u * (3.0 - 5.0 * u_squared), 3, 4),
        ],
        first_over_radius: state.scaled(-1.0, 2, 3),
        second_remainder_over_radius: state.scaled(3.0 * state.u, 3, 4),
    }
}

fn multiquadric_terms(scale: DerivativeScale, radius: f64) -> AwayTerms {
    let state = RationalState::new(scale, radius);
    let value = if state.dimensionless.is_finite() {
        -state.dimensionless.hypot(1.0)
    } else {
        f64::NEG_INFINITY
    };
    AwayTerms {
        derivatives: [
            value,
            state.scaled(-state.u, 1, 0),
            state.scaled(-1.0, 2, 3),
            state.scaled(3.0 * state.u, 3, 4),
        ],
        first_over_radius: state.scaled(-1.0, 2, 1),
        second_remainder_over_radius: state.scaled(state.u, 3, 2),
    }
}

fn matern_terms(smoothness: MaternSmoothness, scale: DerivativeScale, radius: f64) -> AwayTerms {
    let t = radius * scale.inverse;
    if !t.is_finite() || t >= 2048.0 {
        return AwayTerms {
            derivatives: [0.0; 4],
            first_over_radius: -0.0,
            second_remainder_over_radius: 0.0,
        };
    }
    let exponent = -t;
    match smoothness {
        MaternSmoothness::OneHalf => {
            let first_over_radius =
                signed_log_value(exponent + scale.log_inverse - radius.ln(), -1.0);
            let second_remainder_over_radius = signed_log_value(
                exponent + scale.log_inverse + t.ln_1p() - 2.0 * radius.ln(),
                1.0,
            );
            AwayTerms {
                derivatives: [
                    scaled_exponential(exponent, 1.0, scale, 0),
                    scaled_exponential(exponent, -1.0, scale, 1),
                    scaled_exponential(exponent, 1.0, scale, 2),
                    scaled_exponential(exponent, -1.0, scale, 3),
                ],
                first_over_radius,
                second_remainder_over_radius,
            }
        }
        MaternSmoothness::ThreeHalves => AwayTerms {
            derivatives: [
                scaled_exponential(exponent, 1.0 + t, scale, 0),
                scaled_exponential(exponent, -t, scale, 1),
                scaled_exponential(exponent, t - 1.0, scale, 2),
                scaled_exponential(exponent, 2.0 - t, scale, 3),
            ],
            first_over_radius: scaled_exponential(exponent, -1.0, scale, 2),
            second_remainder_over_radius: scaled_exponential(exponent, 1.0, scale, 3),
        },
        MaternSmoothness::FiveHalves => {
            let t_squared = t * t;
            AwayTerms {
                derivatives: [
                    scaled_exponential(exponent, 1.0 + t + t_squared / 3.0, scale, 0),
                    scaled_exponential(exponent, -t * (1.0 + t) / 3.0, scale, 1),
                    scaled_exponential(exponent, (t_squared - t - 1.0) / 3.0, scale, 2),
                    scaled_exponential(exponent, t * (3.0 - t) / 3.0, scale, 3),
                ],
                first_over_radius: scaled_exponential(exponent, -(1.0 + t) / 3.0, scale, 2),
                second_remainder_over_radius: scaled_exponential(exponent, t / 3.0, scale, 3),
            }
        }
    }
}

fn scaled_exponential(
    exponent: f64,
    polynomial: f64,
    scale: DerivativeScale,
    scale_power: u8,
) -> f64 {
    if polynomial == 0.0 {
        return polynomial;
    }
    let direct = exponent.exp() * (polynomial * scale.power(scale_power));
    if direct != 0.0 && direct.is_finite() {
        return direct;
    }
    signed_log_value(
        exponent + polynomial.abs().ln() + f64::from(scale_power) * scale.log_inverse,
        polynomial.signum(),
    )
}

fn signed_log_value(log_magnitude: f64, sign: f64) -> f64 {
    sign * log_magnitude.exp()
}

struct RationalState {
    scale: DerivativeScale,
    dimensionless: f64,
    u: f64,
    inverse_hypot: f64,
    log_inverse_hypot: f64,
}

impl RationalState {
    fn new(scale: DerivativeScale, radius: f64) -> Self {
        let dimensionless = radius * scale.inverse;
        if dimensionless.is_finite() {
            let hypot = dimensionless.hypot(1.0);
            let inverse_hypot = hypot.recip();
            Self {
                scale,
                dimensionless,
                u: dimensionless / hypot,
                inverse_hypot,
                log_inverse_hypot: -hypot.ln(),
            }
        } else {
            let log_dimensionless = radius.ln() + scale.log_inverse;
            Self {
                scale,
                dimensionless,
                u: 1.0,
                inverse_hypot: 0.0,
                log_inverse_hypot: -log_dimensionless,
            }
        }
    }

    fn scaled(&self, polynomial: f64, scale_power: u8, inverse_hypot_power: i32) -> f64 {
        if polynomial == 0.0 {
            return polynomial;
        }
        let direct = polynomial
            * self.scale.power(scale_power)
            * self.inverse_hypot.powi(inverse_hypot_power);
        if direct != 0.0 && direct.is_finite() {
            return direct;
        }
        signed_log_value(
            polynomial.abs().ln()
                + f64::from(scale_power) * self.scale.log_inverse
                + f64::from(inverse_hypot_power) * self.log_inverse_hypot,
            polynomial.signum(),
        )
    }
}

fn validate_derivative(
    order: KernelDerivativeOrder,
    value: f64,
) -> Result<(), SmoothKernelEvaluationError> {
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
