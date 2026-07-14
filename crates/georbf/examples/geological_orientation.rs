//! Converts planar and linear geological angles to validated directions.

use std::error::Error;

use georbf::{AngleUnit, LinearOrientation, OrientationPolarity, PlanarOrientation};

fn main() -> Result<(), Box<dyn Error>> {
    let plane = PlanarOrientation::<3>::from_strike_dip(
        0.0,
        45.0,
        AngleUnit::Degrees,
        OrientationPolarity::Positive,
    )?;
    let lineation = LinearOrientation::<3>::from_azimuth_plunge(
        135.0,
        20.0,
        AngleUnit::Degrees,
        OrientationPolarity::Unknown,
    )?;

    println!(
        "right-hand-rule plane normal: {:?}",
        plane.normal().components()
    );
    println!(
        "axial lineation representative: {:?}",
        lineation.direction().components()
    );
    println!("lineation sign is unknown: {}", lineation.is_axial());
    Ok(())
}
