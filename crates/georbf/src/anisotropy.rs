//! Fixed global anisotropy metrics and their Cartesian chain rule.
//!
//! A metric is available only in the supported dimensions:
//!
//! ```compile_fail
//! use georbf::GlobalAnisotropy;
//!
//! fn unsupported(_: Option<GlobalAnisotropy<4>>) {}
//! ```

use std::error::Error;
use std::fmt;

use crate::dimension::{Dim, SupportedDimension};
use crate::geometry::{Point, UnitDirection};
use crate::kernel_calculus::{
    KernelArgument, KernelCalculusError, RadialSeparation, SpatialKernelJet,
};
use crate::transform::{TransformError, invert_matrix};

/// Caller-selected condition-number acceptance policy.
///
/// `Unbounded` has no implicit ill-conditioning cutoff: construction still
/// requires a finite transform, finite metric, finitely representable inverse,
/// singular values, and condition number. `Maximum` additionally rejects a
/// condition number greater than its finite value, which must be at least one.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnisotropyConditionPolicy {
    /// Accept every otherwise representable invertible transform.
    Unbounded,
    /// Accept only a condition number no greater than this value.
    Maximum(f64),
}

/// Deterministic diagnostics for a validated global transform.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct AnisotropyDiagnostics<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    singular_values: [f64; D],
    condition_number: f64,
}

impl<const D: usize> AnisotropyDiagnostics<D>
where
    Dim<D>: SupportedDimension,
{
    /// Returns singular values in nonincreasing order.
    #[must_use]
    pub const fn singular_values(&self) -> &[f64; D] {
        &self.singular_values
    }

    /// Returns `sigma_max / sigma_min` in the Euclidean matrix norm.
    #[must_use]
    pub const fn condition_number(&self) -> f64 {
        self.condition_number
    }
}

/// Error returned by global anisotropy construction or evaluation.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum AnisotropyError<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    /// An axis length is NaN or infinite.
    NonFiniteAxisLength {
        /// Axis or length role index.
        axis: usize,
        /// Rejected value.
        value: f64,
    },
    /// An axis length is zero or negative.
    NonPositiveAxisLength {
        /// Axis or length role index.
        axis: usize,
        /// Rejected value.
        value: f64,
    },
    /// The reciprocal axis length is outside the finite `f64` domain.
    NonRepresentableReciprocalLength {
        /// Axis or length role index.
        axis: usize,
        /// Rejected positive length.
        value: f64,
    },
    /// A user-transform component is NaN or infinite.
    NonFiniteTransformComponent {
        /// Matrix row.
        row: usize,
        /// Matrix column.
        column: usize,
        /// Rejected value.
        value: f64,
    },
    /// Exact inversion failed or did not have a finite representation.
    TransformInversion {
        /// Underlying coordinate-transform error.
        source: TransformError,
        /// Diagnostic singular values in nonincreasing order.
        singular_values: [f64; D],
        /// Diagnostic Euclidean condition number, or positive infinity for a
        /// numerically singular transform.
        condition_number: f64,
    },
    /// Forming `A^T A` produced a non-finite component.
    NonRepresentableMetricComponent {
        /// Matrix row.
        row: usize,
        /// Matrix column.
        column: usize,
    },
    /// A user-metric component is NaN or infinite.
    NonFiniteMetricComponent {
        /// Matrix row.
        row: usize,
        /// Matrix column.
        column: usize,
        /// Rejected value.
        value: f64,
    },
    /// A user metric is not exactly symmetric.
    NonSymmetricMetric {
        /// First mismatched row.
        row: usize,
        /// First mismatched column.
        column: usize,
        /// `B[row][column]`.
        forward: f64,
        /// `B[column][row]`.
        reverse: f64,
    },
    /// Exact scaled leading-minor validation or unregularized Cholesky found a
    /// nonpositive SPD diagnostic.
    MetricNotPositiveDefinite {
        /// Zero-based Cholesky pivot.
        pivot: usize,
        /// Residual presented to the square root.
        residual: f64,
    },
    /// A Cholesky factor component was not finitely representable.
    NonRepresentableMetricFactor {
        /// Factor row.
        row: usize,
        /// Factor column.
        column: usize,
    },
    /// The caller's orthogonality tolerance is NaN or infinite.
    NonFiniteOrthogonalityTolerance {
        /// Rejected tolerance.
        value: f64,
    },
    /// The caller's orthogonality tolerance is outside `[0, 1)`.
    InvalidOrthogonalityTolerance {
        /// Rejected tolerance.
        value: f64,
    },
    /// Two ellipsoid axes exceed the caller-selected orthogonality tolerance.
    NonOrthogonalAxes {
        /// First axis.
        first: usize,
        /// Second axis.
        second: usize,
        /// Computed dot product.
        dot_product: f64,
        /// Caller-selected tolerance.
        tolerance: f64,
    },
    /// A maximum condition number is non-finite or less than one.
    InvalidMaximumConditionNumber {
        /// Rejected maximum.
        value: f64,
    },
    /// A singular value was not positive and finitely representable.
    NonRepresentableSingularValue {
        /// Index after nonincreasing sorting.
        index: usize,
    },
    /// The Euclidean condition number was not finitely representable.
    NonRepresentableConditionNumber,
    /// The explicit maximum condition number was exceeded.
    ConditionNumberExceeded {
        /// Caller-selected maximum.
        maximum: f64,
        /// Diagnostics for the rejected transform.
        diagnostics: AnisotropyDiagnostics<D>,
    },
    /// Subtracting the original query and center overflowed.
    NonFiniteDisplacementComponent {
        /// Coordinate axis.
        axis: usize,
    },
    /// Applying `A` to a finite displacement was non-representable.
    NonFiniteTransformedDisplacementComponent {
        /// Transformed coordinate axis.
        axis: usize,
    },
    /// Stable radius construction failed after the linear map.
    Separation {
        /// Underlying separation error.
        source: KernelCalculusError,
    },
    /// A chain-rule gradient component was non-representable.
    NonFiniteFirstDerivative {
        /// Original-coordinate axis.
        axis: usize,
    },
    /// A chain-rule Hessian component was non-representable.
    NonFiniteSecondDerivative {
        /// First original-coordinate axis.
        row: usize,
        /// Second original-coordinate axis.
        column: usize,
    },
    /// A chain-rule third-derivative component was non-representable.
    NonFiniteThirdDerivative {
        /// First original-coordinate axis.
        first: usize,
        /// Second original-coordinate axis.
        second: usize,
        /// Third original-coordinate axis.
        third: usize,
    },
}

