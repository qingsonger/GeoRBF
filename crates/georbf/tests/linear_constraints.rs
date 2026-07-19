//! Independent truth and error-path tests for semantic linear constraints.

use std::error::Error;
use std::num::NonZeroUsize;

use georbf::{
    AffineExpression, AffineTerm, CanonicalizationError, Dim, Enforcement, ExecutionOptions,
    FunctionalAtom, FunctionalExpr, FunctionalProvenance, FunctionalTerm, InsideOrientation,
    LevelDefinition, LevelId, LevelMembership, LevelOrder, LevelProblem, LevelValue,
    LinearConstraint, LinearConstraintError, MonotonicitySense, ObservationFunctional,
    ObservationId, Point, ProblemIrError, RegionSide, SemanticProblemIr, SemanticProvenance,
    SourceLocation, SupportedDimension, UnitDirection, VariableBlock,
};

type TestResult = Result<(), Box<dyn Error>>;

fn provenance(identifier: u64) -> Result<SemanticProvenance, Box<dyn Error>> {
    Ok(SemanticProvenance::try_new(
        ObservationId::new(identifier),
        SourceLocation::try_new(
            "linear.csv".to_owned(),
            NonZeroUsize::new(usize::try_from(identifier)? + 1).ok_or("line")?,
        )?,
        "m".to_owned(),
        format!("constraints[{identifier}]"),
        Some("linear".to_owned()),
    )?)
}

fn value_functional<const D: usize>(
    variable: usize,
) -> Result<ObservationFunctional<D>, Box<dyn Error>>
where
    Dim<D>: SupportedDimension,
{
    let mut coordinates = [0.0; D];
    coordinates[0] = f64::from(u32::try_from(variable)?);
    Ok(ObservationFunctional::new(FunctionalExpr::try_new([
        FunctionalTerm::try_new(
            1.0,
            FunctionalAtom::value(
                Point::try_new(coordinates)?,
                FunctionalProvenance::new(u64::try_from(variable)?),
            ),
        )?,
    ])?))
}

fn assert_send_sync<T: Send + Sync>() {}

fn derivative_functional<const D: usize>(
    variable: usize,
) -> Result<ObservationFunctional<D>, Box<dyn Error>>
where
    Dim<D>: SupportedDimension,
{
    let mut direction = [0.0; D];
    direction[0] = 1.0;
    Ok(ObservationFunctional::new(FunctionalExpr::try_new([
        FunctionalTerm::try_new(
            1.0,
            FunctionalAtom::directional_derivative(
                Point::try_new([0.0; D])?,
                UnitDirection::try_new(direction)?,
                FunctionalProvenance::new(u64::try_from(variable)?),
            ),
        )?,
    ])?))
}

fn linearize<const D: usize>(
    functional: &ObservationFunctional<D>,
    variable_count: usize,
) -> Result<AffineExpression, ProblemIrError>
where
    Dim<D>: SupportedDimension,
{
    let mut coefficients = vec![0.0; variable_count];
    for term in functional.expression().terms() {
        let variable = usize::try_from(term.atom().provenance().identifier())
            .map_err(|_| ProblemIrError::VariableCountOverflow)?;
        coefficients[variable] += term.coefficient();
    }
    let terms = coefficients
        .into_iter()
        .enumerate()
        .filter(|(_, coefficient)| *coefficient != 0.0)
        .map(|(variable, coefficient)| AffineTerm::try_new(variable, coefficient))
        .collect::<Result<Vec<_>, _>>()?;
    AffineExpression::try_new(terms, 0.0)
}

fn one_block(count: usize) -> Result<VariableBlock, ProblemIrError> {
    VariableBlock::try_new(
        "field".to_owned(),
        NonZeroUsize::new(count).ok_or(ProblemIrError::EmptyVariableSpace)?,
    )
}

fn row_terms(row: &georbf::CanonicalLinearBound) -> Vec<(usize, f64)> {
    row.row()
        .terms()
        .iter()
        .map(|term| (term.variable(), term.coefficient()))
        .collect()
}

