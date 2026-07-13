//! Dimension-generic Cartesian derivatives for stationary radial kernels.
//!
//! This module expands a supplied radial jet for `k(x, y) = phi(||x-y||)`.
//! It does not define concrete kernels, parameters, or definiteness metadata.
//! Unsupported dimensions fail at compile time:
//!
//! ```compile_fail
//! use georbf::RadialSeparation;
//!
//! fn unsupported(_: Option<RadialSeparation<0>>) {}
//! ```
//!
//! ```compile_fail
//! use georbf::SpatialKernelJet;
//!
//! fn unsupported(_: Option<SpatialKernelJet<4>>) {}
//! ```

use std::error::Error;
use std::fmt;

use crate::dimension::{Dim, SupportedDimension};
use crate::geometry::Point;

/// Which kernel argument a Cartesian derivative acts on.
///
/// For a stationary radial kernel with displacement `d = x - y`, a query
/// derivative is a displacement derivative and a center derivative contributes
/// one minus sign.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum KernelArgument {
    /// The query argument `x`.
    Query,
    /// The center argument `y`.
    Center,
}

/// Radial derivative identified by its order.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RadialDerivativeOrder {
    /// The radial value `phi(r)`.
    Value,
    /// The first radial derivative `phi'(r)`.
    First,
    /// The second radial derivative `phi''(r)`.
    Second,
    /// The third radial derivative `phi'''(r)`.
    Third,
}

/// Stable radial coefficient used by the Cartesian tensor expansion.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RadialExpansionCoefficient {
    /// The coefficient `phi'(r) / r` used by the tangential Hessian term.
    FirstOverRadius,
    /// The coefficient `(phi''(r) - phi'(r) / r) / r` used by the third tensor.
    SecondRemainderOverRadius,
}

/// Whether a radial jet describes a center or a positive-radius point pair.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RadialJetLocation {
    /// Coincident points with `r = 0` and an analytic Euclidean center limit.
    Center,
    /// Distinct points with a finite, positive radius.
    AwayFromCenter,
}

/// Error returned by radial kernel derivative validation or expansion.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum KernelCalculusError {
    /// A radial value or derivative is NaN or infinite.
    NonFiniteRadialDerivative {
        /// Rejected derivative order.
        order: RadialDerivativeOrder,
        /// Rejected value.
        value: f64,
    },
    /// A stable radial expansion coefficient is NaN or infinite.
    NonFiniteRadialExpansionCoefficient {
        /// Rejected coefficient.
        coefficient: RadialExpansionCoefficient,
        /// Rejected value.
        value: f64,
    },
    /// D=2 or D=3 expansion was requested without stable radial coefficients.
    MissingRadialExpansionCoefficients {
        /// Spatial dimension that requires the coefficients.
        dimension: usize,
    },
    /// Subtracting query and center coordinates produced a non-finite value.
    NonFiniteDisplacementComponent {
        /// Coordinate axis of the failed subtraction.
        axis: usize,
    },
    /// The Euclidean radius is larger than the finite `f64` domain.
    NonRepresentableRadius,
    /// A center jet was paired with distinct points or an away jet with a center.
    JetLocationMismatch {
        /// Location determined from the point separation.
        separation: RadialJetLocation,
        /// Location declared by the radial jet.
        jet: RadialJetLocation,
    },
    /// A first spatial derivative could not be represented as a finite `f64`.
    NonFiniteFirstDerivative {
        /// Derivative axis.
        axis: usize,
    },
    /// A second spatial derivative could not be represented as a finite `f64`.
    NonFiniteSecondDerivative {
        /// First derivative axis.
        row: usize,
        /// Second derivative axis.
        column: usize,
    },
    /// A third spatial derivative could not be represented as a finite `f64`.
    NonFiniteThirdDerivative {
        /// First derivative axis.
        first: usize,
        /// Second derivative axis.
        second: usize,
        /// Third derivative axis.
        third: usize,
    },
}

