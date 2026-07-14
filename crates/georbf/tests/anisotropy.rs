//! Independent truth and property tests for fixed global anisotropy.

use std::error::Error;

use georbf::{
    AnisotropyConditionPolicy, AnisotropyError, GlobalAnisotropy, KernelArgument, Point,
    RadialExpansionCoefficients, RadialJet, SpatialKernelJet, UnitDirection,
};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

fn assert_close(actual: f64, expected: f64, relative: f64, absolute: f64) {
    let tolerance = absolute.max(relative * actual.abs().max(expected.abs()));
    assert!(
        (actual - expected).abs() <= tolerance,
        "actual={actual:.17e}, expected={expected:.17e}, tolerance={tolerance:.17e}"
    );
}

fn assert_vector<const D: usize>(actual: &[f64; D], expected: &[f64; D], tolerance: f64) {
    for axis in 0..D {
        assert_close(actual[axis], expected[axis], tolerance, tolerance);
    }
}

fn assert_matrix<const D: usize>(actual: &[[f64; D]; D], expected: &[[f64; D]; D], tolerance: f64) {
    for row in 0..D {
        assert_vector(&actual[row], &expected[row], tolerance);
    }
}

#[test]
fn all_global_families_have_independent_distance_truth() -> TestResult {
    let isotropic = GlobalAnisotropy::<1>::try_isotropic(2.0)?;
    let separation =
        isotropic.try_transform_separation(Point::try_new([7.0])?, Point::try_new([1.0])?)?;
    assert_eq!(separation.radius().to_bits(), 3.0_f64.to_bits());
    assert_eq!(isotropic.transform(), &[[0.5]]);
    assert_eq!(isotropic.metric(), &[[0.25]]);
    assert_eq!(isotropic.inverse_transform(), &[[2.0]]);
    assert_eq!(
        isotropic.diagnostics().singular_values()[0].to_bits(),
        0.5_f64.to_bits()
    );
    assert_eq!(
        isotropic.diagnostics().condition_number().to_bits(),
        1.0_f64.to_bits()
    );

    let spheroidal = GlobalAnisotropy::<3>::try_spheroidal(
        UnitDirection::try_new([0.0, 0.0, 1.0])?,
        4.0,
        2.0,
        AnisotropyConditionPolicy::Maximum(2.0),
    )?;
    let separation = spheroidal.try_transform_separation(
        Point::try_new([2.0, 0.0, 4.0])?,
        Point::try_new([0.0, 0.0, 0.0])?,
    )?;
    assert_close(separation.radius(), 2.0_f64.sqrt(), 1.0e-15, 1.0e-15);
    assert_matrix(
        spheroidal.metric(),
        &[[0.25, 0.0, 0.0], [0.0, 0.25, 0.0], [0.0, 0.0, 0.0625]],
        0.0,
    );
    assert_close(
        spheroidal.diagnostics().condition_number(),
        2.0,
        1.0e-15,
        1.0e-15,
    );

    let c = std::f64::consts::FRAC_1_SQRT_2;
    let axes = [
        UnitDirection::try_new([c, c])?,
        UnitDirection::try_new([-c, c])?,
    ];
    let ellipsoidal = GlobalAnisotropy::<2>::try_ellipsoidal(
        axes,
        [2.0, 4.0],
        2.0 * f64::EPSILON,
        AnisotropyConditionPolicy::Maximum(2.1),
    )?;
    let displacement = [-2.0 * c, 6.0 * c];
    let separation = ellipsoidal
        .try_transform_separation(Point::try_new(displacement)?, Point::try_new([0.0, 0.0])?)?;
    assert_close(separation.radius(), 2.0_f64.sqrt(), 1.0e-15, 1.0e-15);

    let metric = [[4.0, 1.0], [1.0, 1.0]];
    let user_metric =
        GlobalAnisotropy::<2>::try_from_metric(metric, AnisotropyConditionPolicy::Unbounded)?;
    let separation = user_metric
        .try_transform_separation(Point::try_new([1.0, 2.0])?, Point::try_new([0.0, 0.0])?)?;
    assert_close(separation.radius(), 12.0_f64.sqrt(), 1.0e-15, 1.0e-15);
    assert_eq!(user_metric.metric(), &metric);

    let user_transform = GlobalAnisotropy::<2>::try_from_transform(
        [[2.0, 1.0], [0.0, 3.0]],
        AnisotropyConditionPolicy::Unbounded,
    )?;
    assert_matrix(user_transform.metric(), &[[4.0, 2.0], [2.0, 10.0]], 0.0);
    assert_matrix(
        user_transform.inverse_transform(),
        &[[0.5, -1.0 / 6.0], [0.0, 1.0 / 3.0]],
        1.0e-15,
    );
    Ok(())
}

