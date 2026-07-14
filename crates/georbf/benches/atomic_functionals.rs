//! Dependency-free deterministic benchmark for atomic functional actions.

use std::convert::Infallible;
use std::error::Error;
use std::hint::black_box;
use std::time::{Duration, Instant};

use georbf::{
    CenterRepresenter, FunctionalAtom, FunctionalExpr, FunctionalProvenance, FunctionalTerm,
    Gaussian, ObservationFunctional, Point, PolynomialSpace, RadialSeparation, ScalarFieldSample,
    SpatialKernelJet, UnitDirection,
};

fn elapsed_per_iteration(elapsed: Duration, iterations: u32) -> f64 {
    elapsed.as_secs_f64() * 1.0e9 / f64::from(iterations)
}

fn run<const D: usize>(
    label: &str,
    query_components: [f64; D],
    center_components: [f64; D],
    direction_components: [f64; D],
    iterations: u32,
) -> Result<(), Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    let query = Point::try_new(query_components)?;
    let center = Point::try_new(center_components)?;
    let direction = UnitDirection::try_new(direction_components)?;
    let value = FunctionalAtom::value(query, FunctionalProvenance::new(1));
    let derivative =
        FunctionalAtom::directional_derivative(query, direction, FunctionalProvenance::new(2));
    let expression = FunctionalExpr::try_new([
        FunctionalTerm::try_new(1.25, value)?,
        FunctionalTerm::try_new(-0.5, derivative)?,
    ])?;
    let observation = ObservationFunctional::new(expression.clone());
    let center_expression = FunctionalExpr::try_new([
        FunctionalTerm::try_new(
            0.75,
            FunctionalAtom::value(center, FunctionalProvenance::new(3)),
        )?,
        FunctionalTerm::try_new(
            0.25,
            FunctionalAtom::directional_derivative(center, direction, FunctionalProvenance::new(4)),
        )?,
    ])?;
    let representer = CenterRepresenter::new(center_expression);
    let sample = ScalarFieldSample::try_new(1.5, [0.25; D])?;
    let samples = [sample, sample];
    let polynomial = PolynomialSpace::try_new(4)?;
    let separation = RadialSeparation::try_new(query, center)?;
    let radial = Gaussian::try_new(2.0)?.radial_jet(separation)?;
    let jet = SpatialKernelJet::try_new(separation, radial)?;

    let started = Instant::now();
    let mut sample_checksum = 0.0;
    for _ in 0..iterations {
        sample_checksum += black_box(expression.try_apply_samples(black_box(&samples))?);
    }
    let sample_elapsed = started.elapsed();

    let polynomial_iterations = iterations / 10;
    let started = Instant::now();
    let mut polynomial_checksum = 0.0;
    for _ in 0..polynomial_iterations {
        let output = black_box(expression.try_apply_polynomial(black_box(&polynomial))?);
        polynomial_checksum += black_box(output[output.len() - 1]);
    }
    let polynomial_elapsed = started.elapsed();

    let started = Instant::now();
    let mut kernel_checksum = 0.0;
    for _ in 0..iterations {
        kernel_checksum += black_box(
            observation
                .try_apply_kernel(&representer, |_, _, _| Ok::<_, Infallible>(jet.into()))?,
        );
    }
    let kernel_elapsed = started.elapsed();

    println!(
        "{label}: sample={:.2} ns/iter polynomial={:.2} ns/iter kernel={:.2} ns/iter checksums=[{:.17e},{:.17e},{:.17e}]",
        elapsed_per_iteration(sample_elapsed, iterations),
        elapsed_per_iteration(polynomial_elapsed, polynomial_iterations),
        elapsed_per_iteration(kernel_elapsed, iterations),
        black_box(sample_checksum),
        black_box(polynomial_checksum),
        black_box(kernel_checksum),
    );
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let smoke = std::env::args().any(|argument| argument == "--smoke");
    let iterations = if smoke { 2_000 } else { 200_000 };
    println!("atomic-functionals deterministic single-thread benchmark");
    run("D=1", [1.25], [-0.5], [1.0], iterations)?;
    run("D=2", [1.25, -0.75], [-0.5, 0.25], [1.0, 2.0], iterations)?;
    run(
        "D=3",
        [1.25, -0.75, 0.5],
        [-0.5, 0.25, -1.0],
        [1.0, 2.0, -1.0],
        iterations,
    )?;
    Ok(())
}