impl fmt::Display for KernelCalculusError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteRadialDerivative { order, value } => {
                write!(
                    formatter,
                    "radial derivative {order:?} must be finite, got {value}"
                )
            }
            Self::NonFiniteRadialExpansionCoefficient { coefficient, value } => write!(
                formatter,
                "radial expansion coefficient {coefficient:?} must be finite, got {value}"
            ),
            Self::MissingRadialExpansionCoefficients { dimension } => write!(
                formatter,
                "radial expansion coefficients are required in D={dimension}"
            ),
            Self::NonFiniteDisplacementComponent { axis } => write!(
                formatter,
                "query-center displacement on axis {axis} is not finite"
            ),
            Self::NonRepresentableRadius => {
                formatter.write_str("query-center radius is not finitely representable")
            }
            Self::JetLocationMismatch { separation, jet } => write!(
                formatter,
                "point separation is {separation:?}, but radial jet is {jet:?}"
            ),
            Self::NonFiniteFirstDerivative { axis } => write!(
                formatter,
                "first spatial derivative on axis {axis} is not finite"
            ),
            Self::NonFiniteSecondDerivative { row, column } => write!(
                formatter,
                "second spatial derivative ({row}, {column}) is not finite"
            ),
            Self::NonFiniteThirdDerivative {
                first,
                second,
                third,
            } => write!(
                formatter,
                "third spatial derivative ({first}, {second}, {third}) is not finite"
            ),
        }
    }
}

impl Error for KernelCalculusError {}

/// Finite, cancellation-resistant coefficients for an away-from-center jet.
///
/// For positive `r`, these values are
///
/// ```text
/// a(r) = phi'(r) / r
/// b(r) = (phi''(r) - a(r)) / r.
/// ```
///
/// A concrete radial implementation supplies `a` and `b` from stable closed
/// forms. In particular, it must not reconstruct `b` by subtracting two nearly
/// equal rounded values close to the center. The kernel-calculus layer does not
/// know a kernel formula from which it could recover that lost information.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct RadialExpansionCoefficients {
    first_over_radius: f64,
    second_remainder_over_radius: f64,
}

impl RadialExpansionCoefficients {
    /// Constructs finite away-from-center expansion coefficients.
    ///
    /// # Errors
    ///
    /// Returns
    /// [`KernelCalculusError::NonFiniteRadialExpansionCoefficient`] for the
    /// first invalid coefficient.
    pub fn try_new(
        first_over_radius: f64,
        second_remainder_over_radius: f64,
    ) -> Result<Self, KernelCalculusError> {
        validate_radial_expansion_coefficient(
            RadialExpansionCoefficient::FirstOverRadius,
            first_over_radius,
        )?;
        validate_radial_expansion_coefficient(
            RadialExpansionCoefficient::SecondRemainderOverRadius,
            second_remainder_over_radius,
        )?;
        Ok(Self {
            first_over_radius,
            second_remainder_over_radius,
        })
    }

    /// Returns `phi'(r) / r`.
    #[must_use]
    pub const fn first_over_radius(&self) -> f64 {
        self.first_over_radius
    }

    /// Returns `(phi''(r) - phi'(r) / r) / r`.
    #[must_use]
    pub const fn second_remainder_over_radius(&self) -> f64 {
        self.second_remainder_over_radius
    }
}

/// Finite radial values required for zero-through-third spatial derivatives.
///
/// An away jet stores `phi(r)` through `phi'''(r)` for a positive radius and
/// may carry the stable Cartesian expansion coefficients required in D=2 and
/// D=3. D=1 needs no radial quotient. A center jet explicitly promises a
/// smooth Euclidean extension through third spatial order: its gradient and
/// third spatial tensor are zero and its Hessian is `phi''(0) I`. Kernel
/// capability checks remain the responsibility of the later kernel metadata
/// requirement.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct RadialJet {
    location: RadialJetLocation,
    derivatives: [f64; 4],
    expansion_coefficients: Option<RadialExpansionCoefficients>,
}

