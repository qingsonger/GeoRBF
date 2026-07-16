//! Deterministic dense Cholesky and pivoted-LBLT solve benchmark.

use std::error::Error;
use std::hint::black_box;
use std::time::Instant;

use georbf::{
    ConditionPolicy, DenseEqualitySystem, DenseFactorization, DenseSolveOptions, Regularization,
};

fn spd_system(dimension: usize) -> Result<DenseEqualitySystem, Box<dyn Error>> {
    let entries = dimension.checked_mul(dimension).ok_or("matrix shape")?;
    let mut matrix = vec![0.0; entries];
    let expected = (0..dimension)
        .map(|index| Ok(f64::from(u32::try_from(index + 1)?) * 0.03125))
        .collect::<Result<Vec<_>, std::num::TryFromIntError>>()?;
    for index in 0..dimension {
        matrix[index * dimension + index] = 4.0;
        if index + 1 < dimension {
            matrix[index * dimension + index + 1] = -1.0;
            matrix[(index + 1) * dimension + index] = -1.0;
        }
    }
    from_solution(dimension, matrix, &expected)
}

fn indefinite_system(dimension: usize) -> Result<DenseEqualitySystem, Box<dyn Error>> {
    if !dimension.is_multiple_of(2) {
        return Err("indefinite benchmark dimension must be even".into());
    }
    let entries = dimension.checked_mul(dimension).ok_or("matrix shape")?;
    let mut matrix = vec![0.0; entries];
    let expected = (0..dimension)
        .map(|index| Ok(f64::from(u32::try_from(index + 1)?) * -0.015_625))
        .collect::<Result<Vec<_>, std::num::TryFromIntError>>()?;
    for block in 0..dimension / 2 {
        let first = 2 * block;
        let second = first + 1;
        let coupling = 1.0 + f64::from(u32::try_from(block)?) * 0.001;
        matrix[first * dimension + second] = coupling;
        matrix[second * dimension + first] = coupling;
        matrix[second * dimension + second] = if block.is_multiple_of(2) { 0.25 } else { -0.25 };
    }
    from_solution(dimension, matrix, &expected)
}

fn from_solution(
    dimension: usize,
    matrix: Vec<f64>,
    solution: &[f64],
) -> Result<DenseEqualitySystem, Box<dyn Error>> {
    let rhs = (0..dimension)
        .map(|row| {
            (0..dimension)
                .map(|column| matrix[row * dimension + column] * solution[column])
                .sum::<f64>()
        })
        .collect();
    Ok(DenseEqualitySystem::try_from_row_major(
        dimension, matrix, rhs,
    )?)
}

fn run(
    label: &str,
    system: &DenseEqualitySystem,
    factorization: DenseFactorization,
    iterations: u32,
) -> Result<(), Box<dyn Error>> {
    let options = DenseSolveOptions::try_new(
        factorization,
        Regularization::None,
        ConditionPolicy::default(),
        4,
    )?;
    let mut checksum = 0.0;
    let started = Instant::now();
    for _ in 0..iterations {
        let solution = black_box(system).try_solve(black_box(options))?;
        checksum += black_box(solution.values().iter().sum::<f64>());
        checksum += black_box(solution.diagnostics().effective_rank.condition_estimate);
        checksum += black_box(solution.diagnostics().final_residual.original_infinity);
    }
    let microseconds = started.elapsed().as_secs_f64() * 1.0e6 / f64::from(iterations);
    println!("{label}: {microseconds:.2} us/solve checksum={checksum:.17e}");
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let smoke = std::env::args().any(|argument| argument == "--smoke");
    let iterations = if smoke { 2 } else { 100 };
    let spd = spd_system(64)?;
    let indefinite = indefinite_system(64)?;
    println!("dense-equality deterministic single-thread benchmark");
    run(
        "64x64 checked Cholesky",
        &spd,
        DenseFactorization::Cholesky,
        iterations,
    )?;
    run(
        "64x64 pivoted LBLT",
        &indefinite,
        DenseFactorization::PivotedLblt,
        iterations,
    )?;
    Ok(())
}
