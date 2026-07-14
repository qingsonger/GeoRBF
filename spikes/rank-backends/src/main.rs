use std::hint::black_box;
use std::time::{Duration, Instant};

#[cfg(feature = "nalgebra-backend")]
use nalgebra::{DMatrix, linalg::SVD};

const EQUILIBRATION_PASSES: usize = 8;
#[cfg(feature = "nalgebra-backend")]
const SVD_MAX_ITERATIONS: usize = 10_000;

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

#[derive(Clone, Debug, PartialEq)]
struct RankReport {
    rrqr_rank: usize,
    svd_rank: usize,
    rrqr_threshold: f64,
    svd_threshold: f64,
    r_diagonal: Vec<f64>,
    singular_values: Vec<f64>,
}

#[derive(Clone, Debug)]
struct MatrixCase {
    rows: usize,
    cols: usize,
    values: Vec<f64>,
}

impl MatrixCase {
    fn new(rows: usize, cols: usize, values: Vec<f64>) -> Result<Self, String> {
        let expected = rows
            .checked_mul(cols)
            .ok_or_else(|| "matrix shape overflows usize".to_owned())?;
        if rows == 0 || cols == 0 {
            return Err("matrix must be nonempty".to_owned());
        }
        if values.len() != expected {
            return Err(format!(
                "matrix data length {} does not match {rows}x{cols}",
                values.len()
            ));
        }
        if values.iter().any(|value| !value.is_finite()) {
            return Err("matrix entries must be finite".to_owned());
        }
        Ok(Self { rows, cols, values })
    }

    fn equilibrated(&self) -> Self {
        let mut values = self.values.clone();
        for _ in 0..EQUILIBRATION_PASSES {
            for row in 0..self.rows {
                let row_start = row * self.cols;
                let row_end = row_start + self.cols;
                let scale = values[row_start..row_end]
                    .iter()
                    .map(|value| value.abs())
                    .fold(0.0_f64, f64::max);
                if scale > 0.0 {
                    for value in &mut values[row_start..row_end] {
                        *value /= scale;
                    }
                }
            }
            for col in 0..self.cols {
                let scale = (0..self.rows)
                    .map(|row| values[row * self.cols + col].abs())
                    .fold(0.0_f64, f64::max);
                if scale > 0.0 {
                    for row in 0..self.rows {
                        values[row * self.cols + col] /= scale;
                    }
                }
            }
        }
        Self {
            rows: self.rows,
            cols: self.cols,
            values,
        }
    }

    #[cfg(test)]
    fn diagonally_scaled(&self, row_scales: &[f64], col_scales: &[f64]) -> Result<Self, String> {
        if row_scales.len() != self.rows || col_scales.len() != self.cols {
            return Err("diagonal scale shape mismatch".to_owned());
        }
        if row_scales
            .iter()
            .chain(col_scales)
            .any(|scale| !scale.is_finite() || *scale == 0.0)
        {
            return Err("diagonal scales must be finite and nonzero".to_owned());
        }
        let values = (0..self.rows)
            .flat_map(|row| {
                (0..self.cols).map(move |col| {
                    self.values[row * self.cols + col] * row_scales[row] * col_scales[col]
                })
            })
            .collect();
        Self::new(self.rows, self.cols, values)
    }
}

fn threshold_rank(values: &[f64], dimension: usize) -> (usize, f64) {
    let leading = values.iter().copied().fold(0.0_f64, f64::max);
    let dimension = u32::try_from(dimension).map_or(f64::INFINITY, f64::from);
    let threshold = dimension * f64::EPSILON * leading;
    (
        values.iter().filter(|value| **value > threshold).count(),
        threshold,
    )
}

fn analyze(case: &MatrixCase, backend: Backend) -> Result<RankReport, String> {
    let equilibrated = case.equilibrated();
    match backend {
        #[cfg(feature = "faer-backend")]
        Backend::Faer => analyze_faer(&equilibrated),
        #[cfg(feature = "nalgebra-backend")]
        Backend::Nalgebra => analyze_nalgebra(&equilibrated),
    }
}

#[cfg(feature = "faer-backend")]
fn analyze_faer(case: &MatrixCase) -> Result<RankReport, String> {
    let matrix = faer::Mat::from_fn(case.rows, case.cols, |row, col| {
        case.values[row * case.cols + col]
    });
    let qr = matrix.as_ref().col_piv_qr();
    let size = case.rows.min(case.cols);
    let r_diagonal = (0..size)
        .map(|index| qr.R()[(index, index)].abs())
        .collect::<Vec<_>>();
    let singular_values = matrix
        .as_ref()
        .singular_values()
        .map_err(|error| format!("faer SVD failed: {error:?}"))?;
    let dimension = case.rows.max(case.cols);
    let (rrqr_rank, rrqr_threshold) = threshold_rank(&r_diagonal, dimension);
    let (svd_rank, svd_threshold) = threshold_rank(&singular_values, dimension);
    Ok(RankReport {
        rrqr_rank,
        svd_rank,
        rrqr_threshold,
        svd_threshold,
        r_diagonal,
        singular_values,
    })
}