impl RadialJet {
    /// Constructs an away-from-center jet from finite radial derivatives.
    ///
    /// This data is sufficient for D=1. D=2 and D=3 expansion returns
    /// [`KernelCalculusError::MissingRadialExpansionCoefficients`]; use
    /// [`Self::try_away_with_expansion`] there.
    ///
    /// # Errors
    ///
    /// Returns [`KernelCalculusError::NonFiniteRadialDerivative`] for the first
    /// invalid radial value in order zero through three.
    pub fn try_away(
        value: f64,
        first: f64,
        second: f64,
        third: f64,
    ) -> Result<Self, KernelCalculusError> {
        let derivatives = [value, first, second, third];
        validate_radial_derivatives(&derivatives)?;
        Ok(Self {
            location: RadialJetLocation::AwayFromCenter,
            derivatives,
            expansion_coefficients: None,
        })
    }

    /// Constructs an away jet with stable D=2/D=3 expansion coefficients.
    ///
    /// The derivatives and coefficients must be evaluated at the same positive
    /// radius. Concrete radial implementations compute both coefficients from
    /// cancellation-resistant closed forms.
    ///
    /// # Errors
    ///
    /// Returns [`KernelCalculusError::NonFiniteRadialDerivative`] for the first
    /// invalid radial value in order zero through three. The coefficient type
    /// validates its own finite-value invariant before this call.
    pub fn try_away_with_expansion(
        value: f64,
        first: f64,
        second: f64,
        third: f64,
        expansion_coefficients: RadialExpansionCoefficients,
    ) -> Result<Self, KernelCalculusError> {
        let derivatives = [value, first, second, third];
        validate_radial_derivatives(&derivatives)?;
        Ok(Self {
            location: RadialJetLocation::AwayFromCenter,
            derivatives,
            expansion_coefficients: Some(expansion_coefficients),
        })
    }

    /// Constructs an analytic smooth-center jet.
    ///
    /// The first and third radial derivatives at the center are encoded as
    /// zero, which is required by the promised smooth Euclidean extension.
    ///
    /// # Errors
    ///
    /// Returns [`KernelCalculusError::NonFiniteRadialDerivative`] when `value`
    /// or `second` is invalid.
    pub fn try_center(value: f64, second: f64) -> Result<Self, KernelCalculusError> {
        validate_radial_derivative(RadialDerivativeOrder::Value, value)?;
        validate_radial_derivative(RadialDerivativeOrder::Second, second)?;
        Ok(Self {
            location: RadialJetLocation::Center,
            derivatives: [value, 0.0, second, 0.0],
            expansion_coefficients: None,
        })
    }

    /// Returns whether this is a center or positive-radius jet.
    #[must_use]
    pub const fn location(&self) -> RadialJetLocation {
        self.location
    }

    /// Returns `phi(r)`.
    #[must_use]
    pub const fn value(&self) -> f64 {
        self.derivatives[0]
    }

    /// Returns `phi'(r)`, which is zero for a smooth-center jet.
    #[must_use]
    pub const fn first_derivative(&self) -> f64 {
        self.derivatives[1]
    }

    /// Returns `phi''(r)`.
    #[must_use]
    pub const fn second_derivative(&self) -> f64 {
        self.derivatives[2]
    }

    /// Returns `phi'''(r)`, which is zero for a smooth-center jet.
    #[must_use]
    pub const fn third_derivative(&self) -> f64 {
        self.derivatives[3]
    }

    /// Returns the stable expansion coefficients for an away jet.
    ///
    /// A center jet returns `None` because it uses direct analytic limits.
    #[must_use]
    pub const fn expansion_coefficients(&self) -> Option<&RadialExpansionCoefficients> {
        self.expansion_coefficients.as_ref()
    }
}

