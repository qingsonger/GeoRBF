//! Independent level-variable semantic and canonicalization tests.

use std::error::Error;
use std::io;
use std::num::NonZeroUsize;

use georbf::{
    AffineExpression, AffineTerm, ContrastDiagnostic, Dim, FunctionalAtom, FunctionalExpr,
    FunctionalProvenance, FunctionalTerm, LevelCanonicalizationError, LevelDefinition, LevelId,
    LevelMembership, LevelOrder, LevelPrior, LevelProblem, LevelProblemError, LevelValue,
    ObservationFunctional, ObservationId, Point, SemanticProvenance, SoftLoss, SourceLocation,
    SupportedDimension, VariableBlock,
};

type TestResult = Result<(), Box<dyn Error>>;

fn provenance(identifier: u64, field_path: &str) -> Result<SemanticProvenance, Box<dyn Error>> {
    Ok(SemanticProvenance::try_new(
        ObservationId::new(identifier),
        SourceLocation::try_new(
            "levels.csv".to_owned(),
            NonZeroUsize::new(usize::try_from(identifier)? + 1).ok_or("line")?,
        )?,
        "m".to_owned(),
        field_path.to_owned(),
        Some("levels".to_owned()),
    )?)
}

fn functional<const D: usize>(
    identifier: u64,
    coordinate: f64,
) -> Result<ObservationFunctional<D>, Box<dyn Error>>
where
    Dim<D>: SupportedDimension,
{
    let mut coordinates = [0.0; D];
    coordinates[0] = coordinate;
    let expression = FunctionalExpr::try_new([FunctionalTerm::try_new(
        1.0,
        FunctionalAtom::value(
            Point::try_new(coordinates)?,
            FunctionalProvenance::new(identifier),
        ),
    )?])?;
    Ok(ObservationFunctional::new(expression))
}

fn definition(
    identifier: u64,
    value: LevelValue,
    provenance_id: u64,
) -> Result<LevelDefinition, Box<dyn Error>> {
    Ok(LevelDefinition::new(
        LevelId::new(identifier),
        value,
        provenance(provenance_id, &format!("levels[{identifier}]"))?,
    ))
}

fn membership(
    level_id: u64,
    coordinate: f64,
    provenance_id: u64,
) -> Result<LevelMembership<1>, Box<dyn Error>> {
    Ok(LevelMembership::new(
        LevelId::new(level_id),
        functional(provenance_id, coordinate)?,
        provenance(provenance_id, &format!("memberships[{provenance_id}]"))?,
    ))
}

fn order(
    lower: u64,
    upper: u64,
    gap: f64,
    provenance_id: u64,
) -> Result<LevelOrder, Box<dyn Error>> {
    Ok(LevelOrder::try_new(
        LevelId::new(lower),
        LevelId::new(upper),
        gap,
        provenance(provenance_id, &format!("orders[{provenance_id}]"))?,
    )?)
}

