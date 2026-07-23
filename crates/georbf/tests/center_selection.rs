//! Independent truth and failure-path tests for rank-safe center selection.

use std::error::Error;
use std::num::NonZeroUsize;

use georbf::{
    CenterSelectionError, CenterSelectionKind, CenterSelectionOptions, CenterSelectionProblem,
    CenterSelectionStrategy, CpdOrder, DenseRankDecision, DenseSolveError, KernelDefiniteness,
    Point,
};

type TestResult = Result<(), Box<dyn Error>>;

const MEMORY_LIMIT: usize = 64 * 1024 * 1024;
const SPD: KernelDefiniteness = KernelDefiniteness::StrictlyPositiveDefinite;

fn options(strategy: CenterSelectionStrategy) -> Result<CenterSelectionOptions, Box<dyn Error>> {
    Ok(CenterSelectionOptions::new(
        strategy,
        NonZeroUsize::new(MEMORY_LIMIT).ok_or("test memory limit")?,
    ))
}

fn rejected<T>(
    result: Result<T, CenterSelectionError>,
    message: &'static str,
) -> Result<CenterSelectionError, Box<dyn Error>> {
    match result {
        Ok(_) => Err(message.into()),
        Err(error) => Ok(error),
    }
}

fn diagonal_problem<const D: usize>(
    locations: Vec<Point<D>>,
    diagonal: &[f64],
    targets: Vec<f64>,
) -> Result<CenterSelectionProblem<D>, CenterSelectionError>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    let count = diagonal.len();
    let mut gram = vec![0.0; count * count];
    for index in 0..count {
        gram[index * count + index] = diagonal[index];
    }
    CenterSelectionProblem::try_from_row_major(SPD, locations, gram, targets)
}

fn gaussian_problem() -> Result<CenterSelectionProblem<1>, Box<dyn Error>> {
    let coordinates = [0.0_f64, 1.0, 3.0, 10.0];
    let locations = coordinates
        .iter()
        .map(|&coordinate| Point::try_new([coordinate]))
        .collect::<Result<Vec<_>, _>>()?;
    let mut gram = Vec::with_capacity(coordinates.len() * coordinates.len());
    for &left in &coordinates {
        for &right in &coordinates {
            gram.push((-(left - right).powi(2)).exp());
        }
    }
    Ok(CenterSelectionProblem::try_from_row_major(
        SPD,
        locations,
        gram,
        vec![1.0, -2.0, 0.5, 3.0],
    )?)
}

#[test]
fn all_and_user_strategies_preserve_exact_stable_order() -> TestResult {
    let problem = gaussian_problem()?;
    let all = problem.try_select(&options(CenterSelectionStrategy::AllRepresenters)?)?;
    assert_eq!(all.indices(), &[0, 1, 2, 3]);
    assert_eq!(all.diagnostics().kind, CenterSelectionKind::AllRepresenters);
    assert_eq!(all.diagnostics().rank.decision, DenseRankDecision::FullRank);

    let user = problem.try_select(&options(CenterSelectionStrategy::UserProvided(vec![
        3, 1, 0,
    ]))?)?;
    assert_eq!(user.indices(), &[3, 1, 0]);
    assert_eq!(user.diagnostics().kind, CenterSelectionKind::UserProvided);
    assert_eq!(user.diagnostics().rank.dimension, 3);
    Ok(())
}

#[test]
fn farthest_point_matches_independent_one_dimensional_truth() -> TestResult {
    let problem = gaussian_problem()?;
    let selection = problem.try_select(&options(CenterSelectionStrategy::FarthestPoint {
        count: NonZeroUsize::new(3).ok_or("count")?,
        seed: 0,
    })?)?;
    assert_eq!(selection.indices(), &[0, 3, 2]);
    assert_eq!(
        selection.diagnostics().kind,
        CenterSelectionKind::FarthestPoint
    );
    assert_eq!(selection.diagnostics().seed, Some(0));
    Ok(())
}