impl<const D: usize> fmt::Display for AnisotropyError<D>
where
    Dim<D>: SupportedDimension,
{
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteAxisLength { axis, value } => {
                write!(formatter, "axis length {axis} must be finite, got {value}")
            }
            Self::NonPositiveAxisLength { axis, value } => {
                write!(
                    formatter,
                    "axis length {axis} must be positive, got {value}"
                )
            }
            Self::NonRepresentableReciprocalLength { axis, value } => write!(
                formatter,
                "reciprocal of axis length {axis}={value} is not finitely representable"
            ),
            Self::NonFiniteTransformComponent { row, column, value } => write!(
                formatter,
                "anisotropy transform component ({row}, {column}) must be finite, got {value}"
            ),
            Self::TransformInversion {
                source,
                condition_number,
                ..
            } => write!(
                formatter,
                "anisotropy transform is not finitely invertible (diagnostic condition number {condition_number}): {source}"
            ),
            Self::NonRepresentableMetricComponent { row, column } => write!(
                formatter,
                "anisotropy metric component ({row}, {column}) is not finitely representable"
            ),
            Self::NonFiniteMetricComponent { row, column, value } => write!(
                formatter,
                "anisotropy metric component ({row}, {column}) must be finite, got {value}"
            ),
            Self::NonSymmetricMetric {
                row,
                column,
                forward,
                reverse,
            } => write!(
                formatter,
                "anisotropy metric must be exactly symmetric: ({row}, {column})={forward}, but ({column}, {row})={reverse}"
            ),
            Self::MetricNotPositiveDefinite { pivot, residual } => write!(
                formatter,
                "anisotropy metric is not positive definite at pivot {pivot}: residual {residual}"
            ),
            Self::NonRepresentableMetricFactor { row, column } => write!(
                formatter,
                "anisotropy metric Cholesky factor ({row}, {column}) is not finitely representable"
            ),
            Self::NonFiniteOrthogonalityTolerance { value } => write!(
                formatter,
                "orthogonality tolerance must be finite, got {value}"
            ),
            Self::InvalidOrthogonalityTolerance { value } => write!(
                formatter,
                "orthogonality tolerance must be in [0, 1), got {value}"
            ),
            Self::NonOrthogonalAxes {
                first,
                second,
                dot_product,
                tolerance,
            } => write!(
                formatter,
                "ellipsoid axes {first} and {second} have dot product {dot_product}, exceeding tolerance {tolerance}"
            ),
            Self::InvalidMaximumConditionNumber { value } => write!(
                formatter,
                "maximum condition number must be finite and at least one, got {value}"
            ),
            Self::NonRepresentableSingularValue { index } => write!(
                formatter,
                "anisotropy singular value {index} is not positive and finitely representable"
            ),
            Self::NonRepresentableConditionNumber => {
                formatter.write_str("anisotropy condition number is not finitely representable")
            }
            Self::ConditionNumberExceeded {
                maximum,
                diagnostics,
            } => write!(
                formatter,
                "anisotropy condition number {} exceeds explicit maximum {maximum}",
                diagnostics.condition_number()
            ),
            Self::NonFiniteDisplacementComponent { axis } => write!(
                formatter,
                "query-center displacement on axis {axis} is not finite"
            ),
            Self::NonFiniteTransformedDisplacementComponent { axis } => write!(
                formatter,
                "anisotropy-transformed displacement on axis {axis} is not finite"
            ),
            Self::Separation { source } => {
                write!(formatter, "anisotropic separation failed: {source}")
            }
            Self::NonFiniteFirstDerivative { axis } => write!(
                formatter,
                "anisotropy chain-rule first derivative on axis {axis} is not finite"
            ),
            Self::NonFiniteSecondDerivative { row, column } => write!(
                formatter,
                "anisotropy chain-rule second derivative ({row}, {column}) is not finite"
            ),
            Self::NonFiniteThirdDerivative {
                first,
                second,
                third,
            } => write!(
                formatter,
                "anisotropy chain-rule third derivative ({first}, {second}, {third}) is not finite"
            ),
        }
    }
}

