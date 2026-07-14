//! Complete polynomial spaces for conditionally positive-definite kernels.
//!
//! A CPD kernel of order `m` requires every monomial whose total degree is at
//! most `m - 1`. Terms are ordered first by increasing total degree and then,
//! within one degree, lexicographically from the first axis with larger
//! exponents first. For example, the two-dimensional degree-two terms are
//! `[2, 0]`, `[1, 1]`, `[0, 2]`.
//!
//! Polynomial spaces are available only in one, two, or three dimensions:
//!
//! ```compile_fail
//! use georbf::PolynomialSpace;
//!
//! let _ = PolynomialSpace::<0>::try_new(1);
//! ```
//!
//! ```compile_fail
//! use georbf::PolynomialSpace;
//!
//! let _ = PolynomialSpace::<4>::try_new(1);
//! ```

use std::error::Error;
use std::fmt;

use crate::dimension::{Dim, SupportedDimension};
use crate::geometry::Point;
use crate::kernel::CpdOrder;

/// A nonnegative Cartesian multi-index in a supported spatial dimension.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub struct MultiIndex<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    exponents: [usize; D],
    total_degree: usize,
}

impl<const D: usize> MultiIndex<D>
where
    Dim<D>: SupportedDimension,
{
    /// Constructs a multi-index and validates that its total degree is
    /// representable.
    ///
    /// # Errors
    ///
    /// Returns [`PolynomialSpaceError::DegreeOverflow`] when the exponent sum
    /// exceeds [`usize::MAX`].
    pub fn try_new(exponents: [usize; D]) -> Result<Self, PolynomialSpaceError> {
        let total_degree = checked_total_degree(&exponents)?;
        Ok(Self {
            exponents,
            total_degree,
        })
    }

    /// Borrows the exponents in Cartesian axis order.
    #[must_use]
    pub const fn exponents(&self) -> &[usize; D] {
        &self.exponents
    }

    /// Returns the sum of all exponents.
    #[must_use]
    pub const fn total_degree(&self) -> usize {
        self.total_degree
    }
}

/// Caller-provided polynomial output whose length was invalid.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PolynomialOutput {
    /// One monomial value per basis term.
    Values,
    /// One Cartesian gradient per basis term.
    Gradients,
}

/// Error returned while constructing or evaluating a complete polynomial space.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PolynomialSpaceError {
    /// CPD order zero has no polynomial-side-space meaning.
    ZeroOrder,
    /// A multi-index total degree overflowed the platform index type.
    DegreeOverflow,
    /// The exact complete-space term count is not representable.
    TermCountOverflow {
        /// Requested positive CPD order.
        order: usize,
        /// Compile-time spatial dimension.
        dimension: usize,
    },
    /// Storage for the exact term count could not be reserved.
    AllocationFailed {
        /// Exact number of terms that was requested.
        term_count: usize,
    },
    /// Caller-provided output storage has the wrong length.
    OutputLengthMismatch {
        /// Kind of output storage.
        output: PolynomialOutput,
        /// Required number of entries.
        expected: usize,
        /// Supplied number of entries.
        actual: usize,
    },
    /// A monomial value or derivative is not finitely representable.
    NonFiniteEvaluation {
        /// Zero-based basis-term index in deterministic order.
        term_index: usize,
        /// Cartesian derivative axis, or `None` for a value.
        derivative_axis: Option<usize>,
    },
}

impl fmt::Display for PolynomialSpaceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroOrder => formatter.write_str("CPD order must be positive"),
            Self::DegreeOverflow => formatter.write_str("multi-index total degree overflowed"),
            Self::TermCountOverflow { order, dimension } => write!(
                formatter,
                "complete polynomial term count overflowed for order {order} in D={dimension}"
            ),
            Self::AllocationFailed { term_count } => write!(
                formatter,
                "could not reserve storage for {term_count} polynomial terms"
            ),
            Self::OutputLengthMismatch {
                output,
                expected,
                actual,
            } => write!(
                formatter,
                "{output:?} output length must be {expected}, got {actual}"
            ),
            Self::NonFiniteEvaluation {
                term_index,
                derivative_axis: Some(axis),
            } => write!(
                formatter,
                "polynomial term {term_index} derivative on axis {axis} is not finite"
            ),
            Self::NonFiniteEvaluation {
                term_index,
                derivative_axis: None,
            } => write!(
                formatter,
                "polynomial term {term_index} value is not finite"
            ),
        }
    }
}

