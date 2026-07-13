//! Independent truth, invariance, and error-path tests for radial calculus.

use std::error::Error;

use georbf::{
    Dim, KernelArgument, KernelCalculusError, Point, RadialDerivativeOrder,
    RadialExpansionCoefficient, RadialExpansionCoefficients, RadialJet, RadialJetLocation,
    RadialSeparation, SpatialKernelJet, SupportedDimension,
};

const REL_TOLERANCE: f64 = 2.0e-12;

type TestResult<T = ()> = Result<T, Box<dyn Error>>;
type Matrix<const D: usize> = [[f64; D]; D];
type ThirdTensor<const D: usize> = [[[f64; D]; D]; D];
type PolynomialTruth<const D: usize> = (f64, [f64; D], Matrix<D>, ThirdTensor<D>);

fn assert_close(actual: f64, expected: f64, relative: f64, absolute: f64) {
    let tolerance = absolute + relative * actual.abs().max(expected.abs());
    assert!(
        (actual - expected).abs() <= tolerance,
        "actual {actual:.17e}, expected {expected:.17e}, tolerance {tolerance:.3e}"
    );
}

fn assert_array_close<const D: usize>(actual: &[f64; D], expected: &[f64; D]) {
    for index in 0..D {
        assert_close(actual[index], expected[index], REL_TOLERANCE, 0.0);
    }
}

fn assert_matrix_close<const D: usize>(actual: &[[f64; D]; D], expected: &[[f64; D]; D]) {
    for row in 0..D {
        assert_array_close(&actual[row], &expected[row]);
    }
}

fn assert_tensor_close<const D: usize>(actual: &[[[f64; D]; D]; D], expected: &[[[f64; D]; D]; D]) {
    for first in 0..D {
        assert_matrix_close(&actual[first], &expected[first]);
    }
}

fn sixth_power_jet<const D: usize>(
    query: [f64; D],
    center: [f64; D],
) -> TestResult<SpatialKernelJet<D>>
where
    Dim<D>: SupportedDimension,
{
    let separation = RadialSeparation::try_new(Point::try_new(query)?, Point::try_new(center)?)?;
    let radius = separation.radius();
    let radial = if separation.is_center() {
        RadialJet::try_center(0.0, 0.0)?
    } else {
        let expansion_coefficients =
            RadialExpansionCoefficients::try_new(6.0 * radius.powi(4), 24.0 * radius.powi(3))?;
        RadialJet::try_away_with_expansion(
            radius.powi(6),
            6.0 * radius.powi(5),
            30.0 * radius.powi(4),
            120.0 * radius.powi(3),
            expansion_coefficients,
        )?
    };
    Ok(SpatialKernelJet::try_new(separation, radial)?)
}

fn sixth_power_truth<const D: usize>(query: [f64; D], center: [f64; D]) -> PolynomialTruth<D> {
    let displacement = std::array::from_fn::<_, D, _>(|index| query[index] - center[index]);
    let squared_radius = displacement.iter().map(|value| value * value).sum::<f64>();
    let value = squared_radius.powi(3);
    let gradient = displacement.map(|value| 6.0 * squared_radius.powi(2) * value);
    let hessian = std::array::from_fn(|row| {
        std::array::from_fn(|column| {
            24.0 * squared_radius * displacement[row] * displacement[column]
                + if row == column {
                    6.0 * squared_radius.powi(2)
                } else {
                    0.0
                }
        })
    });
    let third = std::array::from_fn(|first| {
        std::array::from_fn(|second| {
            std::array::from_fn(|third| {
                let delta_terms = if first == second {
                    displacement[third]
                } else {
                    0.0
                } + if first == third {
                    displacement[second]
                } else {
                    0.0
                } + if second == third {
                    displacement[first]
                } else {
                    0.0
                };
                48.0 * displacement[first] * displacement[second] * displacement[third]
                    + 24.0 * squared_radius * delta_terms
            })
        })
    });
    (value, gradient, hessian, third)
}

