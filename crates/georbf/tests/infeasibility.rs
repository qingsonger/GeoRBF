//! Independent truth tests for constraint review and infeasibility evidence.

#![allow(clippy::float_cmp)]

use std::error::Error;
use std::num::{NonZeroU32, NonZeroUsize};

use georbf::{
    AffineExpression, AffineTerm, CanonicalizationError, ConvexBackendStatus, ConvexSolveError,
    ConvexSolveOptions, Dim, Enforcement, ExecutionOptions, FunctionalAtom, FunctionalExpr,
    FunctionalOrientation, FunctionalProvenance, FunctionalSimilarity, FunctionalTerm,
    HardAffineConstraintKind, ObservationFunctional, ObservationId, Point, ProblemIrError,
    SemanticConstraint, SemanticExpression, SemanticProblemIr, SemanticProvenance,
    SemanticRelation, SoftLoss, SourceLocation, SupportedDimension, VariableBlock,
    try_review_constraints, try_solve_canonical,
};

type TestResult = Result<(), Box<dyn Error>>;

fn provenance(identifier: u64) -> Result<SemanticProvenance, ProblemIrError> {
    SemanticProvenance::try_new(
        ObservationId::new(identifier),
        SourceLocation::try_new(
            "infeasibility.csv".to_owned(),
            NonZeroUsize::new(usize::try_from(identifier).unwrap_or(1).saturating_add(1))
                .ok_or(ProblemIrError::MemoryEstimateOverflow)?,
        )?,
        "m".to_owned(),
        format!("field.constraints[{identifier}]"),
        Some("infeasibility-review".to_owned()),
    )
}

fn expression<const D: usize>(identifier: u64) -> Result<SemanticExpression<D>, Box<dyn Error>>
where
    Dim<D>: SupportedDimension,
{
    let atom = FunctionalAtom::value(
        Point::try_new([0.0; D])?,
        FunctionalProvenance::new(identifier),
    );
    Ok(SemanticExpression::try_new(
        ObservationFunctional::new(FunctionalExpr::try_new([FunctionalTerm::try_new(
            1.0, atom,
        )?])?),
        0.0,
    )?)
}

fn constraint<const D: usize>(
    identifier: u64,
    relation: SemanticRelation<D>,
    enforcement: Enforcement,
) -> Result<SemanticConstraint<D>, ProblemIrError>
where
    Dim<D>: SupportedDimension,
{
    SemanticConstraint::try_new(provenance(identifier)?, relation, enforcement)
}

fn affine(terms: &[(usize, f64)]) -> Result<AffineExpression, ProblemIrError> {
    AffineExpression::try_new(
        terms
            .iter()
            .map(|(variable, coefficient)| AffineTerm::try_new(*variable, *coefficient))
            .collect::<Result<Vec<_>, _>>()?,
        0.0,
    )
}

fn block(count: usize) -> Result<VariableBlock, ProblemIrError> {
    VariableBlock::try_new(
        "z".to_owned(),
        NonZeroUsize::new(count).ok_or(ProblemIrError::VariableCountOverflow)?,
    )
}