#[test]
fn fixed_unknown_and_prior_compile_to_explicit_variables() -> TestResult {
    let prior = LevelPrior::try_new(2.0, 0.25, SoftLoss::Huber { delta: 1.5 })?;
    let problem = LevelProblem::try_new(
        [
            definition(10, LevelValue::try_fixed(0.0)?, 100)?,
            definition(20, LevelValue::unknown(), 101)?,
            definition(30, LevelValue::Prior(prior), 102)?,
        ],
        [
            membership(10, 0.0, 200)?,
            membership(20, 1.0, 201)?,
            membership(30, 2.0, 202)?,
        ],
        [order(10, 20, 0.5, 300)?, order(20, 30, 0.75, 301)?],
    )?;

    assert_eq!(
        problem.diagnostics().topological_order(),
        [LevelId::new(10), LevelId::new(20), LevelId::new(30)]
    );
    assert_eq!(problem.diagnostics().gauge_anchor_count(), 2);

    let mut next_field_variable = 0_usize;
    let compiled = problem.try_compile(
        [VariableBlock::try_new(
            "field".to_owned(),
            NonZeroUsize::new(3).ok_or("field block")?,
        )?],
        |_, _| {
            let variable = next_field_variable;
            next_field_variable += 1;
            AffineExpression::try_new([AffineTerm::try_new(variable, 2.0)?], 0.25)
        },
    )?;

    let canonical = compiled.canonical_problem();
    assert_eq!(canonical.variable_count(), 6);
    assert_eq!(
        canonical.variable_blocks().collect::<Vec<_>>(),
        [("field", 0, 3), ("levels", 3, 3)]
    );
    assert_eq!(canonical.equalities().len(), 4);
    assert_eq!(canonical.linear_bounds().len(), 2);
    assert!(canonical.capabilities().has_equalities);
    assert!(canonical.capabilities().has_linear_bounds);
    assert!(!canonical.capabilities().has_second_order_cones);

    let first = &canonical.equalities()[0];
    assert_eq!(
        first
            .row()
            .terms()
            .iter()
            .map(|term| (term.variable(), term.coefficient()))
            .collect::<Vec<_>>(),
        [(0, 2.0), (3, -1.0)]
    );
    assert_eq!(first.rhs().to_bits(), (-0.25_f64).to_bits());
    let fixed = &canonical.equalities()[3];
    assert_eq!(fixed.row().terms()[0].variable(), 3);
    assert_eq!(fixed.rhs().to_bits(), 0.0_f64.to_bits());

    let first_order = &canonical.linear_bounds()[0];
    assert_eq!(
        first_order
            .row()
            .terms()
            .iter()
            .map(|term| (term.variable(), term.coefficient()))
            .collect::<Vec<_>>(),
        [(3, -1.0), (4, 1.0)]
    );
    assert_eq!(first_order.lower(), Some(0.5));
    assert_eq!(first_order.upper(), None);

    assert_eq!(compiled.priors().len(), 1);
    assert_eq!(compiled.priors()[0].level_id(), LevelId::new(30));
    assert_eq!(compiled.priors()[0].variable(), 5);
    assert_eq!(compiled.priors()[0].prior(), prior);
    assert_eq!(compiled.level_variable(LevelId::new(20)), Some(4));
    assert_eq!(compiled.level_variable(LevelId::new(99)), None);
    Ok(())
}

#[test]
fn cycle_is_rejected_with_order_sources() -> TestResult {
    let result = LevelProblem::try_new(
        [
            definition(1, LevelValue::try_fixed(0.0)?, 10)?,
            definition(2, LevelValue::unknown(), 11)?,
            definition(3, LevelValue::unknown(), 12)?,
        ],
        [membership(1, 0.0, 20)?, membership(2, 1.0, 21)?],
        [
            order(1, 2, 1.0, 30)?,
            order(2, 3, 1.0, 31)?,
            order(3, 1, 1.0, 32)?,
        ],
    );
    let Err(LevelProblemError::OrderCycle { sources }) = result else {
        return Err(io::Error::other("expected order cycle").into());
    };
    assert_eq!(sources.len(), 3);
    Ok(())
}

#[test]
fn transitive_fixed_order_conflict_is_source_aware() -> TestResult {
    let result = LevelProblem::try_new(
        [
            definition(1, LevelValue::try_fixed(0.0)?, 10)?,
            definition(2, LevelValue::unknown(), 11)?,
            definition(3, LevelValue::try_fixed(1.0)?, 12)?,
        ],
        [membership(1, 0.0, 20)?, membership(3, 2.0, 21)?],
        [order(1, 2, 0.75, 30)?, order(2, 3, 0.75, 31)?],
    );
    let Err(LevelProblemError::FixedOrderConflict {
        lower,
        upper,
        required_gap,
        fixed_gap,
        sources,
    }) = result
    else {
        return Err(io::Error::other("expected transitive fixed conflict").into());
    };
    assert_eq!(lower, LevelId::new(1));
    assert_eq!(upper, LevelId::new(3));
    assert_eq!(required_gap.to_bits(), 1.5_f64.to_bits());
    assert_eq!(fixed_gap.to_bits(), 1.0_f64.to_bits());
    assert_eq!(sources.len(), 4);
    Ok(())
}

