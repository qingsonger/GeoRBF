//! Dependency-free deterministic benchmark for local mixture Hessian evaluation.

use std::error::Error;
use std::hint::black_box;
use std::time::{Duration, Instant};

use georbf::{
    AnisotropyConditionPolicy, Dim, Gaussian, GlobalAnisotropy, KernelDefinition,
    KernelDerivativeOrder, LocalTrendComponent, LocalTrendMixture, OperationalDomain, Point,
    SmoothSpatialWeight, SupportedDimension,
};

struct Measurement {
    dimension: usize,
    iterations: u32,
    elapsed: Duration,
    checksum: f64,
}

fn run<const D: usize>(
    local_transform: [[f64; D]; D],
    query: [f64; D],
    center: [f64; D],
    iterations: u32,
) -> Result<Measurement, Box<dyn Error>>
where
    Dim<D>: SupportedDimension,
{
    let background = LocalTrendComponent::new(
        KernelDefinition::from(Gaussian::try_new(1.0)?),
        GlobalAnisotropy::try_isotropic(1.0)?,
        SmoothSpatialWeight::try_constant(0.3)?,
    );
    let local = LocalTrendComponent::new(
        KernelDefinition::from(Gaussian::try_new(0.8)?),
        GlobalAnisotropy::try_from_transform(
            local_transform,
            AnisotropyConditionPolicy::Unbounded,
        )?,
        SmoothSpatialWeight::try_gaussian(Point::try_new([0.25; D])?, 1.4, 1.5)?,
    );
    let mixture = LocalTrendMixture::try_new(
        vec![background, local],
        0,
        OperationalDomain::try_new(Point::try_new([-4.0; D])?, Point::try_new([4.0; D])?)?,
        0.2,
    )?;

    let center = Point::try_new(center)?;
    let start = Instant::now();
    let mut checksum = 0.0;
    for iteration in 0..iterations {
        let mut current = query;
        current[0] += f64::from(iteration % 19) * 1.0e-7;
        let evaluation = black_box(mixture.try_evaluate(
            Point::try_new(current)?,
            center,
            KernelDerivativeOrder::Second,
        )?);
        let gradient = evaluation
            .gradient()
            .ok_or_else(|| std::io::Error::other("gradient demand was not retained"))?;
        let hessian = evaluation
            .hessian()
            .ok_or_else(|| std::io::Error::other("Hessian demand was not retained"))?;
        checksum += evaluation.value() + gradient[D - 1] + hessian[D - 1][D - 1];
    }
    Ok(Measurement {
        dimension: D,
        iterations,
        elapsed: start.elapsed(),
        checksum: black_box(checksum),
    })
}

fn report(measurement: &Measurement) {
    let nanoseconds = measurement.elapsed.as_secs_f64() * 1.0e9;
    println!(
        "D={} iterations={} elapsed={:.6}s ns/iteration={:.2} checksum={:.17e}",
        measurement.dimension,
        measurement.iterations,
        measurement.elapsed.as_secs_f64(),
        nanoseconds / f64::from(measurement.iterations),
        measurement.checksum,
    );
}

fn main() -> Result<(), Box<dyn Error>> {
    let smoke = std::env::args().any(|argument| argument == "--smoke");
    let iterations = if smoke { 10_000 } else { 500_000 };
    println!("local-trend deterministic single-thread Hessian benchmark");
    report(&run([[0.8]], [0.6], [-0.4], iterations)?);
    report(&run(
        [[0.8, 0.2], [-0.1, 1.1]],
        [0.6, -0.3],
        [-0.4, 0.5],
        iterations,
    )?);
    report(&run(
        [[0.8, 0.2, 0.0], [-0.1, 1.1, 0.1], [0.0, -0.15, 0.9]],
        [0.6, -0.3, 0.2],
        [-0.4, 0.5, -0.1],
        iterations,
    )?);
    Ok(())
}
