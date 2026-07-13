//! Integration tests for dimension-safe geometry primitives.

use georbf::{Dim, Direction, GeometryError, Point, SupportedDimension, UnitDirection, Vector};

fn non_finite_index<T>(result: &Result<T, GeometryError>) -> Option<usize> {
    match result {
        Err(GeometryError::NonFiniteComponent { index, .. }) => Some(*index),
        _ => None,
    }
}

fn assert_unit<const D: usize>(direction: &UnitDirection<D>)
where
    Dim<D>: SupportedDimension,
{
    let squared_norm = direction
        .components()
        .iter()
        .map(|component| component * component)
        .sum::<f64>();
    assert!((squared_norm - 1.0).abs() <= 8.0 * f64::EPSILON);
}

fn assert_close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() <= 8.0 * f64::EPSILON);
}

fn assert_components<const D: usize>(actual: [f64; D], expected: [f64; D]) {
    for (actual, expected) in actual.into_iter().zip(expected) {
        assert_close(actual, expected);
    }
}

#[test]
fn finite_points_and_vectors_round_trip_in_every_supported_dimension() -> Result<(), GeometryError>
{
    let point_1d = Point::<1>::try_new([1.25])?;
    let point_2d = Point::<2>::try_from([1.25, -2.5])?;
    let point_3d = Point::<3>::try_new([1.25, -2.5, 5.0])?;
    let vector_1d = Vector::<1>::try_new([-3.0])?;
    let vector_2d = Vector::<2>::try_from([-3.0, 4.0])?;
    let vector_3d = Vector::<3>::try_new([-3.0, 4.0, 12.0])?;

    assert_components(point_1d.into_components(), [1.25]);
    assert_components(*point_2d.components(), [1.25, -2.5]);
    assert_components(point_3d.into_components(), [1.25, -2.5, 5.0]);
    assert_components(*vector_1d.components(), [-3.0]);
    assert_components(vector_2d.into_components(), [-3.0, 4.0]);
    assert_components(*vector_3d.components(), [-3.0, 4.0, 12.0]);
    Ok(())
}

#[test]
fn points_and_vectors_reject_non_finite_components_with_their_index() {
    assert_eq!(non_finite_index(&Point::<1>::try_new([f64::NAN])), Some(0));
    assert_eq!(
        non_finite_index(&Point::<2>::try_new([0.0, f64::INFINITY])),
        Some(1)
    );
    assert_eq!(
        non_finite_index(&Point::<3>::try_new([0.0, f64::NEG_INFINITY, 1.0])),
        Some(1)
    );
    assert_eq!(
        non_finite_index(&Vector::<1>::try_new([f64::INFINITY])),
        Some(0)
    );
    assert_eq!(
        non_finite_index(&Vector::<2>::try_new([f64::NAN, 0.0])),
        Some(0)
    );
    assert_eq!(
        non_finite_index(&Vector::<3>::try_new([0.0, 1.0, f64::NEG_INFINITY])),
        Some(2)
    );
}

#[test]
fn directions_reject_zero_and_non_finite_components() {
    assert!(matches!(
        Direction::<1>::try_new([-0.0]),
        Err(GeometryError::ZeroDirection)
    ));
    assert!(matches!(
        Direction::<2>::try_new([0.0, 0.0]),
        Err(GeometryError::ZeroDirection)
    ));
    assert!(matches!(
        UnitDirection::<3>::try_new([0.0, -0.0, 0.0]),
        Err(GeometryError::ZeroDirection)
    ));
    assert_eq!(
        non_finite_index(&Direction::<2>::try_new([f64::NAN, 1.0])),
        Some(0)
    );
    assert_eq!(
        non_finite_index(&UnitDirection::<3>::try_new([1.0, f64::INFINITY, 0.0])),
        Some(1)
    );
}

#[test]
fn one_dimensional_unit_directions_preserve_sign() -> Result<(), GeometryError> {
    let positive = UnitDirection::<1>::try_new([42.0])?;
    let negative = UnitDirection::<1>::try_new([-42.0])?;

    assert_components(*positive.components(), [1.0]);
    assert_components(*negative.components(), [-1.0]);
    Ok(())
}

#[test]
fn unit_directions_normalize_in_two_and_three_dimensions() -> Result<(), GeometryError> {
    let direction_2d = UnitDirection::<2>::try_new([3.0, 4.0])?;
    let direction_3d = Direction::<3>::try_new([2.0, -3.0, 6.0])?;
    let direction_3d = direction_3d.unit();

    assert_close(direction_2d.components()[0], 0.6);
    assert_close(direction_2d.components()[1], 0.8);
    assert_unit(&direction_2d);
    assert_unit(&direction_3d);
    Ok(())
}

#[test]
fn normalization_is_stable_for_extreme_finite_magnitudes() -> Result<(), GeometryError> {
    let minimum_subnormal = f64::from_bits(1);
    let huge = UnitDirection::<3>::try_new([f64::MAX, -f64::MAX, 0.0])?;
    let tiny = UnitDirection::<3>::try_new([minimum_subnormal, 0.0, -minimum_subnormal])?;

    assert_unit(&huge);
    assert_unit(&tiny);
    assert_close(huge.components()[0], std::f64::consts::FRAC_1_SQRT_2);
    assert_close(huge.components()[1], -std::f64::consts::FRAC_1_SQRT_2);
    assert_close(tiny.components()[0], std::f64::consts::FRAC_1_SQRT_2);
    assert_close(tiny.components()[2], -std::f64::consts::FRAC_1_SQRT_2);
    Ok(())
}

#[test]
fn unit_direction_is_scale_invariant_for_representative_finite_scales() -> Result<(), GeometryError>
{
    let reference = UnitDirection::<3>::try_new([1.0, -2.0, 2.0])?;

    for scale in [
        f64::from_bits(1),
        f64::MIN_POSITIVE,
        1.0e-200,
        1.0,
        1.0e200,
        f64::MAX / 4.0,
    ] {
        let scaled = UnitDirection::<3>::try_new([scale, -2.0 * scale, 2.0 * scale])?;
        for (actual, expected) in scaled.components().iter().zip(reference.components()) {
            assert_close(*actual, *expected);
        }
    }
    Ok(())
}

#[test]
fn validated_directions_convert_without_losing_components() -> Result<(), GeometryError> {
    let direction = Direction::<2>::try_new([5.0, -12.0])?;
    let vector: Vector<2> = direction.into();
    assert_components(vector.into_components(), [5.0, -12.0]);

    let unit = UnitDirection::<2>::try_new([5.0, -12.0])?;
    let direction: Direction<2> = unit.into();
    assert_unit(&unit);
    assert_components(*direction.components(), *unit.components());
    Ok(())
}

#[test]
fn geometry_primitives_are_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<Point<1>>();
    assert_send_sync::<Vector<2>>();
    assert_send_sync::<Direction<3>>();
    assert_send_sync::<UnitDirection<3>>();
}