#[test]
fn spheroidal_distance_is_rotation_covariant_in_2d_and_3d() -> TestResult {
    let angle = 0.731_f64;
    let (sine, cosine) = angle.sin_cos();
    let rotate_2d = |value: [f64; 2]| {
        [
            cosine.mul_add(value[0], -sine * value[1]),
            sine.mul_add(value[0], cosine * value[1]),
        ]
    };
    let axis = [0.6, 0.8];
    let query = [2.0, -1.5];
    let center = [-0.25, 0.75];
    let original = GlobalAnisotropy::<2>::try_spheroidal(
        UnitDirection::try_new(axis)?,
        5.0,
        1.25,
        AnisotropyConditionPolicy::Unbounded,
    )?;
    let rotated = GlobalAnisotropy::<2>::try_spheroidal(
        UnitDirection::try_new(rotate_2d(axis))?,
        5.0,
        1.25,
        AnisotropyConditionPolicy::Unbounded,
    )?;
    let radius = original
        .try_transform_separation(Point::try_new(query)?, Point::try_new(center)?)?
        .radius();
    let rotated_radius = rotated
        .try_transform_separation(
            Point::try_new(rotate_2d(query))?,
            Point::try_new(rotate_2d(center))?,
        )?
        .radius();
    assert_close(rotated_radius, radius, 2.0e-15, 2.0e-15);

    let rotate_3d = |value: [f64; 3]| {
        [
            cosine.mul_add(value[0], -sine * value[1]),
            sine.mul_add(value[0], cosine * value[1]),
            value[2],
        ]
    };
    let axis = [0.36, 0.48, 0.8];
    let query = [2.0, -1.5, 0.75];
    let center = [-0.25, 0.75, -2.0];
    let original = GlobalAnisotropy::<3>::try_spheroidal(
        UnitDirection::try_new(axis)?,
        4.0,
        1.0,
        AnisotropyConditionPolicy::Unbounded,
    )?;
    let rotated = GlobalAnisotropy::<3>::try_spheroidal(
        UnitDirection::try_new(rotate_3d(axis))?,
        4.0,
        1.0,
        AnisotropyConditionPolicy::Unbounded,
    )?;
    let radius = original
        .try_transform_separation(Point::try_new(query)?, Point::try_new(center)?)?
        .radius();
    let rotated_radius = rotated
        .try_transform_separation(
            Point::try_new(rotate_3d(query))?,
            Point::try_new(rotate_3d(center))?,
        )?
        .radius();
    assert_close(rotated_radius, radius, 3.0e-15, 3.0e-15);
    Ok(())
}

#[test]
fn coordinate_and_axis_length_scaling_cancel_exactly() -> TestResult {
    let axes = [
        UnitDirection::try_new([1.0, 0.0, 0.0])?,
        UnitDirection::try_new([0.0, 1.0, 0.0])?,
        UnitDirection::try_new([0.0, 0.0, 1.0])?,
    ];
    let base = GlobalAnisotropy::<3>::try_ellipsoidal(
        axes,
        [2.0, 3.0, 5.0],
        0.0,
        AnisotropyConditionPolicy::Unbounded,
    )?;
    let scale = 1.0e75;
    let scaled = GlobalAnisotropy::<3>::try_ellipsoidal(
        axes,
        [2.0 * scale, 3.0 * scale, 5.0 * scale],
        0.0,
        AnisotropyConditionPolicy::Unbounded,
    )?;
    let query = [4.0, -9.0, 20.0];
    let center = [-2.0, 3.0, -5.0];
    let radius = base
        .try_transform_separation(Point::try_new(query)?, Point::try_new(center)?)?
        .radius();
    let scaled_radius = scaled
        .try_transform_separation(
            Point::try_new(query.map(|value| value * scale))?,
            Point::try_new(center.map(|value| value * scale))?,
        )?
        .radius();
    assert_close(scaled_radius, radius, 2.0e-15, 2.0e-15);
    Ok(())
}