fn check_sixth_power_truth<const D: usize>(query: [f64; D], center: [f64; D]) -> TestResult
where
    Dim<D>: SupportedDimension,
{
    let jet = sixth_power_jet(query, center)?;
    let (value, gradient, hessian, third) = sixth_power_truth(query, center);
    assert_close(jet.value(), value, REL_TOLERANCE, 0.0);
    assert_array_close(&jet.first_derivative(KernelArgument::Query), &gradient);
    assert_matrix_close(
        &jet.second_derivative([KernelArgument::Query, KernelArgument::Query]),
        &hessian,
    );
    assert_tensor_close(
        &jet.third_derivative([
            KernelArgument::Query,
            KernelArgument::Query,
            KernelArgument::Query,
        ]),
        &third,
    );
    Ok(())
}

#[test]
fn radial_calculus_matches_independent_polynomial_truth_in_every_dimension() -> TestResult {
    check_sixth_power_truth([2.0], [-1.0])?;
    check_sixth_power_truth([1.5, -0.5], [-0.5, 1.5])?;
    check_sixth_power_truth([1.0, -2.0, 0.5], [-1.0, 1.0, -0.5])?;
    Ok(())
}

fn check_center_limits<const D: usize>(point: [f64; D]) -> TestResult
where
    Dim<D>: SupportedDimension,
{
    let point = Point::try_new(point)?;
    let separation = RadialSeparation::try_new(point, point)?;
    let radial = RadialJet::try_center(7.0, 2.0)?;
    let jet = SpatialKernelJet::try_new(separation, radial)?;

    assert_close(jet.value(), 7.0, 0.0, 0.0);
    assert_array_close(&jet.first_derivative(KernelArgument::Query), &[0.0; D]);
    assert_array_close(&jet.first_derivative(KernelArgument::Center), &[0.0; D]);

    let xx = jet.second_derivative([KernelArgument::Query, KernelArgument::Query]);
    let xy = jet.second_derivative([KernelArgument::Query, KernelArgument::Center]);
    let yy = jet.second_derivative([KernelArgument::Center, KernelArgument::Center]);
    for row in 0..D {
        for column in 0..D {
            let expected = if row == column { 2.0 } else { 0.0 };
            assert_close(xx[row][column], expected, 0.0, 0.0);
            assert_close(xy[row][column], -expected, 0.0, 0.0);
            assert_close(yy[row][column], expected, 0.0, 0.0);
        }
    }

    for arguments in [
        [KernelArgument::Query; 3],
        [
            KernelArgument::Query,
            KernelArgument::Query,
            KernelArgument::Center,
        ],
        [
            KernelArgument::Query,
            KernelArgument::Center,
            KernelArgument::Center,
        ],
        [KernelArgument::Center; 3],
    ] {
        assert_tensor_close(&jet.third_derivative(arguments), &[[[0.0; D]; D]; D]);
    }
    Ok(())
}

#[test]
fn smooth_center_uses_analytic_limits_without_radial_quotients() -> TestResult {
    check_center_limits([0.0])?;
    check_center_limits([3.0, -4.0])?;
    check_center_limits([1.0, 2.0, 3.0])?;
    Ok(())
}

