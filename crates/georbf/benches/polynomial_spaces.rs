//! Dependency-free deterministic benchmark for complete polynomial spaces.

use std::error::Error;
use std::hint::black_box;
use std::time::{Duration, Instant};

use georbf::{Dim, Point, PolynomialSpace, SupportedDimension};

struct Measurement {
    dimension: usize,
    term_count: usize,
    generation_iterations: u32,
    generation_elapsed: Duration,
    evaluation_iterations: u32,
    evaluation_elapsed: Duration,
    generation_checksum: usize,
    checksum: f64,
}

fn run<const D: usize>(
    base: [f64; D],
    generation_iterations: u32,
    evaluation_iterations: u32,
) -> Result<Measurement, Box<dyn Error>>
where
    Dim<D>: SupportedDimension,
{
    const ORDER: usize = 8;
    let generation_start = Instant::now();
    let mut generated_terms = 0_usize;
    for _ in 0..generation_iterations {
        generated_terms = generated_terms
            .wrapping_add(black_box(PolynomialSpace::<D>::try_new(ORDER)?).term_count());
    }
    let generation_elapsed = generation_start.elapsed();

    let space = PolynomialSpace::<D>::try_new(ORDER)?;
    let mut values = vec![0.0; space.term_count()];
    let mut gradients = vec![[0.0; D]; space.term_count()];
    let evaluation_start = Instant::now();
    let mut checksum = 0.0;
    for iteration in 0..evaluation_iterations {
        let mut coordinates = base;
        coordinates[0] += f64::from(iteration % 17) * 1.0e-7;
        space.try_evaluate(
            Point::try_new(coordinates)?,
            black_box(&mut values),
            black_box(&mut gradients),
        )?;
        checksum += black_box(values[space.term_count() - 1])
            + black_box(gradients[space.term_count() - 1][D - 1]);
    }
    let evaluation_elapsed = evaluation_start.elapsed();

    Ok(Measurement {
        dimension: D,
        term_count: space.term_count(),
        generation_iterations,
        generation_elapsed,
        evaluation_iterations,
        evaluation_elapsed,
        generation_checksum: black_box(generated_terms),
        checksum: black_box(checksum),
    })
}

fn report(measurement: &Measurement) {
    let generation_ns = measurement.generation_elapsed.as_secs_f64() * 1.0e9
        / f64::from(measurement.generation_iterations);
    let evaluation_ns = measurement.evaluation_elapsed.as_secs_f64() * 1.0e9
        / f64::from(measurement.evaluation_iterations);
    println!(
        "D={} terms={} generation_iterations={} generation_ns/iteration={:.2} generation_checksum={} evaluation_iterations={} evaluation_ns/iteration={:.2} checksum={:.17e}",
        measurement.dimension,
        measurement.term_count,
        measurement.generation_iterations,
        generation_ns,
        measurement.generation_checksum,
        measurement.evaluation_iterations,
        evaluation_ns,
        measurement.checksum,
    );
}

fn main() -> Result<(), Box<dyn Error>> {
    let smoke = std::env::args().any(|argument| argument == "--smoke");
    let generation_iterations = if smoke { 100 } else { 10_000 };
    let evaluation_iterations = if smoke { 1_000 } else { 100_000 };

    println!("complete-polynomial deterministic single-thread benchmark (CPD order 8)");
    report(&run([0.75], generation_iterations, evaluation_iterations)?);
    report(&run(
        [0.75, -0.5],
        generation_iterations,
        evaluation_iterations,
    )?);
    report(&run(
        [0.75, -0.5, 0.25],
        generation_iterations,
        evaluation_iterations,
    )?);
    Ok(())
}
