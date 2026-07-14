//! Dependency-free deterministic benchmark for global anisotropy.

use std::error::Error;
use std::hint::black_box;
use std::time::{Duration, Instant};

use georbf::{
    AnisotropyConditionPolicy, Dim, GlobalAnisotropy, KernelArgument, Point,
    RadialExpansionCoefficients, RadialJet, SpatialKernelJet, SupportedDimension,
};

struct Measurement {
    dimension: usize,
    iterations: u32,
    elapsed: Duration,
    checksum: f64,
}

fn run<const D: usize>(
    transform: [[f64; D]; D],
    base: [f64; D],
    iterations: u32,
) -> Result<Measurement, Box<dyn Error>>
where
    Dim<D>: SupportedDimension,
{
    let anisotropy =
        GlobalAnisotropy::try_from_transform(transform, AnisotropyConditionPolicy::Unbounded)?;
    let center = Point::try_new([0.0; D])?;
    let start = Instant::now();
    let mut checksum = 0.0;

    for iteration in 0..iterations {
        let perturbation = f64::from(iteration % 17) * 1.0e-6;
        let mut query = base;
        query[0] += perturbation;
        let separation = anisotropy.try_transform_separation(Point::try_new(query)?, center)?;
        let radius = separation.radius();
        let radial = RadialJet::try_away_with_expansion(
            radius.powi(6),
            6.0 * radius.powi(5),
            30.0 * radius.powi(4),
            120.0 * radius.powi(3),
            RadialExpansionCoefficients::try_new(6.0 * radius.powi(4), 24.0 * radius.powi(3))?,
        )?;
        let transformed = SpatialKernelJet::try_new(separation, radial)?;
        let jet = black_box(anisotropy.try_transform_spatial_jet(transformed)?);
        checksum += jet.value()
            + jet.first_derivative(KernelArgument::Query)[D - 1]
            + jet.second_derivative([KernelArgument::Query; 2])[D - 1][D - 1]
            + jet.third_derivative([KernelArgument::Query; 3])[D - 1][D - 1][D - 1];
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
    let nanoseconds_per_iteration = nanoseconds / f64::from(measurement.iterations);
    println!(
        "D={} iterations={} elapsed={:.6}s ns/iteration={:.2} checksum={:.17e}",
        measurement.dimension,
        measurement.iterations,
        measurement.elapsed.as_secs_f64(),
        nanoseconds_per_iteration,
        measurement.checksum,
    );
}

fn main() -> Result<(), Box<dyn Error>> {
    let smoke = std::env::args().any(|argument| argument == "--smoke");
    let iterations = if smoke { 20_000 } else { 1_000_000 };

    println!("global-anisotropy deterministic single-thread benchmark");
    report(&run([[0.5]], [0.75], iterations)?);
    report(&run(
        [[0.5, 0.125], [-0.25, 0.75]],
        [0.75, -1.25],
        iterations,
    )?);
    report(&run(
        [[0.5, 0.125, 0.0], [-0.25, 0.75, 0.1], [0.0, 0.0, 1.25]],
        [0.75, -1.25, 0.5],
        iterations,
    )?);
    Ok(())
}
