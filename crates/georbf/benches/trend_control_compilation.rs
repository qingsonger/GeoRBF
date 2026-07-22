//! Dependency-free deterministic benchmark for regional trend-control compilation.

use std::error::Error;
use std::hint::black_box;
use std::time::{Duration, Instant};

use georbf::{
    AnisotropyConditionPolicy, Gaussian, GlobalAnisotropy, KernelDefinition, LocalTrendBackground,
    LocalTrendControl, OperationalDomain, Point, SmoothRegion, SmoothSpatialWeight,
    TrendControlOrientation, TrendControlPolicy, TrendDirectionSource, UnitDirection,
    try_compile_local_trend_controls,
};

struct Measurement {
    controls: usize,
    iterations: u32,
    elapsed: Duration,
    checksum: f64,
}

fn run(control_count: usize, iterations: u32) -> Result<Measurement, Box<dyn Error>> {
    let background = LocalTrendBackground::new(
        KernelDefinition::from(Gaussian::try_new(1.0)?),
        GlobalAnisotropy::try_isotropic(1.0)?,
        SmoothSpatialWeight::try_constant(0.4)?,
    );
    let region = SmoothRegion::try_new(
        Point::try_new([-4.0, -4.0, -4.0])?,
        Point::try_new([4.0, 4.0, 4.0])?,
        0.5,
    )?;
    let mut controls = Vec::with_capacity(control_count);
    for index in 0..control_count {
        let offset = f64::from(u32::try_from(index)?) * 0.02;
        controls.push(LocalTrendControl::new(
            Point::try_new([offset, -0.5 * offset, 0.25 * offset])?,
            KernelDefinition::from(Gaussian::try_new(0.8)?),
            TrendControlOrientation::Spheroidal {
                principal_axis: TrendDirectionSource::Explicit(UnitDirection::try_new([
                    1.0,
                    0.1 + offset,
                    0.2,
                ])?),
                axial_length: 2.5,
                transverse_length: 0.75,
            },
            1.2,
            1.0 + offset,
            Some(region),
        ));
    }
    let policy = TrendControlPolicy::try_new(
        AnisotropyConditionPolicy::Maximum(10.0),
        1.0e-12,
        1.0e-6,
        std::f64::consts::FRAC_PI_4,
    )?;
    let domain = OperationalDomain::try_new(
        Point::try_new([-5.0, -5.0, -5.0])?,
        Point::try_new([5.0, 5.0, 5.0])?,
    )?;

    let start = Instant::now();
    let mut checksum = 0.0;
    for _ in 0..iterations {
        let compiled = black_box(try_compile_local_trend_controls(
            background,
            black_box(&controls),
            None,
            domain,
            0.25,
            policy,
        )?);
        checksum += compiled
            .mixture()
            .diagnostics()
            .maximum_anisotropy_condition_number()
            + compiled.diagnostics().maximum_direction_jump_radians();
    }
    Ok(Measurement {
        controls: control_count,
        iterations,
        elapsed: start.elapsed(),
        checksum: black_box(checksum),
    })
}

fn report(measurement: &Measurement) {
    let nanoseconds = measurement.elapsed.as_secs_f64() * 1.0e9;
    println!(
        "controls={} iterations={} elapsed={:.6}s ns/compilation={:.2} checksum={:.17e}",
        measurement.controls,
        measurement.iterations,
        measurement.elapsed.as_secs_f64(),
        nanoseconds / f64::from(measurement.iterations),
        measurement.checksum,
    );
}

fn main() -> Result<(), Box<dyn Error>> {
    let smoke = std::env::args().any(|argument| argument == "--smoke");
    let iterations = if smoke { 200 } else { 10_000 };
    println!("regional trend-control deterministic single-thread compilation benchmark");
    report(&run(4, iterations)?);
    report(&run(16, iterations)?);
    Ok(())
}
