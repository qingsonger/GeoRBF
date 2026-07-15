use std::hint::black_box;
use std::time::{Duration, Instant};

#[cfg(not(any(feature = "faer-backend", feature = "nalgebra-backend")))]
compile_error!(
    "enable at least one dense factorization backend feature: faer-backend or nalgebra-backend"
);

#[cfg(feature = "faer-backend")]
use faer::prelude::Solve;
#[cfg(feature = "nalgebra-backend")]
use nalgebra::{DMatrix, DVector, linalg::LBLT};

const MAX_REFINEMENT_STEPS: usize = 3;
const ACCEPTED_BACKWARD_ERROR: f64 = 1.0e-8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Backend {
    #[cfg(feature = "faer-backend")]
    Faer,
    #[cfg(feature = "nalgebra-backend")]
    Nalgebra,
}

impl Backend {
    const ALL: &'static [Self] = &[
        #[cfg(feature = "faer-backend")]
        Self::Faer,
        #[cfg(feature = "nalgebra-backend")]
        Self::Nalgebra,
    ];

    const fn label(self) -> &'static str {
        match self {
            #[cfg(feature = "faer-backend")]
            Self::Faer => "faer-0.24.4",
            #[cfg(feature = "nalgebra-backend")]
            Self::Nalgebra => "nalgebra-0.35.0",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Factorization {
    Cholesky,
    PivotedLblt,
}

impl Factorization {
    const fn label(self) -> &'static str {
        match self {
            Self::Cholesky => "cholesky",
            Self::PivotedLblt => "pivoted-lblt",
        }
    }
}

#[derive(Clone, Debug)]
struct LinearCase {
    dimension: usize,
    matrix: Vec<f64>,
    rhs: Vec<f64>,
}

impl LinearCase {
    fn new(dimension: usize, matrix: Vec<f64>, rhs: Vec<f64>) -> Result<Self, String> {
        let expected = dimension
            .checked_mul(dimension)
            .ok_or_else(|| "matrix shape overflows usize".to_owned())?;
        if dimension == 0 {
            return Err("matrix must be nonempty".to_owned());
        }
        if matrix.len() != expected || rhs.len() != dimension {
            return Err(format!(
                "data shape does not match {dimension}x{dimension} matrix and RHS"
            ));
        }
        if matrix.iter().chain(&rhs).any(|value| !value.is_finite()) {
            return Err("matrix and RHS entries must be finite".to_owned());
        }
        for row in 0..dimension {
            for column in 0..row {
                if matrix[row * dimension + column].to_bits()
                    != matrix[column * dimension + row].to_bits()
                {
                    return Err("matrix must be exactly symmetric".to_owned());
                }
            }
        }
        Ok(Self {
            dimension,
            matrix,
            rhs,
        })
    }

    fn from_solution(dimension: usize, matrix: Vec<f64>, solution: &[f64]) -> Result<Self, String> {
        if solution.len() != dimension || solution.iter().any(|value| !value.is_finite()) {
            return Err("truth solution must be finite and match the dimension".to_owned());
        }
        let rhs = matrix_vector_product(dimension, &matrix, solution)?;
        Self::new(dimension, matrix, rhs)
    }

    fn with_rhs(&self, rhs: Vec<f64>) -> Result<Self, String> {
        Self::new(self.dimension, self.matrix.clone(), rhs)
    }
}

#[derive(Clone, Debug, PartialEq)]
struct SolveReport {
    solution: Vec<f64>,
    residual_inf: f64,
    backward_error: f64,
}

#[derive(Clone, Debug, PartialEq)]
struct RefinementReport {
    solution: Vec<f64>,
    initial_residual_inf: f64,
    final_residual_inf: f64,
    final_backward_error: f64,
    accepted_steps: usize,
}

fn matrix_vector_product(
    dimension: usize,
    matrix: &[f64],
    vector: &[f64],
) -> Result<Vec<f64>, String> {
    if matrix.len() != dimension.saturating_mul(dimension) || vector.len() != dimension {
        return Err("matrix-vector shape mismatch".to_owned());
    }
    let mut product = vec![0.0; dimension];
    for row in 0..dimension {
        let mut sum = 0.0;
        for column in 0..dimension {
            sum = matrix[row * dimension + column].mul_add(vector[column], sum);
        }
        product[row] = sum;
    }
    Ok(product)
}

fn residual_vector(case: &LinearCase, solution: &[f64]) -> Result<Vec<f64>, String> {
    let product = matrix_vector_product(case.dimension, &case.matrix, solution)?;
    Ok(case
        .rhs
        .iter()
        .zip(product)
        .map(|(rhs, value)| rhs - value)
        .collect())
}

