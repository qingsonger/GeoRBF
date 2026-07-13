//! Expands a caller-supplied radial jet into Cartesian kernel derivatives.

use std::error::Error;

use georbf::{
    KernelArgument, Point, RadialExpansionCoefficients, RadialJet, RadialSeparation,
    SpatialKernelJet,
};

fn main() -> Result<(), Box<dyn Error>> {
    let separation = RadialSeparation::try_new(
        Point::try_new([1.0, -2.0, 0.5])?,
        Point::try_new([-1.0, 1.0, -0.5])?,
    )?;
    let radius = separation.radius();

    // Demonstration data for phi(r) = r^6; concrete kernel families are a
    // separate capability and are not selected by the calculus layer.
    let radial = RadialJet::try_away_with_expansion(
        radius.powi(6),
        6.0 * radius.powi(5),
        30.0 * radius.powi(4),
        120.0 * radius.powi(3),
        // Stable closed forms for phi'(r)/r and
        // (phi''(r) - phi'(r)/r)/r.
        RadialExpansionCoefficients::try_new(6.0 * radius.powi(4), 24.0 * radius.powi(3))?,
    )?;
    let spatial = SpatialKernelJet::try_new(separation, radial)?;

    println!("radius: {}", separation.radius());
    println!("value: {}", spatial.value());
    println!(
        "query gradient: {:?}",
        spatial.first_derivative(KernelArgument::Query)
    );
    println!(
        "query-center Hessian: {:?}",
        spatial.second_derivative([KernelArgument::Query, KernelArgument::Center])
    );
    Ok(())
}
