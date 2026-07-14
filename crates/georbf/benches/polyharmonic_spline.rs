//! Dependency-free deterministic benchmark for a concrete polyharmonic kernel.

use std::error::Error;
use std::hint::black_box;
use std::time::{Duration, Instant};

use georbf::{
    Dim, KernelArgument, Point, PolyharmonicSpline, RadialSeparation, SpatialKernelJet,
    SupportedDimension,
};

struct Measurement {
    dimension: usize,
    iterations: u32,
    elapsed: Duration,
    checksum: f64,
}

fn run<const D: usize>(base: [f64; D], iterations: u32) -> Result<Measurement, Box<dyn Error>>
where
    Dim<D>: SupportedDimension,
{
    let kernel = PolyharmonicSpline::try_new(5)?;
    let center = Point::try_new([0.0; D])?;
    let start = Instant::now();
    let mut checksum = 0.0;

    for iteration in 0..iterations {
        let perturbation = f64::from(iteration % 17) * 1.0e-6;
        let mut query = base;
        query[0] += perturbation;
        let separation = RadialSeparation::try_new(Point::try_new(query)?, center)?;
        let radial = kernel.radial_jet(separation)?;
        let jet = black_box(SpatialKernelJet::try_new(separation, radial)?);
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

    println!("polyharmonic-spline deterministic single-thread benchmark (power=5)");
    report(&run([0.75], iterations)?);
    report(&run([0.75, -1.25], iterations)?);
    report(&run([0.75, -1.25, 0.5], iterations)?);
    Ok(())
}
