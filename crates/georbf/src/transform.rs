//! Affine coordinate normalization without a linear-algebra dependency.
//!
//! A transform is available only in the supported dimensions:
//!
//! ```compile_fail
//! use georbf::AffineNormalization;
//!
//! fn unsupported(_: Option<AffineNormalization<4>>) {}
//! ```

use std::error::Error;
use std::fmt;

use crate::dimension::{Dim, SupportedDimension};
use crate::geometry::{Point, Vector};

/// Coordinate operation that produced a non-finite result.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TransformOperation {
    /// Applying `S^-1 (x - mu)`.
    NormalizePoint,
    /// Applying `mu + S x_tilde`.
    DenormalizePoint,
    /// Applying `S^-T g_tilde`.
    GradientToOriginal,
    /// Applying `S^-T H_tilde S^-1`.
    HessianToOriginal,
}

/// Error returned by affine normalization construction or evaluation.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum TransformError {
    /// A scale-matrix component is NaN or infinite.
    NonFiniteScaleComponent {
        /// Matrix row.
        row: usize,
        /// Matrix column.
        column: usize,
        /// Rejected value.
        value: f64,
    },
    /// The scale matrix is singular under exact floating-point elimination.
    SingularScaleMatrix,
    /// The inverse cannot be represented by finite `f64` components.
    NonRepresentableInverse,
    /// A normalized-coordinate Hessian component is NaN or infinite.
    NonFiniteHessianComponent {
        /// Matrix row.
        row: usize,
        /// Matrix column.
        column: usize,
        /// Rejected value.
        value: f64,
    },
    /// A finite input produced an unrepresentable operation result.
    NonFiniteResult {
        /// Operation that overflowed or otherwise became non-finite.
        operation: TransformOperation,
    },
}

impl fmt::Display for TransformError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteScaleComponent { row, column, value } => write!(
                formatter,
                "scale matrix component ({row}, {column}) must be finite, got {value}"
            ),
            Self::SingularScaleMatrix => formatter.write_str("scale matrix must be invertible"),
            Self::NonRepresentableInverse => {
                formatter.write_str("scale-matrix inverse is not finite and representable")
            }
            Self::NonFiniteHessianComponent { row, column, value } => write!(
                formatter,
                "Hessian component ({row}, {column}) must be finite, got {value}"
            ),
            Self::NonFiniteResult { operation } => {
                write!(
                    formatter,
                    "coordinate operation {operation:?} produced a non-finite result"
                )
            }
        }
    }
}

impl Error for TransformError {}

/// Invertible affine normalization `x_tilde = S^-1 (x - mu)`.
///
/// Construction uses exact zero-pivot decisions with partial pivoting. It does
/// not add a tolerance, jitter, regularization, or pseudoinverse.
///
/// ```
/// use georbf::{AffineNormalization, Point};
///
/// let transform = AffineNormalization::<2>::try_new(
///     Point::try_new([10.0, -2.0])?,
///     [[2.0, 0.0], [0.0, 4.0]],
/// )?;
/// let normalized = transform.normalize_point(Point::try_new([14.0, 6.0])?)?;
/// assert_eq!(normalized.components(), &[2.0, 2.0]);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct AffineNormalization<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    center: Point<D>,
    scale_matrix: [[f64; D]; D],
    inverse_scale_matrix: [[f64; D]; D],
}

