//! Independent tests for geological orientation conversions.

use std::error::Error;
use std::f64::consts::{FRAC_1_SQRT_2, FRAC_PI_2, PI};

use georbf::{
    AngleUnit, GeometryError, LinearOrientation, OrientationAngleField, OrientationError,
    OrientationPolarity, PlanarOrientation,
};

type TestResult = Result<(), Box<dyn Error>>;

fn assert_close(actual: f64, expected: f64) {
    let scale = actual.abs().max(expected.abs()).max(1.0);
    assert!(
        (actual - expected).abs() <= 64.0 * f64::EPSILON * scale,
        "expected {expected:e}, got {actual:e}"
    );
}

fn assert_components<const D: usize>(actual: &[f64; D], expected: [f64; D]) {
    for (actual, expected) in actual.iter().copied().zip(expected) {
        assert_close(actual, expected);
    }
}

fn assert_unit<const D: usize>(components: &[f64; D]) {
    let squared_norm = components.iter().map(|value| value * value).sum::<f64>();
    assert_close(squared_norm, 1.0);
}

#[test]
fn planar_conversions_match_independent_cardinal_and_oblique_truth() -> TestResult {
    let horizontal = PlanarOrientation::<3>::from_dip_direction_dip(
        217.0,
        0.0,
        AngleUnit::Degrees,
        OrientationPolarity::Positive,
    )?;
    assert_eq!(
        horizontal.normal().components().map(f64::to_bits),
        [0, 0, 1.0_f64.to_bits()]
    );

    let north_vertical = PlanarOrientation::<3>::from_dip_direction_dip(
        0.0,
        90.0,
        AngleUnit::Degrees,
        OrientationPolarity::Positive,
    )?;
    assert_components(north_vertical.normal().components(), [0.0, 1.0, 0.0]);

    let east_thirty = PlanarOrientation::<3>::from_dip_direction_dip(
        90.0,
        30.0,
        AngleUnit::Degrees,
        OrientationPolarity::Positive,
    )?;
    assert_components(
        east_thirty.normal().components(),
        [0.5, 0.0, 3.0_f64.sqrt() / 2.0],
    );

    let section = PlanarOrientation::<2>::from_signed_dip(
        -30.0,
        AngleUnit::Degrees,
        OrientationPolarity::Positive,
    )?;
    assert_components(section.normal().components(), [-0.5, 3.0_f64.sqrt() / 2.0]);
    Ok(())
}

#[test]
fn lineation_conversions_match_independent_cardinal_and_oblique_truth() -> TestResult {
    let north_down = LinearOrientation::<3>::from_azimuth_plunge(
        0.0,
        30.0,
        AngleUnit::Degrees,
        OrientationPolarity::Positive,
    )?;
    assert_components(
        north_down.direction().components(),
        [0.0, 3.0_f64.sqrt() / 2.0, -0.5],
    );

    let east_up = LinearOrientation::<3>::from_azimuth_plunge(
        90.0,
        -30.0,
        AngleUnit::Degrees,
        OrientationPolarity::Positive,
    )?;
    assert_components(
        east_up.direction().components(),
        [3.0_f64.sqrt() / 2.0, 0.0, 0.5],
    );

    let vertical = LinearOrientation::<3>::from_azimuth_plunge(
        293.0,
        90.0,
        AngleUnit::Degrees,
        OrientationPolarity::Positive,
    )?;
    assert_eq!(
        vertical.direction().components().map(f64::to_bits),
        [0, 0, (-1.0_f64).to_bits()]
    );

    let section = LinearOrientation::<2>::from_plunge(
        45.0,
        AngleUnit::Degrees,
        OrientationPolarity::Positive,
    )?;
    assert_components(
        section.direction().components(),
        [FRAC_1_SQRT_2, -FRAC_1_SQRT_2],
    );
    Ok(())
}

