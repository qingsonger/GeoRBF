//! Independent truth tests for solver-neutral per-constraint soft objectives.

#![allow(clippy::expect_used, clippy::float_cmp, clippy::unwrap_used)]

use std::error::Error;
use std::io;
use std::num::NonZeroUsize;

use georbf::{
    AffineExpression, AffineTerm, CanonicalSoftObjective, CanonicalSoftRelation, Enforcement,
    ExecutionOptions, FunctionalAtom, FunctionalExpr, FunctionalProvenance, FunctionalTerm,
    ObservationFunctional, ObservationId, Point, SemanticConstraint, SemanticExpression,
    SemanticProblemIr, SemanticProvenance, SemanticRelation, SoftLoss, SourceLocation,
    VariableBlock,
};

type TestResult = Result<(), Box<dyn Error>>;

fn expression(
    identifier: u64,
    semantic_constant: f64,
) -> Result<SemanticExpression<1>, Box<dyn Error>> {
    let atom = FunctionalAtom::value(
        Point::try_new([0.0])?,
        FunctionalProvenance::new(identifier),
    );
    let functional =
        ObservationFunctional::new(FunctionalExpr::try_new([FunctionalTerm::try_new(
            1.0, atom,
        )?])?);
    Ok(SemanticExpression::try_new(functional, semantic_constant)?)
}

fn provenance(identifier: u64) -> Result<SemanticProvenance, Box<dyn Error>> {
    Ok(SemanticProvenance::try_new(
        ObservationId::new(identifier),
        SourceLocation::try_new(
            "soft-losses.csv".to_owned(),
            NonZeroUsize::new(usize::try_from(identifier)? + 1).ok_or("positive source line")?,
        )?,
        "m".to_owned(),
        format!("field.constraints[{identifier}]"),
        Some("soft-loss-truth".to_owned()),
    )?)
}

fn soft_constraint(
    identifier: u64,
    relation: SemanticRelation<1>,
    scale: f64,
    loss: SoftLoss,
) -> Result<SemanticConstraint<1>, Box<dyn Error>> {
    Ok(SemanticConstraint::try_new(
        provenance(identifier)?,
        relation,
        Enforcement::Soft { scale, loss },
    )?)
}