#[test]
fn farthest_point_seeded_exact_tie_is_repeatable() -> TestResult {
    let problem = diagonal_problem(
        vec![
            Point::try_new([-1.0])?,
            Point::try_new([0.0])?,
            Point::try_new([1.0])?,
        ],
        &[1.0, 1.0, 1.0],
        vec![0.0; 3],
    )?;
    let request = options(CenterSelectionStrategy::FarthestPoint {
        count: NonZeroUsize::new(3).ok_or("count")?,
        seed: 1,
    })?;
    let first = problem.try_select(&request)?;
    let second = problem.try_select(&request)?;

    assert_eq!(first.indices(), &[1, 2, 0]);
    assert_eq!(first, second);
    assert_eq!(first.diagnostics().seed, Some(1));
    Ok(())
}

#[test]
fn residual_greedy_matches_diagonal_interpolation_truth() -> TestResult {
    let locations = vec![
        Point::try_new([0.0])?,
        Point::try_new([1.0])?,
        Point::try_new([2.0])?,
    ];
    let problem = diagonal_problem(locations, &[1.0, 1.0, 1.0], vec![0.25, -5.0, 2.0])?;
    let selection = problem.try_select(&options(CenterSelectionStrategy::ResidualGreedy {
        count: NonZeroUsize::new(2).ok_or("count")?,
        seed: 17,
    })?)?;
    assert_eq!(selection.indices(), &[1, 2]);
    let greedy = selection.diagnostics().greedy.ok_or("greedy diagnostics")?;
    assert_eq!(greedy.minimum_accepted_pivot.to_bits(), 1.0_f64.to_bits());
    assert_eq!(greedy.maximum_accepted_pivot.to_bits(), 1.0_f64.to_bits());
    Ok(())
}

#[test]
fn power_greedy_matches_diagonal_schur_truth() -> TestResult {
    let locations = vec![
        Point::try_new([0.0])?,
        Point::try_new([1.0])?,
        Point::try_new([2.0])?,
    ];
    let problem = diagonal_problem(locations, &[1.0, 4.0, 9.0], vec![0.0; 3])?;
    let selection = problem.try_select(&options(CenterSelectionStrategy::PowerGreedy {
        count: NonZeroUsize::new(3).ok_or("count")?,
        seed: 99,
    })?)?;
    assert_eq!(selection.indices(), &[2, 1, 0]);
    let greedy = selection.diagnostics().greedy.ok_or("greedy diagnostics")?;
    assert_eq!(greedy.minimum_accepted_pivot.to_bits(), 1.0_f64.to_bits());
    assert_eq!(greedy.maximum_accepted_pivot.to_bits(), 9.0_f64.to_bits());
    Ok(())
}

#[test]
fn greedy_rank_classification_is_invariant_under_basis_scaling() -> TestResult {
    let small = 2.0_f64.powi(-100);
    let locations = vec![Point::try_new([0.0])?, Point::try_new([1.0])?];
    let scaled = diagonal_problem(locations.clone(), &[1.0, small], vec![2.0, 1.0])?;
    let identity = diagonal_problem(locations, &[1.0, 1.0], vec![2.0, 1.0])?;

    for strategy in [
        CenterSelectionStrategy::ResidualGreedy {
            count: NonZeroUsize::new(2).ok_or("count")?,
            seed: 7,
        },
        CenterSelectionStrategy::PowerGreedy {
            count: NonZeroUsize::new(2).ok_or("count")?,
            seed: 7,
        },
    ] {
        let request = options(strategy)?;
        for problem in [&scaled, &identity] {
            let selection = problem.try_select(&request)?;
            assert_eq!(selection.indices().len(), 2);
            assert_eq!(
                selection.diagnostics().rank.decision,
                DenseRankDecision::FullRank
            );
        }
    }
    Ok(())
}

