//! Demonstrates coordinate metadata and affine normalization.

use std::error::Error;

use georbf::{
    AffineNormalization, AngleUnit, AxisOrder, CoordinateMetadata, CrsMetadata, Handedness,
    LengthUnit, Point, Vector, VerticalDirection,
};

fn main() -> Result<(), Box<dyn Error>> {
    let metadata = CoordinateMetadata::new(
        LengthUnit::try_new("m")?,
        CrsMetadata::from_epsg(4978)?,
        AxisOrder::<3>::identity(),
        VerticalDirection::Up,
        Handedness::Right,
        AngleUnit::Radians,
    );
    let transform = AffineNormalization::<3>::try_new(
        Point::try_new([100.0, 200.0, 50.0])?,
        [[10.0, 0.0, 0.0], [0.0, 20.0, 0.0], [0.0, 0.0, 5.0]],
    )?;

    let normalized = transform.normalize_point(Point::try_new([110.0, 240.0, 55.0])?)?;
    let gradient = transform.gradient_to_original(Vector::try_new([1.0, 2.0, 3.0])?)?;

    println!("unit: {}", metadata.length_unit().identifier());
    println!("normalized point: {:?}", normalized.components());
    println!("original-coordinate gradient: {:?}", gradient.components());
    Ok(())
}