#[test]
#[allow(clippy::too_many_lines)]
fn all_semantic_forms_compile_with_explicit_signs() -> TestResult {
    let constraints = vec![
        LinearConstraint::try_lower(
            provenance(10)?,
            value_functional::<1>(0)?,
            1.0,
            Enforcement::Hard,
        )?,
        LinearConstraint::try_upper(
            provenance(11)?,
            value_functional::<1>(1)?,
            2.0,
            Enforcement::Hard,
        )?,
        LinearConstraint::try_interval(
            provenance(12)?,
            value_functional::<1>(2)?,
            3.0,
            4.0,
            Enforcement::Hard,
        )?,
        LinearConstraint::try_region(
            provenance(13)?,
            value_functional::<1>(3)?,
            5.0,
            RegionSide::Inside,
            InsideOrientation::InsideAtOrBelow,
            Enforcement::Hard,
        )?,
        LinearConstraint::try_region(
            provenance(14)?,
            value_functional::<1>(4)?,
            6.0,
            RegionSide::Outside,
            InsideOrientation::InsideAtOrBelow,
            Enforcement::Hard,
        )?,
        LinearConstraint::try_region(
            provenance(15)?,
            value_functional::<1>(5)?,
            7.0,
            RegionSide::Inside,
            InsideOrientation::InsideAtOrAbove,
            Enforcement::Hard,
        )?,
        LinearConstraint::try_region(
            provenance(16)?,
            value_functional::<1>(6)?,
            8.0,
            RegionSide::Outside,
            InsideOrientation::InsideAtOrAbove,
            Enforcement::Hard,
        )?,
        LinearConstraint::try_scalar_gap(
            provenance(17)?,
            value_functional::<1>(7)?,
            value_functional::<1>(8)?,
            0.75,
            Enforcement::Hard,
        )?,
        LinearConstraint::try_monotonicity(
            provenance(18)?,
            derivative_functional::<1>(9)?,
            MonotonicitySense::Increasing,
            0.25,
            Enforcement::Hard,
        )?,
        LinearConstraint::try_monotonicity(
            provenance(19)?,
            derivative_functional::<1>(10)?,
            MonotonicitySense::Decreasing,
            0.5,
            Enforcement::Hard,
        )?,
    ];
    let semantic = constraints
        .into_iter()
        .map(LinearConstraint::try_into_semantic_constraint)
        .collect::<Result<Vec<_>, _>>()?;
    let problem = SemanticProblemIr::try_new(semantic, ExecutionOptions::default())?;
    let canonical =
        problem.try_compile([one_block(11)?], |functional, _| linearize(functional, 11))?;

    let rows = canonical.linear_bounds();
    assert_eq!(rows.len(), 10);
    assert_eq!((rows[0].lower(), rows[0].upper()), (Some(1.0), None));
    assert_eq!((rows[1].lower(), rows[1].upper()), (None, Some(2.0)));
    assert_eq!((rows[2].lower(), rows[2].upper()), (Some(3.0), Some(4.0)));
    assert_eq!((rows[3].lower(), rows[3].upper()), (None, Some(5.0)));
    assert_eq!((rows[4].lower(), rows[4].upper()), (Some(6.0), None));
    assert_eq!((rows[5].lower(), rows[5].upper()), (Some(7.0), None));
    assert_eq!((rows[6].lower(), rows[6].upper()), (None, Some(8.0)));
    assert_eq!(row_terms(&rows[7]), [(7, -1.0), (8, 1.0)]);
    assert_eq!((rows[7].lower(), rows[7].upper()), (Some(0.75), None));
    assert_eq!((rows[8].lower(), rows[8].upper()), (Some(0.25), None));
    assert_eq!((rows[9].lower(), rows[9].upper()), (None, Some(-0.5)));
    assert_eq!(
        rows.iter()
            .map(|row| row.provenance().observation_id().identifier())
            .collect::<Vec<_>>(),
        (10_u64..20).collect::<Vec<_>>()
    );
    Ok(())
}