impl<const D: usize> Error for AnisotropyError<D>
where
    Dim<D>: SupportedDimension,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::TransformInversion { source, .. } => Some(source),
            Self::Separation { source } => Some(source),
            _ => None,
        }
    }
}

/// A fixed invertible global transform `r_A(x,y) = ||A(x-y)||`.
///
/// The stored metric is `B=A^T A`. Length parameters use the active coordinate
/// length unit: an axis length `ell` contributes inverse scale `1/ell` to `A`.
/// Construction never inserts jitter, regularizes, symmetrizes input, computes
/// a pseudoinverse, or applies an implicit condition-number cutoff.
///
/// ```
/// use georbf::{GlobalAnisotropy, Point};
///
/// let anisotropy = GlobalAnisotropy::<2>::try_isotropic(2.0)?;
/// let separation = anisotropy.try_transform_separation(
///     Point::try_new([3.0, 4.0])?,
///     Point::try_new([0.0, 0.0])?,
/// )?;
/// assert_eq!(separation.radius(), 2.5);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct GlobalAnisotropy<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    transform: [[f64; D]; D],
    inverse_transform: [[f64; D]; D],
    metric: [[f64; D]; D],
    diagnostics: AnisotropyDiagnostics<D>,
}

impl<const D: usize> GlobalAnisotropy<D>
where
    Dim<D>: SupportedDimension,
{
    /// Constructs `A=I/ell` from one positive coordinate length.
    ///
    /// # Errors
    ///
    /// Returns a structured length or representation error.
    pub fn try_isotropic(length: f64) -> Result<Self, AnisotropyError<D>> {
        let reciprocal = reciprocal_length(length, 0)?;
        let transform = std::array::from_fn(|row| {
            std::array::from_fn(|column| if row == column { reciprocal } else { 0.0 })
        });
        Self::try_build(transform, None, AnisotropyConditionPolicy::Unbounded)
    }

    /// Constructs a spheroidal transform from one principal direction.
    ///
    /// The principal direction has `axial_length`; its orthogonal complement
    /// has `transverse_length`. A stable orthonormal frame stores the principal
    /// direction as the first row of `A`; the remaining rows span its
    /// orthogonal complement. This avoids subtracting nearly equal large
    /// inverse scales. In D=1 the transverse subspace is empty, so the result
    /// depends only on `axial_length` while both supplied lengths are still
    /// validated.
    ///
    /// # Errors
    ///
    /// Returns a structured length, condition-policy, or representation error.
    pub fn try_spheroidal(
        principal_axis: UnitDirection<D>,
        axial_length: f64,
        transverse_length: f64,
        condition_policy: AnisotropyConditionPolicy,
    ) -> Result<Self, AnisotropyError<D>> {
        let axial_reciprocal = reciprocal_length(axial_length, 0)?;
        let transverse_reciprocal = reciprocal_length(transverse_length, 1)?;
        let transform =
            spheroidal_transform(principal_axis, axial_reciprocal, transverse_reciprocal);
        Self::try_build(transform, None, condition_policy)
    }

    /// Constructs an ellipsoid from principal unit axes and axis lengths.
    ///
    /// Axis `i` is row `i` of `A` after division by `axis_lengths[i]`.
    /// Distinct axes must have an absolute dot product no greater than the
    /// explicit `orthogonality_tolerance`. The constructor does not alter,
    /// normalize, or orthogonalize the supplied unit directions.
    ///
    /// # Errors
    ///
    /// Returns a structured length, tolerance, orthogonality, condition, or
    /// representation error.
    pub fn try_ellipsoidal(
        principal_axes: [UnitDirection<D>; D],
        axis_lengths: [f64; D],
        orthogonality_tolerance: f64,
        condition_policy: AnisotropyConditionPolicy,
    ) -> Result<Self, AnisotropyError<D>> {
        validate_orthogonality_tolerance(orthogonality_tolerance)?;
        for first in 0..D {
            for second in (first + 1)..D {
                let dot_product = principal_axes[first]
                    .components()
                    .iter()
                    .zip(principal_axes[second].components())
                    .map(|(left, right)| left * right)
                    .sum::<f64>();
                if dot_product.abs() > orthogonality_tolerance {
                    return Err(AnisotropyError::NonOrthogonalAxes {
                        first,
                        second,
                        dot_product,
                        tolerance: orthogonality_tolerance,
                    });
                }
            }
        }

        let mut transform = [[0.0; D]; D];
        for (row, output_row) in transform.iter_mut().enumerate() {
            let reciprocal = reciprocal_length(axis_lengths[row], row)?;
            for (column, output) in output_row.iter_mut().enumerate() {
                let value = principal_axes[row].components()[column] * reciprocal;
                if !value.is_finite() {
                    return Err(AnisotropyError::NonFiniteTransformComponent {
                        row,
                        column,
                        value,
                    });
                }
                *output = value;
            }
        }
        Self::try_build(transform, None, condition_policy)
    }

    /// Constructs an anisotropy from an arbitrary finite invertible `A`.
    ///
    /// # Errors
    ///
    /// Rejects non-finite components, a singular or non-representably
    /// invertible matrix, a non-representable metric or diagnostic, and an
    /// explicitly disallowed condition number.
    pub fn try_from_transform(
        transform: [[f64; D]; D],
        condition_policy: AnisotropyConditionPolicy,
    ) -> Result<Self, AnisotropyError<D>> {
        Self::try_build(transform, None, condition_policy)
    }

    /// Constructs `A` by unregularized Cholesky factorization of an SPD `B`.
    ///
    /// The metric must be finite and exactly symmetric. No tolerance,
    /// symmetrization, diagonal adjustment, jitter, or eigenvalue clipping is
    /// applied. If `B=L L^T`, the stored transform is `A=L^T`.
    ///
    /// # Errors
    ///
    /// Returns a structured metric, factorization, inverse, diagnostic, or
    /// explicit condition-policy error.
    pub fn try_from_metric(
        metric: [[f64; D]; D],
        condition_policy: AnisotropyConditionPolicy,
    ) -> Result<Self, AnisotropyError<D>> {
        validate_metric(&metric)?;
        let transform = cholesky_transpose(metric)?;
        Self::try_build(transform, Some(metric), condition_policy)
    }

    /// Borrows the distance transform `A`.
    #[must_use]
    pub const fn transform(&self) -> &[[f64; D]; D] {
        &self.transform
    }

    /// Borrows the stored finite inverse `A^-1`.
    #[must_use]
    pub const fn inverse_transform(&self) -> &[[f64; D]; D] {
        &self.inverse_transform
    }

    /// Borrows the symmetric positive-definite metric `B=A^T A`.
    #[must_use]
    pub const fn metric(&self) -> &[[f64; D]; D] {
        &self.metric
    }

    /// Borrows singular-value and Euclidean condition diagnostics.
    pub const fn diagnostics(&self) -> &AnisotropyDiagnostics<D> {
        &self.diagnostics
    }

    /// Forms `A(x-y)` directly and computes its stable radial separation.
    ///
    /// # Errors
    ///
    /// Returns a structured displacement, transformed-displacement, or radius
    /// error if a finite result is not representable.
    pub fn try_transform_separation(
        &self,
        query: Point<D>,
        center: Point<D>,
    ) -> Result<RadialSeparation<D>, AnisotropyError<D>> {
        let mut displacement = [0.0; D];
        for (axis, output) in displacement.iter_mut().enumerate() {
            *output = query.components()[axis] - center.components()[axis];
            if !output.is_finite() {
                return Err(AnisotropyError::NonFiniteDisplacementComponent { axis });
            }
        }

        let mut transformed = [0.0; D];
        for (row, output) in transformed.iter_mut().enumerate() {
            let mut sum = 0.0;
            for (column, component) in displacement.iter().copied().enumerate() {
                let term = self.transform[row][column] * component;
                if !term.is_finite() {
                    return Err(AnisotropyError::NonFiniteTransformedDisplacementComponent {
                        axis: row,
                    });
                }
                sum += term;
                if !sum.is_finite() {
                    return Err(AnisotropyError::NonFiniteTransformedDisplacementComponent {
                        axis: row,
                    });
                }
            }
            *output = sum;
        }

        RadialSeparation::try_from_displacement(transformed)
            .map_err(|source| AnisotropyError::Separation { source })
    }

    /// Applies the constant-linear-map chain rule through third order.
    ///
    /// The input jet is interpreted in transformed coordinates `z=A(x-y)`.
    /// The returned [`SpatialKernelJet`] stores original-query derivatives and
    /// therefore retains its existing exact minus sign for every center
    /// argument requested through its accessors.
    ///
    /// # Errors
    ///
    /// Returns the first original-coordinate derivative component whose
    /// complete product-and-sum evaluation is not finitely representable.
    #[allow(clippy::needless_range_loop)]
    pub fn try_transform_spatial_jet(
        &self,
        transformed_jet: SpatialKernelJet<D>,
    ) -> Result<SpatialKernelJet<D>, AnisotropyError<D>> {
        let source_first = transformed_jet.first_derivative(KernelArgument::Query);
        let source_second =
            transformed_jet.second_derivative([KernelArgument::Query, KernelArgument::Query]);
        let source_third = transformed_jet.third_derivative([
            KernelArgument::Query,
            KernelArgument::Query,
            KernelArgument::Query,
        ]);

        let mut first = [0.0; D];
        for (axis, output) in first.iter_mut().enumerate() {
            let mut sum = 0.0;
            for (transformed_axis, derivative) in source_first.iter().copied().enumerate() {
                let term = self.transform[transformed_axis][axis] * derivative;
                if !term.is_finite() {
                    return Err(AnisotropyError::NonFiniteFirstDerivative { axis });
                }
                sum += term;
                if !sum.is_finite() {
                    return Err(AnisotropyError::NonFiniteFirstDerivative { axis });
                }
            }
            *output = sum;
        }

        let mut second = [[0.0; D]; D];
        for row in 0..D {
            for column in row..D {
                let mut sum = 0.0;
                for first_axis in 0..D {
                    for second_axis in 0..D {
                        let term = self.transform[first_axis][row]
                            * source_second[first_axis][second_axis]
                            * self.transform[second_axis][column];
                        if !term.is_finite() {
                            return Err(AnisotropyError::NonFiniteSecondDerivative { row, column });
                        }
                        sum += term;
                        if !sum.is_finite() {
                            return Err(AnisotropyError::NonFiniteSecondDerivative { row, column });
                        }
                    }
                }
                second[row][column] = sum;
                second[column][row] = sum;
            }
        }

        let mut third = [[[0.0; D]; D]; D];
        for first in 0..D {
            for second_axis in first..D {
                for third_axis in second_axis..D {
                    let mut sum = 0.0;
                    for a in 0..D {
                        for b in 0..D {
                            for c in 0..D {
                                let term = self.transform[a][first]
                                    * source_third[a][b][c]
                                    * self.transform[b][second_axis]
                                    * self.transform[c][third_axis];
                                if !term.is_finite() {
                                    return Err(AnisotropyError::NonFiniteThirdDerivative {
                                        first,
                                        second: second_axis,
                                        third: third_axis,
                                    });
                                }
                                sum += term;
                                if !sum.is_finite() {
                                    return Err(AnisotropyError::NonFiniteThirdDerivative {
                                        first,
                                        second: second_axis,
                                        third: third_axis,
                                    });
                                }
                            }
                        }
                    }
                    set_symmetric_third(&mut third, first, second_axis, third_axis, sum);
                }
            }
        }

        Ok(SpatialKernelJet::from_query_derivatives(
            transformed_jet.value(),
            first,
            second,
            third,
        ))
    }

    fn try_build(
        transform: [[f64; D]; D],
        supplied_metric: Option<[[f64; D]; D]>,
        condition_policy: AnisotropyConditionPolicy,
    ) -> Result<Self, AnisotropyError<D>> {
        validate_condition_policy(condition_policy)?;
        validate_transform(&transform)?;

        let singular_values = transform_singular_values(transform)?;
        let raw_condition_number = if singular_values[D - 1] > 0.0 {
            singular_values[0] / singular_values[D - 1]
        } else {
            f64::INFINITY
        };
        let inverse_transform = match invert_matrix(transform) {
            Ok(inverse) => inverse,
            Err(source) => {
                return Err(AnisotropyError::TransformInversion {
                    source,
                    singular_values,
                    condition_number: raw_condition_number,
                });
            }
        };
        let metric = match supplied_metric {
            Some(metric) => metric,
            None => metric_from_transform(&transform)?,
        };
        validate_positive_definite_metric(&metric)?;
        let diagnostics = diagnostics_from_singular_values(singular_values)?;

        if let AnisotropyConditionPolicy::Maximum(maximum) = condition_policy
            && diagnostics.condition_number > maximum
        {
            return Err(AnisotropyError::ConditionNumberExceeded {
                maximum,
                diagnostics,
            });
        }

        Ok(Self {
            transform,
            inverse_transform,
            metric,
            diagnostics,
        })
    }
}

