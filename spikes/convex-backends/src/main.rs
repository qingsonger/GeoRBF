use std::hint::black_box;
use std::time::{Duration, Instant};

#[cfg(not(any(feature = "clarabel-backend", feature = "osqp-backend")))]
compile_error!("enable at least one convex backend feature: clarabel-backend or osqp-backend");

#[cfg(feature = "clarabel-backend")]
use clarabel::{
    algebra::CscMatrix as ClarabelCsc,
    solver::{
        DefaultSettings, DefaultSolver, IPSolver, NonnegativeConeT, SecondOrderConeT, SolverStatus,
        ZeroConeT,
    },
};
#[cfg(feature = "osqp-backend")]
use osqp::{CscMatrix as OsqpCsc, Problem as OsqpProblem, Settings as OsqpSettings};

const SOLUTION_TOLERANCE: f64 = 2.0e-7;
const RESIDUAL_TOLERANCE: f64 = 2.0e-7;
#[cfg(test)]
const CERTIFICATE_TOLERANCE: f64 = 2.0e-7;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Backend {
    #[cfg(feature = "clarabel-backend")]
    Clarabel,
    #[cfg(feature = "osqp-backend")]
    Osqp,
}

impl Backend {
    const QP_BACKENDS: &'static [Self] = &[
        #[cfg(feature = "clarabel-backend")]
        Self::Clarabel,
        #[cfg(feature = "osqp-backend")]
        Self::Osqp,
    ];

    const fn label(self) -> &'static str {
        match self {
            #[cfg(feature = "clarabel-backend")]
            Self::Clarabel => "clarabel-0.11.1",
            #[cfg(feature = "osqp-backend")]
            Self::Osqp => "osqp-1.0.1",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct SolveReport {
    solution: Vec<f64>,
    objective: f64,
    equality_residual_inf: f64,
    cone_or_bound_violation: f64,
    iterations: u32,
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq)]
struct CertificateReport {
    stationarity_residual_inf: f64,
    separating_value: f64,
}

fn require_finite(label: &str, values: &[f64]) -> Result<(), String> {
    if values.iter().any(|value| !value.is_finite()) {
        return Err(format!("{label} contains a nonfinite entry"));
    }
    Ok(())
}

#[cfg(test)]
fn infinity_norm(label: &str, values: &[f64]) -> Result<f64, String> {
    require_finite(label, values)?;
    Ok(values.iter().map(|value| value.abs()).fold(0.0, f64::max))
}

fn qp_objective(solution: &[f64], linear: &[f64]) -> Result<f64, String> {
    if solution.len() != linear.len() {
        return Err("QP objective dimensions differ".to_owned());
    }
    require_finite("QP solution", solution)?;
    require_finite("QP linear objective", linear)?;
    let objective = solution
        .iter()
        .zip(linear)
        .map(|(value, coefficient)| 0.5 * value * value + coefficient * value)
        .sum::<f64>();
    if !objective.is_finite() {
        return Err("QP objective is nonfinite".to_owned());
    }
    Ok(objective)
}

#[cfg(test)]
fn qp_review(solution: &[f64], linear: &[f64]) -> Result<SolveReport, String> {
    if solution.len() != 2 {
        return Err("analytic QP solution must have two entries".to_owned());
    }
    let equality_residual = solution[0] + solution[1] - 2.0;
    let violations = [
        (-solution[0]).max(0.0),
        (-solution[1]).max(0.0),
        (solution[0] - 1.2).max(0.0),
        (solution[1] - 2.0).max(0.0),
    ];
    let equality_residual_inf = infinity_norm("QP equality residual", &[equality_residual])?;
    let cone_or_bound_violation = infinity_norm("QP bound violation", &violations)?;
    if equality_residual_inf > RESIDUAL_TOLERANCE || cone_or_bound_violation > RESIDUAL_TOLERANCE {
        return Err(format!(
            "QP failed original-unit review: equality={equality_residual_inf:.17e}, bound={cone_or_bound_violation:.17e}"
        ));
    }
    Ok(SolveReport {
        solution: solution.to_vec(),
        objective: qp_objective(solution, linear)?,
        equality_residual_inf,
        cone_or_bound_violation,
        iterations: 0,
    })
}