#[test]
fn malformed_linear_semantics_are_rejected() -> TestResult {
    assert!(matches!(
        LinearConstraint::try_lower(
            provenance(1)?,
            value_functional::<1>(0)?,
            f64::NAN,
            Enforcement::Hard,
        ),
        Err(LinearConstraintError::NonFiniteThreshold { .. })
    ));
    assert!(matches!(
        LinearConstraint::try_interval(
            provenance(2)?,
            value_functional::<1>(0)?,
            2.0,
            1.0,
            Enforcement::Hard,
        ),
        Err(LinearConstraintError::ReversedInterval { .. })
    ));
    assert!(matches!(
        LinearConstraint::try_scalar_gap(
            provenance(3)?,
            value_functional::<1>(0)?,
            value_functional::<1>(1)?,
            -f64::EPSILON,
            Enforcement::Hard,
        ),
        Err(LinearConstraintError::InvalidMinimumGap { .. })
    ));
    assert!(matches!(
        LinearConstraint::try_monotonicity(
            provenance(4)?,
            derivative_functional::<1>(0)?,
            MonotonicitySense::Increasing,
            f64::INFINITY,
            Enforcement::Hard,
        ),
        Err(LinearConstraintError::InvalidMinimumRate { .. })
    ));
    assert!(matches!(
        LinearConstraint::try_monotonicity(
            provenance(5)?,
            value_functional::<1>(0)?,
            MonotonicitySense::Increasing,
            0.0,
            Enforcement::Hard,
        ),
        Err(LinearConstraintError::MonotonicityRequiresDirectionalDerivative)
    ));
    let point = Point::try_new([0.0])?;
    let direction = UnitDirection::try_new([1.0])?;
    let scaled = ObservationFunctional::new(FunctionalExpr::try_new([FunctionalTerm::try_new(
        2.0,
        FunctionalAtom::directional_derivative(point, direction, FunctionalProvenance::new(0)),
    )?])?);
    assert!(matches!(
        LinearConstraint::try_monotonicity(
            provenance(7)?,
            scaled,
            MonotonicitySense::Increasing,
            0.0,
            Enforcement::Hard,
        ),
        Err(LinearConstraintError::InvalidMonotonicityCoefficient { .. })
    ));
    let multiple = ObservationFunctional::new(FunctionalExpr::try_new([
        FunctionalTerm::try_new(
            1.0,
            FunctionalAtom::directional_derivative(point, direction, FunctionalProvenance::new(0)),
        )?,
        FunctionalTerm::try_new(
            1.0,
            FunctionalAtom::directional_derivative(point, direction, FunctionalProvenance::new(1)),
        )?,
    ])?);
    assert!(matches!(
        LinearConstraint::try_monotonicity(
            provenance(8)?,
            multiple,
            MonotonicitySense::Increasing,
            0.0,
            Enforcement::Hard,
        ),
        Err(LinearConstraintError::InvalidMonotonicityTermCount { count: 2 })
    ));
    assert!(matches!(
        LinearConstraint::try_upper(
            provenance(6)?,
            value_functional::<1>(0)?,
            0.0,
            Enforcement::Soft {
                scale: 0.0,
                loss: georbf::SoftLoss::SquaredL2,
            },
        ),
        Err(LinearConstraintError::Ir(
            ProblemIrError::InvalidSoftScale { .. }
        ))
    ));
    Ok(())
}

fn compile_dimension<const D: usize>() -> TestResult
where
    Dim<D>: SupportedDimension,
{
    let constraint = LinearConstraint::try_monotonicity(
        provenance(u64::try_from(D)?)?,
        derivative_functional::<D>(0)?,
        MonotonicitySense::Increasing,
        0.0,
        Enforcement::Hard,
    )?
    .try_into_semantic_constraint()?;
    let problem = SemanticProblemIr::try_new([constraint], ExecutionOptions::default())?;
    let canonical =
        problem.try_compile([one_block(1)?], |functional, _| linearize(functional, 1))?;
    assert_eq!(canonical.linear_bounds().len(), 1);
    Ok(())
}

#[test]
fn linear_semantics_are_dimension_safe_and_thread_safe() -> TestResult {
    compile_dimension::<1>()?;
    compile_dimension::<2>()?;
    compile_dimension::<3>()?;
    assert_send_sync::<LinearConstraint<1>>();
    assert_send_sync::<LinearConstraint<2>>();
    assert_send_sync::<LinearConstraint<3>>();
    Ok(())
}

