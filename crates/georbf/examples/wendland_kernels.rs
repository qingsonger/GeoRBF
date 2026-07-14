//! Evaluate a Wendland C4 kernel inside and at its compact-support boundary.

use std::error::Error;

use georbf::{
    KernelArgument, Point, RadialSeparation, SpatialKernelJet, Wendland, WendlandSmoothness,
};

fn main() -> Result<(), Box<dyn Error>> {
    let kernel = Wendland::try_new(WendlandSmoothness::C4, 2.0)?;
    let center = Point::try_new([0.0, 0.0, 0.0])?;
    let query = Point::try_new([0.75, -0.25, 0.5])?;
    let separation = RadialSeparation::try_new(query, center)?;
    let spatial = SpatialKernelJet::try_new(separation, kernel.radial_jet(separation)?)?;

    println!("{} value: {}", kernel.metadata().name(), spatial.value());
    println!(
        "query gradient: {:?}",
        spatial.first_derivative(KernelArgument::Query)
    );
    println!(
        "value at support boundary: {}",
        kernel.radial_value(kernel.support_radius())?
    );
    Ok(())
}
