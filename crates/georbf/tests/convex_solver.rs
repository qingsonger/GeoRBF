//! Independent analytic truth and failure-path tests for canonical QP/SOCP dispatch.

#![allow(clippy::float_cmp)]

use std::error::Error;
use std::num::{NonZeroU32, NonZeroUsize};

use georbf::{
    AffineExpression, AffineTerm, ConvexBackendStatus, ConvexSolveError, ConvexSolveOptions,
    Enforcement, ExecutionOptions, FunctionalAtom, FunctionalExpr, FunctionalProvenance,
    FunctionalTerm, ObservationFunctional, ObservationId, Point, SemanticConstraint,
    SemanticExpression, SemanticProblemIr, SemanticProvenance, SemanticRelation, SoftLoss,
    SourceLocation, VariableBlock, try_solve_canonical,
};

type TestResult = Result<(), Box<dyn Error>>;

fn expression(identifier: u64) -> Result<SemanticExpression<1>, Box<dyn Error>> {
    let atom = FunctionalAtom::value(
        Point::try_new([0.0])?,
        FunctionalProvenance::new(identifier),
    );
    Ok(SemanticExpression::try_new(
        ObservationFunctional::new(FunctionalExpr::try_new([FunctionalTerm::try_new(
            1.0, atom,
        )?])?),
        0.0,
    )?)
}

fn provenance(identifier: u64) -> Result<SemanticProvenance, Box<dyn Error>> {
    Ok(SemanticProvenance::try_new(
        ObservationId::new(identifier),
        SourceLocation::try_new(
            "convex-truth.csv".to_owned(),
            NonZeroUsize::new(usize::try_from(identifier)? + 1).ok_or("source line")?,
        )?,
        "m".to_owned(),
        format!("field.constraints[{identifier}]"),
        Some("convex-truth".to_owned()),
    )?)
}

fn constraint(
    identifier: u64,
    relation: SemanticRelation<1>,
    enforcement: Enforcement,
) -> Result<SemanticConstraint<1>, Box<dyn Error>> {
    Ok(SemanticConstraint::try_new(
        provenance(identifier)?,
        relation,
        enforcement,
    )?)
}

fn affine(
    terms: &[(usize, f64)],
    constant: f64,
) -> Result<AffineExpression, georbf::ProblemIrError> {
    AffineExpression::try_new(
        terms
            .iter()
            .map(|(variable, coefficient)| AffineTerm::try_new(*variable, *coefficient))
            .collect::<Result<Vec<_>, _>>()?,
        constant,
    )
}

fn block(count: usize) -> Result<VariableBlock, Box<dyn Error>> {
    Ok(VariableBlock::try_new(
        "z".to_owned(),
        NonZeroUsize::new(count).ok_or("variable count")?,
    )?)
}

fn options() -> Result<ConvexSolveOptions, Box<dyn Error>> {
    Ok(ConvexSolveOptions::try_new(
        1.0e-9,
        NonZeroU32::new(300).ok_or("iterations")?,
        Some(10.0),
        NonZeroUsize::new(64 * 1024 * 1024).ok_or("memory")?,
    )?)
}

fn close(actual: f64, expected: f64) -> bool {
    (actual - expected).abs() <= 5.0e-7 * expected.abs().max(1.0)
}

#[test]
fn qp_truth_recovers_independent_analytic_solution() -> TestResult {
    let semantic = SemanticProblemIr::try_new(
        [
            constraint(
                1,
                SemanticRelation::Equality {
                    expression: expression(1)?,
                    target: 2.0,
                },
                Enforcement::Hard,
            )?,
            constraint(
                2,
                SemanticRelation::LinearBound {
                    expression: expression(2)?,
                    lower: Some(0.0),
                    upper: Some(1.2),
                },
                Enforcement::Hard,
            )?,
            constraint(
                3,
                SemanticRelation::LinearBound {
                    expression: expression(3)?,
                    lower: Some(0.0),
                    upper: Some(2.0),
                },
                Enforcement::Hard,
            )?,
            constraint(
                4,
                SemanticRelation::Equality {
                    expression: expression(4)?,
                    target: 1.0,
                },
                Enforcement::Soft {
                    scale: 1.0,
                    loss: SoftLoss::SquaredL2,
                },
            )?,
            constraint(
                5,
                SemanticRelation::Equality {
                    expression: expression(5)?,
                    target: 2.0,
                },
                Enforcement::Soft {
                    scale: 1.0,
                    loss: SoftLoss::SquaredL2,
                },
            )?,
        ],
        ExecutionOptions::default(),
    )?;
    let canonical = semantic.try_compile([block(2)?], |functional, _| {
        match functional.expression().terms()[0]
            .atom()
            .provenance()
            .identifier()
        {
            1 => affine(&[(0, 1.0), (1, 1.0)], 0.0),
            2 | 4 => affine(&[(0, 1.0)], 0.0),
            3 | 5 => affine(&[(1, 1.0)], 0.0),
            _ => affine(&[], 0.0),
        }
    })?;
    let solution = try_solve_canonical(&canonical, options()?)?;
    assert!(close(solution.values()[0], 0.5));
    assert!(close(solution.values()[1], 1.5));
    assert_eq!(solution.diagnostics().status, ConvexBackendStatus::Solved);
    assert_eq!(solution.diagnostics().original_variable_count, 2);
    assert_eq!(solution.diagnostics().auxiliary_variable_count, 2);
    assert!(solution.diagnostics().kkt.normalized_primal_residual <= 6.4e-8);
    assert!(solution.diagnostics().kkt.normalized_dual_residual <= 6.4e-8);
    assert!(
        solution
            .diagnostics()
            .constraints
            .iter()
            .filter(|entry| entry.kind != georbf::ConvexConstraintKind::SoftObjective)
            .all(|entry| entry.normalized_residual <= 6.4e-8)
    );
    Ok(())
}