#[cfg(feature = "clarabel-backend")]
fn clarabel_settings() -> DefaultSettings<f64> {
    DefaultSettings {
        verbose: false,
        max_iter: 200,
        max_threads: 1,
        tol_gap_abs: 1.0e-9,
        tol_gap_rel: 1.0e-9,
        tol_feas: 1.0e-9,
        tol_infeas_abs: 1.0e-9,
        tol_infeas_rel: 1.0e-9,
        equilibrate_enable: true,
        presolve_enable: false,
        static_regularization_enable: false,
        dynamic_regularization_enable: false,
        iterative_refinement_enable: true,
        ..DefaultSettings::default()
    }
}

#[cfg(feature = "osqp-backend")]
fn osqp_settings() -> OsqpSettings {
    OsqpSettings::default()
        .verbose(false)
        .max_iter(20_000)
        .eps_abs(1.0e-9)
        .eps_rel(1.0e-9)
        .eps_prim_inf(1.0e-9)
        .eps_dual_inf(1.0e-9)
        .adaptive_rho(true)
        .adaptive_rho_interval(Some(25))
        .polishing(false)
        .warm_starting(false)
}

#[cfg(test)]
fn solve_qp_truth(backend: Backend) -> Result<SolveReport, String> {
    let linear = [-1.0, -2.0];
    match backend {
        #[cfg(feature = "clarabel-backend")]
        Backend::Clarabel => {
            let quadratic = ClarabelCsc::new(2, 2, vec![0, 1, 2], vec![0, 1], vec![1.0, 1.0]);
            let constraints =
                ClarabelCsc::from(&[[1.0, 1.0], [-1.0, 0.0], [0.0, -1.0], [1.0, 0.0], [0.0, 1.0]]);
            let rhs = [2.0, 0.0, 0.0, 1.2, 2.0];
            let cones = [ZeroConeT(1), NonnegativeConeT(4)];
            let mut solver = DefaultSolver::new(
                &quadratic,
                &linear,
                &constraints,
                &rhs,
                &cones,
                clarabel_settings(),
            )
            .map_err(|error| format!("Clarabel QP setup failed: {error}"))?;
            solver.solve();
            if solver.solution.status != SolverStatus::Solved {
                return Err(format!("Clarabel QP returned {:?}", solver.solution.status));
            }
            let mut report = qp_review(&solver.solution.x, &linear)?;
            report.iterations = solver.solution.iterations;
            Ok(report)
        }
        #[cfg(feature = "osqp-backend")]
        Backend::Osqp => {
            let quadratic = OsqpCsc::from(&[[1.0, 0.0], [0.0, 1.0]]).into_upper_tri();
            let constraints = OsqpCsc::from(&[[1.0, 1.0], [1.0, 0.0], [0.0, 1.0]]);
            let lower = [2.0, 0.0, 0.0];
            let upper = [2.0, 1.2, 2.0];
            let mut problem = OsqpProblem::new(
                quadratic,
                &linear,
                constraints,
                &lower,
                &upper,
                &osqp_settings(),
            )
            .map_err(|error| format!("OSQP QP setup failed: {error}"))?;
            let status = problem.solve();
            let solution = status
                .solution()
                .ok_or_else(|| format!("OSQP QP returned {status:?}"))?;
            let mut report = qp_review(solution.x(), &linear)?;
            report.iterations = status.iter();
            Ok(report)
        }
    }
}