#[test]
fn right_hand_rule_strike_matches_explicit_dip_direction() -> TestResult {
    for (strike, dip_direction) in [(0.0, 90.0), (90.0, 180.0), (270.0, 0.0), (359.0, 89.0)] {
        let from_strike = PlanarOrientation::<3>::from_strike_dip(
            strike,
            37.0,
            AngleUnit::Degrees,
            OrientationPolarity::Positive,
        )?;
        let explicit = PlanarOrientation::<3>::from_dip_direction_dip(
            dip_direction,
            37.0,
            AngleUnit::Degrees,
            OrientationPolarity::Positive,
        )?;
        assert_components(
            explicit.normal().components(),
            *from_strike.normal().components(),
        );
    }
    Ok(())
}

#[test]
fn degree_and_radian_inputs_are_equivalent() -> TestResult {
    let plane_degrees = PlanarOrientation::<3>::from_strike_dip(
        123.0,
        47.0,
        AngleUnit::Degrees,
        OrientationPolarity::Positive,
    )?;
    let plane_radians = PlanarOrientation::<3>::from_strike_dip(
        123.0 * PI / 180.0,
        47.0 * PI / 180.0,
        AngleUnit::Radians,
        OrientationPolarity::Positive,
    )?;
    assert_components(
        plane_degrees.normal().components(),
        *plane_radians.normal().components(),
    );

    let line_degrees = LinearOrientation::<3>::from_azimuth_plunge(
        302.0,
        -19.0,
        AngleUnit::Degrees,
        OrientationPolarity::Positive,
    )?;
    let line_radians = LinearOrientation::<3>::from_azimuth_plunge(
        302.0 * PI / 180.0,
        -19.0 * PI / 180.0,
        AngleUnit::Radians,
        OrientationPolarity::Positive,
    )?;
    assert_components(
        line_degrees.direction().components(),
        *line_radians.direction().components(),
    );
    Ok(())
}

#[test]
fn polarity_reverses_known_directions_and_preserves_axial_metadata() -> TestResult {
    let positive = PlanarOrientation::<3>::from_normal_components(
        [2.0, -3.0, 6.0],
        OrientationPolarity::Positive,
    )?;
    let negative = PlanarOrientation::<3>::from_normal_components(
        [2.0, -3.0, 6.0],
        OrientationPolarity::Negative,
    )?;
    let unknown = PlanarOrientation::<3>::from_normal_components(
        [2.0, -3.0, 6.0],
        OrientationPolarity::Unknown,
    )?;

    for ((positive, negative), unknown) in positive
        .normal()
        .components()
        .iter()
        .zip(negative.normal().components())
        .zip(unknown.normal().components())
    {
        assert_close(*negative, -*positive);
        assert_close(*unknown, *positive);
    }
    assert_eq!(positive.polarity(), OrientationPolarity::Positive);
    assert_eq!(negative.polarity(), OrientationPolarity::Negative);
    assert_eq!(unknown.polarity(), OrientationPolarity::Unknown);
    assert!(!positive.is_axial());
    assert!(unknown.is_axial());

    let axial_line = LinearOrientation::<2>::from_direction_components(
        [3.0, 4.0],
        OrientationPolarity::Unknown,
    )?;
    assert!(axial_line.is_axial());
    assert_components(axial_line.into_direction().components(), [0.6, 0.8]);
    Ok(())
}