#[test]
fn exact_same_and_sign_reversed_rows_report_source_aware_infeasibility() -> TestResult {
    let same_row = SemanticProblemIr::try_new(
        [
            LinearConstraint::try_lower(
                provenance(20)?,
                value_functional::<1>(0)?,
                2.0,
                Enforcement::Hard,
            )?
            .try_into_semantic_constraint()?,
            LinearConstraint::try_upper(
                provenance(21)?,
                value_functional::<1>(0)?,
                1.0,
                Enforcement::Hard,
            )?
            .try_into_semantic_constraint()?,
        ],
        ExecutionOptions::default(),
    )?;
    assert!(matches!(
        same_row.try_compile([one_block(1)?], |functional, _| {
            linearize(functional, 1)
        }),
        Err(CanonicalizationError::Ir(
            ProblemIrError::InfeasibleLinearBounds { .. }
        ))
    ));

    let constraints = [
        LinearConstraint::try_lower(
            provenance(1)?,
            value_functional::<1>(0)?,
            2.0,
            Enforcement::Hard,
        )?
        .try_into_semantic_constraint()?,
        LinearConstraint::try_lower(
            provenance(2)?,
            value_functional::<1>(0)?,
            0.0,
            Enforcement::Hard,
        )?
        .try_into_semantic_constraint()?,
    ];
    let problem = SemanticProblemIr::try_new(constraints, ExecutionOptions::default())?;
    let Err(error) = problem.try_compile([one_block(1)?], |_, source| {
        let coefficient = if source.observation_id() == ObservationId::new(1) {
            1.0
        } else {
            -1.0
        };
        AffineExpression::try_new([AffineTerm::try_new(0, coefficient)?], 0.0)
    }) else {
        return Err("x >= 2 and -x >= 0 must be infeasible".into());
    };
    match error {
        CanonicalizationError::Ir(ProblemIrError::InfeasibleLinearBounds {
            sources,
            lower,
            upper,
        }) => {
            assert_eq!(sources.len(), 2);
            assert_eq!(sources[0].observation_id(), ObservationId::new(1));
            assert_eq!(sources[1].observation_id(), ObservationId::new(2));
            assert_eq!((lower, upper), (2.0, -0.0));
        }
        other => return Err(format!("unexpected conflict: {other:?}").into()),
    }

    let feasible = SemanticProblemIr::try_new(
        [
            LinearConstraint::try_lower(
                provenance(3)?,
                value_functional::<1>(0)?,
                1.0,
                Enforcement::Hard,
            )?
            .try_into_semantic_constraint()?,
            LinearConstraint::try_upper(
                provenance(4)?,
                value_functional::<1>(0)?,
                1.0,
                Enforcement::Hard,
            )?
            .try_into_semantic_constraint()?,
        ],
        ExecutionOptions::default(),
    )?;
    assert!(
        feasible
            .try_compile([one_block(1)?], |functional, _| {
                linearize(functional, 1)
            })
            .is_ok()
    );

    Ok(())
}

#[test]
fn soft_bounds_remain_objectives_and_are_not_hard_conflicts() -> TestResult {
    let soft = SemanticProblemIr::try_new(
        [
            LinearConstraint::try_lower(
                provenance(30)?,
                value_functional::<1>(0)?,
                2.0,
                Enforcement::Soft {
                    scale: 1.0,
                    loss: georbf::SoftLoss::SquaredL2,
                },
            )?
            .try_into_semantic_constraint()?,
            LinearConstraint::try_upper(
                provenance(31)?,
                value_functional::<1>(0)?,
                1.0,
                Enforcement::Soft {
                    scale: 1.0,
                    loss: georbf::SoftLoss::SquaredL2,
                },
            )?
            .try_into_semantic_constraint()?,
        ],
        ExecutionOptions::default(),
    )?;
    let canonical = soft.try_compile([one_block(1)?], |functional, _| linearize(functional, 1))?;
    assert!(canonical.linear_bounds().is_empty());
    assert_eq!(canonical.soft_objectives().len(), 2);
    Ok(())
}