#[test]
fn separation_radius_and_unit_displacement_are_stable() -> TestResult {
    let tiny = f64::from_bits(1);
    let one = RadialSeparation::try_new(Point::try_new([tiny])?, Point::try_new([0.0])?)?;
    assert_eq!(one.radius().to_bits(), tiny.to_bits());
    let Some(one_unit) = one.unit_displacement() else {
        return Err("away separation must have a unit displacement".into());
    };
    assert_array_close(one_unit, &[1.0]);

    let two_scale = f64::MAX / 4.0;
    let two = RadialSeparation::try_new(
        Point::try_new([two_scale, -two_scale])?,
        Point::try_new([0.0, 0.0])?,
    )?;
    assert_close(two.radius(), two_scale * 2.0_f64.sqrt(), 4.0e-16, 0.0);
    let Some(two_unit) = two.unit_displacement() else {
        return Err("away separation must have a unit displacement".into());
    };
    assert_array_close(two_unit, &[2.0_f64.sqrt().recip(), -2.0_f64.sqrt().recip()]);

    let three = RadialSeparation::try_new(
        Point::try_new([3.0, 4.0, 12.0])?,
        Point::try_new([0.0, 0.0, 0.0])?,
    )?;
    assert_close(three.radius(), 13.0, 0.0, 0.0);
    let Some(three_unit) = three.unit_displacement() else {
        return Err("away separation must have a unit displacement".into());
    };
    assert_array_close(three_unit, &[3.0 / 13.0, 4.0 / 13.0, 12.0 / 13.0]);

    let center =
        RadialSeparation::try_new(Point::try_new([-0.0, 2.0])?, Point::try_new([0.0, 2.0])?)?;
    assert!(center.is_center());
    assert_eq!(center.radius().to_bits(), 0.0_f64.to_bits());
    assert!(center.unit_displacement().is_none());
    Ok(())
}

#[test]
fn query_center_signs_exchange_and_tensor_symmetry_are_exact() -> TestResult {
    let query = [1.0, -2.0, 0.5];
    let center = [-1.0, 1.0, -0.5];
    let jet = sixth_power_jet(query, center)?;

    let gx = jet.first_derivative(KernelArgument::Query);
    let gy = jet.first_derivative(KernelArgument::Center);
    assert_array_close(&gy, &gx.map(|value| -value));

    let xx = jet.second_derivative([KernelArgument::Query, KernelArgument::Query]);
    assert_matrix_close(
        &jet.second_derivative([KernelArgument::Center, KernelArgument::Center]),
        &xx,
    );
    assert_matrix_close(
        &jet.second_derivative([KernelArgument::Query, KernelArgument::Center]),
        &xx.map(|row| row.map(|value| -value)),
    );

    let xxx = jet.third_derivative([KernelArgument::Query; 3]);
    assert_tensor_close(
        &jet.third_derivative([
            KernelArgument::Query,
            KernelArgument::Query,
            KernelArgument::Center,
        ]),
        &xxx.map(|matrix| matrix.map(|row| row.map(|value| -value))),
    );
    assert_tensor_close(
        &jet.third_derivative([
            KernelArgument::Query,
            KernelArgument::Center,
            KernelArgument::Center,
        ]),
        &xxx,
    );
    assert_tensor_close(
        &jet.third_derivative([KernelArgument::Center; 3]),
        &xxx.map(|matrix| matrix.map(|row| row.map(|value| -value))),
    );

    for (first, first_matrix) in xxx.iter().enumerate() {
        for (second, second_row) in first_matrix.iter().enumerate() {
            for (third, value) in second_row.iter().copied().enumerate() {
                assert_eq!(value.to_bits(), xxx[first][third][second].to_bits());
                assert_eq!(value.to_bits(), xxx[second][first][third].to_bits());
                assert_eq!(value.to_bits(), xxx[third][second][first].to_bits());
            }
        }
    }

    let exchanged = sixth_power_jet(center, query)?;
    assert_array_close(
        &exchanged.first_derivative(KernelArgument::Query),
        &gx.map(|value| -value),
    );
    assert_matrix_close(
        &exchanged.second_derivative([KernelArgument::Query; 2]),
        &xx,
    );
    assert_tensor_close(
        &exchanged.third_derivative([KernelArgument::Query; 3]),
        &xxx.map(|matrix| matrix.map(|row| row.map(|value| -value))),
    );
    Ok(())
}

fn gaussian_value<const D: usize>(point: [f64; D], alpha: f64) -> f64 {
    let squared_radius = point.iter().map(|value| value * value).sum::<f64>();
    (-alpha * squared_radius).exp()
}