fn spheroidal_transform<const D: usize>(
    principal_axis: UnitDirection<D>,
    axial_reciprocal: f64,
    transverse_reciprocal: f64,
) -> [[f64; D]; D]
where
    Dim<D>: SupportedDimension,
{
    let axis = *principal_axis.components();
    let mut frame = [[0.0; D]; D];

    if D == 1 {
        frame[0][0] = 1.0;
    } else if D == 2 {
        frame[0] = axis;
        frame[1][0] = -axis[1];
        frame[1][1] = axis[0];
    } else {
        frame[0] = axis;
        let reference = axis
            .iter()
            .enumerate()
            .min_by(|(_, left), (_, right)| left.abs().total_cmp(&right.abs()))
            .map_or(0, |(index, _)| index);
        let transverse = match reference {
            0 => [0.0, axis[2], -axis[1]],
            1 => [-axis[2], 0.0, axis[0]],
            _ => [axis[1], -axis[0], 0.0],
        };
        let transverse = normalize_three(transverse);
        let second_transverse = normalize_three([
            axis[1].mul_add(transverse[2], -axis[2] * transverse[1]),
            axis[2].mul_add(transverse[0], -axis[0] * transverse[2]),
            axis[0].mul_add(transverse[1], -axis[1] * transverse[0]),
        ]);
        frame[1].copy_from_slice(&transverse);
        frame[2].copy_from_slice(&second_transverse);
    }

    for (row, values) in frame.iter_mut().enumerate() {
        let reciprocal = if row == 0 {
            axial_reciprocal
        } else {
            transverse_reciprocal
        };
        for value in values {
            *value *= reciprocal;
        }
    }
    frame
}