#[test]
fn singular_values_match_closed_form_two_by_two_truth() -> TestResult {
    let anisotropy = GlobalAnisotropy::<2>::try_from_transform(
        [[3.0, 1.0], [0.0, 2.0]],
        AnisotropyConditionPolicy::Unbounded,
    )?;
    let discriminant = 13.0_f64.sqrt();
    let expected = [(7.0 + discriminant).sqrt(), (7.0 - discriminant).sqrt()];
    assert_vector(
        anisotropy.diagnostics().singular_values(),
        &expected,
        2.0e-15,
    );
    assert_close(
        anisotropy.diagnostics().condition_number(),
        expected[0] / expected[1],
        3.0e-15,
        3.0e-15,
    );
    Ok(())
}

#[test]
fn high_axis_ratios_preserve_representable_spheroids_and_diagnostics() -> TestResult {
    let one = GlobalAnisotropy::<1>::try_spheroidal(
        UnitDirection::try_new([1.0])?,
        1.0,
        1.0e-100,
        AnisotropyConditionPolicy::Unbounded,
    )?;
    assert_eq!(one.transform()[0][0].to_bits(), 1.0_f64.to_bits());
    assert_eq!(one.metric()[0][0].to_bits(), 1.0_f64.to_bits());

    let two = GlobalAnisotropy::<2>::try_spheroidal(
        UnitDirection::try_new([1.0, 0.0])?,
        1.0,
        1.0e-100,
        AnisotropyConditionPolicy::Unbounded,
    )?;
    assert_matrix(two.transform(), &[[1.0, 0.0], [0.0, 1.0e100]], 0.0);
    assert_matrix(two.metric(), &[[1.0, 0.0], [0.0, 1.0e200]], 0.0);
    assert_vector(
        two.diagnostics().singular_values(),
        &[1.0e100, 1.0],
        2.0e-15,
    );
    assert_close(two.diagnostics().condition_number(), 1.0e100, 2.0e-15, 0.0);

    let three = GlobalAnisotropy::<3>::try_from_transform(
        [[1.0e100, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0e-50]],
        AnisotropyConditionPolicy::Unbounded,
    )?;
    assert_vector(
        three.diagnostics().singular_values(),
        &[1.0e100, 1.0, 1.0e-50],
        2.0e-15,
    );
    assert_close(
        three.diagnostics().condition_number(),
        1.0e150,
        3.0e-15,
        0.0,
    );
    Ok(())
}

#[test]
fn exact_spd_sign_handles_three_dimensional_determinant_boundaries() -> TestResult {
    let positive_correlation = -0.5 + f64::EPSILON;
    let positive = [
        [1.0, positive_correlation, positive_correlation],
        [positive_correlation, 1.0, positive_correlation],
        [positive_correlation, positive_correlation, 1.0],
    ];
    let accepted =
        GlobalAnisotropy::<3>::try_from_metric(positive, AnisotropyConditionPolicy::Unbounded)?;
    assert_eq!(accepted.metric(), &positive);

    let negative_correlation = -0.5 - f64::EPSILON;
    let negative = [
        [1.0, negative_correlation, negative_correlation],
        [negative_correlation, 1.0, negative_correlation],
        [negative_correlation, negative_correlation, 1.0],
    ];
    assert!(matches!(
        GlobalAnisotropy::<3>::try_from_metric(negative, AnisotropyConditionPolicy::Unbounded),
        Err(AnisotropyError::MetricNotPositiveDefinite { pivot: 2, .. })
    ));
    Ok(())
}

