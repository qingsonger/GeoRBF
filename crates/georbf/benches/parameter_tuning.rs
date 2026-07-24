//! Deterministic production parameter-tuning benchmark.

use std::error::Error;
use std::hint::black_box;
use std::num::NonZeroUsize;
use std::time::Instant;

use georbf::{
    CrossValidationEvidence, GeneralizedCrossValidationEvidence, Point, PowerFunctionEvidence,
    TuningBounds, TuningEvaluationFailure, TuningEvaluator, TuningFold, TuningParameters,
    TuningProblem, TuningStrategy,
};

struct BenchmarkEvaluator;

impl TuningEvaluator for BenchmarkEvaluator {
    fn cross_validation(
        &mut self,
        candidate: &TuningParameters,
        fold: &TuningFold,
    ) -> Result<CrossValidationEvidence, TuningEvaluationFailure> {
        let length = candidate
            .length()
            .ok_or(TuningEvaluationFailure::new("missing length"))?;
        let weight = f64::from(
            u32::try_from(fold.validation_indices().len())
                .map_err(|_| TuningEvaluationFailure::new("fold too large"))?,
        );
        Ok(CrossValidationEvidence {
            weighted_squared_error: (length - 2.0).powi(2) * weight,
            weight,
        })
    }

    fn generalized_cross_validation(
        &mut self,
        candidate: &TuningParameters,
    ) -> Result<GeneralizedCrossValidationEvidence, TuningEvaluationFailure> {
        let length = candidate
            .length()
            .ok_or(TuningEvaluationFailure::new("missing length"))?;
        Ok(GeneralizedCrossValidationEvidence {
            residual_sum_squares: (length - 2.0).powi(2),
            observation_count: 64,
            effective_degrees_of_freedom: 12.0,
        })
    }

    fn power_function(
        &mut self,
        candidate: &TuningParameters,
    ) -> Result<PowerFunctionEvidence, TuningEvaluationFailure> {
        let length = candidate
            .length()
            .ok_or(TuningEvaluationFailure::new("missing length"))?;
        Ok(PowerFunctionEvidence {
            maximum_squared_power: (length - 2.0).powi(2),
            sample_count: 128,
        })
    }
}

fn fixture(candidate_count: u32) -> Result<TuningProblem<2>, Box<dyn Error>> {
    let locations = (0_u32..64)
        .map(|index| {
            let x = f64::from(index % 8);
            let y = f64::from(index / 8);
            Point::try_new([x, y])
        })
        .collect::<Result<Vec<_>, _>>()?;
    let candidates = (0..candidate_count)
        .map(|index| {
            let fraction = f64::from(index) / f64::from(candidate_count - 1);
            TuningParameters::try_new(
                Some(0.5 + 3.5 * fraction),
                Some(1.0 + 7.0 * fraction),
                Some(1.0e-12 * f64::from(index)),
                Some(1.0 + 3.0 * fraction),
                Some(0.75 + 5.25 * fraction),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TuningProblem::try_new(
        locations,
        candidates,
        TuningBounds::try_new(
            Some((0.5, 4.0)),
            Some((1.0, 8.0)),
            Some((0.0, 1.0e-9)),
            Some((1.0, 4.0)),
            Some((0.75, 6.0)),
        )?,
    )?)
}

fn run(
    problem: &TuningProblem<2>,
    strategy: TuningStrategy,
    iterations: u32,
) -> Result<(f64, f64), Box<dyn Error>> {
    let mut checksum = 0.0;
    let started = Instant::now();
    for _ in 0..iterations {
        let result = black_box(problem).try_tune(black_box(strategy), &mut BenchmarkEvaluator)?;
        checksum += f64::from(u32::try_from(result.selected_index())?);
        checksum += result
            .diagnostics()
            .candidates
            .iter()
            .filter_map(|candidate| candidate.score)
            .sum::<f64>();
    }
    let nanoseconds = started.elapsed().as_secs_f64() * 1.0e9 / f64::from(iterations);
    Ok((nanoseconds, checksum))
}

fn main() -> Result<(), Box<dyn Error>> {
    let smoke = std::env::args().any(|argument| argument == "--smoke");
    let candidate_count = if smoke { 16 } else { 128 };
    let iterations = if smoke { 1 } else { 100 };
    let problem = fixture(candidate_count)?;
    let strategies = [
        (
            "fixed",
            TuningStrategy::Fixed {
                candidate_index: usize::try_from(candidate_count / 2)?,
            },
        ),
        (
            "distance-heuristic",
            TuningStrategy::DistanceHeuristic { seed: 0x5eed },
        ),
        (
            "cross-validation",
            TuningStrategy::CrossValidation {
                folds: NonZeroUsize::new(5).ok_or("fold count")?,
                seed: 0x5eed,
            },
        ),
        (
            "generalized-cross-validation",
            TuningStrategy::GeneralizedCrossValidation { seed: 0x5eed },
        ),
        (
            "power-function",
            TuningStrategy::PowerFunction { seed: 0x5eed },
        ),
    ];

    println!("strategy,candidates,observations,nanoseconds,checksum");
    for (name, strategy) in strategies {
        let (nanoseconds, checksum) = run(&problem, strategy, iterations)?;
        println!("{name},{candidate_count},64,{nanoseconds:.2},{checksum:.17e}");
    }
    Ok(())
}