#[cfg(feature = "clarabel-backend")]
#[cfg(test)]
fn solve_socp_truth() -> Result<SolveReport, String> {
    let quadratic = ClarabelCsc::zeros((3, 3));
    let linear = [1.0, 0.0, 0.0];
    let constraints = ClarabelCsc::from(&[
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [-1.0, 0.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, 0.0, -1.0],
    ]);
    let rhs = [3.0, 4.0, 0.0, 0.0, 0.0];
    let cones = [ZeroConeT(2), SecondOrderConeT(3)];
    let mut solver = DefaultSolver::new(
        &quadratic,
        &linear,
        &constraints,
        &rhs,
        &cones,
        clarabel_settings(),
    )
    .map_err(|error| format!("Clarabel SOCP setup failed: {error}"))?;
    solver.solve();
    if solver.solution.status != SolverStatus::Solved {
        return Err(format!(
            "Clarabel SOCP returned {:?}",
            solver.solution.status
        ));
    }
    let solution = &solver.solution.x;
    require_finite("SOCP solution", solution)?;
    if solution.len() != 3 {
        return Err("analytic SOCP solution must have three entries".to_owned());
    }
    let equality_residual_inf = infinity_norm(
        "SOCP equality residual",
        &[solution[1] - 3.0, solution[2] - 4.0],
    )?;
    let cone_or_bound_violation = solution[1]
        .hypot(solution[2])
        .mul_add(1.0, -solution[0])
        .max(0.0);
    if equality_residual_inf > RESIDUAL_TOLERANCE || cone_or_bound_violation > RESIDUAL_TOLERANCE {
        return Err(format!(
            "SOCP failed original-unit review: equality={equality_residual_inf:.17e}, cone={cone_or_bound_violation:.17e}"
        ));
    }
    Ok(SolveReport {
        solution: solution.clone(),
        objective: solution[0],
        equality_residual_inf,
        cone_or_bound_violation,
        iterations: solver.solution.iterations,
    })
}

#[cfg(feature = "clarabel-backend")]
#[cfg(test)]
fn clarabel_infeasible(linear_only: bool) -> Result<CertificateReport, String> {
    let (dimension, quadratic, linear, constraints, rhs, cones) = if linear_only {
        (
            1,
            ClarabelCsc::new(1, 1, vec![0, 1], vec![0], vec![1.0]),
            vec![0.0],
            ClarabelCsc::from(&[[-1.0], [1.0]]),
            vec![-1.0, 0.0],
            vec![NonnegativeConeT(2)],
        )
    } else {
        (
            2,
            ClarabelCsc::zeros((2, 2)),
            vec![0.0, 0.0],
            ClarabelCsc::from(&[[1.0, 0.0], [0.0, 1.0], [-1.0, 0.0], [0.0, -1.0]]),
            vec![0.0, 1.0, 0.0, 0.0],
            vec![ZeroConeT(2), SecondOrderConeT(2)],
        )
    };
    let mut solver = DefaultSolver::new(
        &quadratic,
        &linear,
        &constraints,
        &rhs,
        &cones,
        clarabel_settings(),
    )
    .map_err(|error| format!("Clarabel infeasible setup failed: {error}"))?;
    solver.solve();
    if solver.solution.status != SolverStatus::PrimalInfeasible {
        return Err(format!(
            "Clarabel infeasible case returned {:?}",
            solver.solution.status
        ));
    }
    let certificate = &solver.solution.z;
    require_finite("Clarabel infeasibility certificate", certificate)?;
    if linear_only {
        if certificate
            .iter()
            .any(|dual| *dual < -CERTIFICATE_TOLERANCE)
        {
            return Err("Clarabel QP certificate leaves the nonnegative dual cone".to_owned());
        }
    } else if certificate.len() != 4
        || certificate[2] + CERTIFICATE_TOLERANCE < certificate[3].abs()
    {
        return Err("Clarabel SOCP certificate leaves the Lorentz dual cone".to_owned());
    }
    let mut stationarity = vec![0.0; dimension];
    let rows: Vec<Vec<f64>> = if linear_only {
        vec![vec![-1.0], vec![1.0]]
    } else {
        vec![
            vec![1.0, 0.0],
            vec![0.0, 1.0],
            vec![-1.0, 0.0],
            vec![0.0, -1.0],
        ]
    };
    for (row, dual) in rows.iter().zip(certificate) {
        for (column, value) in row.iter().enumerate() {
            stationarity[column] = value.mul_add(*dual, stationarity[column]);
        }
    }
    let separating_value = rhs
        .iter()
        .zip(certificate)
        .map(|(value, dual)| value * dual)
        .sum::<f64>();
    let stationarity_residual_inf =
        infinity_norm("Clarabel certificate stationarity", &stationarity)?;
    if stationarity_residual_inf > CERTIFICATE_TOLERANCE || separating_value >= -1.0e-8 {
        return Err(format!(
            "Clarabel certificate failed review: stationarity={stationarity_residual_inf:.17e}, separator={separating_value:.17e}"
        ));
    }
    Ok(CertificateReport {
        stationarity_residual_inf,
        separating_value,
    })
}