fn gaussian_spatial_jet<const D: usize>(
    point: [f64; D],
    alpha: f64,
) -> TestResult<SpatialKernelJet<D>>
where
    Dim<D>: SupportedDimension,
{
    let separation = RadialSeparation::try_new(Point::try_new(point)?, Point::try_new([0.0; D])?)?;
    let radius = separation.radius();
    let value = (-alpha * radius * radius).exp();
    let radial = RadialJet::try_away_with_expansion(
        value,
        -2.0 * alpha * radius * value,
        (4.0 * alpha * alpha * radius * radius - 2.0 * alpha) * value,
        (12.0 * alpha * alpha * radius - 8.0 * alpha * alpha * alpha * radius.powi(3)) * value,
        RadialExpansionCoefficients::try_new(
            -2.0 * alpha * value,
            4.0 * alpha * alpha * radius * value,
        )?,
    )?;
    Ok(SpatialKernelJet::try_new(separation, radial)?)
}

fn check_finite_difference<const D: usize>(point: [f64; D]) -> TestResult
where
    Dim<D>: SupportedDimension,
{
    let alpha = 0.7;
    let step = 2.0e-3;
    let jet = gaussian_spatial_jet(point, alpha)?;
    let sample = |offset: f64| {
        let mut shifted = point;
        shifted[0] += offset;
        gaussian_value(shifted, alpha)
    };
    let minus_two = sample(-2.0 * step);
    let minus_one = sample(-step);
    let zero = sample(0.0);
    let plus_one = sample(step);
    let plus_two = sample(2.0 * step);

    let first = (minus_two - 8.0 * minus_one + 8.0 * plus_one - plus_two) / (12.0 * step);
    let second = (-plus_two + 16.0 * plus_one - 30.0 * zero + 16.0 * minus_one - minus_two)
        / (12.0 * step * step);
    let third = (plus_two - 2.0 * plus_one + 2.0 * minus_one - minus_two) / (2.0 * step.powi(3));

    assert_close(
        jet.first_derivative(KernelArgument::Query)[0],
        first,
        2.0e-9,
        2.0e-10,
    );
    assert_close(
        jet.second_derivative([KernelArgument::Query; 2])[0][0],
        second,
        2.0e-8,
        2.0e-8,
    );
    assert_close(
        jet.third_derivative([KernelArgument::Query; 3])[0][0][0],
        third,
        3.0e-5,
        3.0e-5,
    );
    Ok(())
}

#[test]
fn gaussian_spatial_derivatives_match_independent_finite_differences() -> TestResult {
    check_finite_difference([0.7])?;
    check_finite_difference([0.7, -0.4])?;
    check_finite_difference([0.7, -0.4, 0.9])?;
    Ok(())
}

#[test]
fn stable_coefficients_preserve_near_center_gaussian_third_derivatives() -> TestResult {
    let alpha: f64 = 0.7;
    let radius: f64 = 1.0e-10;
    let value = (-alpha * radius * radius).exp();
    let jet = gaussian_spatial_jet([radius, 0.0, 0.0], alpha)?;
    let third = jet.third_derivative([KernelArgument::Query; 3]);

    let expected_pure =
        (12.0 * alpha * alpha * radius - 8.0 * alpha.powi(3) * radius.powi(3)) * value;
    let expected_mixed = 4.0 * alpha * alpha * radius * value;
    assert_close(third[0][0][0], expected_pure, 2.0e-15, 0.0);
    assert_close(third[0][1][1], expected_mixed, 2.0e-15, 0.0);
    assert_close(third[0][2][2], expected_mixed, 2.0e-15, 0.0);
    Ok(())
}