fn hard_constraint(
    identifier: u64,
    relation: SemanticRelation<1>,
) -> Result<SemanticConstraint<1>, Box<dyn Error>> {
    Ok(SemanticConstraint::try_new(
        provenance(identifier)?,
        relation,
        Enforcement::Hard,
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

fn affine_value(expression: &AffineExpression, variables: &[f64]) -> f64 {
    expression.constant()
        + expression
            .terms()
            .iter()
            .map(|term| term.coefficient() * variables[term.variable()])
            .sum::<f64>()
}

fn relation_violation(relation: &CanonicalSoftRelation, variables: &[f64]) -> f64 {
    match relation {
        CanonicalSoftRelation::Equality(relation) => {
            affine_value(relation.row(), variables) - relation.rhs()
        }
        CanonicalSoftRelation::LinearBound(relation) => {
            let value = affine_value(relation.row(), variables);
            let below = relation.lower().map_or(0.0, |lower| lower - value);
            let above = relation.upper().map_or(0.0, |upper| value - upper);
            0.0_f64.max(below).max(above)
        }
        CanonicalSoftRelation::SecondOrderCone(relation) => {
            let lhs_norm = relation
                .lhs()
                .iter()
                .map(|expression| affine_value(expression, variables).powi(2))
                .sum::<f64>()
                .sqrt();
            0.0_f64.max(lhs_norm - affine_value(relation.rhs(), variables))
        }
    }
}

fn loss_value(objective: &CanonicalSoftObjective, variables: &[f64]) -> f64 {
    let scaled = relation_violation(objective.relation(), variables) / objective.scale();
    match objective.loss() {
        SoftLoss::SquaredL2 => scaled * scaled,
        SoftLoss::AbsoluteL1 => scaled.abs(),
        SoftLoss::Huber { delta } => {
            let magnitude = scaled.abs();
            if magnitude <= delta {
                0.5 * magnitude * magnitude
            } else {
                delta * (magnitude - 0.5 * delta)
            }
        }
    }
}

fn one_block() -> Result<VariableBlock, georbf::ProblemIrError> {
    VariableBlock::try_new("z".to_owned(), NonZeroUsize::new(2).unwrap())
}

#[test]
fn l2_l1_and_huber_use_independent_residual_scales() -> TestResult {
    let problem = SemanticProblemIr::try_new(
        [
            soft_constraint(
                1,
                SemanticRelation::Equality {
                    expression: expression(11, 1.0)?,
                    target: 5.0,
                },
                2.0,
                SoftLoss::SquaredL2,
            )?,
            soft_constraint(
                2,
                SemanticRelation::Equality {
                    expression: expression(12, -2.0)?,
                    target: 0.0,
                },
                0.5,
                SoftLoss::AbsoluteL1,
            )?,
            soft_constraint(
                3,
                SemanticRelation::Equality {
                    expression: expression(13, 0.5)?,
                    target: -1.5,
                },
                2.0,
                SoftLoss::Huber { delta: 1.5 },
            )?,
        ],
        ExecutionOptions::default(),
    )?;
    let canonical = problem.try_compile([one_block()?], |functional, _| {
        match functional.expression().terms()[0]
            .atom()
            .provenance()
            .identifier()
        {
            11 => affine(&[(0, 2.0)], 2.0),
            12 => affine(&[(1, 1.0)], 2.0),
            13 => affine(&[(0, 1.0), (1, -1.0)], 0.0),
            _ => unreachable!("known truth-functional identifier"),
        }
    })?;

    assert!(canonical.equalities().is_empty());
    let objectives = canonical.soft_objectives();
    assert_eq!(objectives.len(), 3);
    assert_eq!(loss_value(&objectives[0], &[3.0, -1.0]), 4.0);
    assert_eq!(loss_value(&objectives[1], &[3.0, -1.0]), 2.0);
    assert_eq!(loss_value(&objectives[2], &[3.0, -1.0]), 3.375);
    assert_eq!(
        objectives
            .iter()
            .map(|objective| objective.provenance().observation_id())
            .collect::<Vec<_>>(),
        [
            ObservationId::new(1),
            ObservationId::new(2),
            ObservationId::new(3)
        ]
    );
    let capabilities = canonical.capabilities();
    assert!(capabilities.soft_objectives.has_any());
    assert!(capabilities.soft_objectives.has_squared_l2());
    assert!(capabilities.soft_objectives.has_absolute_l1());
    assert!(capabilities.soft_objectives.has_huber());
    assert_eq!(canonical.scaling().soft_objective(), &[1.0, 1.0, 1.0]);
    Ok(())
}

#[test]
fn soft_only_relations_report_required_geometry_capabilities() -> TestResult {
    fn compile(relation: SemanticRelation<1>) -> Result<georbf::CanonicalProblem, Box<dyn Error>> {
        let problem = SemanticProblemIr::try_new(
            [soft_constraint(1, relation, 1.0, SoftLoss::SquaredL2)?],
            ExecutionOptions::default(),
        )?;
        Ok(problem.try_compile([one_block()?], |_, _| affine(&[(0, 1.0)], 0.0))?)
    }

    let equality = compile(SemanticRelation::Equality {
        expression: expression(1, 0.0)?,
        target: 0.0,
    })?;
    assert!(equality.equalities().is_empty());
    assert!(equality.capabilities().has_equalities);
    assert!(!equality.capabilities().has_linear_bounds);
    assert!(!equality.capabilities().has_second_order_cones);

    let bound = compile(SemanticRelation::LinearBound {
        expression: expression(2, 0.0)?,
        lower: Some(-1.0),
        upper: Some(1.0),
    })?;
    assert!(bound.linear_bounds().is_empty());
    assert!(!bound.capabilities().has_equalities);
    assert!(bound.capabilities().has_linear_bounds);
    assert!(!bound.capabilities().has_second_order_cones);

    let cone = compile(SemanticRelation::SecondOrderCone {
        lhs: vec![expression(3, 0.0)?],
        rhs: expression(4, 0.0)?,
    })?;
    assert!(cone.second_order_cones().is_empty());
    assert!(!cone.capabilities().has_equalities);
    assert!(!cone.capabilities().has_linear_bounds);
    assert!(cone.capabilities().has_second_order_cones);
    Ok(())
}

#[test]
#[allow(clippy::too_many_lines)]
fn mixed_hard_and_soft_relations_preserve_hard_families() -> TestResult {
    let problem = SemanticProblemIr::try_new(
        [
            hard_constraint(
                1,
                SemanticRelation::Equality {
                    expression: expression(1, 0.0)?,
                    target: 2.0,
                },
            )?,
            soft_constraint(
                2,
                SemanticRelation::LinearBound {
                    expression: expression(2, 1.0)?,
                    lower: Some(-1.0),
                    upper: Some(3.0),
                },
                0.25,
                SoftLoss::AbsoluteL1,
            )?,
            hard_constraint(
                3,
                SemanticRelation::LinearBound {
                    expression: expression(3, -1.0)?,
                    lower: Some(0.0),
                    upper: None,
                },
            )?,
            soft_constraint(
                4,
                SemanticRelation::Equality {
                    expression: expression(4, 0.5)?,
                    target: 1.0,
                },
                2.0,
                SoftLoss::SquaredL2,
            )?,
            hard_constraint(
                5,
                SemanticRelation::SecondOrderCone {
                    lhs: vec![expression(5, 0.0)?],
                    rhs: expression(6, 0.0)?,
                },
            )?,
            soft_constraint(
                6,
                SemanticRelation::SecondOrderCone {
                    lhs: vec![expression(7, 1.0)?, expression(8, -1.0)?],
                    rhs: expression(9, 0.5)?,
                },
                0.5,
                SoftLoss::Huber { delta: 2.0 },
            )?,
        ],
        ExecutionOptions::default(),
    )?;
    let canonical = problem.try_compile([one_block()?], |functional, _| {
        let identifier = functional.expression().terms()[0]
            .atom()
            .provenance()
            .identifier();
        match identifier {
            1 | 2 | 3 | 4 | 5 | 7 => affine(&[(0, 1.0)], 0.0),
            6 | 8 | 9 => affine(&[(1, 1.0)], 0.0),
            _ => unreachable!("known mixed-relation identifier"),
        }
    })?;

    assert_eq!(canonical.equalities().len(), 1);
    assert_eq!(
        canonical.equalities()[0].provenance().observation_id(),
        ObservationId::new(1)
    );
    assert_eq!(canonical.linear_bounds().len(), 1);
    assert_eq!(
        canonical.linear_bounds()[0].provenance().observation_id(),
        ObservationId::new(3)
    );
    assert_eq!(canonical.second_order_cones().len(), 1);
    assert_eq!(
        canonical.second_order_cones()[0]
            .provenance()
            .observation_id(),
        ObservationId::new(5)
    );
    assert_eq!(
        canonical
            .soft_objectives()
            .iter()
            .map(|objective| objective.provenance().observation_id())
            .collect::<Vec<_>>(),
        [
            ObservationId::new(2),
            ObservationId::new(4),
            ObservationId::new(6)
        ]
    );

    let bound = &canonical.soft_objectives()[0];
    assert_eq!(relation_violation(bound.relation(), &[1.0, 0.0]), 0.0);
    assert_eq!(relation_violation(bound.relation(), &[4.0, 0.0]), 2.0);
    let cone = &canonical.soft_objectives()[2];
    let expected = (3.0_f64.powi(2) + (-3.0_f64).powi(2)).sqrt() + 1.5;
    assert_eq!(relation_violation(cone.relation(), &[2.0, -2.0]), expected);
    Ok(())
}

#[test]
fn positive_scalar_unit_rescaling_preserves_objective_value() -> TestResult {
    fn compile(unit: f64) -> Result<georbf::CanonicalProblem, Box<dyn Error>> {
        let problem = SemanticProblemIr::try_new(
            [soft_constraint(
                1,
                SemanticRelation::Equality {
                    expression: expression(10, unit)?,
                    target: 5.0 * unit,
                },
                2.0 * unit,
                SoftLoss::Huber { delta: 1.25 },
            )?],
            ExecutionOptions::default(),
        )?;
        Ok(problem.try_compile([one_block()?], |_, _| {
            affine(&[(0, 3.0 * unit)], 2.0 * unit)
        })?)
    }

    let base = compile(1.0)?;
    let rescaled = compile(1.0e120)?;
    let base_value = loss_value(&base.soft_objectives()[0], &[4.0, 0.0]);
    let rescaled_value = loss_value(&rescaled.soft_objectives()[0], &[4.0, 0.0]);
    let tolerance = 16.0 * f64::EPSILON * base_value.abs().max(rescaled_value.abs());
    assert!((base_value - rescaled_value).abs() <= tolerance);
    Ok(())
}

#[test]
fn public_soft_objective_values_are_send_sync() -> TestResult {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<CanonicalSoftRelation>();
    assert_send_sync::<CanonicalSoftObjective>();

    let invalid_huber = soft_constraint(
        1,
        SemanticRelation::Equality {
            expression: expression(1, 0.0)?,
            target: 0.0,
        },
        1.0,
        SoftLoss::Huber { delta: f64::NAN },
    );
    assert!(invalid_huber.is_err());
    let invalid_scale = soft_constraint(
        2,
        SemanticRelation::Equality {
            expression: expression(2, 0.0)?,
            target: 0.0,
        },
        f64::INFINITY,
        SoftLoss::SquaredL2,
    );
    assert!(invalid_scale.is_err());
    Ok(())
}

#[test]
fn unexpected_relation_shape_is_never_reclassified() -> TestResult {
    let problem = SemanticProblemIr::try_new(
        [soft_constraint(
            1,
            SemanticRelation::LinearBound {
                expression: expression(1, 0.0)?,
                lower: Some(-1.0),
                upper: Some(1.0),
            },
            1.0,
            SoftLoss::SquaredL2,
        )?],
        ExecutionOptions::default(),
    )?;
    let canonical = problem.try_compile([one_block()?], |_, _| affine(&[(0, 1.0)], 0.0))?;
    match canonical.soft_objectives()[0].relation() {
        CanonicalSoftRelation::LinearBound(_) => Ok(()),
        _ => Err(io::Error::other("soft bound was reclassified").into()),
    }
}