/// Finite Euclidean separation for a query and center point.
///
/// The radius and optional unit displacement are computed by maximum-component
/// scaling. This avoids overflow or underflow in the intermediate squared norm.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct RadialSeparation<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    radius: f64,
    unit_displacement: Option<[f64; D]>,
}

impl<const D: usize> RadialSeparation<D>
where
    Dim<D>: SupportedDimension,
{
    /// Computes `d = x - y`, `r = ||d||`, and `u = d/r` when `r > 0`.
    ///
    /// # Errors
    ///
    /// Returns [`KernelCalculusError::NonFiniteDisplacementComponent`] when a
    /// finite point subtraction overflows, or
    /// [`KernelCalculusError::NonRepresentableRadius`] when the radius itself
    /// is outside the finite `f64` domain.
    pub fn try_new(query: Point<D>, center: Point<D>) -> Result<Self, KernelCalculusError> {
        let mut displacement = [0.0; D];
        for (axis, value) in displacement.iter_mut().enumerate() {
            *value = query.components()[axis] - center.components()[axis];
            if !value.is_finite() {
                return Err(KernelCalculusError::NonFiniteDisplacementComponent { axis });
            }
        }

        let scale = displacement
            .iter()
            .copied()
            .map(f64::abs)
            .fold(0.0_f64, f64::max);
        if scale == 0.0 {
            return Ok(Self {
                radius: 0.0,
                unit_displacement: None,
            });
        }

        let scaled_squared_norm = displacement
            .iter()
            .map(|component| {
                let scaled = component / scale;
                scaled * scaled
            })
            .sum::<f64>();
        let scaled_norm = scaled_squared_norm.sqrt();
        let radius = scale * scaled_norm;
        if !radius.is_finite() {
            return Err(KernelCalculusError::NonRepresentableRadius);
        }
        let unit_displacement = displacement.map(|component| (component / scale) / scaled_norm);
        Ok(Self {
            radius,
            unit_displacement: Some(unit_displacement),
        })
    }

    /// Returns the finite nonnegative radius `||x-y||`.
    #[must_use]
    pub const fn radius(&self) -> f64 {
        self.radius
    }

    /// Returns whether the query and center coincide exactly.
    #[must_use]
    pub const fn is_center(&self) -> bool {
        self.unit_displacement.is_none()
    }

    /// Borrows `u = (x-y)/||x-y||`, or returns `None` at a center.
    #[must_use]
    pub const fn unit_displacement(&self) -> Option<&[f64; D]> {
        self.unit_displacement.as_ref()
    }

    fn location(&self) -> RadialJetLocation {
        if self.is_center() {
            RadialJetLocation::Center
        } else {
            RadialJetLocation::AwayFromCenter
        }
    }
}

/// Cartesian value and derivative tensors expanded from a radial jet.
///
/// Stored tensors are derivatives with respect to the query argument `x`.
/// Accessors apply one exact minus sign for each requested center argument.
/// Arrays are fixed-size, and construction performs no heap allocation or
/// dynamic dispatch.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct SpatialKernelJet<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    value: f64,
    first: [f64; D],
    second: [[f64; D]; D],
    third: [[[f64; D]; D]; D],
}