#[test]
fn radial_calculus_is_rotation_covariant_through_third_order() -> TestResult {
    let alpha = 0.7;
    let point = [0.7, -0.4, 0.9];
    let angle: f64 = 0.37;
    let (sine, cosine) = angle.sin_cos();
    let rotation = [[cosine, -sine, 0.0], [sine, cosine, 0.0], [0.0, 0.0, 1.0]];
    let rotated_point: [f64; 3] = std::array::from_fn(|row| {
        (0..3)
            .map(|column| rotation[row][column] * point[column])
            .sum::<f64>()
    });

    let original = gaussian_spatial_jet(point, alpha)?;
    let rotated = gaussian_spatial_jet(rotated_point, alpha)?;
    assert_close(rotated.value(), original.value(), REL_TOLERANCE, 0.0);

    let gradient = original.first_derivative(KernelArgument::Query);
    let expected_gradient: [f64; 3] = std::array::from_fn(|row| {
        (0..3)
            .map(|column| rotation[row][column] * gradient[column])
            .sum::<f64>()
    });
    assert_array_close(
        &rotated.first_derivative(KernelArgument::Query),
        &expected_gradient,
    );

    let hessian = original.second_derivative([KernelArgument::Query; 2]);
    let expected_hessian: Matrix<3> = std::array::from_fn(|row| {
        std::array::from_fn(|column| {
            (0..3)
                .flat_map(|first| {
                    (0..3).map(move |second| {
                        rotation[row][first] * rotation[column][second] * hessian[first][second]
                    })
                })
                .sum::<f64>()
        })
    });
    assert_matrix_close(
        &rotated.second_derivative([KernelArgument::Query; 2]),
        &expected_hessian,
    );

    let third = original.third_derivative([KernelArgument::Query; 3]);
    let expected_third: ThirdTensor<3> = std::array::from_fn(|first| {
        std::array::from_fn(|second| {
            std::array::from_fn(|third_axis| {
                let mut value = 0.0;
                for source_first in 0..3 {
                    for source_second in 0..3 {
                        for source_third in 0..3 {
                            value += rotation[first][source_first]
                                * rotation[second][source_second]
                                * rotation[third_axis][source_third]
                                * third[source_first][source_second][source_third];
                        }
                    }
                }
                value
            })
        })
    });
    assert_tensor_close(
        &rotated.third_derivative([KernelArgument::Query; 3]),
        &expected_third,
    );
    Ok(())
}

#[test]
fn invalid_radial_separation_and_location_states_are_structured_errors() -> TestResult {
    for (value, expected_order) in [
        (f64::NAN, RadialDerivativeOrder::Value),
        (f64::INFINITY, RadialDerivativeOrder::First),
        (f64::NEG_INFINITY, RadialDerivativeOrder::Second),
        (f64::NAN, RadialDerivativeOrder::Third),
    ] {
        let values = match expected_order {
            RadialDerivativeOrder::Value => [value, 0.0, 0.0, 0.0],
            RadialDerivativeOrder::First => [0.0, value, 0.0, 0.0],
            RadialDerivativeOrder::Second => [0.0, 0.0, value, 0.0],
            RadialDerivativeOrder::Third => [0.0, 0.0, 0.0, value],
        };
        assert!(matches!(
            RadialJet::try_away(values[0], values[1], values[2], values[3]),
            Err(KernelCalculusError::NonFiniteRadialDerivative { order, .. })
                if order == expected_order
        ));
    }
    assert!(matches!(
        RadialJet::try_center(0.0, f64::NAN),
        Err(KernelCalculusError::NonFiniteRadialDerivative {
            order: RadialDerivativeOrder::Second,
            ..
        })
    ));
    for (first_over_radius, second_remainder_over_radius, expected) in [
        (f64::NAN, 0.0, RadialExpansionCoefficient::FirstOverRadius),
        (
            0.0,
            f64::INFINITY,
            RadialExpansionCoefficient::SecondRemainderOverRadius,
        ),
    ] {
        assert!(matches!(
            RadialExpansionCoefficients::try_new(
                first_over_radius,
                second_remainder_over_radius,
            ),
            Err(KernelCalculusError::NonFiniteRadialExpansionCoefficient {
                coefficient,
                ..
            }) if coefficient == expected
        ));
    }

    assert!(matches!(
        RadialSeparation::try_new(Point::try_new([f64::MAX])?, Point::try_new([-f64::MAX])?,),
        Err(KernelCalculusError::NonFiniteDisplacementComponent { axis: 0 })
    ));
    assert!(matches!(
        RadialSeparation::try_new(
            Point::try_new([f64::MAX, f64::MAX])?,
            Point::try_new([0.0, 0.0])?,
        ),
        Err(KernelCalculusError::NonRepresentableRadius)
    ));

    let center = RadialSeparation::try_new(Point::try_new([1.0])?, Point::try_new([1.0])?)?;
    let away_jet = RadialJet::try_away(1.0, 0.0, 0.0, 0.0)?;
    assert!(matches!(
        SpatialKernelJet::try_new(center, away_jet),
        Err(KernelCalculusError::JetLocationMismatch {
            separation: RadialJetLocation::Center,
            jet: RadialJetLocation::AwayFromCenter,
        })
    ));

    let away = RadialSeparation::try_new(Point::try_new([2.0])?, Point::try_new([1.0])?)?;
    let center_jet = RadialJet::try_center(1.0, 2.0)?;
    assert!(matches!(
        SpatialKernelJet::try_new(away, center_jet),
        Err(KernelCalculusError::JetLocationMismatch {
            separation: RadialJetLocation::AwayFromCenter,
            jet: RadialJetLocation::Center,
        })
    ));

    let two_dimensional =
        RadialSeparation::try_new(Point::try_new([1.0, 0.0])?, Point::try_new([0.0, 0.0])?)?;
    let derivatives_only = RadialJet::try_away(1.0, 2.0, 3.0, 4.0)?;
    assert!(matches!(
        SpatialKernelJet::try_new(two_dimensional, derivatives_only),
        Err(KernelCalculusError::MissingRadialExpansionCoefficients { dimension: 2 })
    ));
    Ok(())
}