#[test]
fn chain_rule_matches_independent_polynomial_truth_through_third_order() -> TestResult {
    let anisotropy = GlobalAnisotropy::<2>::try_from_transform(
        [[2.0, 0.5], [-1.0, 3.0]],
        AnisotropyConditionPolicy::Unbounded,
    )?;
    let displacement = [0.7, -1.1];
    let separation = anisotropy
        .try_transform_separation(Point::try_new(displacement)?, Point::try_new([0.0, 0.0])?)?;
    let radius = separation.radius();
    let radial = RadialJet::try_away_with_expansion(
        radius.powi(6),
        6.0 * radius.powi(5),
        30.0 * radius.powi(4),
        120.0 * radius.powi(3),
        RadialExpansionCoefficients::try_new(6.0 * radius.powi(4), 24.0 * radius.powi(3))?,
    )?;
    let transformed = SpatialKernelJet::try_new(separation, radial)?;
    let jet = anisotropy.try_transform_spatial_jet(transformed)?;

    let metric = anisotropy.metric();
    let metric_times_displacement = [
        metric[0][0].mul_add(displacement[0], metric[0][1] * displacement[1]),
        metric[1][0].mul_add(displacement[0], metric[1][1] * displacement[1]),
    ];
    let q = displacement[0].mul_add(
        metric_times_displacement[0],
        displacement[1] * metric_times_displacement[1],
    );
    assert_close(jet.value(), q.powi(3), 2.0e-14, 2.0e-14);

    let expected_gradient = metric_times_displacement.map(|value| 6.0 * q.powi(2) * value);
    let gradient = jet.first_derivative(KernelArgument::Query);
    assert_vector(&gradient, &expected_gradient, 3.0e-14);

    let expected_hessian = std::array::from_fn(|row| {
        std::array::from_fn(|column| {
            24.0 * q * metric_times_displacement[row] * metric_times_displacement[column]
                + 6.0 * q.powi(2) * metric[row][column]
        })
    });
    let hessian = jet.second_derivative([KernelArgument::Query; 2]);
    assert_matrix(&hessian, &expected_hessian, 5.0e-14);

    let expected_third: [[[f64; 2]; 2]; 2] = std::array::from_fn(|first| {
        std::array::from_fn(|second| {
            std::array::from_fn(|third| {
                48.0 * metric_times_displacement[first]
                    * metric_times_displacement[second]
                    * metric_times_displacement[third]
                    + 24.0
                        * q
                        * (metric[first][second] * metric_times_displacement[third]
                            + metric[first][third] * metric_times_displacement[second]
                            + metric[second][third] * metric_times_displacement[first])
            })
        })
    });
    let third = jet.third_derivative([KernelArgument::Query; 3]);
    for first in 0..2 {
        assert_matrix(&third[first], &expected_third[first], 8.0e-14);
    }

    let center_first = jet.first_derivative(KernelArgument::Center);
    assert_vector(&center_first, &gradient.map(|value| -value), 0.0);
    let mixed_second = jet.second_derivative([KernelArgument::Query, KernelArgument::Center]);
    assert_matrix(
        &mixed_second,
        &hessian.map(|row| row.map(|value| -value)),
        0.0,
    );
    let one_center = jet.third_derivative([
        KernelArgument::Query,
        KernelArgument::Center,
        KernelArgument::Query,
    ]);
    let two_centers = jet.third_derivative([
        KernelArgument::Center,
        KernelArgument::Query,
        KernelArgument::Center,
    ]);
    for first in 0..2 {
        assert_matrix(
            &one_center[first],
            &third[first].map(|row| row.map(|v| -v)),
            0.0,
        );
        assert_matrix(&two_centers[first], &third[first], 0.0);
    }
    Ok(())
}

#[test]
fn analytic_center_limit_becomes_twice_the_metric() -> TestResult {
    let anisotropy = GlobalAnisotropy::<3>::try_from_transform(
        [[2.0, 0.5, 0.0], [0.0, 3.0, 1.0], [0.0, 0.0, 4.0]],
        AnisotropyConditionPolicy::Unbounded,
    )?;
    let point = Point::try_new([1.0, -2.0, 3.0])?;
    let separation = anisotropy.try_transform_separation(point, point)?;
    assert!(separation.is_center());
    let transformed = SpatialKernelJet::try_new(separation, RadialJet::try_center(0.0, 2.0)?)?;
    let jet = anisotropy.try_transform_spatial_jet(transformed)?;
    assert!(
        jet.first_derivative(KernelArgument::Query)
            .iter()
            .all(|value| value.to_bits() == 0.0_f64.to_bits())
    );
    assert_matrix(
        &jet.second_derivative([KernelArgument::Query; 2]),
        &anisotropy.metric().map(|row| row.map(|value| 2.0 * value)),
        0.0,
    );
    assert_eq!(
        jet.third_derivative([KernelArgument::Query; 3]),
        [[[0.0; 3]; 3]; 3]
    );
    Ok(())
}

