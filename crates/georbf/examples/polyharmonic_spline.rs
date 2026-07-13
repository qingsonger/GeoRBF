//! Evaluate a D=2 order-two surface spline through the shared calculus.

use std::error::Error;

use georbf::{KernelArgument, Point, RadialSeparation, SpatialKernelJet, SurfaceSpline};

fn main() -> Result<(), Box<dyn Error>> {
    let kernel = SurfaceSpline::<2>::try_new(2)?;
    let query = Point::try_new([0.75, -0.25])?;
    let center = Point::try_new([0.0, 0.0])?;
    let separation = RadialSeparation::try_new(query, center)?;
    let spatial = SpatialKernelJet::try_new(separation, kernel.radial_jet(separation)?)?;

    println!("family: {}", kernel.metadata().name());
    println!("surface order: {}", kernel.order());
    println!("derived power: {}", kernel.power());
    println!("value: {}", spatial.value());
    println!(
        "query gradient: {:?}",
        spatial.first_derivative(KernelArgument::Query)
    );
    Ok(())
}