#[test]
fn one_dimensional_away_expansion_needs_no_radial_quotients() -> TestResult {
    let radius = f64::from_bits(1);
    let separation = RadialSeparation::try_new(Point::try_new([radius])?, Point::try_new([0.0])?)?;
    let radial = RadialJet::try_away(1.0, f64::MAX, -f64::MAX, f64::MAX)?;
    let jet = SpatialKernelJet::try_new(separation, radial)?;

    assert_eq!(
        jet.first_derivative(KernelArgument::Query)[0].to_bits(),
        f64::MAX.to_bits()
    );
    assert_eq!(
        jet.second_derivative([KernelArgument::Query; 2])[0][0].to_bits(),
        (-f64::MAX).to_bits()
    );
    assert_eq!(
        jet.third_derivative([KernelArgument::Query; 3])[0][0][0].to_bits(),
        f64::MAX.to_bits()
    );
    Ok(())
}

#[test]
fn nonrepresentable_third_derivative_is_reported() -> TestResult {
    let third_separation = RadialSeparation::try_new(
        Point::try_new([1.0, 1.0, 1.0])?,
        Point::try_new([0.0, 0.0, 0.0])?,
    )?;
    let third_radial = RadialJet::try_away_with_expansion(
        0.0,
        0.0,
        0.0,
        0.0,
        RadialExpansionCoefficients::try_new(0.0, f64::MAX)?,
    )?;
    assert!(matches!(
        SpatialKernelJet::try_new(third_separation, third_radial),
        Err(KernelCalculusError::NonFiniteThirdDerivative {
            first: 0,
            second: 0,
            third: 0,
        })
    ));
    Ok(())
}

#[test]
fn kernel_calculus_values_are_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<RadialExpansionCoefficients>();
    assert_send_sync::<RadialJet>();
    assert_send_sync::<RadialSeparation<1>>();
    assert_send_sync::<RadialSeparation<2>>();
    assert_send_sync::<RadialSeparation<3>>();
    assert_send_sync::<SpatialKernelJet<1>>();
    assert_send_sync::<SpatialKernelJet<2>>();
    assert_send_sync::<SpatialKernelJet<3>>();
}