impl<const D: usize> SpatialKernelJet<D>
where
    Dim<D>: SupportedDimension,
{
    /// Expands a radial jet for the supplied query-center separation.
    ///
    /// At a center, construction uses the analytic limits directly and never
    /// evaluates an expression containing `1/r`.
    ///
    /// # Errors
    ///
    /// Returns a location mismatch when the separation and radial jet disagree,
    /// or a structured derivative error when a Cartesian component is not
    /// finitely representable.
    pub fn try_new(
        separation: RadialSeparation<D>,
        radial: RadialJet,
    ) -> Result<Self, KernelCalculusError> {
        let separation_location = separation.location();
        if separation_location != radial.location() {
            return Err(KernelCalculusError::JetLocationMismatch {
                separation: separation_location,
                jet: radial.location(),
            });
        }

        if separation.is_center() {
            Ok(center_spatial_jet(radial))
        } else {
            away_spatial_jet(separation, radial)
        }
    }

    /// Returns the kernel value.
    #[must_use]
    pub const fn value(&self) -> f64 {
        self.value
    }

    /// Returns the first derivative for the selected kernel argument.
    #[must_use]
    pub fn first_derivative(&self, argument: KernelArgument) -> [f64; D] {
        apply_first_sign(self.first, argument)
    }

    /// Returns the second derivative tensor for the selected arguments.
    #[must_use]
    pub fn second_derivative(&self, arguments: [KernelArgument; 2]) -> [[f64; D]; D] {
        apply_second_sign(self.second, arguments_have_negative_sign(&arguments))
    }

    /// Returns the third derivative tensor for the selected arguments.
    #[must_use]
    pub fn third_derivative(&self, arguments: [KernelArgument; 3]) -> [[[f64; D]; D]; D] {
        apply_third_sign(self.third, arguments_have_negative_sign(&arguments))
    }
}

fn center_spatial_jet<const D: usize>(radial: RadialJet) -> SpatialKernelJet<D>
where
    Dim<D>: SupportedDimension,
{
    SpatialKernelJet {
        value: radial.value(),
        first: [0.0; D],
        second: std::array::from_fn(|row| {
            std::array::from_fn(|column| {
                if row == column {
                    radial.second_derivative()
                } else {
                    0.0
                }
            })
        }),
        third: [[[0.0; D]; D]; D],
    }
}

fn away_spatial_jet<const D: usize>(
    separation: RadialSeparation<D>,
    radial: RadialJet,
) -> Result<SpatialKernelJet<D>, KernelCalculusError>
where
    Dim<D>: SupportedDimension,
{
    let Some(unit) = separation.unit_displacement else {
        return Err(KernelCalculusError::JetLocationMismatch {
            separation: RadialJetLocation::Center,
            jet: radial.location(),
        });
    };
    let first = away_first_derivative(&radial, &unit)?;
    let expansion_coefficients = match radial.expansion_coefficients {
        Some(coefficients) => coefficients,
        None if D == 1 => RadialExpansionCoefficients {
            first_over_radius: 0.0,
            second_remainder_over_radius: 0.0,
        },
        None => {
            return Err(KernelCalculusError::MissingRadialExpansionCoefficients { dimension: D });
        }
    };
    let second = away_second_derivative(&radial, &expansion_coefficients, &unit)?;
    let third = away_third_derivative(&radial, &expansion_coefficients, &unit)?;
    Ok(SpatialKernelJet {
        value: radial.value(),
        first,
        second,
        third,
    })
}

fn away_first_derivative<const D: usize>(
    radial: &RadialJet,
    unit: &[f64; D],
) -> Result<[f64; D], KernelCalculusError> {
    let mut first = [0.0; D];
    for (axis, output) in first.iter_mut().enumerate() {
        *output = radial.first_derivative() * unit[axis];
        if !output.is_finite() {
            return Err(KernelCalculusError::NonFiniteFirstDerivative { axis });
        }
    }
    Ok(first)
}

fn away_second_derivative<const D: usize>(
    radial: &RadialJet,
    expansion_coefficients: &RadialExpansionCoefficients,
    unit: &[f64; D],
) -> Result<[[f64; D]; D], KernelCalculusError> {
    let mut second = [[0.0; D]; D];
    for row in 0..D {
        for column in row..D {
            let unit_product = unit[row] * unit[column];
            let tangential_factor = f64::from(row == column) - unit_product;
            let radial_part = radial.second_derivative() * unit_product;
            let value = expansion_coefficients
                .first_over_radius()
                .mul_add(tangential_factor, radial_part);
            if !value.is_finite() {
                return Err(KernelCalculusError::NonFiniteSecondDerivative { row, column });
            }
            second[row][column] = value;
            second[column][row] = value;
        }
    }
    Ok(second)
}

