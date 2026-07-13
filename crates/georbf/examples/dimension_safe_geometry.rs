//! Constructs validated three-dimensional geometry values.

use georbf::{Point, UnitDirection};

fn main() -> Result<(), georbf::GeometryError> {
    let point = Point::<3>::try_new([1.0, 2.0, 3.0])?;
    let direction = UnitDirection::<3>::try_new([3.0, 0.0, 4.0])?;

    println!("point: {:?}", point.components());
    println!("unit direction: {:?}", direction.components());

    Ok(())
}