fn normalize_three(components: [f64; 3]) -> [f64; 3] {
    let scale = components
        .iter()
        .copied()
        .map(f64::abs)
        .fold(0.0_f64, f64::max);
    let scaled = components.map(|value| value / scale);
    let norm = scaled.iter().map(|value| value * value).sum::<f64>().sqrt();
    scaled.map(|value| value / norm)
}

fn reciprocal_length<const D: usize>(value: f64, axis: usize) -> Result<f64, AnisotropyError<D>>
where
    Dim<D>: SupportedDimension,
{
    if !value.is_finite() {
        return Err(AnisotropyError::NonFiniteAxisLength { axis, value });
    }
    if value <= 0.0 {
        return Err(AnisotropyError::NonPositiveAxisLength { axis, value });
    }
    let reciprocal = value.recip();
    if !reciprocal.is_finite() {
        return Err(AnisotropyError::NonRepresentableReciprocalLength { axis, value });
    }
    Ok(reciprocal)
}

fn validate_condition_policy<const D: usize>(
    policy: AnisotropyConditionPolicy,
) -> Result<(), AnisotropyError<D>>
where
    Dim<D>: SupportedDimension,
{
    if let AnisotropyConditionPolicy::Maximum(value) = policy
        && (!value.is_finite() || value < 1.0)
    {
        return Err(AnisotropyError::InvalidMaximumConditionNumber { value });
    }
    Ok(())
}