#[test]
fn every_constructor_canonicalizes_zero_bits_after_polarity() -> TestResult {
    let plane_2d = PlanarOrientation::<2>::from_signed_dip(
        0.0,
        AngleUnit::Degrees,
        OrientationPolarity::Negative,
    )?;
    assert_eq!(
        plane_2d.normal().components().map(f64::to_bits),
        [0, (-1.0_f64).to_bits()]
    );

    let vertical_plane_3d = PlanarOrientation::<3>::from_dip_direction_dip(
        0.0,
        90.0,
        AngleUnit::Degrees,
        OrientationPolarity::Negative,
    )?;
    assert_eq!(
        vertical_plane_3d.normal().components().map(f64::to_bits),
        [0, (-1.0_f64).to_bits(), 0]
    );
    let horizontal_plane_3d = PlanarOrientation::<3>::from_dip_direction_dip(
        217.0,
        0.0,
        AngleUnit::Degrees,
        OrientationPolarity::Negative,
    )?;
    assert_eq!(
        horizontal_plane_3d.normal().components().map(f64::to_bits),
        [0, 0, (-1.0_f64).to_bits()]
    );

    let line_2d = LinearOrientation::<2>::from_plunge(
        0.0,
        AngleUnit::Degrees,
        OrientationPolarity::Negative,
    )?;
    assert_eq!(
        line_2d.direction().components().map(f64::to_bits),
        [(-1.0_f64).to_bits(), 0]
    );

    let horizontal_line_3d = LinearOrientation::<3>::from_azimuth_plunge(
        90.0,
        0.0,
        AngleUnit::Degrees,
        OrientationPolarity::Negative,
    )?;
    assert_eq!(
        horizontal_line_3d
            .direction()
            .components()
            .map(f64::to_bits),
        [(-1.0_f64).to_bits(), 0, 0]
    );
    let vertical_line_3d = LinearOrientation::<3>::from_azimuth_plunge(
        293.0,
        90.0,
        AngleUnit::Degrees,
        OrientationPolarity::Negative,
    )?;
    assert_eq!(
        vertical_line_3d.direction().components().map(f64::to_bits),
        [0, 0, 1.0_f64.to_bits()]
    );

    for polarity in [
        OrientationPolarity::Positive,
        OrientationPolarity::Negative,
        OrientationPolarity::Unknown,
    ] {
        let plane = PlanarOrientation::<2>::from_normal_components([-0.0, 2.0], polarity)?;
        let line = LinearOrientation::<3>::from_direction_components([-0.0, 2.0, -0.0], polarity)?;
        assert_eq!(plane.normal().components()[0].to_bits(), 0);
        assert_eq!(line.direction().components()[0].to_bits(), 0);
        assert_eq!(line.direction().components()[2].to_bits(), 0);
    }
    Ok(())
}

#[test]
fn angle_conversions_covary_under_vertical_axis_rotations() -> TestResult {
    let azimuth = 43.0_f64;
    let rotation = 37.0_f64;
    let plane = PlanarOrientation::<3>::from_dip_direction_dip(
        azimuth,
        51.0,
        AngleUnit::Degrees,
        OrientationPolarity::Positive,
    )?;
    let rotated_plane = PlanarOrientation::<3>::from_dip_direction_dip(
        azimuth + rotation,
        51.0,
        AngleUnit::Degrees,
        OrientationPolarity::Positive,
    )?;

    let gamma = rotation.to_radians();
    let (sin_gamma, cos_gamma) = gamma.sin_cos();
    let [x, y, z] = plane.normal().into_components();
    assert_components(
        rotated_plane.normal().components(),
        [
            cos_gamma * x + sin_gamma * y,
            -sin_gamma * x + cos_gamma * y,
            z,
        ],
    );

    let line = LinearOrientation::<3>::from_azimuth_plunge(
        azimuth,
        -23.0,
        AngleUnit::Degrees,
        OrientationPolarity::Positive,
    )?;
    let rotated_line = LinearOrientation::<3>::from_azimuth_plunge(
        azimuth + rotation,
        -23.0,
        AngleUnit::Degrees,
        OrientationPolarity::Positive,
    )?;
    let [x, y, z] = line.direction().into_components();
    assert_components(
        rotated_line.direction().components(),
        [
            cos_gamma * x + sin_gamma * y,
            -sin_gamma * x + cos_gamma * y,
            z,
        ],
    );
    Ok(())
}

#[test]
fn component_conversions_covary_under_two_dimensional_rotation() -> TestResult {
    let angle = 0.731_f64;
    let (sin_angle, cos_angle) = angle.sin_cos();
    let original = [3.0, 4.0];
    let rotated = [
        cos_angle * original[0] - sin_angle * original[1],
        sin_angle * original[0] + cos_angle * original[1],
    ];
    let orientation =
        PlanarOrientation::<2>::from_normal_components(original, OrientationPolarity::Positive)?;
    let rotated_orientation =
        PlanarOrientation::<2>::from_normal_components(rotated, OrientationPolarity::Positive)?;
    let [x, y] = orientation.normal().into_components();
    assert_components(
        rotated_orientation.normal().components(),
        [cos_angle * x - sin_angle * y, sin_angle * x + cos_angle * y],
    );
    Ok(())
}