#[test]
#[allow(clippy::too_many_lines)]
fn construction_rejects_invalid_lengths_axes_metrics_and_policies() -> TestResult {
    assert!(matches!(
        GlobalAnisotropy::<2>::try_isotropic(f64::NAN),
        Err(AnisotropyError::NonFiniteAxisLength { axis: 0, .. })
    ));
    assert!(matches!(
        GlobalAnisotropy::<2>::try_isotropic(0.0),
        Err(AnisotropyError::NonPositiveAxisLength { axis: 0, .. })
    ));
    assert!(matches!(
        GlobalAnisotropy::<1>::try_isotropic(f64::from_bits(1)),
        Err(AnisotropyError::NonRepresentableReciprocalLength { axis: 0, .. })
    ));
    assert!(matches!(
        GlobalAnisotropy::<2>::try_from_transform(
            [[1.0, f64::INFINITY], [0.0, 1.0]],
            AnisotropyConditionPolicy::Unbounded
        ),
        Err(AnisotropyError::NonFiniteTransformComponent {
            row: 0,
            column: 1,
            ..
        })
    ));
    let singular = GlobalAnisotropy::<2>::try_from_transform(
        [[1.0, 2.0], [2.0, 4.0]],
        AnisotropyConditionPolicy::Unbounded,
    );
    let Err(AnisotropyError::TransformInversion {
        singular_values,
        condition_number,
        ..
    }) = singular
    else {
        return Err(format!("unexpected singular-transform result: {singular:?}").into());
    };
    assert!(singular_values[0].is_finite());
    assert!(singular_values[0] > 0.0);
    assert_eq!(singular_values[1].to_bits(), 0.0_f64.to_bits());
    assert!(condition_number.is_infinite());
    assert!(matches!(
        GlobalAnisotropy::<2>::try_from_transform(
            [[1.0, 1.0], [1.0e-12, 2.0e-12]],
            AnisotropyConditionPolicy::Unbounded
        ),
        Err(AnisotropyError::MetricNotPositiveDefinite { .. })
    ));

    assert!(matches!(
        GlobalAnisotropy::<2>::try_from_metric(
            [[1.0, f64::NAN], [f64::NAN, 1.0]],
            AnisotropyConditionPolicy::Unbounded
        ),
        Err(AnisotropyError::NonFiniteMetricComponent {
            row: 0,
            column: 1,
            ..
        })
    ));
    assert!(matches!(
        GlobalAnisotropy::<2>::try_from_metric(
            [[1.0, 0.25], [0.5, 1.0]],
            AnisotropyConditionPolicy::Unbounded
        ),
        Err(AnisotropyError::NonSymmetricMetric {
            row: 0,
            column: 1,
            ..
        })
    ));
    for metric in [
        [[1.0, 2.0], [2.0, 1.0]],
        [[1.0, 1.0], [1.0, 1.0]],
        [[2.0, 2.000_000_01], [2.000_000_01, 2.000_000_02]],
    ] {
        assert!(matches!(
            GlobalAnisotropy::<2>::try_from_metric(metric, AnisotropyConditionPolicy::Unbounded),
            Err(AnisotropyError::MetricNotPositiveDefinite { .. })
        ));
    }

    let axes = [
        UnitDirection::try_new([1.0, 0.0])?,
        UnitDirection::try_new([0.5, 1.0])?,
    ];
    assert!(matches!(
        GlobalAnisotropy::<2>::try_ellipsoidal(
            axes,
            [1.0, 2.0],
            0.1,
            AnisotropyConditionPolicy::Unbounded
        ),
        Err(AnisotropyError::NonOrthogonalAxes { .. })
    ));
    for tolerance in [f64::NAN, f64::INFINITY] {
        assert!(matches!(
            GlobalAnisotropy::<2>::try_ellipsoidal(
                axes,
                [1.0, 2.0],
                tolerance,
                AnisotropyConditionPolicy::Unbounded
            ),
            Err(AnisotropyError::NonFiniteOrthogonalityTolerance { .. })
        ));
    }
    for tolerance in [-1.0, 1.0] {
        assert!(matches!(
            GlobalAnisotropy::<2>::try_ellipsoidal(
                axes,
                [1.0, 2.0],
                tolerance,
                AnisotropyConditionPolicy::Unbounded
            ),
            Err(AnisotropyError::InvalidOrthogonalityTolerance { .. })
        ));
    }

    for maximum in [0.5, f64::NAN, f64::INFINITY] {
        assert!(matches!(
            GlobalAnisotropy::<2>::try_from_transform(
                [[1.0, 0.0], [0.0, 1.0]],
                AnisotropyConditionPolicy::Maximum(maximum)
            ),
            Err(AnisotropyError::InvalidMaximumConditionNumber { .. })
        ));
    }
    let condition_error = GlobalAnisotropy::<2>::try_from_transform(
        [[1.0, 0.0], [0.0, 0.01]],
        AnisotropyConditionPolicy::Maximum(50.0),
    );
    let Err(AnisotropyError::ConditionNumberExceeded {
        maximum,
        diagnostics,
    }) = condition_error
    else {
        return Err(format!("unexpected condition result: {condition_error:?}").into());
    };
    assert_eq!(maximum.to_bits(), 50.0_f64.to_bits());
    assert_close(diagnostics.condition_number(), 100.0, 1.0e-14, 1.0e-14);
    Ok(())
}