#[cfg(feature = "osqp-backend")]
#[cfg(test)]
fn osqp_qp_infeasible() -> Result<CertificateReport, String> {
    let quadratic = OsqpCsc::from(&[[1.0]]).into_upper_tri();
    let linear = [0.0];
    let constraints = OsqpCsc::from(&[[1.0], [1.0]]);
    let lower = [1.0, -1.0];
    let upper = [2.0, 0.0];
    let mut problem = OsqpProblem::new(
        quadratic,
        &linear,
        constraints,
        &lower,
        &upper,
        &osqp_settings(),
    )
    .map_err(|error| format!("OSQP infeasible setup failed: {error}"))?;
    let status = problem.solve();
    let certificate = match &status {
        osqp::Status::PrimalInfeasible(certificate) => certificate.delta_y(),
        _ => return Err(format!("OSQP infeasible QP returned {status:?}")),
    };
    require_finite("OSQP infeasibility certificate", certificate)?;
    if certificate.len() != 2 {
        return Err("OSQP infeasibility certificate has the wrong length".to_owned());
    }
    let stationarity_residual_inf = (certificate[0] + certificate[1]).abs();
    let separating_value = lower
        .iter()
        .zip(&upper)
        .zip(certificate)
        .map(|((lower, upper), dual)| {
            if *dual < 0.0 {
                lower * dual
            } else {
                upper * dual
            }
        })
        .sum::<f64>();
    if stationarity_residual_inf > CERTIFICATE_TOLERANCE || separating_value >= -1.0e-8 {
        return Err(format!(
            "OSQP certificate failed review: stationarity={stationarity_residual_inf:.17e}, separator={separating_value:.17e}"
        ));
    }
    Ok(CertificateReport {
        stationarity_residual_inf,
        separating_value,
    })
}

