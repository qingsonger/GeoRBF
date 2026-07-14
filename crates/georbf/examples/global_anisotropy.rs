//! Evaluate a Gaussian through a rotated global ellipsoidal metric.

use std::error::Error;

use georbf::{
    AnisotropyConditionPolicy, Gaussian, GlobalAnisotropy, KernelArgument, Point, SpatialKernelJet,
    UnitDirection,
};

fn main() -> Result<(), Box<dyn Error>> {
    let angle = 30.0_f64.to_radians();
    let (sine, cosine) = angle.sin_cos();
    let anisotropy = GlobalAnisotropy::<2>::try_ellipsoidal(
        [
            UnitDirection::try_new([cosine, sine])?,
            UnitDirection::try_new([-sine, cosine])?,
        ],
        [2.0, 5.0],
        4.0 * f64::EPSILON,
        AnisotropyConditionPolicy::Maximum(3.0),
    )?;

    let separation = anisotropy
        .try_transform_separation(Point::try_new([3.0, -1.0])?, Point::try_new([0.0, 0.0])?)?;
    let gaussian = Gaussian::try_new(1.0)?;
    let transformed = SpatialKernelJet::try_new(separation, gaussian.radial_jet(separation)?)?;
    let original = anisotropy.try_transform_spatial_jet(transformed)?;

    println!("anisotropic radius: {}", separation.radius());
    println!("kernel value: {}", original.value());
    println!(
        "original-coordinate gradient: {:?}",
        original.first_derivative(KernelArgument::Query)
    );
    println!(
        "condition number: {}",
        anisotropy.diagnostics().condition_number()
    );
    Ok(())
}