fn validate_orthogonality_tolerance<const D: usize>(value: f64) -> Result<(), AnisotropyError<D>>
where
    Dim<D>: SupportedDimension,
{
    if !value.is_finite() {
        return Err(AnisotropyError::NonFiniteOrthogonalityTolerance { value });
    }
    if !(0.0..1.0).contains(&value) {
        return Err(AnisotropyError::InvalidOrthogonalityTolerance { value });
    }
    Ok(())
}

fn validate_transform<const D: usize>(transform: &[[f64; D]; D]) -> Result<(), AnisotropyError<D>>
where
    Dim<D>: SupportedDimension,
{
    for (row, values) in transform.iter().enumerate() {
        for (column, value) in values.iter().copied().enumerate() {
            if !value.is_finite() {
                return Err(AnisotropyError::NonFiniteTransformComponent { row, column, value });
            }
        }
    }
    Ok(())
}

fn metric_from_transform<const D: usize>(
    transform: &[[f64; D]; D],
) -> Result<[[f64; D]; D], AnisotropyError<D>>
where
    Dim<D>: SupportedDimension,
{
    let mut metric = [[0.0; D]; D];
    for row in 0..D {
        for column in row..D {
            let mut sum = 0.0;
            for transform_row in transform {
                let term = transform_row[row] * transform_row[column];
                if !term.is_finite() {
                    return Err(AnisotropyError::NonRepresentableMetricComponent { row, column });
                }
                sum += term;
                if !sum.is_finite() {
                    return Err(AnisotropyError::NonRepresentableMetricComponent { row, column });
                }
            }
            metric[row][column] = sum;
            metric[column][row] = sum;
        }
    }
    Ok(metric)
}

fn validate_metric<const D: usize>(metric: &[[f64; D]; D]) -> Result<(), AnisotropyError<D>>
where
    Dim<D>: SupportedDimension,
{
    for (row, values) in metric.iter().enumerate() {
        for (column, value) in values.iter().copied().enumerate() {
            if !value.is_finite() {
                return Err(AnisotropyError::NonFiniteMetricComponent { row, column, value });
            }
        }
    }
    for (row, values) in metric.iter().enumerate() {
        for (column, forward) in values.iter().copied().enumerate().skip(row + 1) {
            let reverse = metric[column][row];
            if forward.partial_cmp(&reverse) != Some(std::cmp::Ordering::Equal) {
                return Err(AnisotropyError::NonSymmetricMetric {
                    row,
                    column,
                    forward,
                    reverse,
                });
            }
        }
    }
    validate_positive_definite_metric(metric)
}

#[derive(Clone, Copy)]
struct ExactExpansion {
    components: [f64; 64],
    length: usize,
}

impl ExactExpansion {
    const fn zero() -> Self {
        Self {
            components: [0.0; 64],
            length: 0,
        }
    }

    fn add_component(&mut self, component: f64) {
        let mut output = [0.0; 64];
        let mut output_length = 0;
        let mut accumulator = component;
        for value in self.components.iter().copied().take(self.length) {
            let (sum, error) = two_sum(accumulator, value);
            if error != 0.0 {
                output[output_length] = error;
                output_length += 1;
            }
            accumulator = sum;
        }
        if accumulator != 0.0 || output_length == 0 {
            output[output_length] = accumulator;
            output_length += 1;
        }
        self.components = output;
        self.length = output_length;
    }

    fn add_product(&mut self, left: f64, right: f64, sign: f64) {
        let product = left * right;
        let error = left.mul_add(right, -product);
        self.add_component(sign * error);
        self.add_component(sign * product);
    }