fn diagonal_qp_case(size: usize, backend: Backend) -> Result<SolveReport, String> {
    if size == 0 {
        return Err("benchmark QP size must be nonzero".to_owned());
    }
    let targets = (0..size)
        .map(|index| {
            let numerator = u32::try_from(index + 1).map_or(f64::INFINITY, f64::from);
            let denominator = u32::try_from(size + 1).map_or(f64::INFINITY, f64::from);
            numerator / denominator
        })
        .collect::<Vec<_>>();
    require_finite("benchmark targets", &targets)?;
    match backend {
        #[cfg(feature = "clarabel-backend")]
        Backend::Clarabel => {
            let quadratic = ClarabelCsc::new(
                size,
                size,
                (0..=size).collect(),
                (0..size).collect(),
                vec![1.0; size],
            );
            let mut column_pointers = Vec::with_capacity(size + 1);
            let mut row_indices = Vec::with_capacity(2 * size);
            let mut values = Vec::with_capacity(2 * size);
            column_pointers.push(0);
            for column in 0..size {
                row_indices.extend([column, size + column]);
                values.extend([-1.0, 1.0]);
                column_pointers.push(row_indices.len());
            }
            let constraints =
                ClarabelCsc::new(2 * size, size, column_pointers, row_indices, values);
            let mut rhs = vec![0.0; size];
            rhs.extend(vec![1.0; size]);
            let cones = [NonnegativeConeT(2 * size)];
            let linear = targets.iter().map(|value| -value).collect::<Vec<_>>();
            let mut solver = DefaultSolver::new(
                &quadratic,
                &linear,
                &constraints,
                &rhs,
                &cones,
                clarabel_settings(),
            )
            .map_err(|error| format!("Clarabel benchmark QP setup failed: {error}"))?;
            solver.solve();
            if solver.solution.status != SolverStatus::Solved {
                return Err(format!(
                    "Clarabel benchmark QP returned {:?}",
                    solver.solution.status
                ));
            }
            benchmark_qp_review(&solver.solution.x, &targets, solver.solution.iterations)
        }
        #[cfg(feature = "osqp-backend")]
        Backend::Osqp => {
            let identity = || {
                (0..size.saturating_mul(size)).map(|index| {
                    if index / size == index % size {
                        1.0
                    } else {
                        0.0
                    }
                })
            };
            let quadratic = OsqpCsc::from_row_iter(size, size, identity());
            let constraints = OsqpCsc::from_row_iter(size, size, identity());
            let lower = vec![0.0; size];
            let upper = vec![1.0; size];
            let linear = targets.iter().map(|value| -value).collect::<Vec<_>>();
            let mut problem = OsqpProblem::new(
                quadratic,
                &linear,
                constraints,
                &lower,
                &upper,
                &osqp_settings(),
            )
            .map_err(|error| format!("OSQP benchmark QP setup failed: {error}"))?;
            let status = problem.solve();
            let solution = status
                .solution()
                .ok_or_else(|| format!("OSQP benchmark QP returned {status:?}"))?;
            benchmark_qp_review(solution.x(), &targets, status.iter())
        }
    }
}

fn benchmark_qp_review(
    solution: &[f64],
    targets: &[f64],
    iterations: u32,
) -> Result<SolveReport, String> {
    if solution.len() != targets.len() {
        return Err("benchmark QP solution length differs from truth".to_owned());
    }
    require_finite("benchmark QP solution", solution)?;
    let truth_error = solution
        .iter()
        .zip(targets)
        .map(|(actual, expected)| (actual - expected).abs())
        .fold(0.0, f64::max);
    let bound_violation = solution
        .iter()
        .map(|value| (-value).max(0.0).max((value - 1.0).max(0.0)))
        .fold(0.0, f64::max);
    if truth_error > SOLUTION_TOLERANCE || bound_violation > RESIDUAL_TOLERANCE {
        return Err(format!(
            "benchmark QP failed review: truth={truth_error:.17e}, bound={bound_violation:.17e}"
        ));
    }
    let linear = targets.iter().map(|value| -value).collect::<Vec<_>>();
    Ok(SolveReport {
        solution: solution.to_vec(),
        objective: qp_objective(solution, &linear)?,
        equality_residual_inf: truth_error,
        cone_or_bound_violation: bound_violation,
        iterations,
    })
}

