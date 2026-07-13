//! Integration tests for coordinate metadata and affine normalization.

use std::error::Error;

use georbf::{
    AffineNormalization, AngleUnit, AxisOrder, CoordinateMetadata, CoordinateMetadataError,
    CoordinateMetadataField, CrsMetadata, Handedness, LengthUnit, Point, TransformError,
    TransformOperation, UnitError, Vector, VerticalDirection,
};

fn assert_close(actual: f64, expected: f64) {
    let scale = actual.abs().max(expected.abs()).max(1.0);
    assert!((actual - expected).abs() <= 32.0 * f64::EPSILON * scale);
}

fn assert_components<const D: usize>(actual: &[f64; D], expected: [f64; D]) {
    for (actual, expected) in actual.iter().copied().zip(expected) {
        assert_close(actual, expected);
    }
}

fn assert_matrix<const D: usize>(actual: &[[f64; D]; D], expected: [[f64; D]; D]) {
    for (actual_row, expected_row) in actual.iter().zip(expected) {
        assert_components(actual_row, expected_row);
    }
}

#[test]
fn metadata_preserves_units_crs_and_axis_conventions() -> Result<(), Box<dyn Error>> {
    let unit = LengthUnit::try_new("m")?;
    let crs = CrsMetadata::try_new(Some(4978), Some("GEODCRS[\"WGS 84\"]".to_owned()))?;
    let axis_order = AxisOrder::<3>::try_new([1, 0, 2])?;
    let metadata = CoordinateMetadata::new(
        unit,
        crs,
        axis_order,
        VerticalDirection::Up,
        Handedness::Right,
        AngleUnit::Degrees,
    );

    assert_eq!(metadata.length_unit().identifier(), "m");
    assert_eq!(metadata.crs().epsg(), Some(4978));
    assert_eq!(metadata.crs().wkt(), Some("GEODCRS[\"WGS 84\"]"));
    assert_eq!(metadata.axis_order().components(), &[1, 0, 2]);
    assert_eq!(metadata.vertical_direction(), VerticalDirection::Up);
    assert_eq!(metadata.handedness(), Handedness::Right);
    assert_eq!(metadata.angle_unit(), AngleUnit::Degrees);
    Ok(())
}

#[test]
fn metadata_rejects_invalid_identifiers_crs_and_axis_orders() {
    assert!(matches!(
        LengthUnit::try_new(""),
        Err(UnitError::EmptyIdentifier)
    ));
    assert!(matches!(
        LengthUnit::try_new(" m"),
        Err(UnitError::SurroundingWhitespace)
    ));
    assert!(matches!(
        LengthUnit::try_new("m\0custom"),
        Err(UnitError::ControlCharacter { byte_index: 1 })
    ));
    assert!(matches!(
        CrsMetadata::from_epsg(0),
        Err(CoordinateMetadataError::InvalidEpsgCode)
    ));
    assert!(matches!(
        CrsMetadata::from_wkt("   "),
        Err(CoordinateMetadataError::EmptyWkt)
    ));
    assert!(matches!(
        CrsMetadata::from_wkt(" WKT"),
        Err(CoordinateMetadataError::SurroundingWhitespaceInWkt)
    ));
    assert!(matches!(
        AxisOrder::<2>::try_new([0, 2]),
        Err(CoordinateMetadataError::AxisOutOfRange {
            position: 1,
            axis: 2,
            dimension: 2
        })
    ));
    assert!(matches!(
        AxisOrder::<3>::try_new([0, 0, 2]),
        Err(CoordinateMetadataError::DuplicateAxis {
            position: 1,
            axis: 0
        })
    ));

    assert_eq!(AxisOrder::<1>::identity().components(), &[0]);
    assert_eq!(AxisOrder::<2>::identity().components(), &[0, 1]);
    assert_eq!(AxisOrder::<3>::identity().components(), &[0, 1, 2]);
}