impl Error for PolynomialSpaceError {}

/// The complete monomial space required by a positive CPD order.
///
/// Construction generates exactly `binomial(D + m - 1, D)` terms for CPD
/// order `m`. Evaluation performs no heap allocation and writes into storage
/// supplied by the caller. An evaluation error leaves that storage unchanged.
///
/// ```
/// use georbf::{Point, PolynomialSpace};
///
/// let space = PolynomialSpace::<2>::try_new(3)?;
/// let mut values = vec![0.0; space.term_count()];
/// let mut gradients = vec![[0.0; 2]; space.term_count()];
/// space.try_evaluate(
///     Point::try_new([2.0, 3.0])?,
///     &mut values,
///     &mut gradients,
/// )?;
///
/// assert_eq!(values, [1.0, 2.0, 3.0, 4.0, 6.0, 9.0]);
/// assert_eq!(gradients[4], [3.0, 2.0]);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
#[must_use]
pub struct PolynomialSpace<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    cpd_order: CpdOrder,
    terms: Vec<MultiIndex<D>>,
}

impl<const D: usize> PolynomialSpace<D>
where
    Dim<D>: SupportedDimension,
{
    /// Generates the complete polynomial space for the supplied CPD order.
    ///
    /// # Errors
    ///
    /// Returns [`PolynomialSpaceError::ZeroOrder`] for order zero,
    /// [`PolynomialSpaceError::TermCountOverflow`] when the exact binomial
    /// count is not representable, or [`PolynomialSpaceError::AllocationFailed`]
    /// when storage for that exact count cannot be reserved.
    pub fn try_new(order: usize) -> Result<Self, PolynomialSpaceError> {
        let cpd_order = CpdOrder::try_new(order).map_err(|_| PolynomialSpaceError::ZeroOrder)?;
        let term_count = complete_term_count(order, D)?;
        let mut terms = Vec::new();
        terms
            .try_reserve_exact(term_count)
            .map_err(|_| PolynomialSpaceError::AllocationFailed { term_count })?;

        let mut exponents = [0; D];
        for degree in 0..=cpd_order.maximum_polynomial_degree() {
            append_exact_degree(0, degree, degree, &mut exponents, &mut terms);
        }

        debug_assert_eq!(terms.len(), term_count);
        Ok(Self { cpd_order, terms })
    }

    /// Returns the positive CPD order represented by this space.
    pub const fn cpd_order(&self) -> CpdOrder {
        self.cpd_order
    }

    /// Returns the maximum total polynomial degree, namely `m - 1`.
    #[must_use]
    pub const fn maximum_degree(&self) -> usize {
        self.cpd_order.maximum_polynomial_degree()
    }

    /// Returns the exact number of basis terms.
    #[must_use]
    pub fn term_count(&self) -> usize {
        self.terms.len()
    }

    /// Borrows all basis multi-indices in deterministic order.
    pub fn terms(&self) -> &[MultiIndex<D>] {
        &self.terms
    }

    /// Evaluates every basis value into caller-provided storage.
    ///
    /// The output remains unchanged on error.
    ///
    /// # Errors
    ///
    /// Returns [`PolynomialSpaceError::OutputLengthMismatch`] unless `values`
    /// has exactly [`Self::term_count`] entries. Returns
    /// [`PolynomialSpaceError::NonFiniteEvaluation`] when a result is not
    /// finitely representable.
    pub fn try_evaluate_values(
        &self,
        point: Point<D>,
        values: &mut [f64],
    ) -> Result<(), PolynomialSpaceError> {
        self.validate_output_length(PolynomialOutput::Values, values.len())?;
        self.validate_values(point)?;
        self.write_values(point, values);
        Ok(())
    }

    /// Evaluates every Cartesian basis gradient into caller-provided storage.
    ///
    /// The output remains unchanged on error. Derivatives lower the relevant
    /// exponent directly and never divide by a coordinate.
    ///
    /// # Errors
    ///
    /// Returns [`PolynomialSpaceError::OutputLengthMismatch`] unless
    /// `gradients` has exactly [`Self::term_count`] entries. Returns
    /// [`PolynomialSpaceError::NonFiniteEvaluation`] when a result is not
    /// finitely representable.
    pub fn try_evaluate_gradients(
        &self,
        point: Point<D>,
        gradients: &mut [[f64; D]],
    ) -> Result<(), PolynomialSpaceError> {
        self.validate_output_length(PolynomialOutput::Gradients, gradients.len())?;
        self.validate_gradients(point)?;
        self.write_gradients(point, gradients);
        Ok(())
    }

    /// Evaluates every basis value and Cartesian gradient.
    ///
    /// Both outputs remain unchanged on error.
    ///
    /// # Errors
    ///
    /// Returns [`PolynomialSpaceError::OutputLengthMismatch`] for an invalid
    /// output length or [`PolynomialSpaceError::NonFiniteEvaluation`] for a
    /// result that is not finitely representable.
    pub fn try_evaluate(
        &self,
        point: Point<D>,
        values: &mut [f64],
        gradients: &mut [[f64; D]],
    ) -> Result<(), PolynomialSpaceError> {
        self.validate_output_length(PolynomialOutput::Values, values.len())?;
        self.validate_output_length(PolynomialOutput::Gradients, gradients.len())?;
        self.validate_values(point)?;
        self.validate_gradients(point)?;
        self.write_values(point, values);
        self.write_gradients(point, gradients);
        Ok(())
    }

    fn validate_output_length(
        &self,
        output: PolynomialOutput,
        actual: usize,
    ) -> Result<(), PolynomialSpaceError> {
        let expected = self.term_count();
        if actual == expected {
            Ok(())
        } else {
            Err(PolynomialSpaceError::OutputLengthMismatch {
                output,
                expected,
                actual,
            })
        }
    }

    fn validate_values(&self, point: Point<D>) -> Result<(), PolynomialSpaceError> {
        for (term_index, term) in self.terms.iter().enumerate() {
            if !evaluate_monomial(term.exponents, *point.components(), None).is_finite() {
                return Err(PolynomialSpaceError::NonFiniteEvaluation {
                    term_index,
                    derivative_axis: None,
                });
            }
        }
        Ok(())
    }

    fn validate_gradients(&self, point: Point<D>) -> Result<(), PolynomialSpaceError> {
        for (term_index, term) in self.terms.iter().enumerate() {
            for axis in 0..D {
                if !evaluate_monomial(term.exponents, *point.components(), Some(axis)).is_finite() {
                    return Err(PolynomialSpaceError::NonFiniteEvaluation {
                        term_index,
                        derivative_axis: Some(axis),
                    });
                }
            }
        }
        Ok(())
    }

    fn write_values(&self, point: Point<D>, values: &mut [f64]) {
        for (value, term) in values.iter_mut().zip(&self.terms) {
            *value = evaluate_monomial(term.exponents, *point.components(), None);
        }
    }

    fn write_gradients(&self, point: Point<D>, gradients: &mut [[f64; D]]) {
        for (gradient, term) in gradients.iter_mut().zip(&self.terms) {
            for (axis, component) in gradient.iter_mut().enumerate() {
                *component = evaluate_monomial(term.exponents, *point.components(), Some(axis));
            }
        }
    }
}

