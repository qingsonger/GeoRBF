//! Evaluate Gaussian and Matérn kernels through the shared D=3 calculus.

use std::error::Error;

use georbf::{
    Gaussian, KernelArgument, Matern, MaternSmoothness, Point, RadialSeparation, SpatialKernelJet,
};

fn main() -> Result<(), Box<dyn Error>> {
    let query = Point::try_new([0.75, -0.25, 0.5])?;
    let center = Point::try_new([0.0, 0.0, 0.0])?;
    let separation = RadialSeparation::try_new(query, center)?;

    let gaussian = Gaussian::try_new(1.5)?;
    let gaussian_jet = SpatialKernelJet::try_new(separation, gaussian.radial_jet(separation)?)?;
    println!(
        "{} value: {}",
        gaussian.metadata().name(),
        gaussian_jet.value()
    );

    let matern = Matern::try_new(MaternSmoothness::FiveHalves, 1.5)?;
    let matern_jet = SpatialKernelJet::try_new(separation, matern.radial_jet(separation)?)?;
    println!("{} value: {}", matern.metadata().name(), matern_jet.value());
    println!(
        "Matérn query gradient: {:?}",
        matern_jet.first_derivative(KernelArgument::Query)
    );
    Ok(())
}
