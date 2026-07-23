//! Export renderer-neutral local anisotropy diagnostics at caller sample points.

use std::error::Error;

use georbf::{
    AnisotropyConditionPolicy, Gaussian, GlobalAnisotropy, KernelDefinition, LocalTrendBackground,
    LocalTrendControl, OperationalDomain, Point, SmoothSpatialWeight, TrendControlOrientation,
    TrendControlPolicy, TrendDirectionSource, UnitDirection, try_compile_local_trend_controls,
    try_export_anisotropy_diagnostics,
};

fn main() -> Result<(), Box<dyn Error>> {
    let background = LocalTrendBackground::new(
        KernelDefinition::from(Gaussian::try_new(1.0)?),
        GlobalAnisotropy::try_isotropic(1.0)?,
        SmoothSpatialWeight::try_constant(0.4)?,
    );
    let control = LocalTrendControl::new(
        Point::try_new([0.0, 0.0])?,
        KernelDefinition::from(Gaussian::try_new(0.8)?),
        TrendControlOrientation::Spheroidal {
            principal_axis: TrendDirectionSource::Explicit(UnitDirection::try_new([1.0, 1.0])?),
            axial_length: 2.5,
            transverse_length: 0.75,
        },
        1.2,
        1.5,
        None,
    );
    let compiled = try_compile_local_trend_controls(
        background,
        &[control],
        None,
        OperationalDomain::try_new(Point::try_new([-3.0, -3.0])?, Point::try_new([3.0, 3.0])?)?,
        0.25,
        TrendControlPolicy::try_new(
            AnisotropyConditionPolicy::Maximum(10.0),
            1.0e-10,
            1.0e-6,
            std::f64::consts::FRAC_PI_4,
        )?,
    )?;
    let export = try_export_anisotropy_diagnostics(
        &compiled,
        &[Point::try_new([0.0, 0.0])?, Point::try_new([2.0, 0.0])?],
    )?;

    println!("controls: {}", export.controls().len());
    println!("background: {:?}", export.background());
    for sample in export.samples() {
        println!(
            "position={:?} weights={:?} coverage={}",
            sample.position(),
            sample.component_weights(),
            sample.coverage().squared_weight_sum()
        );
    }
    Ok(())
}