#[test]
fn identical_functional_with_distinct_fixed_values_is_rejected() -> TestResult {
    let shared = functional::<1>(90, 4.0)?;
    let result = LevelProblem::try_new(
        [
            definition(1, LevelValue::try_fixed(0.0)?, 10)?,
            definition(2, LevelValue::try_fixed(1.0)?, 11)?,
        ],
        [
            LevelMembership::new(
                LevelId::new(1),
                shared.clone(),
                provenance(20, "memberships[0]")?,
            ),
            LevelMembership::new(LevelId::new(2), shared, provenance(21, "memberships[1]")?),
        ],
        [],
    );
    let Err(LevelProblemError::FixedMembershipConflict {
        first_level,
        second_level,
        ..
    }) = result
    else {
        return Err(io::Error::other("expected fixed membership conflict").into());
    };
    assert_eq!(first_level, LevelId::new(1));
    assert_eq!(second_level, LevelId::new(2));
    Ok(())
}

#[test]
fn missing_gauge_is_checked_per_connected_component() -> TestResult {
    let result = LevelProblem::try_new(
        [
            definition(1, LevelValue::try_fixed(0.0)?, 10)?,
            definition(2, LevelValue::unknown(), 11)?,
            definition(3, LevelValue::unknown(), 12)?,
        ],
        [membership(1, 0.0, 20)?],
        [order(2, 3, 1.0, 30)?],
    );
    let Err(LevelProblemError::MissingGauge { diagnostic }) = result else {
        return Err(io::Error::other("expected missing gauge").into());
    };
    assert_eq!(diagnostic.component_count(), 1);
    Ok(())
}

#[test]
fn prior_anchors_gauge_but_equal_anchors_still_lack_contrast() -> TestResult {
    let prior = LevelPrior::try_new(0.0, 1.0, SoftLoss::SquaredL2)?;
    let result = LevelProblem::try_new(
        [
            definition(1, LevelValue::try_fixed(0.0)?, 10)?,
            definition(2, LevelValue::Prior(prior), 11)?,
        ],
        [membership(1, 0.0, 20)?, membership(2, 1.0, 21)?],
        [order(1, 2, 0.0, 30)?],
    );
    let Err(LevelProblemError::MissingContrast { diagnostic }) = result else {
        return Err(io::Error::other("expected missing contrast").into());
    };
    assert_eq!(
        diagnostic,
        ContrastDiagnostic::try_new(LevelId::new(1), LevelId::new(2))?
    );
    Ok(())
}

#[test]
fn isolated_unknown_and_undefined_references_are_rejected() -> TestResult {
    let isolated = LevelProblem::try_new(
        [
            definition(1, LevelValue::try_fixed(0.0)?, 10)?,
            definition(2, LevelValue::unknown(), 11)?,
        ],
        [membership(1, 0.0, 20)?],
        [],
    );
    let Err(LevelProblemError::IsolatedUnknownLevel { level_id }) = isolated else {
        return Err(io::Error::other("expected isolated unknown level").into());
    };
    assert_eq!(level_id, LevelId::new(2));

    let undefined = LevelProblem::try_new(
        [
            definition(1, LevelValue::try_fixed(0.0)?, 10)?,
            definition(2, LevelValue::try_fixed(1.0)?, 11)?,
        ],
        [membership(9, 0.0, 20)?],
        [],
    );
    let Err(LevelProblemError::UndefinedLevel {
        level_id,
        observation_id,
    }) = undefined
    else {
        return Err(io::Error::other("expected undefined level").into());
    };
    assert_eq!(level_id, LevelId::new(9));
    assert_eq!(observation_id, ObservationId::new(20));
    Ok(())
}