#[test]
fn seeded_ties_are_repeatable_and_seed_is_recorded() -> TestResult {
    let locations = (0_u32..5)
        .map(|index| Point::try_new([f64::from(index)]))
        .collect::<Result<Vec<_>, _>>()?;
    let problem = diagonal_problem(locations, &[1.0; 5], vec![1.0; 5])?;
    for strategy in [
        CenterSelectionStrategy::ResidualGreedy {
            count: NonZeroUsize::new(4).ok_or("count")?,
            seed: 0x1234,
        },
        CenterSelectionStrategy::PowerGreedy {
            count: NonZeroUsize::new(4).ok_or("count")?,
            seed: 0x1234,
        },
    ] {
        let first = problem.try_select(&options(strategy.clone())?)?;
        let second = problem.try_select(&options(strategy)?)?;
        assert_eq!(first.indices(), second.indices());
        assert_eq!(first.diagnostics(), second.diagnostics());
        assert_eq!(first.diagnostics().seed, Some(0x1234));
    }
    Ok(())
}

#[test]
fn duplicate_basis_is_rejected_without_jitter_or_pseudoinverse() -> TestResult {
    let problem = CenterSelectionProblem::try_from_row_major(
        SPD,
        vec![Point::try_new([0.0])?, Point::try_new([0.0])?],
        vec![1.0, 1.0, 1.0, 1.0],
        vec![1.0, 1.0],
    )?;

    let greedy = rejected(
        problem.try_select(&options(CenterSelectionStrategy::PowerGreedy {
            count: NonZeroUsize::new(2).ok_or("count")?,
            seed: 0,
        })?),
        "zero Schur pivot must be rejected",
    )?;
    assert!(matches!(
        greedy,
        CenterSelectionError::InsufficientBasisRank {
            selected: 1,
            requested: 2,
            pivot: 0.0,
            ..
        }
    ));

    let all = rejected(
        problem.try_select(&options(CenterSelectionStrategy::AllRepresenters)?),
        "complete duplicate Gram must fail RRQR/SVD review",
    )?;
    assert!(matches!(
        all,
        CenterSelectionError::BasisReview(source)
            if matches!(*source, DenseSolveError::RankDeficient { .. })
    ));
    Ok(())
}

#[test]
fn malformed_and_insufficient_requests_are_structured() -> TestResult {
    assert!(matches!(
        CenterSelectionProblem::<1>::try_from_row_major(SPD, Vec::new(), Vec::new(), Vec::new()),
        Err(CenterSelectionError::EmptyCandidates)
    ));
    assert!(matches!(
        CenterSelectionProblem::try_from_row_major(
            SPD,
            vec![Point::try_new([0.0])?, Point::try_new([1.0])?],
            vec![1.0, 0.5, 0.25, 1.0],
            vec![0.0, 0.0],
        ),
        Err(CenterSelectionError::GramNotExactlySymmetric { .. })
    ));

    let problem = gaussian_problem()?;
    assert!(matches!(
        problem.try_select(&options(CenterSelectionStrategy::UserProvided(vec![1, 1]))?),
        Err(CenterSelectionError::DuplicateUserIndex { .. })
    ));
    assert!(matches!(
        problem.try_select(&options(CenterSelectionStrategy::UserProvided(vec![4]))?),
        Err(CenterSelectionError::UserIndexOutOfBounds { .. })
    ));
    assert!(matches!(
        problem.try_select(&options(CenterSelectionStrategy::PowerGreedy {
            count: NonZeroUsize::new(5).ok_or("count")?,
            seed: 0,
        })?),
        Err(CenterSelectionError::CountExceedsCandidates { .. })
    ));
    Ok(())
}