fn checked_total_degree<const D: usize>(
    exponents: &[usize; D],
) -> Result<usize, PolynomialSpaceError> {
    exponents.iter().try_fold(0_usize, |degree, exponent| {
        degree
            .checked_add(*exponent)
            .ok_or(PolynomialSpaceError::DegreeOverflow)
    })
}

fn complete_term_count(order: usize, dimension: usize) -> Result<usize, PolynomialSpaceError> {
    let n = order
        .checked_add(dimension - 1)
        .ok_or(PolynomialSpaceError::TermCountOverflow { order, dimension })?;
    checked_binomial(n, dimension)
        .ok_or(PolynomialSpaceError::TermCountOverflow { order, dimension })
}

fn checked_binomial(n: usize, k: usize) -> Option<usize> {
    let k = k.min(n - k);
    let mut result = 1_usize;
    for index in 1..=k {
        let mut numerator = n - k + index;
        let divisor_gcd = greatest_common_divisor(result, index);
        result /= divisor_gcd;
        let remaining_divisor = index / divisor_gcd;
        numerator /= remaining_divisor;
        result = result.checked_mul(numerator)?;
    }
    Some(result)
}

const fn greatest_common_divisor(mut left: usize, mut right: usize) -> usize {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left
}

fn append_exact_degree<const D: usize>(
    axis: usize,
    total_degree: usize,
    remaining_degree: usize,
    exponents: &mut [usize; D],
    terms: &mut Vec<MultiIndex<D>>,
) where
    Dim<D>: SupportedDimension,
{
    if axis + 1 == D {
        exponents[axis] = remaining_degree;
        terms.push(MultiIndex {
            exponents: *exponents,
            total_degree,
        });
        return;
    }

    for exponent in (0..=remaining_degree).rev() {
        exponents[axis] = exponent;
        append_exact_degree(
            axis + 1,
            total_degree,
            remaining_degree - exponent,
            exponents,
            terms,
        );
    }
}