#[test]
fn metadata_compatibility_rejects_silent_unit_or_crs_mixing() -> Result<(), Box<dyn Error>> {
    let metres = CoordinateMetadata::new(
        LengthUnit::try_new("m")?,
        CrsMetadata::from_epsg(4978)?,
        AxisOrder::<3>::identity(),
        VerticalDirection::Up,
        Handedness::Right,
        AngleUnit::Radians,
    );
    let mut feet = metres.clone();
    feet = CoordinateMetadata::new(
        LengthUnit::try_new("ft")?,
        feet.crs().clone(),
        *feet.axis_order(),
        feet.vertical_direction(),
        feet.handedness(),
        feet.angle_unit(),
    );
    let different_crs = CoordinateMetadata::new(
        metres.length_unit().clone(),
        CrsMetadata::from_epsg(4326)?,
        *metres.axis_order(),
        metres.vertical_direction(),
        metres.handedness(),
        metres.angle_unit(),
    );

    assert!(matches!(
        metres.ensure_compatible(&feet),
        Err(CoordinateMetadataError::Mismatch {
            field: CoordinateMetadataField::LengthUnit
        })
    ));
    assert!(matches!(
        metres.ensure_compatible(&different_crs),
        Err(CoordinateMetadataError::Mismatch {
            field: CoordinateMetadataField::Crs
        })
    ));
    metres.ensure_compatible(&metres)?;
    Ok(())
}

#[test]
fn affine_points_round_trip_in_every_supported_dimension() -> Result<(), Box<dyn Error>> {
    let transform_1d = AffineNormalization::<1>::try_new(Point::try_new([10.0])?, [[2.0]])?;
    let normalized_1d = transform_1d.normalize_point(Point::try_new([14.0])?)?;
    assert_components(normalized_1d.components(), [2.0]);
    assert_components(
        transform_1d.denormalize_point(normalized_1d)?.components(),
        [14.0],
    );

    let transform_2d =
        AffineNormalization::<2>::try_new(Point::try_new([1.0, -2.0])?, [[0.0, -2.0], [4.0, 0.0]])?;
    let normalized_2d = transform_2d.normalize_point(Point::try_new([-3.0, 6.0])?)?;
    assert_components(normalized_2d.components(), [2.0, 2.0]);
    assert_components(
        transform_2d.denormalize_point(normalized_2d)?.components(),
        [-3.0, 6.0],
    );

    let transform_3d = AffineNormalization::<3>::try_new(
        Point::try_new([10.0, -1.0, 3.0])?,
        [[2.0, 1.0, 0.0], [0.0, 3.0, 1.0], [0.0, 0.0, 4.0]],
    )?;
    let normalized_3d = transform_3d.normalize_point(Point::try_new([10.0, -6.5, 5.0])?)?;
    assert_components(normalized_3d.components(), [1.0, -2.0, 0.5]);
    assert_components(
        transform_3d.denormalize_point(normalized_3d)?.components(),
        [10.0, -6.5, 5.0],
    );
    Ok(())
}

#[test]
fn derivatives_transform_to_original_coordinates_by_analytic_truth() -> Result<(), Box<dyn Error>> {
    let transform_1d = AffineNormalization::<1>::try_new(Point::try_new([0.0])?, [[2.0]])?;
    let gradient_1d = transform_1d.gradient_to_original(Vector::try_new([4.0])?)?;
    assert_components(gradient_1d.components(), [2.0]);
    assert_matrix(&transform_1d.hessian_to_original([[8.0]])?, [[2.0]]);

    let transform =
        AffineNormalization::<2>::try_new(Point::try_new([0.0, 0.0])?, [[2.0, 1.0], [0.0, 3.0]])?;

    let gradient = transform.gradient_to_original(Vector::try_new([4.0, 6.0])?)?;
    assert_components(gradient.components(), [2.0, 4.0 / 3.0]);

    let hessian = transform.hessian_to_original([[8.0, 0.0], [0.0, 18.0]])?;
    assert_matrix(&hessian, [[2.0, -2.0 / 3.0], [-2.0 / 3.0, 20.0 / 9.0]]);

    let transform_3d = AffineNormalization::<3>::try_new(
        Point::try_new([0.0, 0.0, 0.0])?,
        [[2.0, 0.0, 0.0], [0.0, 3.0, 0.0], [0.0, 0.0, 4.0]],
    )?;
    let gradient_3d = transform_3d.gradient_to_original(Vector::try_new([2.0, 6.0, 12.0])?)?;
    assert_components(gradient_3d.components(), [1.0, 2.0, 3.0]);
    assert_matrix(
        &transform_3d.hessian_to_original([[4.0, 0.0, 0.0], [0.0, 18.0, 0.0], [0.0, 0.0, 48.0]])?,
        [[1.0, 0.0, 0.0], [0.0, 2.0, 0.0], [0.0, 0.0, 3.0]],
    );
    Ok(())
}

