//! Evaluate a positive-definite local mixture and inspect its coverage.

use std::error::Error;

use georbf::{
    AnisotropyConditionPolicy, Gaussian, GlobalAnisotropy, KernelDefinition, KernelDerivativeOrder,
    LocalTrendComponent, LocalTrendMixture, OperationalDomain, Point, SmoothSpatialWeight,
};

fn main() -> Result<(), Box<dyn Error>> {
    let background = LocalTrendComponent::new(
        KernelDefinition::from(Gaussian::try_new(1.0)?),
        GlobalAnisotropy::try_isotropic(1.0)?,
        SmoothSpatialWeight::try_constant(0.35)?,
    );
    let local = LocalTrendComponent::new(
        KernelDefinition::from(Gaussian::try_new(0.75)?),
        GlobalAnisotropy::try_from_transform(
            [[0.5, 0.15], [-0.1, 1.25]],
            AnisotropyConditionPolicy::Maximum(3.0),
        )?,
        SmoothSpatialWeight::try_gaussian(Point::try_new([0.5, -0.25])?, 1.5, 1.2)?,
    );
    let mixture = LocalTrendMixture::try_new(
        vec![background, local],
        0,
        OperationalDomain::try_new(Point::try_new([-3.0, -3.0])?, Point::try_new([3.0, 3.0])?)?,
        0.25,
    )?;

    let query = Point::try_new([0.75, -0.4])?;
    let evaluation = mixture.try_evaluate(
        query,
        Point::try_new([-0.5, 0.2])?,
        KernelDerivativeOrder::Second,
    )?;
    let coverage = mixture.try_coverage(query)?;
    println!("value: {}", evaluation.value());
    println!("gradient: {:?}", evaluation.gradient());
    println!("Hessian: {:?}", evaluation.hessian());
    println!("squared weight coverage: {}", coverage.squared_weight_sum());
    println!(
        "background policy ratio: {}",
        mixture.diagnostics().background_policy_ratio()
    );
    Ok(())
}