#[allow(clippy::cast_precision_loss)]
fn evaluate_monomial<const D: usize>(
    mut exponents: [usize; D],
    coordinates: [f64; D],
    derivative_axis: Option<usize>,
) -> f64 {
    let coefficient = if let Some(axis) = derivative_axis {
        let exponent = exponents[axis];
        if exponent == 0 {
            return 0.0;
        }
        exponents[axis] -= 1;
        exponent as f64
    } else {
        1.0
    };

    if exponents
        .iter()
        .zip(coordinates)
        .any(|(exponent, coordinate)| *exponent > 0 && coordinate == 0.0)
    {
        return 0.0;
    }

    let mut scaled = ScaledProduct::from_nonzero(coefficient);
    for (exponent, coordinate) in exponents.into_iter().zip(coordinates) {
        if exponent > 0 {
            scaled = scaled.multiply(ScaledProduct::from_nonzero(coordinate).power(exponent));
        }
    }
    scaled.into_f64()
}

#[derive(Clone, Copy)]
struct ScaledProduct {
    mantissa: f64,
    binary_exponent: i128,
}

impl ScaledProduct {
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    fn from_nonzero(value: f64) -> Self {
        let negative = value.is_sign_negative();
        let bits = value.abs().to_bits();
        let exponent_bits = i128::from((bits >> 52) & 0x7ff);
        let fraction = bits & ((1_u64 << 52) - 1);
        let (mut mantissa, binary_exponent) = if exponent_bits == 0 {
            let leading_bit = fraction.ilog2();
            (
                fraction as f64 / 2.0_f64.powi(leading_bit.cast_signed()),
                -1074 + i128::from(leading_bit),
            )
        } else {
            (
                1.0 + fraction as f64 / 4_503_599_627_370_496.0,
                exponent_bits - 1023,
            )
        };
        if negative {
            mantissa = -mantissa;
        }
        Self {
            mantissa,
            binary_exponent,
        }
    }

    fn multiply(self, other: Self) -> Self {
        let mut mantissa = self.mantissa * other.mantissa;
        let mut binary_exponent = self.binary_exponent + other.binary_exponent;
        if mantissa.abs() >= 2.0 {
            mantissa *= 0.5;
            binary_exponent += 1;
        }
        Self {
            mantissa,
            binary_exponent,
        }
    }

    fn power(mut self, mut exponent: usize) -> Self {
        let mut result = Self {
            mantissa: 1.0,
            binary_exponent: 0,
        };
        while exponent > 0 {
            if exponent & 1 == 1 {
                result = result.multiply(self);
            }
            exponent >>= 1;
            if exponent > 0 {
                self = self.multiply(self);
            }
        }
        result
    }

    #[allow(clippy::cast_possible_truncation)]
    fn into_f64(self) -> f64 {
        let negative = self.mantissa.is_sign_negative();
        let magnitude = if self.binary_exponent > 1023 {
            f64::INFINITY
        } else if self.binary_exponent >= -1022 {
            self.mantissa.abs() * 2.0_f64.powi(self.binary_exponent as i32)
        } else if self.binary_exponent < -1075 {
            0.0
        } else {
            let subnormal_units =
                self.mantissa.abs() * 2.0_f64.powi((self.binary_exponent + 1074) as i32);
            subnormal_units * f64::from_bits(1)
        };
        if negative { -magnitude } else { magnitude }
    }
}