impl<const D: usize> AffineNormalization<D>
where
    Dim<D>: SupportedDimension,
{
    /// Constructs a normalization from finite `mu` and finite, invertible `S`.
    ///
    /// # Errors
    ///
    /// Returns [`TransformError::NonFiniteScaleComponent`] for invalid matrix
    /// data, [`TransformError::SingularScaleMatrix`] for an exact zero pivot,
    /// or [`TransformError::NonRepresentableInverse`] when elimination cannot
    /// produce a finite inverse.
    pub fn try_new(center: Point<D>, scale_matrix: [[f64; D]; D]) -> Result<Self, TransformError> {
        validate_scale_matrix(&scale_matrix)?;
        let inverse_scale_matrix = invert_matrix(scale_matrix)?;
        Ok(Self {
            center,
            scale_matrix,
            inverse_scale_matrix,
        })
    }

    /// Borrows the normalization center `mu`.
    pub const fn center(&self) -> &Point<D> {
        &self.center
    }

    /// Borrows the scale matrix `S`.
    #[must_use]
    pub const fn scale_matrix(&self) -> &[[f64; D]; D] {
        &self.scale_matrix
    }

    /// Borrows the stored inverse scale matrix `S^-1`.
    #[must_use]
    pub const fn inverse_scale_matrix(&self) -> &[[f64; D]; D] {
        &self.inverse_scale_matrix
    }

    /// Applies `x_tilde = S^-1 (x - mu)`.
    ///
    /// # Errors
    ///
    /// Returns [`TransformError::NonFiniteResult`] if subtraction or matrix
    /// multiplication leaves the finite `f64` domain.
    pub fn normalize_point(&self, point: Point<D>) -> Result<Point<D>, TransformError> {
        let operation = TransformOperation::NormalizePoint;
        let mut delta = [0.0; D];
        for (index, value) in delta.iter_mut().enumerate() {
            *value = point.components()[index] - self.center.components()[index];
            ensure_finite(*value, operation)?;
        }
        let normalized = matrix_vector(&self.inverse_scale_matrix, &delta, operation)?;
        point_from_finite(normalized, operation)
    }

    /// Applies `x = mu + S x_tilde`.
    ///
    /// # Errors
    ///
    /// Returns [`TransformError::NonFiniteResult`] if matrix multiplication or
    /// translation leaves the finite `f64` domain.
    pub fn denormalize_point(&self, point: Point<D>) -> Result<Point<D>, TransformError> {
        let operation = TransformOperation::DenormalizePoint;
        let mut original = matrix_vector(&self.scale_matrix, point.components(), operation)?;
        for (index, value) in original.iter_mut().enumerate() {
            *value += self.center.components()[index];
            ensure_finite(*value, operation)?;
        }
        point_from_finite(original, operation)
    }

    /// Applies `g = S^-T g_tilde`.
    ///
    /// # Errors
    ///
    /// Returns [`TransformError::NonFiniteResult`] if the transformed gradient
    /// cannot be represented by finite `f64` components.
    pub fn gradient_to_original(&self, gradient: Vector<D>) -> Result<Vector<D>, TransformError> {
        let operation = TransformOperation::GradientToOriginal;
        let transformed =
            transpose_matrix_vector(&self.inverse_scale_matrix, gradient.components(), operation)?;
        Vector::try_new(transformed).map_err(|_| TransformError::NonFiniteResult { operation })
    }

    /// Applies `H = S^-T H_tilde S^-1`.
    ///
    /// # Errors
    ///
    /// Returns [`TransformError::NonFiniteHessianComponent`] for invalid input
    /// or [`TransformError::NonFiniteResult`] when the transformed matrix is
    /// not representable by finite `f64` components.
    pub fn hessian_to_original(
        &self,
        hessian: [[f64; D]; D],
    ) -> Result<[[f64; D]; D], TransformError> {
        validate_hessian(&hessian)?;
        let operation = TransformOperation::HessianToOriginal;
        let mut right_product = [[0.0; D]; D];
        for (row, output_row) in right_product.iter_mut().enumerate() {
            for (column, output) in output_row.iter_mut().enumerate() {
                *output = finite_dot(
                    (0..D).map(|inner| {
                        hessian[row][inner] * self.inverse_scale_matrix[inner][column]
                    }),
                    operation,
                )?;
            }
        }

        let mut original = [[0.0; D]; D];
        for (row, output_row) in original.iter_mut().enumerate() {
            for (column, output) in output_row.iter_mut().enumerate() {
                *output = finite_dot(
                    (0..D).map(|inner| {
                        self.inverse_scale_matrix[inner][row] * right_product[inner][column]
                    }),
                    operation,
                )?;
            }
        }
        Ok(original)
    }
}

fn validate_scale_matrix<const D: usize>(matrix: &[[f64; D]; D]) -> Result<(), TransformError> {
    for (row, values) in matrix.iter().enumerate() {
        for (column, value) in values.iter().copied().enumerate() {
            if !value.is_finite() {
                return Err(TransformError::NonFiniteScaleComponent { row, column, value });
            }
        }
    }
    Ok(())
}