#[cfg(feature = "clarabel-backend")]
fn socp_scaling_case(size: usize) -> Result<SolveReport, String> {
    if size == 0 {
        return Err("benchmark SOCP size must be nonzero".to_owned());
    }
    let variable_count = size + 1;
    let row_count = size + variable_count;
    let quadratic = ClarabelCsc::zeros((variable_count, variable_count));
    let mut linear = vec![0.0; variable_count];
    linear[0] = 1.0;
    let components = (0..size)
        .map(|index| {
            let value = u32::try_from(index + 1).map_or(f64::INFINITY, f64::from);
            value / 32.0
        })
        .collect::<Vec<_>>();
    require_finite("benchmark SOCP components", &components)?;
    let expected_norm = components
        .iter()
        .map(|value| value * value)
        .sum::<f64>()
        .sqrt();
    let mut column_pointers = Vec::with_capacity(variable_count + 1);
    let mut row_indices = Vec::with_capacity(2 * size + 1);
    let mut values = Vec::with_capacity(2 * size + 1);
    column_pointers.push(0);
    row_indices.push(size);
    values.push(-1.0);
    column_pointers.push(1);
    for component in 0..size {
        row_indices.extend([component, size + 1 + component]);
        values.extend([1.0, -1.0]);
        column_pointers.push(row_indices.len());
    }
    let constraints = ClarabelCsc::new(
        row_count,
        variable_count,
        column_pointers,
        row_indices,
        values,
    );
    let mut rhs = components.clone();
    rhs.extend(vec![0.0; variable_count]);
    let cones = [ZeroConeT(size), SecondOrderConeT(variable_count)];
    let mut solver = DefaultSolver::new(
        &quadratic,
        &linear,
        &constraints,
        &rhs,
        &cones,
        clarabel_settings(),
    )
    .map_err(|error| format!("Clarabel benchmark SOCP setup failed: {error}"))?;
    solver.solve();
    if solver.solution.status != SolverStatus::Solved {
        return Err(format!(
            "Clarabel benchmark SOCP returned {:?}",
            solver.solution.status
        ));
    }
    let solution = &solver.solution.x;
    require_finite("benchmark SOCP solution", solution)?;
    let equality_residual_inf = solution[1..]
        .iter()
        .zip(&components)
        .map(|(actual, expected)| (actual - expected).abs())
        .fold(0.0, f64::max);
    let actual_norm = solution[1..]
        .iter()
        .map(|value| value * value)
        .sum::<f64>()
        .sqrt();
    let cone_violation = (actual_norm - solution[0]).max(0.0);
    if (solution[0] - expected_norm).abs() > SOLUTION_TOLERANCE
        || equality_residual_inf > RESIDUAL_TOLERANCE
        || cone_violation > RESIDUAL_TOLERANCE
    {
        return Err(format!(
            "benchmark SOCP failed review: truth={:.17e}, equality={equality_residual_inf:.17e}, cone={cone_violation:.17e}",
            (solution[0] - expected_norm).abs()
        ));
    }
    Ok(SolveReport {
        solution: solution.clone(),
        objective: solution[0],
        equality_residual_inf,
        cone_or_bound_violation: cone_violation,
        iterations: solver.solution.iterations,
    })
}

fn time_qp_backend(
    size: usize,
    backend: Backend,
    iterations: usize,
) -> Result<(Duration, f64), String> {
    let started = Instant::now();
    let mut checksum = 0.0;
    for _ in 0..iterations {
        let report = black_box(diagonal_qp_case(black_box(size), backend))?;
        checksum += report.objective + report.solution.iter().sum::<f64>();
    }
    Ok((started.elapsed(), black_box(checksum)))
}

#[cfg(feature = "clarabel-backend")]
fn time_socp(size: usize, iterations: usize) -> Result<(Duration, f64), String> {
    let started = Instant::now();
    let mut checksum = 0.0;
    for _ in 0..iterations {
        let report = black_box(socp_scaling_case(black_box(size)))?;
        checksum += report.objective + report.solution.iter().sum::<f64>();
    }
    Ok((started.elapsed(), black_box(checksum)))
}