#[test]
fn duplicate_and_near_duplicate_rows_are_source_aware_and_non_mutating() -> TestResult {
    let semantic = SemanticProblemIr::try_new(
        [
            constraint(
                1,
                SemanticRelation::Equality {
                    expression: expression::<1>(1)?,
                    target: 3.0,
                },
                Enforcement::Hard,
            )?,
            constraint(
                2,
                SemanticRelation::Equality {
                    expression: expression::<1>(2)?,
                    target: 6.0,
                },
                Enforcement::Hard,
            )?,
            constraint(
                3,
                SemanticRelation::Equality {
                    expression: expression::<1>(3)?,
                    target: -3.0,
                },
                Enforcement::Hard,
            )?,
        ],
        ExecutionOptions::default(),
    )?;
    let canonical = semantic.try_compile([block(2)?], |_, source| {
        match source.observation_id().identifier() {
            1 => affine(&[(0, 1.0), (1, 2.0)]),
            2 => affine(&[(0, 2.0), (1, 4.0)]),
            3 => affine(&[(0, -1.0), (1, -(2.0 + 1.0e-15))]),
            _ => unreachable!("known source"),
        }
    })?;

    let before = canonical.clone();
    let first = try_review_constraints(&canonical)?;
    let second = try_review_constraints(&canonical)?;
    assert_eq!(first, second);
    assert_eq!(canonical, before);
    assert_eq!(first.reviewed_hard_affine_constraints, 3);
    assert_eq!(first.pairs.len(), 3);
    assert_eq!(
        first
            .pairs
            .iter()
            .map(|pair| (
                pair.first_provenance.observation_id().identifier(),
                pair.second_provenance.observation_id().identifier(),
                pair.similarity,
            ))
            .collect::<Vec<_>>(),
        vec![
            (1, 2, FunctionalSimilarity::Duplicate),
            (1, 3, FunctionalSimilarity::NearDuplicate),
            (2, 3, FunctionalSimilarity::NearDuplicate),
        ]
    );
    assert_eq!(first.pairs[0].orientation, FunctionalOrientation::Same);
    assert_eq!(first.pairs[1].orientation, FunctionalOrientation::Reversed);
    assert!(first.pairs[1].normalized_row_distance > 0.0);
    assert!(first.pairs[1].normalized_row_distance <= first.pairs[1].comparison_threshold);
    assert!(
        first
            .pairs
            .iter()
            .all(|pair| pair.first_kind == HardAffineConstraintKind::Equality
                && pair.second_kind == HardAffineConstraintKind::Equality)
    );
    assert_eq!(
        canonical.equalities()[1]
            .row()
            .terms()
            .iter()
            .map(|term| term.coefficient())
            .collect::<Vec<_>>(),
        vec![2.0, 4.0]
    );
    Ok(())
}

#[test]
fn exact_affine_conflicts_and_constant_equalities_retain_sources() -> TestResult {
    for row_scale in [1.0e-12, 1.0, 1.0e12] {
        let semantic = SemanticProblemIr::try_new(
            [
                constraint(
                    10,
                    SemanticRelation::Equality {
                        expression: expression::<1>(10)?,
                        target: 2.0,
                    },
                    Enforcement::Hard,
                )?,
                constraint(
                    11,
                    SemanticRelation::LinearBound {
                        expression: expression::<1>(11)?,
                        lower: None,
                        upper: Some(row_scale),
                    },
                    Enforcement::Hard,
                )?,
            ],
            ExecutionOptions::default(),
        )?;
        let result = semantic.try_compile([block(1)?], |_, source| {
            affine(&[(
                0,
                if source.observation_id() == ObservationId::new(10) {
                    1.0
                } else {
                    row_scale
                },
            )])
        });
        let Err(error) = result else {
            return Err("x = 2 and x <= 1 must be exactly inconsistent".into());
        };
        match error {
            CanonicalizationError::Ir(ProblemIrError::InfeasibleHardAffineConstraints {
                sources,
                lower,
                upper,
            }) => {
                assert_eq!((lower, upper), (2.0, 1.0));
                assert_eq!(
                    sources
                        .iter()
                        .map(|source| source.observation_id().identifier())
                        .collect::<Vec<_>>(),
                    vec![10, 11]
                );
            }
            other => return Err(format!("unexpected conflict: {other:?}").into()),
        }
    }

    let constant = SemanticProblemIr::try_new(
        [constraint(
            12,
            SemanticRelation::Equality {
                expression: expression::<1>(12)?,
                target: 1.0,
            },
            Enforcement::Hard,
        )?],
        ExecutionOptions::default(),
    )?;
    assert!(matches!(
        constant.try_compile([block(1)?], |_, _| affine(&[])),
        Err(CanonicalizationError::Ir(
            ProblemIrError::InfeasibleConstantEquality { sources, rhs }
        )) if sources[0].observation_id() == ObservationId::new(12) && rhs == 1.0
    ));
    Ok(())
}

#[test]
fn soft_objectives_are_excluded_from_hard_duplicate_review() -> TestResult {
    let semantic = SemanticProblemIr::try_new(
        [
            constraint(
                20,
                SemanticRelation::Equality {
                    expression: expression::<1>(20)?,
                    target: 0.0,
                },
                Enforcement::Hard,
            )?,
            constraint(
                21,
                SemanticRelation::Equality {
                    expression: expression::<1>(21)?,
                    target: 1.0,
                },
                Enforcement::Soft {
                    scale: 1.0,
                    loss: SoftLoss::SquaredL2,
                },
            )?,
        ],
        ExecutionOptions::default(),
    )?;
    let canonical = semantic.try_compile([block(1)?], |_, _| affine(&[(0, 1.0)]))?;
    let review = try_review_constraints(&canonical)?;
    assert_eq!(review.reviewed_hard_affine_constraints, 1);
    assert_eq!(review.excluded_soft_objectives, 1);
    assert!(review.pairs.is_empty());
    Ok(())
}

