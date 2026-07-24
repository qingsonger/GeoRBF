//! Independent truth and error-path tests for deterministic parameter tuning.

use std::error::Error;
use std::num::NonZeroUsize;

use georbf::{
    CrossValidationEvidence, GeneralizedCrossValidationEvidence, NoopTuningEvaluator, Point,
    PowerFunctionEvidence, TuningBounds, TuningCriterion, TuningError, TuningEvaluationFailure,
    TuningEvaluator, TuningFold, TuningParameter, TuningParameters, TuningProblem,
    TuningScoreEvidence, TuningStrategy,
};

type TestResult = Result<(), Box<dyn Error>>;

fn length_candidate(length: f64) -> Result<TuningParameters, TuningError> {
    TuningParameters::try_new(Some(length), None, None, None, None)
}

fn length_problem<const D: usize>(
    locations: Vec<Point<D>>,
    lengths: &[f64],
) -> Result<TuningProblem<D>, Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    let candidates = lengths
        .iter()
        .copied()
        .map(length_candidate)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TuningProblem::try_new(
        locations,
        candidates,
        TuningBounds::try_new(Some((0.25, 8.0)), None, None, None, None)?,
    )?)
}

#[derive(Clone, Copy)]
enum AnalyticMode {
    CrossValidation { optimum: f64 },
    GeneralizedCrossValidation { optimum: f64 },
    Power { optimum: f64 },
    Constant,
    InvalidCrossValidation,
    InvalidGeneralizedCrossValidation,
    InvalidPower,
    Failure,
}

struct AnalyticEvaluator {
    mode: AnalyticMode,
    calls: usize,
}

impl AnalyticEvaluator {
    const fn new(mode: AnalyticMode) -> Self {
        Self { mode, calls: 0 }
    }
}

impl TuningEvaluator for AnalyticEvaluator {
    fn cross_validation(
        &mut self,
        candidate: &TuningParameters,
        fold: &TuningFold,
    ) -> Result<CrossValidationEvidence, TuningEvaluationFailure> {
        self.calls += 1;
        let length = candidate
            .length()
            .ok_or(TuningEvaluationFailure::new("missing length"))?;
        match self.mode {
            AnalyticMode::CrossValidation { optimum } => {
                let weight = f64::from(
                    u32::try_from(fold.validation_indices().len())
                        .map_err(|_| TuningEvaluationFailure::new("fold too large"))?,
                );
                Ok(CrossValidationEvidence {
                    weighted_squared_error: (length - optimum).powi(2) * weight,
                    weight,
                })
            }
            AnalyticMode::Constant => Ok(CrossValidationEvidence {
                weighted_squared_error: 1.0,
                weight: 1.0,
            }),
            AnalyticMode::InvalidCrossValidation => Ok(CrossValidationEvidence {
                weighted_squared_error: -1.0,
                weight: 0.0,
            }),
            AnalyticMode::Failure => Err(TuningEvaluationFailure::new("analytic failure")),
            _ => Err(TuningEvaluationFailure::new("wrong analytic mode")),
        }
    }

    fn generalized_cross_validation(
        &mut self,
        candidate: &TuningParameters,
    ) -> Result<GeneralizedCrossValidationEvidence, TuningEvaluationFailure> {
        self.calls += 1;
        let length = candidate
            .length()
            .ok_or(TuningEvaluationFailure::new("missing length"))?;
        match self.mode {
            AnalyticMode::GeneralizedCrossValidation { optimum } => {
                Ok(GeneralizedCrossValidationEvidence {
                    residual_sum_squares: (length - optimum).powi(2),
                    observation_count: 4,
                    effective_degrees_of_freedom: 2.0,
                })
            }
            AnalyticMode::InvalidGeneralizedCrossValidation => {
                Ok(GeneralizedCrossValidationEvidence {
                    residual_sum_squares: 1.0,
                    observation_count: 2,
                    effective_degrees_of_freedom: 2.0,
                })
            }
            AnalyticMode::Failure => Err(TuningEvaluationFailure::new("analytic failure")),
            _ => Err(TuningEvaluationFailure::new("wrong analytic mode")),
        }
    }

    fn power_function(
        &mut self,
        candidate: &TuningParameters,
    ) -> Result<PowerFunctionEvidence, TuningEvaluationFailure> {
        self.calls += 1;
        let length = candidate
            .length()
            .ok_or(TuningEvaluationFailure::new("missing length"))?;
        match self.mode {
            AnalyticMode::Power { optimum } => Ok(PowerFunctionEvidence {
                maximum_squared_power: (length - optimum).powi(2),
                sample_count: 8,
            }),
            AnalyticMode::InvalidPower => Ok(PowerFunctionEvidence {
                maximum_squared_power: f64::NAN,
                sample_count: 0,
            }),
            AnalyticMode::Failure => Err(TuningEvaluationFailure::new("analytic failure")),
            _ => Err(TuningEvaluationFailure::new("wrong analytic mode")),
        }
    }
}