fn infinity_norm(values: &[f64]) -> f64 {
    values.iter().map(|value| value.abs()).fold(0.0, f64::max)
}

fn matrix_infinity_norm(case: &LinearCase) -> f64 {
    (0..case.dimension)
        .map(|row| {
            case.matrix[row * case.dimension..(row + 1) * case.dimension]
                .iter()
                .map(|value| value.abs())
                .sum::<f64>()
        })
        .fold(0.0, f64::max)
}

fn residual_metrics(case: &LinearCase, solution: &[f64]) -> Result<(f64, f64), String> {
    if solution.iter().any(|value| !value.is_finite()) {
        return Err("backend produced a nonfinite solution".to_owned());
    }
    let residual_inf = infinity_norm(&residual_vector(case, solution)?);
    let denominator =
        matrix_infinity_norm(case).mul_add(infinity_norm(solution), infinity_norm(&case.rhs));
    let backward_error = if denominator == 0.0 {
        residual_inf
    } else {
        residual_inf / denominator
    };
    Ok((residual_inf, backward_error))
}

fn solve(
    case: &LinearCase,
    backend: Backend,
    method: Factorization,
) -> Result<SolveReport, String> {
    let solution = match backend {
        #[cfg(feature = "faer-backend")]
        Backend::Faer => solve_faer(case, method)?,
        #[cfg(feature = "nalgebra-backend")]
        Backend::Nalgebra => solve_nalgebra(case, method)?,
    };
    let (residual_inf, backward_error) = residual_metrics(case, &solution)?;
    if backward_error > ACCEPTED_BACKWARD_ERROR {
        return Err(format!(
            "{} {} solve failed residual review: {backward_error:.17e}",
            backend.label(),
            method.label()
        ));
    }
    Ok(SolveReport {
        solution,
        residual_inf,
        backward_error,
    })
}

#[cfg(feature = "faer-backend")]
fn solve_faer(case: &LinearCase, method: Factorization) -> Result<Vec<f64>, String> {
    let matrix = faer::Mat::from_fn(case.dimension, case.dimension, |row, column| {
        case.matrix[row * case.dimension + column]
    });
    let rhs = faer::Mat::from_fn(case.dimension, 1, |row, _| case.rhs[row]);
    let solution = match method {
        Factorization::Cholesky => matrix
            .as_ref()
            .llt(faer::Side::Lower)
            .map_err(|error| format!("faer checked Cholesky rejected matrix: {error:?}"))?
            .solve(&rhs),
        Factorization::PivotedLblt => matrix.as_ref().lblt(faer::Side::Lower).solve(&rhs),
    };
    Ok((0..case.dimension)
        .map(|index| solution[(index, 0)])
        .collect())
}

#[cfg(feature = "nalgebra-backend")]
fn solve_nalgebra(case: &LinearCase, method: Factorization) -> Result<Vec<f64>, String> {
    let matrix = DMatrix::from_row_slice(case.dimension, case.dimension, &case.matrix);
    let rhs = DVector::from_column_slice(&case.rhs);
    let solution = match method {
        Factorization::Cholesky => matrix
            .cholesky()
            .ok_or_else(|| "nalgebra checked Cholesky rejected matrix".to_owned())?
            .solve(&rhs),
        Factorization::PivotedLblt => LBLT::new(matrix)
            .solve(&rhs)
            .ok_or_else(|| "nalgebra pivoted LBLT reported a zero pivot".to_owned())?,
    };
    Ok(solution.iter().copied().collect())
}

fn refine(
    case: &LinearCase,
    backend: Backend,
    method: Factorization,
) -> Result<RefinementReport, String> {
    let initial = solve(case, backend, method)?;
    let mut solution = initial.solution;
    let mut residual_inf = initial.residual_inf;
    let mut backward_error = initial.backward_error;
    let initial_residual_inf = residual_inf;
    let mut accepted_steps = 0;

    for _ in 0..MAX_REFINEMENT_STEPS {
        let residual = residual_vector(case, &solution)?;
        if infinity_norm(&residual) == 0.0 {
            break;
        }
        let correction_case = case.with_rhs(residual)?;
        let correction = solve(&correction_case, backend, method)?;
        let candidate = solution
            .iter()
            .zip(correction.solution)
            .map(|(value, delta)| value + delta)
            .collect::<Vec<_>>();
        let (candidate_residual, candidate_backward_error) = residual_metrics(case, &candidate)?;
        if candidate_residual >= residual_inf {
            break;
        }
        solution = candidate;
        residual_inf = candidate_residual;
        backward_error = candidate_backward_error;
        accepted_steps += 1;
    }

    Ok(RefinementReport {
        solution,
        initial_residual_inf,
        final_residual_inf: residual_inf,
        final_backward_error: backward_error,
        accepted_steps,
    })
}