#[test]
fn socp_truth_recovers_three_four_five_solution() -> TestResult {
    let semantic = SemanticProblemIr::try_new(
        [
            constraint(
                10,
                SemanticRelation::Equality {
                    expression: expression(10)?,
                    target: 3.0,
                },
                Enforcement::Hard,
            )?,
            constraint(
                11,
                SemanticRelation::Equality {
                    expression: expression(11)?,
                    target: 4.0,
                },
                Enforcement::Hard,
            )?,
            constraint(
                12,
                SemanticRelation::SecondOrderCone {
                    lhs: vec![expression(10)?, expression(11)?],
                    rhs: expression(12)?,
                },
                Enforcement::Hard,
            )?,
            constraint(
                13,
                SemanticRelation::Equality {
                    expression: expression(12)?,
                    target: 0.0,
                },
                Enforcement::Soft {
                    scale: 1.0,
                    loss: SoftLoss::AbsoluteL1,
                },
            )?,
        ],
        ExecutionOptions::default(),
    )?;
    let canonical = semantic.try_compile([block(3)?], |functional, _| {
        match functional.expression().terms()[0]
            .atom()
            .provenance()
            .identifier()
        {
            10 => affine(&[(1, 1.0)], 0.0),
            11 => affine(&[(2, 1.0)], 0.0),
            12 => affine(&[(0, 1.0)], 0.0),
            _ => affine(&[], 0.0),
        }
    })?;
    let solution = try_solve_canonical(&canonical, options()?)?;
    assert!(close(solution.values()[0], 5.0));
    assert!(close(solution.values()[1], 3.0));
    assert!(close(solution.values()[2], 4.0));
    assert!(solution.diagnostics().kkt.primal_cone_violation <= 6.4e-8);
    assert!(solution.diagnostics().kkt.dual_cone_violation <= 6.4e-8);
    Ok(())
}

#[test]
fn mixed_l1_huber_bounds_and_cone_preserve_hard_constraints() -> TestResult {
    let semantic = SemanticProblemIr::try_new(
        [
            constraint(
                20,
                SemanticRelation::LinearBound {
                    expression: expression(20)?,
                    lower: Some(1.0),
                    upper: Some(2.0),
                },
                Enforcement::Hard,
            )?,
            constraint(
                21,
                SemanticRelation::SecondOrderCone {
                    lhs: vec![expression(21)?],
                    rhs: expression(20)?,
                },
                Enforcement::Hard,
            )?,
            constraint(
                22,
                SemanticRelation::Equality {
                    expression: expression(20)?,
                    target: 1.25,
                },
                Enforcement::Soft {
                    scale: 0.5,
                    loss: SoftLoss::Huber { delta: 0.75 },
                },
            )?,
            constraint(
                23,
                SemanticRelation::LinearBound {
                    expression: expression(21)?,
                    lower: Some(0.5),
                    upper: Some(0.75),
                },
                Enforcement::Soft {
                    scale: 0.25,
                    loss: SoftLoss::AbsoluteL1,
                },
            )?,
            constraint(
                24,
                SemanticRelation::SecondOrderCone {
                    lhs: vec![expression(21)?],
                    rhs: expression(24)?,
                },
                Enforcement::Soft {
                    scale: 1.0,
                    loss: SoftLoss::SquaredL2,
                },
            )?,
        ],
        ExecutionOptions::default(),
    )?;
    let canonical = semantic.try_compile([block(2)?], |functional, _| {
        match functional.expression().terms()[0]
            .atom()
            .provenance()
            .identifier()
        {
            20 => affine(&[(0, 1.0)], 0.0),
            21 => affine(&[(1, 1.0)], 0.0),
            24 => affine(&[], 2.0),
            _ => affine(&[], 0.0),
        }
    })?;
    let solution = try_solve_canonical(&canonical, options()?)?;
    assert!(solution.values()[0] >= 1.0 - 5.0e-7);
    assert!(solution.values()[0] <= 2.0 + 5.0e-7);
    assert!(solution.values()[1].abs() <= solution.values()[0] + 5.0e-7);
    assert!(solution.diagnostics().auxiliary_variable_count >= 4);
    assert_eq!(
        solution
            .diagnostics()
            .constraints
            .iter()
            .filter(|entry| entry.kind == georbf::ConvexConstraintKind::SoftObjective)
            .count(),
        3
    );
    Ok(())
}