fn review_dimension<const D: usize>() -> TestResult
where
    Dim<D>: SupportedDimension,
{
    let semantic = SemanticProblemIr::try_new(
        [
            constraint(
                100 + u64::try_from(D)?,
                SemanticRelation::Equality {
                    expression: expression::<D>(1)?,
                    target: 0.0,
                },
                Enforcement::Hard,
            )?,
            constraint(
                200 + u64::try_from(D)?,
                SemanticRelation::Equality {
                    expression: expression::<D>(2)?,
                    target: 0.0,
                },
                Enforcement::Hard,
            )?,
        ],
        ExecutionOptions::default(),
    )?;
    let canonical = semantic.try_compile([block(1)?], |_, _| affine(&[(0, 1.0)]))?;
    assert_eq!(try_review_constraints(&canonical)?.pairs.len(), 1);
    Ok(())
}

#[test]
fn review_is_dimension_generic_send_and_sync() -> TestResult {
    fn assert_send_sync<T: Send + Sync>() {}
    review_dimension::<1>()?;
    review_dimension::<2>()?;
    review_dimension::<3>()?;
    assert_send_sync::<georbf::ConstraintReviewDiagnostics>();
    assert_send_sync::<georbf::ConstraintPairDiagnostics>();
    Ok(())
}

#[test]
fn general_infeasibility_returns_independently_reviewed_source_certificate() -> TestResult {
    let semantic = SemanticProblemIr::try_new(
        [
            constraint(
                30,
                SemanticRelation::LinearBound {
                    expression: expression::<1>(30)?,
                    lower: Some(1.0),
                    upper: None,
                },
                Enforcement::Hard,
            )?,
            constraint(
                31,
                SemanticRelation::LinearBound {
                    expression: expression::<1>(31)?,
                    lower: Some(1.0),
                    upper: None,
                },
                Enforcement::Hard,
            )?,
            constraint(
                32,
                SemanticRelation::LinearBound {
                    expression: expression::<1>(32)?,
                    lower: None,
                    upper: Some(1.5),
                },
                Enforcement::Hard,
            )?,
        ],
        ExecutionOptions::default(),
    )?;
    let canonical = semantic.try_compile([block(2)?], |_, source| {
        match source.observation_id().identifier() {
            30 => affine(&[(0, 1.0)]),
            31 => affine(&[(1, 1.0)]),
            32 => affine(&[(0, 1.0), (1, 1.0)]),
            _ => unreachable!("known source"),
        }
    })?;
    assert!(try_review_constraints(&canonical)?.pairs.is_empty());
    let options = ConvexSolveOptions::try_new(
        1.0e-9,
        NonZeroU32::new(300).ok_or("iterations")?,
        Some(10.0),
        NonZeroUsize::new(64 * 1024 * 1024).ok_or("memory")?,
    )?;
    let Err(error) = try_solve_canonical(&canonical, options) else {
        return Err("x >= 1, y >= 1, x + y <= 1.5 must be infeasible".into());
    };
    let ConvexSolveError::PrimalInfeasible { certificate } = error else {
        return Err(format!("unexpected solve result: {error}").into());
    };
    assert_eq!(certificate.status, ConvexBackendStatus::PrimalInfeasible);
    assert_eq!(certificate.rows.len(), certificate.normalized_dual.len());
    assert_eq!(
        certificate
            .rows
            .iter()
            .map(|(_, source)| source.observation_id().identifier())
            .collect::<Vec<_>>(),
        vec![30, 31, 32]
    );
    assert!(
        certificate
            .diagnostics
            .normalized_stationarity_residual_infinity
            <= 1.0e-9
    );
    assert!(certificate.diagnostics.dual_cone_violation <= 1.0e-9);
    assert!(certificate.diagnostics.separating_value < 0.0);
    Ok(())
}