#[test]
fn representable_extreme_spd_metric_has_finite_diagnostics() -> TestResult {
    let metric = [[f64::MAX, 0.0], [0.0, f64::MIN_POSITIVE]];
    let anisotropy =
        GlobalAnisotropy::<2>::try_from_metric(metric, AnisotropyConditionPolicy::Unbounded)?;
    assert_eq!(anisotropy.metric(), &metric);
    let singular_values = anisotropy.diagnostics().singular_values();
    assert!(singular_values[0].is_finite());
    assert!(singular_values[1].is_finite());
    assert!(singular_values[0] > singular_values[1]);
    assert!(anisotropy.diagnostics().condition_number().is_finite());

    let displacement_error = anisotropy
        .try_transform_separation(Point::try_new([1.0e200, 0.0])?, Point::try_new([0.0, 0.0])?);
    assert!(matches!(
        displacement_error,
        Err(AnisotropyError::NonFiniteTransformedDisplacementComponent { axis: 0 })
    ));
    let subtraction_error = anisotropy.try_transform_separation(
        Point::try_new([f64::MAX, 0.0])?,
        Point::try_new([-f64::MAX, 0.0])?,
    );
    assert!(matches!(
        subtraction_error,
        Err(AnisotropyError::NonFiniteDisplacementComponent { axis: 0 })
    ));
    Ok(())
}

#[test]
fn derivative_overflow_is_structured() -> TestResult {
    let anisotropy = GlobalAnisotropy::<1>::try_isotropic(0.5)?;
    let separation =
        anisotropy.try_transform_separation(Point::try_new([0.5])?, Point::try_new([0.0])?)?;
    let transformed =
        SpatialKernelJet::try_new(separation, RadialJet::try_away(0.0, f64::MAX, 0.0, 0.0)?)?;
    assert!(matches!(
        anisotropy.try_transform_spatial_jet(transformed),
        Err(AnisotropyError::NonFiniteFirstDerivative { axis: 0 })
    ));
    Ok(())
}

#[test]
fn global_anisotropy_values_are_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<GlobalAnisotropy<1>>();
    assert_send_sync::<GlobalAnisotropy<2>>();
    assert_send_sync::<GlobalAnisotropy<3>>();
    assert_send_sync::<AnisotropyError<3>>();
}