    fn add_triple_product(&mut self, first: f64, second: f64, third: f64, sign: f64) {
        let product = first * second;
        let product_error = first.mul_add(second, -product);
        for partial in [product_error, product] {
            let scaled = partial * third;
            let scaled_error = partial.mul_add(third, -scaled);
            self.add_component(sign * scaled_error);
            self.add_component(sign * scaled);
        }
    }

    fn sign(&self) -> std::cmp::Ordering {
        self.components
            .iter()
            .copied()
            .take(self.length)
            .rev()
            .find(|value| *value != 0.0)
            .map_or(std::cmp::Ordering::Equal, |value| value.total_cmp(&0.0))
    }
}

fn two_sum(left: f64, right: f64) -> (f64, f64) {
    let sum = left + right;
    let right_virtual = sum - left;
    let left_virtual = sum - right_virtual;
    let right_roundoff = right - right_virtual;
    let left_roundoff = left - left_virtual;
    (sum, left_roundoff + right_roundoff)
}

fn validate_positive_definite_metric<const D: usize>(
    metric: &[[f64; D]; D],
) -> Result<(), AnisotropyError<D>>
where
    Dim<D>: SupportedDimension,
{
    for (pivot, row) in metric.iter().enumerate() {
        if row[pivot] <= 0.0 {
            return Err(AnisotropyError::MetricNotPositiveDefinite {
                pivot,
                residual: row[pivot],
            });
        }
    }

    let scaled = power_of_two_equilibrated_metric(metric)?;
    for first in 0..D {
        for second in (first + 1)..D {
            let bound = (scaled[first][first] * scaled[second][second]).sqrt();
            if scaled[first][second].abs() >= bound {
                let residual = scaled[first][first].mul_add(
                    scaled[second][second],
                    -scaled[first][second] * scaled[second][first],
                );
                return Err(AnisotropyError::MetricNotPositiveDefinite {
                    pivot: second,
                    residual,
                });
            }
        }
    }

    if D >= 2 {
        let mut determinant = ExactExpansion::zero();
        determinant.add_product(scaled[0][0], scaled[1][1], 1.0);
        determinant.add_product(scaled[0][1], scaled[1][0], -1.0);
        if determinant.sign() != std::cmp::Ordering::Greater {
            return Err(AnisotropyError::MetricNotPositiveDefinite {
                pivot: 1,
                residual: scaled[0][0].mul_add(scaled[1][1], -scaled[0][1] * scaled[1][0]),
            });
        }
    }

    if D == 3 {
        let mut determinant = ExactExpansion::zero();
        for (first, second, third, sign) in [
            (0, 1, 2, 1.0),
            (1, 2, 0, 1.0),
            (2, 0, 1, 1.0),
            (2, 1, 0, -1.0),
            (1, 0, 2, -1.0),
            (0, 2, 1, -1.0),
        ] {
            determinant.add_triple_product(
                scaled[0][first],
                scaled[1][second],
                scaled[2][third],
                sign,
            );
        }
        if determinant.sign() != std::cmp::Ordering::Greater {
            return Err(AnisotropyError::MetricNotPositiveDefinite {
                pivot: 2,
                residual: approximate_three_determinant(&scaled),
            });
        }
    }
    Ok(())
}

fn power_of_two_equilibrated_metric<const D: usize>(
    metric: &[[f64; D]; D],
) -> Result<[[f64; D]; D], AnisotropyError<D>>
where
    Dim<D>: SupportedDimension,
{
    let exponents: [i32; D] = std::array::from_fn(|axis| {
        let exponent = binary_exponent(metric[axis][axis]);
        -exponent.div_euclid(2)
    });
    let mut scaled = [[0.0; D]; D];
    for (row, output_row) in scaled.iter_mut().enumerate() {
        for (column, output) in output_row.iter_mut().enumerate() {
            let value =
                scale_by_power_of_two(metric[row][column], exponents[row] + exponents[column]);
            if !value.is_finite() || (value == 0.0 && metric[row][column] != 0.0) {
                return Err(AnisotropyError::NonRepresentableMetricFactor { row, column });
            }
            *output = value;
        }
    }
    Ok(scaled)
}

fn binary_exponent(value: f64) -> i32 {
    let bits = value.to_bits();
    let exponent_bits = ((bits >> 52) & 0x7ff) as i32;
    if exponent_bits != 0 {
        exponent_bits - 1023
    } else {
        let fraction = bits & ((1_u64 << 52) - 1);
        let highest_bit = 63 - fraction.leading_zeros().cast_signed();
        -1074 + highest_bit
    }
}

fn scale_by_power_of_two(value: f64, exponent: i32) -> f64 {
    let mut scaled = value;
    let mut remaining = exponent;
    while remaining > 0 {
        let step = remaining.min(1023);
        scaled *= 2.0_f64.powi(step);
        remaining -= step;
    }
    while remaining < 0 {
        let step = remaining.max(-1022);
        scaled *= 2.0_f64.powi(step);
        remaining -= step;
    }
    scaled
}