fn away_third_derivative<const D: usize>(
    radial: &RadialJet,
    expansion_coefficients: &RadialExpansionCoefficients,
    unit: &[f64; D],
) -> Result<[[[f64; D]; D]; D], KernelCalculusError> {
    let mut third = [[[0.0; D]; D]; D];
    for first_axis in 0..D {
        for second_axis in first_axis..D {
            for third_axis in second_axis..D {
                let value = away_third_component(
                    radial,
                    expansion_coefficients,
                    unit,
                    first_axis,
                    second_axis,
                    third_axis,
                )?;
                set_symmetric_third(&mut third, first_axis, second_axis, third_axis, value);
            }
        }
    }
    Ok(third)
}

fn away_third_component<const D: usize>(
    radial: &RadialJet,
    expansion_coefficients: &RadialExpansionCoefficients,
    unit: &[f64; D],
    first: usize,
    second: usize,
    third: usize,
) -> Result<f64, KernelCalculusError> {
    let error = KernelCalculusError::NonFiniteThirdDerivative {
        first,
        second,
        third,
    };
    let unit_product = unit[first] * unit[second] * unit[third];
    let delta_sum = if first == second { unit[third] } else { 0.0 }
        + if first == third { unit[second] } else { 0.0 }
        + if second == third { unit[first] } else { 0.0 };
    let correction_factor = delta_sum - 3.0 * unit_product;
    let radial_part = radial.third_derivative() * unit_product;
    let value = expansion_coefficients
        .second_remainder_over_radius()
        .mul_add(correction_factor, radial_part);
    if value.is_finite() {
        Ok(value)
    } else {
        Err(error)
    }
}

fn validate_radial_derivatives(derivatives: &[f64; 4]) -> Result<(), KernelCalculusError> {
    for (index, value) in derivatives.iter().copied().enumerate() {
        let order = match index {
            0 => RadialDerivativeOrder::Value,
            1 => RadialDerivativeOrder::First,
            2 => RadialDerivativeOrder::Second,
            _ => RadialDerivativeOrder::Third,
        };
        validate_radial_derivative(order, value)?;
    }
    Ok(())
}

fn validate_radial_derivative(
    order: RadialDerivativeOrder,
    value: f64,
) -> Result<(), KernelCalculusError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(KernelCalculusError::NonFiniteRadialDerivative { order, value })
    }
}

fn validate_radial_expansion_coefficient(
    coefficient: RadialExpansionCoefficient,
    value: f64,
) -> Result<(), KernelCalculusError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(KernelCalculusError::NonFiniteRadialExpansionCoefficient { coefficient, value })
    }
}

fn set_symmetric_third<const D: usize>(
    tensor: &mut [[[f64; D]; D]; D],
    first: usize,
    second: usize,
    third: usize,
    value: f64,
) {
    for [i, j, k] in [
        [first, second, third],
        [first, third, second],
        [second, first, third],
        [second, third, first],
        [third, first, second],
        [third, second, first],
    ] {
        tensor[i][j][k] = value;
    }
}

fn apply_first_sign<const D: usize>(tensor: [f64; D], argument: KernelArgument) -> [f64; D] {
    if argument == KernelArgument::Query {
        tensor
    } else {
        tensor.map(|value| -value)
    }
}

fn apply_second_sign<const D: usize>(tensor: [[f64; D]; D], negative: bool) -> [[f64; D]; D] {
    if negative {
        tensor.map(|row| row.map(|value| -value))
    } else {
        tensor
    }
}

fn apply_third_sign<const D: usize>(
    tensor: [[[f64; D]; D]; D],
    negative: bool,
) -> [[[f64; D]; D]; D] {
    if negative {
        tensor.map(|matrix| matrix.map(|row| row.map(|value| -value)))
    } else {
        tensor
    }
}

fn arguments_have_negative_sign(arguments: &[KernelArgument]) -> bool {
    arguments
        .iter()
        .filter(|argument| **argument == KernelArgument::Center)
        .count()
        % 2
        != 0
}