#[test]
fn numeric_validation_rejects_nonfinite_or_nonpositive_values() -> TestResult {
    assert!(matches!(
        LevelValue::try_fixed(f64::NAN),
        Err(LevelProblemError::NonFiniteFixedValue { .. })
    ));
    assert!(matches!(
        LevelPrior::try_new(f64::INFINITY, 1.0, SoftLoss::SquaredL2),
        Err(LevelProblemError::NonFinitePriorMean { .. })
    ));
    assert!(matches!(
        LevelPrior::try_new(0.0, 0.0, SoftLoss::AbsoluteL1),
        Err(LevelProblemError::InvalidPriorScale { .. })
    ));
    assert!(matches!(
        LevelPrior::try_new(0.0, 1.0, SoftLoss::Huber { delta: -1.0 }),
        Err(LevelProblemError::InvalidHuberDelta { .. })
    ));
    assert!(matches!(
        LevelOrder::try_new(
            LevelId::new(1),
            LevelId::new(2),
            -0.1,
            provenance(30, "orders[30]")?
        ),
        Err(LevelProblemError::InvalidMinimumGap { .. })
    ));
    assert!(matches!(
        LevelOrder::try_new(
            LevelId::new(1),
            LevelId::new(1),
            0.0,
            provenance(31, "orders[31]")?
        ),
        Err(LevelProblemError::SelfOrderEdge { .. })
    ));
    let bypassed_constructor = LevelProblem::try_new(
        [
            definition(1, LevelValue::Fixed(f64::NEG_INFINITY), 10)?,
            definition(2, LevelValue::try_fixed(1.0)?, 11)?,
        ],
        [membership(1, 0.0, 20)?],
        [],
    );
    assert!(matches!(
        bypassed_constructor,
        Err(LevelProblemError::NonFiniteFixedValue { .. })
    ));
    Ok(())
}

#[test]
fn compiler_reports_field_linearizer_failures_and_out_of_range_terms() -> TestResult {
    let problem = LevelProblem::try_new(
        [
            definition(1, LevelValue::try_fixed(0.0)?, 10)?,
            definition(2, LevelValue::try_fixed(1.0)?, 11)?,
        ],
        [membership(1, 0.0, 20)?, membership(2, 1.0, 21)?],
        [],
    )?;
    let block = || {
        VariableBlock::try_new(
            "field".to_owned(),
            NonZeroUsize::new(1).ok_or("field block")?,
        )
        .map_err(|error| -> Box<dyn Error> { Box::new(error) })
    };

    let failed = problem.try_compile([block()?], |_, _| {
        Err::<AffineExpression, _>(io::Error::other("linearizer failed"))
    });
    let Err(LevelCanonicalizationError::Linearization {
        membership_index,
        observation_id,
        ..
    }) = failed
    else {
        return Err(io::Error::other("expected linearizer failure").into());
    };
    assert_eq!(membership_index, 0);
    assert_eq!(observation_id, ObservationId::new(20));

    let invalid = problem.try_compile([block()?], |_, _| {
        AffineExpression::try_new([AffineTerm::try_new(1, 1.0)?], 0.0)
    });
    let Err(LevelCanonicalizationError::InvalidLinearization {
        membership_index,
        observation_id,
        ..
    }) = invalid
    else {
        return Err(io::Error::other("expected invalid linearization").into());
    };
    assert_eq!(membership_index, 0);
    assert_eq!(observation_id, ObservationId::new(20));
    Ok(())
}

#[test]
fn public_level_values_are_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<LevelProblem<1>>();
    assert_send_sync::<LevelProblem<2>>();
    assert_send_sync::<LevelProblem<3>>();
    assert_send_sync::<georbf::CompiledLevelProblem>();
    assert_send_sync::<LevelProblemError>();
}