#[test]
fn fixed_returns_exact_candidate_without_evaluation() -> TestResult {
    let problem = length_problem(
        vec![Point::try_new([0.0])?, Point::try_new([1.0])?],
        &[1.0, 2.0, 4.0],
    )?;
    let result = problem.try_tune(
        TuningStrategy::Fixed { candidate_index: 2 },
        &mut NoopTuningEvaluator,
    )?;
    assert_eq!(result.selected_index(), 2);
    assert_eq!(result.parameters().length(), Some(4.0));
    assert_eq!(result.diagnostics().criterion, TuningCriterion::Fixed);
    assert_eq!(result.diagnostics().seed, None);
    assert!(matches!(
        result.diagnostics().candidates[2].evidence,
        TuningScoreEvidence::Fixed
    ));
    assert!(matches!(
        result.diagnostics().candidates[0].evidence,
        TuningScoreEvidence::NotEvaluated
    ));
    Ok(())
}

#[test]
fn distance_heuristic_matches_median_nearest_neighbor_truth() -> TestResult {
    let problem = length_problem(
        vec![
            Point::try_new([0.0])?,
            Point::try_new([1.0])?,
            Point::try_new([4.0])?,
        ],
        &[0.5, 1.0, 2.0],
    )?;
    let result = problem.try_tune(
        TuningStrategy::DistanceHeuristic { seed: 19 },
        &mut NoopTuningEvaluator,
    )?;
    assert_eq!(result.selected_index(), 1);
    assert_eq!(result.parameters().length(), Some(1.0));
    assert_eq!(result.diagnostics().candidates[1].score, Some(0.0));
    assert!(matches!(
        result.diagnostics().candidates[1].evidence,
        TuningScoreEvidence::DistanceHeuristic {
            median_nearest_neighbor_distance: 1.0,
            compared_parameter_count: 1,
        }
    ));
    Ok(())
}

#[test]
fn cross_validation_finds_known_optimum_and_records_folds() -> TestResult {
    let problem = length_problem(
        (0_u32..7)
            .map(|value| Point::try_new([f64::from(value)]))
            .collect::<Result<Vec<_>, _>>()?,
        &[0.5, 1.0, 2.0, 4.0],
    )?;
    let mut evaluator = AnalyticEvaluator::new(AnalyticMode::CrossValidation { optimum: 2.0 });
    let strategy = TuningStrategy::CrossValidation {
        folds: NonZeroUsize::new(3).ok_or("fold count")?,
        seed: 0x5eed,
    };
    let result = problem.try_tune(strategy, &mut evaluator)?;
    assert_eq!(result.parameters().length(), Some(2.0));
    assert_eq!(evaluator.calls, 12);
    assert_eq!(result.diagnostics().folds.len(), 3);
    let mut held_out = result
        .diagnostics()
        .folds
        .iter()
        .flat_map(|fold| fold.validation_indices().iter().copied())
        .collect::<Vec<_>>();
    held_out.sort_unstable();
    assert_eq!(held_out, (0..7).collect::<Vec<_>>());
    assert!(
        result
            .diagnostics()
            .folds
            .iter()
            .all(|fold| !fold.validation_indices().is_empty())
    );
    assert!(matches!(
        &result.diagnostics().candidates[2].evidence,
        TuningScoreEvidence::CrossValidation { fold_losses }
            if fold_losses == &[0.0, 0.0, 0.0]
    ));
    Ok(())
}

#[test]
fn seeded_ties_are_repeatable_and_seed_controls_selection() -> TestResult {
    let problem = length_problem(
        vec![Point::try_new([0.0])?, Point::try_new([1.0])?],
        &[1.0, 2.0, 3.0],
    )?;
    let folds = NonZeroUsize::new(2).ok_or("fold count")?;
    let select = |seed| -> Result<usize, TuningError> {
        let mut evaluator = AnalyticEvaluator::new(AnalyticMode::Constant);
        problem
            .try_tune(
                TuningStrategy::CrossValidation { folds, seed },
                &mut evaluator,
            )
            .map(|result| result.selected_index())
    };
    assert_eq!(select(7)?, select(7)?);
    let mut selected = (0_u64..64).map(select).collect::<Result<Vec<_>, _>>()?;
    selected.sort_unstable();
    selected.dedup();
    assert!(selected.len() > 1);
    Ok(())
}