#[test]
fn affine_inverse_uses_no_hidden_singularity_tolerance() -> Result<(), Box<dyn Error>> {
    let near_singular = AffineNormalization::<2>::try_new(
        Point::try_new([0.0, 0.0])?,
        [[1.0, 1.0], [1.0, 1.0 + f64::EPSILON]],
    )?;
    assert!(
        near_singular
            .inverse_scale_matrix()
            .iter()
            .flatten()
            .all(|value| value.is_finite())
    );

    let tiny = AffineNormalization::<1>::try_new(Point::try_new([0.0])?, [[f64::MIN_POSITIVE]])?;
    assert_components(
        tiny.normalize_point(Point::try_new([f64::MIN_POSITIVE])?)?
            .components(),
        [1.0],
    );

    let huge = AffineNormalization::<1>::try_new(Point::try_new([0.0])?, [[f64::MAX]])?;
    assert_components(
        huge.normalize_point(Point::try_new([f64::MAX])?)?
            .components(),
        [1.0],
    );
    Ok(())
}

#[test]
fn affine_constructor_and_operations_report_invalid_numeric_states() -> Result<(), Box<dyn Error>> {
    let origin_2d = Point::try_new([0.0, 0.0])?;
    assert!(matches!(
        AffineNormalization::<2>::try_new(origin_2d, [[1.0, f64::NAN], [0.0, 1.0]]),
        Err(TransformError::NonFiniteScaleComponent {
            row: 0,
            column: 1,
            ..
        })
    ));
    assert!(matches!(
        AffineNormalization::<2>::try_new(origin_2d, [[1.0, 2.0], [2.0, 4.0]]),
        Err(TransformError::SingularScaleMatrix)
    ));
    assert!(matches!(
        AffineNormalization::<1>::try_new(Point::try_new([0.0])?, [[f64::from_bits(1)]]),
        Err(TransformError::NonRepresentableInverse)
    ));

    let identity = AffineNormalization::<1>::try_new(Point::try_new([-f64::MAX])?, [[1.0]])?;
    assert!(matches!(
        identity.normalize_point(Point::try_new([f64::MAX])?),
        Err(TransformError::NonFiniteResult {
            operation: TransformOperation::NormalizePoint
        })
    ));
    assert!(matches!(
        identity.hessian_to_original([[f64::INFINITY]]),
        Err(TransformError::NonFiniteHessianComponent {
            row: 0,
            column: 0,
            ..
        })
    ));

    let overflowing_denormalization =
        AffineNormalization::<1>::try_new(Point::try_new([f64::MAX])?, [[1.0]])?;
    assert!(matches!(
        overflowing_denormalization.denormalize_point(Point::try_new([f64::MAX])?),
        Err(TransformError::NonFiniteResult {
            operation: TransformOperation::DenormalizePoint
        })
    ));

    let derivative_overflow =
        AffineNormalization::<1>::try_new(Point::try_new([0.0])?, [[f64::MIN_POSITIVE]])?;
    assert!(matches!(
        derivative_overflow.gradient_to_original(Vector::try_new([8.0])?),
        Err(TransformError::NonFiniteResult {
            operation: TransformOperation::GradientToOriginal
        })
    ));
    assert!(matches!(
        derivative_overflow.hessian_to_original([[1.0]]),
        Err(TransformError::NonFiniteResult {
            operation: TransformOperation::HessianToOriginal
        })
    ));
    Ok(())
}

#[test]
fn coordinate_values_are_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<LengthUnit>();
    assert_send_sync::<CrsMetadata>();
    assert_send_sync::<CoordinateMetadata<3>>();
    assert_send_sync::<AffineNormalization<3>>();
}