#[test]
fn malformed_gram_target_shapes_and_values_are_structured() -> TestResult {
    #[derive(Clone, Copy)]
    enum Expected {
        GramLength,
        TargetLength,
        NonFiniteGram,
        NonFiniteTarget,
    }

    let cases = [
        (
            "Gram length",
            vec![1.0, 0.0, 0.0],
            vec![0.0, 0.0],
            Expected::GramLength,
        ),
        (
            "target length",
            vec![1.0, 0.0, 0.0, 1.0],
            vec![0.0],
            Expected::TargetLength,
        ),
        (
            "nonfinite Gram",
            vec![1.0, f64::NAN, f64::NAN, 1.0],
            vec![0.0, 0.0],
            Expected::NonFiniteGram,
        ),
        (
            "nonfinite target",
            vec![1.0, 0.0, 0.0, 1.0],
            vec![0.0, f64::INFINITY],
            Expected::NonFiniteTarget,
        ),
    ];

    for (name, gram, targets, expected) in cases {
        let error = rejected(
            CenterSelectionProblem::try_from_row_major(
                SPD,
                vec![Point::try_new([0.0])?, Point::try_new([1.0])?],
                gram,
                targets,
            ),
            "malformed center-selection problem must be rejected",
        )?;
        let matched = match expected {
            Expected::GramLength => {
                matches!(error, CenterSelectionError::GramLengthMismatch { .. })
            }
            Expected::TargetLength => {
                matches!(error, CenterSelectionError::TargetLengthMismatch { .. })
            }
            Expected::NonFiniteGram => {
                matches!(error, CenterSelectionError::NonFiniteGram { .. })
            }
            Expected::NonFiniteTarget => {
                matches!(error, CenterSelectionError::NonFiniteTarget { .. })
            }
        };
        assert!(matched, "{name} returned {error}");
    }
    Ok(())
}

#[test]
fn cpd_candidate_gram_is_rejected_at_the_typed_capability_boundary() -> TestResult {
    let order = CpdOrder::try_new(1)?;
    let gram = vec![0.0, -1.0, -1.0, 0.0];
    let inverse_root_two = 1.0 / 2.0_f64.sqrt();
    let null_vector = [inverse_root_two, -inverse_root_two];
    let polynomial_action = [1.0, 1.0];
    let polynomial_residual =
        polynomial_action[0] * null_vector[0] + polynomial_action[1] * null_vector[1];
    let projected_energy =
        null_vector[0] * (gram[1] * null_vector[1]) + null_vector[1] * (gram[2] * null_vector[0]);
    assert_eq!(polynomial_residual.to_bits(), 0.0_f64.to_bits());
    assert!((projected_energy - 1.0).abs() <= 2.0 * f64::EPSILON);

    let error = rejected(
        CenterSelectionProblem::try_from_row_major(
            KernelDefiniteness::ConditionallyPositiveDefinite { order },
            vec![Point::try_new([0.0])?, Point::try_new([1.0])?],
            gram,
            vec![0.0, 0.0],
        ),
        "CPD input must fail before generic basis review",
    )?;
    assert!(matches!(
        error,
        CenterSelectionError::ConditionallyPositiveDefiniteUnsupported {
            order: rejected_order
        } if rejected_order == order
    ));
    Ok(())
}

#[test]
fn final_rank_review_enforces_explicit_memory_limit() -> TestResult {
    let problem = gaussian_problem()?;
    let tiny =
        CenterSelectionOptions::new(CenterSelectionStrategy::AllRepresenters, NonZeroUsize::MIN);
    let error = rejected(
        problem.try_select(&tiny),
        "one-byte review limit must fail before backend dispatch",
    )?;
    assert!(matches!(
        error,
        CenterSelectionError::BasisReview(source)
            if matches!(*source, DenseSolveError::MemoryLimitExceeded { .. })
    ));
    Ok(())
}

#[test]
fn compile_time_dimensions_two_and_three_share_the_same_rank_policy() -> TestResult {
    let two = diagonal_problem(
        vec![Point::try_new([0.0, 0.0])?, Point::try_new([1.0, -1.0])?],
        &[1.0, 2.0],
        vec![1.0, -1.0],
    )?;
    let three = diagonal_problem(
        vec![
            Point::try_new([0.0, 0.0, 0.0])?,
            Point::try_new([1.0, -1.0, 2.0])?,
        ],
        &[1.0, 2.0],
        vec![1.0, -1.0],
    )?;
    let request = options(CenterSelectionStrategy::AllRepresenters)?;
    for selection in [two.try_select(&request)?, three.try_select(&request)?] {
        assert_eq!(selection.indices(), &[0, 1]);
        assert_eq!(
            selection.diagnostics().rank.decision,
            DenseRankDecision::FullRank
        );
    }
    Ok(())
}