fn spd_case(dimension: usize) -> Result<(LinearCase, Vec<f64>), String> {
    let mut matrix = vec![0.0; dimension.saturating_mul(dimension)];
    for index in 0..dimension {
        matrix[index * dimension + index] = 4.0;
        if index + 1 < dimension {
            matrix[index * dimension + index + 1] = -1.0;
            matrix[(index + 1) * dimension + index] = -1.0;
        }
    }
    let solution = (0..dimension)
        .map(|index| {
            let value = u32::try_from(index + 1).map_or(f64::INFINITY, f64::from);
            (value * 0.125).sin()
        })
        .collect::<Vec<_>>();
    let case = LinearCase::from_solution(dimension, matrix, &solution)?;
    Ok((case, solution))
}

fn indefinite_case(dimension: usize) -> Result<(LinearCase, Vec<f64>), String> {
    if !dimension.is_multiple_of(2) {
        return Err("indefinite benchmark dimension must be even".to_owned());
    }
    let mut matrix = vec![0.0; dimension.saturating_mul(dimension)];
    for block in 0..dimension / 2 {
        let first = 2 * block;
        let second = first + 1;
        let coupling = 1.0 + u32::try_from(block).map_or(f64::INFINITY, f64::from) * 0.001;
        matrix[first * dimension + second] = coupling;
        matrix[second * dimension + first] = coupling;
        matrix[second * dimension + second] = if block % 2 == 0 { 0.25 } else { -0.25 };
    }
    let solution = (0..dimension)
        .map(|index| {
            let value = u32::try_from(index + 1).map_or(f64::INFINITY, f64::from);
            (value * 0.0625).cos()
        })
        .collect::<Vec<_>>();
    let case = LinearCase::from_solution(dimension, matrix, &solution)?;
    Ok((case, solution))
}

#[cfg(test)]
fn ill_conditioned_case() -> Result<(LinearCase, Vec<f64>), String> {
    let scales = [
        2.0_f64.powi(-10),
        2.0_f64.powi(-6),
        2.0_f64.powi(-2),
        2.0_f64.powi(2),
        2.0_f64.powi(6),
        2.0_f64.powi(10),
    ];
    let dimension = scales.len();
    let mut matrix = vec![0.0; dimension * dimension];
    for row in 0..dimension {
        for column in 0..dimension {
            let base = if row == column {
                2.0
            } else if row.abs_diff(column) == 1 {
                -0.25
            } else {
                0.0
            };
            matrix[row * dimension + column] = base * scales[row] * scales[column];
        }
    }
    let solution = vec![1.0, -2.0, 3.0, -4.0, 5.0, -6.0];
    let case = LinearCase::from_solution(dimension, matrix, &solution)?;
    Ok((case, solution))
}

fn time_backend(
    case: &LinearCase,
    backend: Backend,
    method: Factorization,
    iterations: usize,
) -> Result<(Duration, f64, RefinementReport), String> {
    let started = Instant::now();
    let mut checksum = 0.0;
    let mut last_report = None;
    for _ in 0..iterations {
        let report = black_box(refine(black_box(case), backend, method))?;
        checksum += report.solution.iter().sum::<f64>();
        checksum += report.final_residual_inf;
        last_report = Some(report);
    }
    let report = last_report.ok_or_else(|| "benchmark iterations must be nonzero".to_owned())?;
    Ok((started.elapsed(), black_box(checksum), report))
}