#[test]
fn generalized_cross_validation_uses_documented_formula() -> TestResult {
    let problem = length_problem(
        vec![
            Point::try_new([0.0])?,
            Point::try_new([1.0])?,
            Point::try_new([2.0])?,
            Point::try_new([3.0])?,
        ],
        &[1.0, 2.0, 4.0],
    )?;
    let mut evaluator =
        AnalyticEvaluator::new(AnalyticMode::GeneralizedCrossValidation { optimum: 2.0 });
    let result = problem.try_tune(
        TuningStrategy::GeneralizedCrossValidation { seed: 11 },
        &mut evaluator,
    )?;
    assert_eq!(result.parameters().length(), Some(2.0));
    assert_eq!(result.diagnostics().candidates[0].score, Some(0.25));
    assert_eq!(result.diagnostics().candidates[1].score, Some(0.0));
    Ok(())
}

#[test]
fn power_function_finds_known_worst_case_optimum() -> TestResult {
    let problem = length_problem(
        vec![Point::try_new([0.0])?, Point::try_new([1.0])?],
        &[1.0, 2.0, 4.0],
    )?;
    let mut evaluator = AnalyticEvaluator::new(AnalyticMode::Power { optimum: 4.0 });
    let result = problem.try_tune(TuningStrategy::PowerFunction { seed: 23 }, &mut evaluator)?;
    assert_eq!(result.parameters().length(), Some(4.0));
    assert_eq!(result.diagnostics().candidates[2].score, Some(0.0));
    Ok(())
}

#[test]
fn every_supported_parameter_is_bounded_and_retained() -> TestResult {
    let bounds = TuningBounds::try_new(
        Some((1.0, 2.0)),
        Some((3.0, 4.0)),
        Some((0.0, 0.25)),
        Some((1.0, 5.0)),
        Some((6.0, 8.0)),
    )?;
    let candidate =
        TuningParameters::try_new(Some(1.5), Some(3.5), Some(0.1), Some(2.0), Some(7.0))?;
    let problem =
        TuningProblem::try_new(vec![Point::try_new([0.0, 0.0])?], vec![candidate], bounds)?;
    let result = problem.try_tune(
        TuningStrategy::Fixed { candidate_index: 0 },
        &mut NoopTuningEvaluator,
    )?;
    assert_eq!(result.parameters().length(), Some(1.5));
    assert_eq!(result.parameters().support_radius(), Some(3.5));
    assert_eq!(result.parameters().regularization(), Some(0.1));
    assert_eq!(result.parameters().axis_ratio(), Some(2.0));
    assert_eq!(result.parameters().influence_radius(), Some(7.0));
    assert_eq!(
        result
            .diagnostics()
            .bounds
            .range(TuningParameter::Regularization)
            .map(|range| (range.minimum(), range.maximum())),
        Some((0.0, 0.25))
    );
    Ok(())
}

#[test]
fn malformed_bounds_candidates_and_strategy_are_structured() -> TestResult {
    assert!(matches!(
        TuningBounds::try_new(None, None, None, None, None),
        Err(TuningError::NoActiveParameters)
    ));
    assert!(matches!(
        TuningBounds::try_new(Some((2.0, 1.0)), None, None, None, None),
        Err(TuningError::ReversedBound { .. })
    ));
    assert!(matches!(
        TuningParameters::try_new(None, None, Some(-1.0), None, None),
        Err(TuningError::InvalidParameterDomain {
            parameter: TuningParameter::Regularization,
            ..
        })
    ));
    let bounds = TuningBounds::try_new(Some((1.0, 2.0)), None, None, None, None)?;
    let out_of_bounds = length_candidate(4.0)?;
    assert!(matches!(
        TuningProblem::try_new(
            vec![Point::try_new([0.0])?],
            vec![out_of_bounds],
            bounds.clone(),
        ),
        Err(TuningError::CandidateOutOfBounds { .. })
    ));
    let duplicate = length_candidate(1.0)?;
    assert!(matches!(
        TuningProblem::try_new(
            vec![Point::try_new([0.0])?],
            vec![duplicate.clone(), duplicate],
            bounds,
        ),
        Err(TuningError::DuplicateCandidate { .. })
    ));
    let signed_zero_bounds =
        TuningBounds::try_new(None, None, Some((-0.0, 1.0)), None, None)?;
    assert!(matches!(
        TuningProblem::try_new(
            vec![Point::try_new([0.0])?],
            vec![
                TuningParameters::try_new(None, None, Some(-0.0), None, None)?,
                TuningParameters::try_new(None, None, Some(0.0), None, None)?,
            ],
            signed_zero_bounds,
        ),
        Err(TuningError::DuplicateCandidate { .. })
    ));
    let problem = length_problem(vec![Point::try_new([0.0])?], &[1.0])?;
    assert!(matches!(
        problem.try_tune(
            TuningStrategy::Fixed { candidate_index: 1 },
            &mut NoopTuningEvaluator,
        ),
        Err(TuningError::FixedCandidateOutOfBounds { .. })
    ));
    Ok(())
}