#[test]
fn component_conversion_retains_extreme_normalization_guarantees() -> TestResult {
    let smallest = f64::from_bits(1);
    let huge = PlanarOrientation::<3>::from_normal_components(
        [f64::MAX, -f64::MAX, 0.0],
        OrientationPolarity::Positive,
    )?;
    let tiny = LinearOrientation::<2>::from_direction_components(
        [smallest, -smallest],
        OrientationPolarity::Positive,
    )?;
    assert_unit(huge.normal().components());
    assert_unit(tiny.direction().components());
    assert_components(
        huge.normal().components(),
        [FRAC_1_SQRT_2, -FRAC_1_SQRT_2, 0.0],
    );
    assert_components(
        tiny.direction().components(),
        [FRAC_1_SQRT_2, -FRAC_1_SQRT_2],
    );
    Ok(())
}

#[test]
fn invalid_angles_report_their_field_unit_and_interval() {
    assert!(matches!(
        PlanarOrientation::<3>::from_strike_dip(
            f64::NAN,
            30.0,
            AngleUnit::Degrees,
            OrientationPolarity::Positive
        ),
        Err(OrientationError::NonFiniteAngle {
            field: OrientationAngleField::Strike,
            unit: AngleUnit::Degrees,
            ..
        })
    ));
    assert!(matches!(
        LinearOrientation::<3>::from_azimuth_plunge(
            0.0,
            f64::INFINITY,
            AngleUnit::Radians,
            OrientationPolarity::Positive
        ),
        Err(OrientationError::NonFiniteAngle {
            field: OrientationAngleField::Plunge,
            unit: AngleUnit::Radians,
            ..
        })
    ));

    let invalid_cases = [
        PlanarOrientation::<3>::from_strike_dip(
            -1.0,
            30.0,
            AngleUnit::Degrees,
            OrientationPolarity::Positive,
        ),
        PlanarOrientation::<3>::from_strike_dip(
            360.0,
            30.0,
            AngleUnit::Degrees,
            OrientationPolarity::Positive,
        ),
        PlanarOrientation::<3>::from_dip_direction_dip(
            2.0 * PI,
            FRAC_PI_2,
            AngleUnit::Radians,
            OrientationPolarity::Positive,
        ),
        PlanarOrientation::<3>::from_dip_direction_dip(
            0.0,
            f64::from_bits(90.0_f64.to_bits() + 1),
            AngleUnit::Degrees,
            OrientationPolarity::Positive,
        ),
    ];
    for result in invalid_cases {
        assert!(matches!(
            result,
            Err(OrientationError::AngleOutOfRange { .. })
        ));
    }
    assert!(matches!(
        PlanarOrientation::<2>::from_signed_dip(
            -FRAC_PI_2 - f64::EPSILON,
            AngleUnit::Radians,
            OrientationPolarity::Positive,
        ),
        Err(OrientationError::AngleOutOfRange {
            field: OrientationAngleField::SignedDip,
            maximum_inclusive: true,
            ..
        })
    ));
    assert!(matches!(
        LinearOrientation::<2>::from_plunge(
            f64::from_bits(90.0_f64.to_bits() + 1),
            AngleUnit::Degrees,
            OrientationPolarity::Positive,
        ),
        Err(OrientationError::AngleOutOfRange {
            field: OrientationAngleField::Plunge,
            ..
        })
    ));
}

#[test]
fn invalid_components_preserve_geometry_diagnostics() {
    assert!(matches!(
        PlanarOrientation::<2>::from_normal_components([0.0, -0.0], OrientationPolarity::Unknown,),
        Err(OrientationError::InvalidDirection(
            GeometryError::ZeroDirection
        ))
    ));
    assert!(matches!(
        LinearOrientation::<3>::from_direction_components(
            [0.0, f64::NEG_INFINITY, 1.0],
            OrientationPolarity::Positive,
        ),
        Err(OrientationError::InvalidDirection(
            GeometryError::NonFiniteComponent { index: 1, .. }
        ))
    ));
}

#[test]
fn orientation_values_are_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<PlanarOrientation<2>>();
    assert_send_sync::<PlanarOrientation<3>>();
    assert_send_sync::<LinearOrientation<2>>();
    assert_send_sync::<LinearOrientation<3>>();
    assert_send_sync::<OrientationError>();
}