fn approximate_three_determinant<const D: usize>(matrix: &[[f64; D]; D]) -> f64 {
    let positive = matrix[0][0] * matrix[1][1] * matrix[2][2]
        + matrix[0][1] * matrix[1][2] * matrix[2][0]
        + matrix[0][2] * matrix[1][0] * matrix[2][1];
    let negative = matrix[0][2] * matrix[1][1] * matrix[2][0]
        + matrix[0][1] * matrix[1][0] * matrix[2][2]
        + matrix[0][0] * matrix[1][2] * matrix[2][1];
    positive - negative
}

#[allow(clippy::needless_range_loop)]
fn cholesky_transpose<const D: usize>(
    metric: [[f64; D]; D],
) -> Result<[[f64; D]; D], AnisotropyError<D>>
where
    Dim<D>: SupportedDimension,
{
    let mut lower: [[f64; D]; D] = [[0.0; D]; D];
    for row in 0..D {
        for column in 0..=row {
            let mut residual = metric[row][column];
            for inner in 0..column {
                let product = lower[row][inner] * lower[column][inner];
                if !product.is_finite() {
                    return Err(AnisotropyError::NonRepresentableMetricFactor { row, column });
                }
                residual -= product;
                if !residual.is_finite() {
                    return Err(AnisotropyError::NonRepresentableMetricFactor { row, column });
                }
            }
            if row == column {
                if residual <= 0.0 {
                    return Err(AnisotropyError::MetricNotPositiveDefinite {
                        pivot: row,
                        residual,
                    });
                }
                lower[row][column] = residual.sqrt();
            } else {
                let value = residual / lower[column][column];
                if !value.is_finite() {
                    return Err(AnisotropyError::NonRepresentableMetricFactor { row, column });
                }
                lower[row][column] = value;
            }
        }
    }
    Ok(std::array::from_fn(|row| {
        std::array::from_fn(|column| lower[column][row])
    }))
}

fn transform_singular_values<const D: usize>(
    transform: [[f64; D]; D],
) -> Result<[f64; D], AnisotropyError<D>>
where
    Dim<D>: SupportedDimension,
{
    let scale = transform
        .iter()
        .flatten()
        .copied()
        .map(f64::abs)
        .fold(0.0_f64, f64::max);
    if scale == 0.0 {
        return Ok([0.0; D]);
    }
    let mut working = transform.map(|row| row.map(|value| value / scale));

    // Fixed-sweep one-sided Jacobi SVD for the only supported sizes D<=3.
    // There is no convergence tolerance that could act as a rank decision.
    for _ in 0..32 {
        for left in 0..D {
            for right in (left + 1)..D {
                let mut alpha = 0.0;
                let mut beta = 0.0;
                let mut gamma = 0.0;
                for row in &working {
                    alpha += row[left] * row[left];
                    beta += row[right] * row[right];
                    gamma += row[left] * row[right];
                }
                if gamma == 0.0 {
                    continue;
                }
                let tau = (beta - alpha) / (2.0 * gamma);
                let tangent = if tau >= 0.0 {
                    1.0 / (tau + tau.hypot(1.0))
                } else {
                    -1.0 / (-tau + tau.hypot(1.0))
                };
                if tangent == 0.0 {
                    continue;
                }
                let cosine = 1.0 / tangent.hypot(1.0);
                let sine = cosine * tangent;
                for row in &mut working {
                    let old_left = row[left];
                    let old_right = row[right];
                    row[left] = cosine.mul_add(old_left, -sine * old_right);
                    row[right] = sine.mul_add(old_left, cosine * old_right);
                }
            }
        }
    }

    let mut singular_values = [0.0; D];
    for column in 0..D {
        let column_scale = working
            .iter()
            .map(|row| row[column].abs())
            .fold(0.0_f64, f64::max);
        if column_scale == 0.0 {
            singular_values[column] = 0.0;
            continue;
        }
        let scaled_norm = working
            .iter()
            .map(|row| {
                let value = row[column] / column_scale;
                value * value
            })
            .sum::<f64>()
            .sqrt();
        singular_values[column] = scale * column_scale * scaled_norm;
    }
    for first in 0..D {
        for second in (first + 1)..D {
            if singular_values[second] > singular_values[first] {
                singular_values.swap(first, second);
            }
        }
    }
    for (index, singular_value) in singular_values.iter().copied().enumerate() {
        if !singular_value.is_finite() {
            return Err(AnisotropyError::NonRepresentableSingularValue { index });
        }
    }
    Ok(singular_values)
}

fn diagnostics_from_singular_values<const D: usize>(
    singular_values: [f64; D],
) -> Result<AnisotropyDiagnostics<D>, AnisotropyError<D>>
where
    Dim<D>: SupportedDimension,
{
    for (index, singular_value) in singular_values.iter().copied().enumerate() {
        if singular_value <= 0.0 {
            return Err(AnisotropyError::NonRepresentableSingularValue { index });
        }
    }
    let condition_number = singular_values[0] / singular_values[D - 1];
    if !condition_number.is_finite() {
        return Err(AnisotropyError::NonRepresentableConditionNumber);
    }
    Ok(AnisotropyDiagnostics {
        singular_values,
        condition_number,
    })
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
