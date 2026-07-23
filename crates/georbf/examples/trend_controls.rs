//! Compile regional geological controls into a strict-background local mixture.

use std::error::Error;

use georbf::{
    AnisotropyConditionPolicy, Gaussian, GlobalAnisotropy, KernelDefinition, KernelDerivativeOrder,
    LocalTrendBackground, LocalTrendControl, OperationalDomain, Point, SmoothRegion,
    SmoothSpatialWeight, TrendControlOrientation, TrendControlPolicy, TrendDirectionSource,
    UnitDirection, try_compile_local_trend_controls,
};

fn main() -> Result<(), Box<dyn Error>> {
    let background = LocalTrendBackground::new(
        KernelDefinition::from(Gaussian::try_new(1.0)?),
        GlobalAnisotropy::try_isotropic(1.0)?,
        SmoothSpatialWeight::try_constant(0.4)?,
    );
    let region = SmoothRegion::try_new(
        Point::try_new([-2.0, -2.0])?,
        Point::try_new([2.0, 2.0])?,
        0.5,
    )?;
    let control = LocalTrendControl::new(
        Point::try_new([0.25, -0.25])?,
        KernelDefinition::from(Gaussian::try_new(0.8)?),
        TrendControlOrientation::Spheroidal {
            principal_axis: TrendDirectionSource::Explicit(UnitDirection::try_new([1.0, 1.0])?),
            axial_length: 2.5,
            transverse_length: 0.75,
        },
        1.2,
        1.5,
        Some(region),
    );
    let policy = TrendControlPolicy::try_new(
        AnisotropyConditionPolicy::Maximum(10.0),
        1.0e-10,
        1.0e-6,
        std::f64::consts::FRAC_PI_4,
    )?;
    let compiled = try_compile_local_trend_controls(
        background,
        &[control],
        None,
        OperationalDomain::try_new(Point::try_new([-3.0, -3.0])?, Point::try_new([3.0, 3.0])?)?,
        0.25,
        policy,
    )?;

    let query = Point::try_new([-1.75, 0.0])?;
    let evaluation = compiled.mixture().try_evaluate(
        query,
        Point::try_new([0.5, 0.5])?,
        KernelDerivativeOrder::Second,
    )?;
    println!("components: {}", compiled.mixture().components().len());
    println!("value: {}", evaluation.value());
    println!("gradient: {:?}", evaluation.gradient());
    println!("Hessian: {:?}", evaluation.hessian());
    println!(
        "coverage: {}",
        compiled.mixture().try_coverage(query)?.squared_weight_sum()
    );
    Ok(())
}