fn run_benchmark(smoke: bool) -> Result<(), String> {
    let sizes: &[usize] = if smoke { &[8, 16] } else { &[16, 32, 64] };
    let iterations = if smoke { 1 } else { 3 };
    println!("backend,problem,size,iterations,elapsed_ns,checksum");
    for &size in sizes {
        for &backend in Backend::QP_BACKENDS {
            let (elapsed, checksum) = time_qp_backend(size, backend, iterations)?;
            println!(
                "{},qp,{size},{iterations},{},{checksum:.17e}",
                backend.label(),
                elapsed.as_nanos()
            );
        }
        #[cfg(feature = "clarabel-backend")]
        {
            let (elapsed, checksum) = time_socp(size, iterations)?;
            println!(
                "clarabel-0.11.1,socp,{size},{iterations},{},{checksum:.17e}",
                elapsed.as_nanos()
            );
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
mod convex_spike_cases {
    use super::*;

    fn assert_solution_close(actual: &[f64], expected: &[f64], label: &str) {
        assert_eq!(actual.len(), expected.len(), "{label} solution length");
        for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
            assert!(
                (actual - expected).abs() <= SOLUTION_TOLERANCE,
                "{label} solution[{index}] expected {expected:.17e}, got {actual:.17e}"
            );
        }
    }

    #[test]
    fn analytic_qp_truth_agrees() -> Result<(), String> {
        for &backend in Backend::QP_BACKENDS {
            let report = solve_qp_truth(backend)?;
            assert_solution_close(&report.solution, &[0.5, 1.5], backend.label());
            assert!((report.objective + 2.25).abs() <= SOLUTION_TOLERANCE);
            assert!(report.equality_residual_inf <= RESIDUAL_TOLERANCE);
            assert!(report.cone_or_bound_violation <= RESIDUAL_TOLERANCE);
        }
        Ok(())
    }

    #[cfg(feature = "clarabel-backend")]
    #[test]
    fn analytic_socp_truth_maps_to_lorentz_cone() -> Result<(), String> {
        let report = solve_socp_truth()?;
        assert_solution_close(&report.solution, &[5.0, 3.0, 4.0], "Clarabel SOCP");
        assert!((report.objective - 5.0).abs() <= SOLUTION_TOLERANCE);
        assert!(report.equality_residual_inf <= RESIDUAL_TOLERANCE);
        assert!(report.cone_or_bound_violation <= RESIDUAL_TOLERANCE);
        Ok(())
    }

    #[cfg(feature = "clarabel-backend")]
    #[test]
    fn clarabel_qp_and_socp_infeasibility_certificates_are_reviewed() -> Result<(), String> {
        let qp = clarabel_infeasible(true)?;
        let socp = clarabel_infeasible(false)?;
        assert!(qp.stationarity_residual_inf <= CERTIFICATE_TOLERANCE);
        assert!(socp.stationarity_residual_inf <= CERTIFICATE_TOLERANCE);
        assert!(qp.separating_value < 0.0);
        assert!(socp.separating_value < 0.0);
        Ok(())
    }

    #[cfg(feature = "osqp-backend")]
    #[test]
    fn osqp_qp_infeasibility_certificate_is_reviewed() -> Result<(), String> {
        let report = osqp_qp_infeasible()?;
        assert!(report.stationarity_residual_inf <= CERTIFICATE_TOLERANCE);
        assert!(report.separating_value < 0.0);
        Ok(())
    }

    #[test]
    fn repeated_qp_runs_are_deterministic() -> Result<(), String> {
        for &backend in Backend::QP_BACKENDS {
            let first = solve_qp_truth(backend)?;
            let second = solve_qp_truth(backend)?;
            assert_eq!(first, second, "{} report", backend.label());
        }
        Ok(())
    }

    #[cfg(feature = "clarabel-backend")]
    #[test]
    fn repeated_socp_runs_are_deterministic() -> Result<(), String> {
        assert_eq!(solve_socp_truth()?, solve_socp_truth()?);
        Ok(())
    }

    #[test]
    fn invalid_nonfinite_data_is_rejected_before_dispatch() {
        assert!(require_finite("matrix", &[f64::NAN]).is_err());
        assert!(require_finite("rhs", &[f64::INFINITY]).is_err());
        assert!(qp_objective(&[0.0], &[0.0, 1.0]).is_err());
        assert!(diagonal_qp_case(0, Backend::QP_BACKENDS[0]).is_err());
    }
}