#[cfg(feature = "nalgebra-backend")]
fn analyze_nalgebra(case: &MatrixCase) -> Result<RankReport, String> {
    let matrix = DMatrix::from_row_slice(case.rows, case.cols, &case.values);
    let qr = matrix.clone().col_piv_qr();
    let r = qr.r();
    let size = case.rows.min(case.cols);
    let r_diagonal = (0..size)
        .map(|index| r[(index, index)].abs())
        .collect::<Vec<_>>();
    let svd = SVD::try_new(matrix, false, false, 5.0 * f64::EPSILON, SVD_MAX_ITERATIONS)
        .ok_or_else(|| {
            format!("nalgebra SVD did not converge within {SVD_MAX_ITERATIONS} iterations")
        })?;
    let singular_values = svd.singular_values.iter().copied().collect::<Vec<_>>();
    let dimension = case.rows.max(case.cols);
    let (rrqr_rank, rrqr_threshold) = threshold_rank(&r_diagonal, dimension);
    let (svd_rank, svd_threshold) = threshold_rank(&singular_values, dimension);
    Ok(RankReport {
        rrqr_rank,
        svd_rank,
        rrqr_threshold,
        svd_threshold,
        r_diagonal,
        singular_values,
    })
}

fn benchmark_case(size: usize) -> Result<MatrixCase, String> {
    let values = (0..size)
        .flat_map(|row| {
            (0..size).map(move |col| {
                let phase = u32::try_from((row + 1) * (col + 3)).map_or(f64::INFINITY, f64::from);
                let diagonal = if row == col { 2.0 } else { 0.0 };
                diagonal + (phase * 0.013).sin() + (phase * 0.007).cos()
            })
        })
        .collect();
    MatrixCase::new(size, size, values)
}

fn time_backend(
    case: &MatrixCase,
    backend: Backend,
    iterations: usize,
) -> Result<(Duration, f64), String> {
    let started = Instant::now();
    let mut checksum = 0.0;
    for _ in 0..iterations {
        let report = black_box(analyze(black_box(case), backend))?;
        checksum += report.r_diagonal.iter().sum::<f64>();
        checksum += report.singular_values.iter().sum::<f64>();
    }
    Ok((started.elapsed(), black_box(checksum)))
}

fn run_benchmark(smoke: bool) -> Result<(), String> {
    let sizes: &[usize] = if smoke { &[16, 32] } else { &[32, 64, 128] };
    let iterations = if smoke { 1 } else { 3 };
    println!("backend,size,iterations,elapsed_ns,checksum");
    for &size in sizes {
        let case = benchmark_case(size)?;
        for &backend in Backend::ALL {
            let (elapsed, checksum) = time_backend(&case, backend, iterations)?;
            println!(
                "{},{size},{iterations},{},{checksum:.17e}",
                backend.label(),
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
mod rank_spike_cases {
    use super::*;

    fn near_dependent(delta: f64) -> Result<MatrixCase, String> {
        MatrixCase::new(
            3,
            3,
            vec![1.0, 0.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 2.0 + delta],
        )
    }

    #[test]
    fn full_rank_truth_case_agrees() -> Result<(), String> {
        let case = MatrixCase::new(
            4,
            3,
            vec![1.0, 0.0, 2.0, 0.0, 1.0, -1.0, 1.0, 1.0, 0.0, 2.0, -1.0, 1.0],
        )?;
        for &backend in Backend::ALL {
            let report = analyze(&case, backend)?;
            assert_eq!(report.rrqr_rank, 3, "{} RRQR", backend.label());
            assert_eq!(report.svd_rank, 3, "{} SVD", backend.label());
        }
        Ok(())
    }

    #[test]
    fn exact_rank_deficiency_is_not_hidden() -> Result<(), String> {
        let case = near_dependent(0.0)?;
        for &backend in Backend::ALL {
            let report = analyze(&case, backend)?;
            assert_eq!(report.rrqr_rank, 2, "{} RRQR", backend.label());
            assert_eq!(report.svd_rank, 2, "{} SVD", backend.label());
        }
        Ok(())
    }

    #[test]
    fn near_threshold_cases_receive_svd_review() -> Result<(), String> {
        let resolved = near_dependent(1.0e-12)?;
        let unresolved = near_dependent(f64::EPSILON / 4.0)?;
        for &backend in Backend::ALL {
            let resolved_report = analyze(&resolved, backend)?;
            let unresolved_report = analyze(&unresolved, backend)?;
            assert_eq!(
                resolved_report.svd_rank,
                3,
                "{} resolved SVD",
                backend.label()
            );
            assert_eq!(
                unresolved_report.svd_rank,
                2,
                "{} unresolved SVD",
                backend.label()
            );
        }
        Ok(())
    }

    #[test]
    fn nonzero_row_and_column_scaling_preserves_classification() -> Result<(), String> {
        let case = near_dependent(2.0_f64.powi(-20))?;
        let scaled = case.diagonally_scaled(
            &[2.0_f64.powi(-20), -2.0_f64.powi(18), 2.0_f64.powi(10)],
            &[2.0_f64.powi(22), -2.0_f64.powi(-16), 2.0_f64.powi(8)],
        )?;
        for &backend in Backend::ALL {
            let baseline = analyze(&case, backend)?;
            let rescaled = analyze(&scaled, backend)?;
            assert_eq!(
                baseline.svd_rank,
                rescaled.svd_rank,
                "{} SVD",
                backend.label()
            );
            assert_eq!(baseline.svd_rank, 3, "{} baseline SVD", backend.label());
        }
        Ok(())
    }

    #[test]
    fn repeated_runs_are_deterministic() -> Result<(), String> {
        let case = benchmark_case(24)?;
        for &backend in Backend::ALL {
            let first = analyze(&case, backend)?;
            let second = analyze(&case, backend)?;
            assert_eq!(first, second, "{} report", backend.label());
        }
        Ok(())
    }

    #[test]
    fn invalid_inputs_fail_before_backend_dispatch() {
        assert!(MatrixCase::new(0, 1, Vec::new()).is_err());
        assert!(MatrixCase::new(1, 1, vec![f64::NAN]).is_err());
        assert!(MatrixCase::new(2, 2, vec![1.0]).is_err());
    }
}