#[test]
fn deterministic_repeats_match_values_objective_and_iterations() -> TestResult {
    let semantic = SemanticProblemIr::try_new(
        [
            constraint(
                30,
                SemanticRelation::Equality {
                    expression: expression(30)?,
                    target: 2.0,
                },
                Enforcement::Soft {
                    scale: 1.0,
                    loss: SoftLoss::SquaredL2,
                },
            )?,
            constraint(
                31,
                SemanticRelation::LinearBound {
                    expression: expression(30)?,
                    lower: Some(0.0),
                    upper: Some(3.0),
                },
                Enforcement::Hard,
            )?,
        ],
        ExecutionOptions::default(),
    )?;
    let canonical = semantic.try_compile([block(1)?], |_, _| affine(&[(0, 1.0)], 0.0))?;
    let first = try_solve_canonical(&canonical, options()?)?;
    let second = try_solve_canonical(&canonical, options()?)?;
    assert_eq!(first.values(), second.values());
    assert_eq!(
        first.diagnostics().kkt.original_objective,
        second.diagnostics().kkt.original_objective
    );
    assert_eq!(
        first.diagnostics().iterations,
        second.diagnostics().iterations
    );
    assert_eq!(
        first.diagnostics().constraints,
        second.diagnostics().constraints
    );
    Ok(())
}

#[test]
fn primal_infeasibility_requires_and_returns_reviewed_certificate() -> TestResult {
    let semantic = SemanticProblemIr::try_new(
        [
            constraint(
                40,
                SemanticRelation::Equality {
                    expression: expression(40)?,
                    target: 1.0,
                },
                Enforcement::Hard,
            )?,
            constraint(
                41,
                SemanticRelation::SecondOrderCone {
                    lhs: vec![expression(40)?],
                    rhs: expression(41)?,
                },
                Enforcement::Hard,
            )?,
        ],
        ExecutionOptions::default(),
    )?;
    let canonical = semantic.try_compile([block(1)?], |functional, _| {
        match functional.expression().terms()[0]
            .atom()
            .provenance()
            .identifier()
        {
            40 => affine(&[(0, 1.0)], 0.0),
            _ => affine(&[], 0.0),
        }
    })?;
    let Err(error) = try_solve_canonical(&canonical, options()?) else {
        return Err("contradictory cone unexpectedly solved".into());
    };
    let ConvexSolveError::PrimalInfeasible { certificate } = error else {
        return Err(format!("unexpected infeasibility result: {error}").into());
    };
    assert_eq!(certificate.status, ConvexBackendStatus::PrimalInfeasible);
    assert_eq!(certificate.normalized_dual.len(), certificate.rows.len());
    assert!(certificate.diagnostics.stationarity_residual_infinity <= 6.4e-8);
    assert!(certificate.diagnostics.dual_cone_violation <= 6.4e-8);
    assert!(certificate.diagnostics.separating_value < 0.0);
    Ok(())
}

#[test]
fn invalid_policy_and_memory_limit_fail_before_dispatch() -> TestResult {
    assert!(
        ConvexSolveOptions::try_new(f64::NAN, NonZeroU32::MIN, None, NonZeroUsize::MIN).is_err()
    );
    assert!(ConvexSolveOptions::try_new(1.0e-3, NonZeroU32::MIN, None, NonZeroUsize::MIN).is_err());
    assert!(
        ConvexSolveOptions::try_new(
            1.0e-9,
            NonZeroU32::MIN,
            Some(f64::INFINITY),
            NonZeroUsize::MIN
        )
        .is_err()
    );

    let semantic = SemanticProblemIr::try_new(
        [constraint(
            50,
            SemanticRelation::Equality {
                expression: expression(50)?,
                target: 1.0,
            },
            Enforcement::Soft {
                scale: 1.0,
                loss: SoftLoss::SquaredL2,
            },
        )?],
        ExecutionOptions::default(),
    )?;
    let canonical = semantic.try_compile([block(1)?], |_, _| affine(&[(0, 1.0)], 0.0))?;
    let tiny = ConvexSolveOptions::try_new(1.0e-9, NonZeroU32::MIN, None, NonZeroUsize::MIN)?;
    assert!(matches!(
        try_solve_canonical(&canonical, tiny),
        Err(ConvexSolveError::MemoryLimitExceeded { .. })
    ));
    Ok(())
}

#[test]
fn public_convex_results_are_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<georbf::ConvexSolution>();
    assert_send_sync::<georbf::ConvexSolveDiagnostics>();
    assert_send_sync::<georbf::ConvexInfeasibilityCertificate>();
    assert_send_sync::<georbf::ConvexSolveError>();
}