#[test]
fn field_bounds_compose_with_explicit_level_rows_without_rewriting_them() -> TestResult {
    let levels = LevelProblem::try_new(
        [
            LevelDefinition::new(
                LevelId::new(10),
                LevelValue::try_fixed(0.0)?,
                provenance(100)?,
            ),
            LevelDefinition::new(
                LevelId::new(20),
                LevelValue::try_fixed(1.0)?,
                provenance(101)?,
            ),
        ],
        [
            LevelMembership::new(
                LevelId::new(10),
                value_functional::<1>(0)?,
                provenance(102)?,
            ),
            LevelMembership::new(
                LevelId::new(20),
                value_functional::<1>(1)?,
                provenance(103)?,
            ),
        ],
        [LevelOrder::try_new(
            LevelId::new(10),
            LevelId::new(20),
            0.5,
            provenance(104)?,
        )?],
    )?;
    let field_block = one_block(2)?;
    let compiled_levels = levels.try_compile([field_block.clone()], |functional, _| {
        linearize(functional, 2)
    })?;
    let original = compiled_levels.canonical_problem().clone();

    let field_semantic = SemanticProblemIr::try_new(
        [
            LinearConstraint::try_lower(
                provenance(105)?,
                value_functional::<1>(0)?,
                -2.0,
                Enforcement::Hard,
            )?
            .try_into_semantic_constraint()?,
            LinearConstraint::try_upper(
                provenance(106)?,
                value_functional::<1>(1)?,
                3.0,
                Enforcement::Hard,
            )?
            .try_into_semantic_constraint()?,
        ],
        ExecutionOptions::default(),
    )?;
    let field_canonical =
        field_semantic.try_compile([field_block], |functional, _| linearize(functional, 2))?;
    let combined = compiled_levels.try_compose_field_linear_problem(field_canonical)?;
    let canonical = combined.canonical_problem();

    assert_eq!(
        canonical.variable_blocks().collect::<Vec<_>>(),
        [("field", 0, 2), ("levels", 2, 2)]
    );
    assert_eq!(canonical.equalities(), original.equalities());
    assert_eq!(&canonical.linear_bounds()[..1], original.linear_bounds());
    assert_eq!(canonical.linear_bounds().len(), 3);
    assert_eq!(
        canonical.linear_bounds()[1].provenance().observation_id(),
        ObservationId::new(105)
    );
    assert_eq!(
        canonical.linear_bounds()[2].provenance().observation_id(),
        ObservationId::new(106)
    );
    assert_eq!(combined.level_variable(LevelId::new(10)), Some(2));

    let mismatch = field_semantic.try_compile(
        [VariableBlock::try_new(
            "other".to_owned(),
            NonZeroUsize::new(2).ok_or("block")?,
        )?],
        |functional, _| linearize(functional, 2),
    )?;
    let levels_again =
        levels.try_compile([one_block(2)?], |functional, _| linearize(functional, 2))?;
    assert!(matches!(
        levels_again.try_compose_field_linear_problem(mismatch),
        Err(ProblemIrError::IncompatibleCanonicalVariableSpaces)
    ));
    Ok(())
}

#[test]
fn field_level_composition_rejects_duplicate_observation_ids() -> TestResult {
    let levels = LevelProblem::try_new(
        [
            LevelDefinition::new(
                LevelId::new(10),
                LevelValue::try_fixed(0.0)?,
                provenance(100)?,
            ),
            LevelDefinition::new(
                LevelId::new(20),
                LevelValue::try_fixed(1.0)?,
                provenance(101)?,
            ),
        ],
        [
            LevelMembership::new(
                LevelId::new(10),
                value_functional::<1>(0)?,
                provenance(102)?,
            ),
            LevelMembership::new(
                LevelId::new(20),
                value_functional::<1>(1)?,
                provenance(103)?,
            ),
        ],
        [LevelOrder::try_new(
            LevelId::new(10),
            LevelId::new(20),
            0.5,
            provenance(104)?,
        )?],
    )?;
    let field_block = one_block(2)?;
    let compiled_levels = levels.try_compile([field_block.clone()], |functional, _| {
        linearize(functional, 2)
    })?;
    let duplicate = SemanticProblemIr::try_new(
        [LinearConstraint::try_lower(
            provenance(100)?,
            value_functional::<1>(0)?,
            -2.0,
            Enforcement::Hard,
        )?
        .try_into_semantic_constraint()?],
        ExecutionOptions::default(),
    )?
    .try_compile([field_block], |functional, _| linearize(functional, 2))?;

    assert!(matches!(
        compiled_levels.try_compose_field_linear_problem(duplicate),
        Err(ProblemIrError::DuplicateObservationId { identifier })
            if identifier == ObservationId::new(100)
    ));
    Ok(())
}

#[test]
fn infeasible_constant_row_is_rejected_without_a_solver() -> TestResult {
    let constraint = LinearConstraint::try_lower(
        provenance(8)?,
        value_functional::<1>(0)?,
        1.0,
        Enforcement::Hard,
    )?
    .try_into_semantic_constraint()?;
    let problem = SemanticProblemIr::try_new([constraint], ExecutionOptions::default())?;
    let Err(error) = problem.try_compile([one_block(1)?], |_, _| {
        AffineExpression::try_new(std::iter::empty::<AffineTerm>(), 0.0)
    }) else {
        return Err("zero cannot satisfy zero >= 1".into());
    };
    assert!(matches!(
        error,
        CanonicalizationError::Ir(ProblemIrError::InfeasibleConstantLinearBound {
            lower: Some(1.0),
            upper: None,
            ..
        })
    ));
    Ok(())
}
