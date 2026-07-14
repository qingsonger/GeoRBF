//! Dependency-free deterministic benchmark for smooth global-support kernels.

use std::error::Error;
use std::hint::black_box;
use std::time::{Duration, Instant};

use georbf::{
    Dim, Gaussian, InverseMultiquadric, KernelArgument, Matern, MaternSmoothness, Multiquadric,
    Point, RadialSeparation, SpatialKernelJet, SupportedDimension,
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
    let gaussian = Gaussian::try_new(1.25)?;
    let inverse = InverseMultiquadric::try_new(1.25)?;
    let multiquadric = Multiquadric::try_new(1.25)?;
    let matern_one = Matern::try_new(MaternSmoothness::OneHalf, 1.25)?;
    let matern_three = Matern::try_new(MaternSmoothness::ThreeHalves, 1.25)?;
    let matern_five = Matern::try_new(MaternSmoothness::FiveHalves, 1.25)?;
    let center = Point::try_new([0.0; D])?;
    let start = Instant::now();
    let mut checksum = 0.0;

    for iteration in 0..iterations {
        let perturbation = f64::from(iteration % 17) * 1.0e-6;
        let mut query = base;
        query[0] += perturbation;
        let separation = RadialSeparation::try_new(Point::try_new(query)?, center)?;

        let gaussian = SpatialKernelJet::try_new(separation, gaussian.radial_jet(separation)?)?;
        let inverse = SpatialKernelJet::try_new(separation, inverse.radial_jet(separation)?)?;
        let multiquadric =
            SpatialKernelJet::try_new(separation, multiquadric.radial_jet(separation)?)?;
        let matern_one = SpatialKernelJet::try_new(separation, matern_one.radial_jet(separation)?)?;
        let matern_three =
            SpatialKernelJet::try_new(separation, matern_three.radial_jet(separation)?)?;
        let matern_five =
            SpatialKernelJet::try_new(separation, matern_five.radial_jet(separation)?)?;

        for jet in [
            gaussian,
            inverse,
            multiquadric,
            matern_one,
            matern_three,
            matern_five,
        ] {
            let jet = black_box(jet);
            checksum += jet.value()
                + jet.first_derivative(KernelArgument::Query)[D - 1]
                + jet.second_derivative([KernelArgument::Query; 2])[D - 1][D - 1]
                + jet.third_derivative([KernelArgument::Query; 3])[D - 1][D - 1][D - 1];
        }
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

    println!("smooth-global deterministic single-thread benchmark (six family members)");
    report(&run([0.75], iterations)?);
    report(&run([0.75, -1.25], iterations)?);
    report(&run([0.75, -1.25, 0.5], iterations)?);
    Ok(())
}