fn run_benchmark(smoke: bool) -> Result<(), String> {
    let sizes: &[usize] = if smoke { &[16, 32] } else { &[32, 64, 128] };
    let iterations = if smoke { 1 } else { 3 };
    println!(
        "backend,factorization,size,iterations,elapsed_ns,checksum,initial_residual_inf,final_residual_inf,accepted_refinement_steps"
    );
    for &size in sizes {
        let (spd, _) = spd_case(size)?;
        let (indefinite, _) = indefinite_case(size)?;
        for (method, case) in [
            (Factorization::Cholesky, &spd),
            (Factorization::PivotedLblt, &indefinite),
        ] {
            for &backend in Backend::ALL {
                let (elapsed, checksum, report) = time_backend(case, backend, method, iterations)?;
                println!(
                    "{},{},{size},{iterations},{},{checksum:.17e},{:.17e},{:.17e},{}",
                    backend.label(),
                    method.label(),
                    elapsed.as_nanos(),
                    report.initial_residual_inf,
                    report.final_residual_inf,
                    report.accepted_steps
                );
            }
        }
    }
    Ok(())
}

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let smoke = match args.next().as_deref() {
        None => false,
        Some("--smoke") => true,
        Some(argument) => return Err(format!("unknown argument: {argument}")),
    };
    if let Some(argument) = args.next() {
        return Err(format!("unexpected extra argument: {argument}"));
    }
    run_benchmark(smoke)
}

#[cfg(test)]
mod factorization_spike_cases {
    use super::*;

    fn assert_solution_close(actual: &[f64], expected: &[f64], tolerance: f64, label: &str) {
        assert_eq!(actual.len(), expected.len(), "{label} solution length");
        for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
            assert!(
                (actual - expected).abs() <= tolerance * expected.abs().max(1.0),
                "{label} solution[{index}] expected {expected:.17e}, got {actual:.17e}"
            );
        }
    }

    #[test]
    fn analytic_spd_truth_agrees() -> Result<(), String> {
        let matrix = vec![4.0, 1.0, 1.0, 3.0];
        let expected = vec![1.0, -2.0];
        let case = LinearCase::from_solution(2, matrix, &expected)?;
        for &backend in Backend::ALL {
            for method in [Factorization::Cholesky, Factorization::PivotedLblt] {
                let report = solve(&case, backend, method)?;
                assert_solution_close(&report.solution, &expected, 1.0e-13, backend.label());
                assert!(report.backward_error <= 8.0 * f64::EPSILON);
            }
        }
        Ok(())
    }

    #[test]
    fn leading_zero_indefinite_case_requires_pivoted_lblt() -> Result<(), String> {
        let matrix = vec![0.0, 2.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0, -3.0];
        let expected = vec![1.0, -2.0, 0.5];
        let case = LinearCase::from_solution(3, matrix, &expected)?;
        for &backend in Backend::ALL {
            let report = solve(&case, backend, Factorization::PivotedLblt)?;
            assert_solution_close(&report.solution, &expected, 1.0e-13, backend.label());
            assert!(solve(&case, backend, Factorization::Cholesky).is_err());
        }
        Ok(())
    }

    #[test]
    fn singular_system_is_not_reported_as_success() -> Result<(), String> {
        let case = LinearCase::new(2, vec![1.0, 1.0, 1.0, 1.0], vec![1.0, 0.0])?;
        for &backend in Backend::ALL {
            assert!(
                solve(&case, backend, Factorization::PivotedLblt).is_err(),
                "{} accepted a singular inconsistent system",
                backend.label()
            );
        }
        Ok(())
    }

    #[test]
    fn ill_conditioned_case_has_bounded_explicit_refinement() -> Result<(), String> {
        let (case, expected) = ill_conditioned_case()?;
        for &backend in Backend::ALL {
            for method in [Factorization::Cholesky, Factorization::PivotedLblt] {
                let report = refine(&case, backend, method)?;
                assert!(report.accepted_steps <= MAX_REFINEMENT_STEPS);
                assert!(report.final_residual_inf <= report.initial_residual_inf);
                assert!(report.final_backward_error <= ACCEPTED_BACKWARD_ERROR);
                assert_solution_close(&report.solution, &expected, 1.0e-8, backend.label());
            }
        }
        Ok(())
    }

    #[test]
    fn repeated_runs_are_deterministic() -> Result<(), String> {
        let (case, _) = indefinite_case(24)?;
        for &backend in Backend::ALL {
            let first = refine(&case, backend, Factorization::PivotedLblt)?;
            let second = refine(&case, backend, Factorization::PivotedLblt)?;
            assert_eq!(first, second, "{} report", backend.label());
        }
        Ok(())
    }

    #[test]
    fn invalid_inputs_fail_before_backend_dispatch() {
        assert!(LinearCase::new(0, Vec::new(), Vec::new()).is_err());
        assert!(LinearCase::new(1, vec![f64::NAN], vec![0.0]).is_err());
        assert!(LinearCase::new(2, vec![1.0], vec![0.0, 0.0]).is_err());
        assert!(LinearCase::new(2, vec![1.0, 2.0, 3.0, 4.0], vec![0.0, 0.0]).is_err());
    }
}