#[test]
fn distance_and_fold_preconditions_fail_explicitly() -> TestResult {
    let single = length_problem(vec![Point::try_new([0.0])?], &[1.0, 2.0])?;
    assert!(matches!(
        single.try_tune(
            TuningStrategy::DistanceHeuristic { seed: 0 },
            &mut NoopTuningEvaluator,
        ),
        Err(TuningError::InsufficientLocationsForDistanceHeuristic { .. })
    ));
    let duplicate_locations = length_problem(
        vec![Point::try_new([1.0])?, Point::try_new([1.0])?],
        &[1.0, 2.0],
    )?;
    assert!(matches!(
        duplicate_locations.try_tune(
            TuningStrategy::DistanceHeuristic { seed: 0 },
            &mut NoopTuningEvaluator,
        ),
        Err(TuningError::InvalidNearestNeighborDistance { .. })
    ));
    assert!(matches!(
        single.try_tune(
            TuningStrategy::CrossValidation {
                folds: NonZeroUsize::new(2).ok_or("fold count")?,
                seed: 0,
            },
            &mut AnalyticEvaluator::new(AnalyticMode::Constant),
        ),
        Err(TuningError::FoldCountExceedsObservations { .. })
    ));
    Ok(())
}

#[test]
fn invalid_evidence_and_evaluator_failure_reject_the_search() -> TestResult {
    let problem = length_problem(
        vec![Point::try_new([0.0])?, Point::try_new([1.0])?],
        &[1.0, 2.0],
    )?;
    let folds = NonZeroUsize::new(2).ok_or("fold count")?;
    let mut invalid_cv = AnalyticEvaluator::new(AnalyticMode::InvalidCrossValidation);
    assert!(matches!(
        problem.try_tune(
            TuningStrategy::CrossValidation { folds, seed: 0 },
            &mut invalid_cv,
        ),
        Err(TuningError::InvalidCrossValidationEvidence { .. })
    ));
    let mut invalid_generalized =
        AnalyticEvaluator::new(AnalyticMode::InvalidGeneralizedCrossValidation);
    assert!(matches!(
        problem.try_tune(
            TuningStrategy::GeneralizedCrossValidation { seed: 0 },
            &mut invalid_generalized,
        ),
        Err(TuningError::InvalidGeneralizedCrossValidationEvidence { .. })
    ));
    let mut invalid_power = AnalyticEvaluator::new(AnalyticMode::InvalidPower);
    assert!(matches!(
        problem.try_tune(
            TuningStrategy::PowerFunction { seed: 0 },
            &mut invalid_power,
        ),
        Err(TuningError::InvalidPowerFunctionEvidence { .. })
    ));
    let mut failed = AnalyticEvaluator::new(AnalyticMode::Failure);
    assert!(matches!(
        problem.try_tune(
            TuningStrategy::CrossValidation { folds, seed: 0 },
            &mut failed,
        ),
        Err(TuningError::EvaluationFailed {
            candidate: 0,
            fold: Some(0),
            ..
        })
    ));
    assert_eq!(failed.calls, 1);
    Ok(())
}

#[test]
fn dimensions_one_two_and_three_share_the_same_search() -> TestResult {
    let one = length_problem(
        vec![Point::try_new([0.0])?, Point::try_new([1.0])?],
        &[1.0, 2.0],
    )?;
    let two = length_problem(
        vec![Point::try_new([0.0, 0.0])?, Point::try_new([1.0, 0.0])?],
        &[1.0, 2.0],
    )?;
    let three = length_problem(
        vec![
            Point::try_new([0.0, 0.0, 0.0])?,
            Point::try_new([1.0, 0.0, 0.0])?,
        ],
        &[1.0, 2.0],
    )?;
    for selected in [
        one.try_tune(
            TuningStrategy::DistanceHeuristic { seed: 5 },
            &mut NoopTuningEvaluator,
        )?
        .selected_index(),
        two.try_tune(
            TuningStrategy::DistanceHeuristic { seed: 5 },
            &mut NoopTuningEvaluator,
        )?
        .selected_index(),
        three
            .try_tune(
                TuningStrategy::DistanceHeuristic { seed: 5 },
                &mut NoopTuningEvaluator,
            )?
            .selected_index(),
    ] {
        assert_eq!(selected, 0);
    }
    Ok(())
}