fn validate_hessian<const D: usize>(matrix: &[[f64; D]; D]) -> Result<(), TransformError> {
    for (row, values) in matrix.iter().enumerate() {
        for (column, value) in values.iter().copied().enumerate() {
            if !value.is_finite() {
                return Err(TransformError::NonFiniteHessianComponent { row, column, value });
            }
        }
    }
    Ok(())
}

fn invert_matrix<const D: usize>(
    mut matrix: [[f64; D]; D],
) -> Result<[[f64; D]; D], TransformError> {
    let mut inverse = identity_matrix();

    for pivot_column in 0..D {
        let mut pivot_row = pivot_column;
        let mut pivot_magnitude = 0.0_f64;
        for (row, values) in matrix.iter().enumerate().skip(pivot_column) {
            let magnitude = values[pivot_column].abs();
            if magnitude > pivot_magnitude {
                pivot_magnitude = magnitude;
                pivot_row = row;
            }
        }
        if pivot_magnitude == 0.0 {
            return Err(TransformError::SingularScaleMatrix);
        }

        matrix.swap(pivot_column, pivot_row);
        inverse.swap(pivot_column, pivot_row);
        let pivot = matrix[pivot_column][pivot_column];
        for column in 0..D {
            matrix[pivot_column][column] /= pivot;
            inverse[pivot_column][column] /= pivot;
        }
        ensure_matrix_finite(&matrix)?;
        ensure_matrix_finite(&inverse)?;

        for row in 0..D {
            if row == pivot_column {
                continue;
            }
            let factor = matrix[row][pivot_column];
            for column in 0..D {
                matrix[row][column] -= factor * matrix[pivot_column][column];
                inverse[row][column] -= factor * inverse[pivot_column][column];
            }
            ensure_matrix_finite(&matrix)?;
            ensure_matrix_finite(&inverse)?;
        }
    }
    Ok(inverse)
}

fn identity_matrix<const D: usize>() -> [[f64; D]; D] {
    std::array::from_fn(|row| std::array::from_fn(|column| f64::from(row == column)))
}

fn ensure_matrix_finite<const D: usize>(matrix: &[[f64; D]; D]) -> Result<(), TransformError> {
    if matrix.iter().flatten().all(|value| value.is_finite()) {
        Ok(())
    } else {
        Err(TransformError::NonRepresentableInverse)
    }
}

fn matrix_vector<const D: usize>(
    matrix: &[[f64; D]; D],
    vector: &[f64; D],
    operation: TransformOperation,
) -> Result<[f64; D], TransformError> {
    let mut result = [0.0; D];
    for (row, output) in result.iter_mut().enumerate() {
        *output = finite_dot(
            (0..D).map(|column| matrix[row][column] * vector[column]),
            operation,
        )?;
    }
    Ok(result)
}

fn transpose_matrix_vector<const D: usize>(
    matrix: &[[f64; D]; D],
    vector: &[f64; D],
    operation: TransformOperation,
) -> Result<[f64; D], TransformError> {
    let mut result = [0.0; D];
    for (column, output) in result.iter_mut().enumerate() {
        *output = finite_dot(
            (0..D).map(|row| matrix[row][column] * vector[row]),
            operation,
        )?;
    }
    Ok(result)
}

fn finite_dot(
    terms: impl IntoIterator<Item = f64>,
    operation: TransformOperation,
) -> Result<f64, TransformError> {
    let mut sum = 0.0;
    for term in terms {
        ensure_finite(term, operation)?;
        sum += term;
        ensure_finite(sum, operation)?;
    }
    Ok(sum)
}

fn ensure_finite(value: f64, operation: TransformOperation) -> Result<(), TransformError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(TransformError::NonFiniteResult { operation })
    }
}

fn point_from_finite<const D: usize>(
    components: [f64; D],
    operation: TransformOperation,
) -> Result<Point<D>, TransformError>
where
    Dim<D>: SupportedDimension,
{
    Point::try_new(components).map_err(|_| TransformError::NonFiniteResult { operation })
}
