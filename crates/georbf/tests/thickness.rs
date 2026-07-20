//! Independent scalar-gap and sampled local thickness tests.

use std::error::Error;
use std::io;
use std::num::NonZeroUsize;

use georbf::{
    AffineExpression, AffineTerm, Dim, FunctionalAtom, FunctionalExpr, FunctionalProvenance,
    FunctionalTerm, LevelDefinition, LevelId, LevelMembership, LevelOrder, LevelProblem,
    LevelValue, LocalNormalThickness, LocalNormalThicknessError, ObservationFunctional,
    ObservationId, Point, ProblemIrError, SemanticProvenance, SourceLocation, SupportedDimension,
    ThicknessCanonicalizationError, ThicknessDiagnosticKind, ThicknessGuarantee, VariableBlock,
};

type TestResult = Result<(), Box<dyn Error>>;

fn provenance(identifier: u64, field_path: &str) -> Result<SemanticProvenance, Box<dyn Error>> {
    Ok(SemanticProvenance::try_new(
        ObservationId::new(identifier),
        SourceLocation::try_new(
            "thickness.csv".to_owned(),
            NonZeroUsize::new(usize::try_from(identifier)? + 1).ok_or("line")?,
        )?,
        "m".to_owned(),
        field_path.to_owned(),
        Some("thickness".to_owned()),
    )?)
}

fn value_functional<const D: usize>(
    identifier: u64,
    coordinate: f64,
) -> Result<ObservationFunctional<D>, Box<dyn Error>>
where
    Dim<D>: SupportedDimension,
{
    let mut coordinates = [0.0; D];
    coordinates[0] = coordinate;
    Ok(ObservationFunctional::new(FunctionalExpr::try_new([
        FunctionalTerm::try_new(
            1.0,
            FunctionalAtom::value(
                Point::try_new(coordinates)?,
                FunctionalProvenance::new(identifier),
            ),
        )?,
    ])?))
}

fn level_problem<const D: usize>() -> Result<LevelProblem<D>, Box<dyn Error>>
where
    Dim<D>: SupportedDimension,
{
    let lower = LevelId::new(10);
    let upper = LevelId::new(20);
    Ok(LevelProblem::try_new(
        [
            LevelDefinition::new(
                lower,
                LevelValue::try_fixed(0.0)?,
                provenance(100, "levels.lower")?,
            ),
            LevelDefinition::new(
                upper,
                LevelValue::try_fixed(10.0)?,
                provenance(101, "levels.upper")?,
            ),
        ],
        [
            LevelMembership::new(
                lower,
                value_functional(200, 0.0)?,
                provenance(200, "memberships.lower")?,
            ),
            LevelMembership::new(
                upper,
                value_functional(201, 10.0)?,
                provenance(201, "memberships.upper")?,
            ),
        ],
        [LevelOrder::try_new(
            lower,
            upper,
            2.0,
            provenance(300, "orders.scalar_gap")?,
        )?],
    )?)
}

fn compile_level_problem<const D: usize>() -> Result<georbf::CompiledLevelProblem, Box<dyn Error>>
where
    Dim<D>: SupportedDimension,
{
    let mut membership_variable = 0_usize;
    Ok(level_problem::<D>()?.try_compile(
        [VariableBlock::try_new(
            "field".to_owned(),
            NonZeroUsize::new(D + 2).ok_or("field block")?,
        )?],
        |_, _| {
            let variable = membership_variable;
            membership_variable += 1;
            AffineExpression::try_new([AffineTerm::try_new(variable, 1.0)?], 0.0)
        },
    )?)
}

fn local_constraint<const D: usize>(
    identifier: u64,
    minimum_thickness: f64,
) -> Result<LocalNormalThickness<D>, Box<dyn Error>>
where
    Dim<D>: SupportedDimension,
{
    Ok(LocalNormalThickness::try_new(
        LevelId::new(10),
        LevelId::new(20),
        Point::try_new([0.5; D])?,
        minimum_thickness,
        provenance(identifier, &format!("local_thickness[{identifier}]"))?,
    )?)
}

struct LooseUpperBound<T> {
    item: Option<T>,
}

impl<T> Iterator for LooseUpperBound<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.item.take()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(usize::MAX))
    }
}

fn compile_local<const D: usize>(
    minimum_thickness: f64,
) -> Result<georbf::CompiledLevelProblem, Box<dyn Error>>
where
    Dim<D>: SupportedDimension,
{
    Ok(
        compile_level_problem::<D>()?.try_compose_local_normal_thickness(
            [local_constraint(400, minimum_thickness)?],
            |functional, _| {
                let [term] = functional.expression().terms() else {
                    return Err(ProblemIrError::MemoryEstimateOverflow);
                };
                let FunctionalAtom::DirectionalDerivative { direction, .. } = term.atom() else {
                    return Err(ProblemIrError::MemoryEstimateOverflow);
                };
                AffineExpression::try_new(
                    direction
                        .components()
                        .iter()
                        .copied()
                        .enumerate()
                        .filter(|(_, component)| *component != 0.0)
                        .map(|(axis, component)| AffineTerm::try_new(axis + 2, component))
                        .collect::<Result<Vec<_>, _>>()?,
                    0.0,
                )
            },
        )?,
    )
}

fn assert_dimension<const D: usize>() -> TestResult
where
    Dim<D>: SupportedDimension,
{
    let compiled = compile_local::<D>(2.0)?;
    let canonical = compiled.canonical_problem();
    assert_eq!(canonical.linear_bounds().len(), 1);
    assert_eq!(canonical.second_order_cones().len(), 1);
    assert!(canonical.capabilities().has_linear_bounds);
    assert!(canonical.capabilities().has_second_order_cones);

    let scalar_gap = &canonical.linear_bounds()[0];
    assert_eq!(scalar_gap.lower(), Some(2.0));
    assert_eq!(scalar_gap.upper(), None);
    assert_eq!(
        scalar_gap.provenance().observation_id(),
        ObservationId::new(300)
    );

    let cone = &canonical.second_order_cones()[0];
    assert_eq!(cone.lhs().len(), D);
    for (axis, row) in cone.lhs().iter().enumerate() {
        assert_eq!(row.constant().to_bits(), 0.0_f64.to_bits());
        assert_eq!(row.terms().len(), 1);
        assert_eq!(row.terms()[0].variable(), axis + 2);
        assert_eq!(row.terms()[0].coefficient().to_bits(), 2.0_f64.to_bits());
    }
    assert_eq!(
        cone.rhs()
            .terms()
            .iter()
            .map(|term| (term.variable(), term.coefficient()))
            .collect::<Vec<_>>(),
        [(D + 2, -1.0), (D + 3, 1.0)]
    );
    assert_eq!(cone.rhs().constant().to_bits(), 0.0_f64.to_bits());
    assert_eq!(cone.provenance().observation_id(), ObservationId::new(400));
    Ok(())
}

#[test]
fn local_cones_are_dimension_safe_and_preserve_ordered_signs() -> TestResult {
    assert_dimension::<1>()?;
    assert_dimension::<2>()?;
    assert_dimension::<3>()?;
    Ok(())
}

#[test]
fn loose_iterator_upper_bound_does_not_reject_a_trivial_constraint() -> TestResult {
    let constraints = LooseUpperBound {
        item: Some(local_constraint::<1>(400, 1.0)?),
    };
    let compiled = compile_level_problem::<1>()?.try_compose_local_normal_thickness(
        constraints,
        |functional, _| {
            let FunctionalAtom::DirectionalDerivative { direction, .. } =
                functional.expression().terms()[0].atom()
            else {
                return Err(ProblemIrError::MemoryEstimateOverflow);
            };
            let axis = direction
                .components()
                .iter()
                .position(|component| component.to_bits() == 1.0_f64.to_bits())
                .ok_or(ProblemIrError::MemoryEstimateOverflow)?;
            AffineExpression::try_new([AffineTerm::try_new(axis + 2, 1.0)?], 0.0)
        },
    )?;
    assert_eq!(compiled.canonical_problem().second_order_cones().len(), 1);
    Ok(())
}

#[test]
fn sample_point_and_complete_provenance_cross_compilation_boundaries() -> TestResult {
    let point = Point::try_new([1.25, -2.5, 3.75])?;
    let expected_provenance = SemanticProvenance::try_new(
        ObservationId::new(777),
        SourceLocation::try_new(
            "sections/alpha-thickness.csv".to_owned(),
            NonZeroUsize::new(37).ok_or("line")?,
        )?,
        "metres".to_owned(),
        "fields.stratigraphy.local_thickness.samples[7]".to_owned(),
        Some("section-alpha".to_owned()),
    )?;
    let constraint = LocalNormalThickness::try_new(
        LevelId::new(10),
        LevelId::new(20),
        point,
        2.0,
        expected_provenance.clone(),
    )?;
    let mut expected_axis = 0_usize;
    let compiled = compile_level_problem::<3>()?.try_compose_local_normal_thickness(
        [constraint],
        |functional, provenance| {
            assert_eq!(provenance, &expected_provenance);
            let [term] = functional.expression().terms() else {
                return Err(ProblemIrError::MemoryEstimateOverflow);
            };
            let FunctionalAtom::DirectionalDerivative {
                point: callback_point,
                direction,
                ..
            } = term.atom()
            else {
                return Err(ProblemIrError::MemoryEstimateOverflow);
            };
            assert_eq!(callback_point, point);
            let axis = direction
                .components()
                .iter()
                .position(|component| component.to_bits() == 1.0_f64.to_bits())
                .ok_or(ProblemIrError::MemoryEstimateOverflow)?;
            assert_eq!(axis, expected_axis);
            expected_axis += 1;
            AffineExpression::try_new([AffineTerm::try_new(axis + 2, 1.0)?], 0.0)
        },
    )?;
    assert_eq!(expected_axis, 3);
    assert_eq!(
        compiled.canonical_problem().second_order_cones()[0].provenance(),
        &expected_provenance
    );
    Ok(())
}

#[test]
fn scalar_gap_and_local_cone_have_distinct_diagnostics_and_canonical_shapes() -> TestResult {
    let problem = level_problem::<2>()?;
    let scalar = problem.orders()[0].thickness_diagnostics();
    let local = local_constraint::<2>(400, 2.0)?;
    assert_eq!(scalar.kind(), ThicknessDiagnosticKind::ScalarLevelGap);
    assert_eq!(scalar.guarantee(), ThicknessGuarantee::ScalarOnly);
    assert_eq!(
        local.diagnostics().kind(),
        ThicknessDiagnosticKind::SampledLocalNormalCone
    );
    assert_eq!(
        local.diagnostics().guarantee(),
        ThicknessGuarantee::SampledLocalFirstOrder
    );
    assert_ne!(scalar, local.diagnostics());
    Ok(())
}

#[test]
fn analytic_parallel_level_case_is_exactly_on_the_local_boundary() -> TestResult {
    let compiled = compile_local::<2>(2.0)?;
    let cone = &compiled.canonical_problem().second_order_cones()[0];
    let variables = [0.0, 10.0, 3.0, 4.0, 0.0, 10.0];
    let evaluate = |row: &AffineExpression| {
        row.constant()
            + row
                .terms()
                .iter()
                .map(|term| term.coefficient() * variables[term.variable()])
                .sum::<f64>()
    };
    let norm = cone
        .lhs()
        .iter()
        .map(|row| evaluate(row).powi(2))
        .sum::<f64>()
        .sqrt();
    let gap = evaluate(cone.rhs());
    assert_eq!(norm.to_bits(), 10.0_f64.to_bits());
    assert_eq!(gap.to_bits(), 10.0_f64.to_bits());
    Ok(())
}

#[test]
fn construction_rejects_equal_levels_and_nonpositive_or_nonfinite_thickness() -> TestResult {
    let point = Point::try_new([0.0, 0.0])?;
    let make = |lower, upper, value| {
        LocalNormalThickness::try_new(
            LevelId::new(lower),
            LevelId::new(upper),
            point,
            value,
            provenance(400, "local").map_err(|error| io::Error::other(error.to_string()))?,
        )
        .map_err(|error| io::Error::other(format!("{error:?}")))
    };
    assert!(matches!(
        LocalNormalThickness::try_new(
            LevelId::new(10),
            LevelId::new(10),
            point,
            1.0,
            provenance(400, "local")?,
        ),
        Err(LocalNormalThicknessError::EqualLevels { .. })
    ));
    for value in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(make(10, 20, value).is_err());
    }
    Ok(())
}

#[test]
fn composition_rejects_empty_unknown_and_duplicate_constraints() -> TestResult {
    let empty = compile_level_problem::<2>()?.try_compose_local_normal_thickness::<2, _>(
        [],
        |_, _| -> Result<AffineExpression, ProblemIrError> {
            Err(ProblemIrError::MemoryEstimateOverflow)
        },
    );
    assert!(matches!(
        empty,
        Err(ThicknessCanonicalizationError::EmptyConstraints)
    ));

    let unknown = LocalNormalThickness::try_new(
        LevelId::new(10),
        LevelId::new(99),
        Point::try_new([0.0, 0.0])?,
        1.0,
        provenance(400, "unknown")?,
    )?;
    let unknown_result = compile_level_problem::<2>()?.try_compose_local_normal_thickness(
        [unknown],
        |_, _| -> Result<AffineExpression, ProblemIrError> {
            Err(ProblemIrError::MemoryEstimateOverflow)
        },
    );
    assert!(matches!(
        unknown_result,
        Err(ThicknessCanonicalizationError::UnknownUpperLevel {
            level_id,
            ..
        }) if level_id == LevelId::new(99)
    ));

    let mut duplicate_linearizer_called = false;
    let duplicate_existing = compile_level_problem::<2>()?.try_compose_local_normal_thickness(
        [local_constraint::<2>(300, 1.0)?],
        |functional, _| {
            duplicate_linearizer_called = true;
            let FunctionalAtom::DirectionalDerivative { direction, .. } =
                functional.expression().terms()[0].atom()
            else {
                return Err(ProblemIrError::MemoryEstimateOverflow);
            };
            let axis = direction
                .components()
                .iter()
                .position(|component| component.to_bits() == 1.0_f64.to_bits())
                .ok_or(ProblemIrError::MemoryEstimateOverflow)?;
            AffineExpression::try_new([AffineTerm::try_new(axis + 2, 1.0)?], 0.0)
        },
    );
    assert!(matches!(
        duplicate_existing,
        Err(ThicknessCanonicalizationError::Ir(
            ProblemIrError::DuplicateObservationId { identifier }
        )) if identifier == ObservationId::new(300)
    ));
    assert!(!duplicate_linearizer_called);

    let duplicate_local = compile_level_problem::<2>()?.try_compose_local_normal_thickness(
        [
            local_constraint::<2>(400, 1.0)?,
            local_constraint::<2>(400, 2.0)?,
        ],
        |functional, _| {
            let FunctionalAtom::DirectionalDerivative { direction, .. } =
                functional.expression().terms()[0].atom()
            else {
                return Err(ProblemIrError::MemoryEstimateOverflow);
            };
            let axis = direction
                .components()
                .iter()
                .position(|component| component.to_bits() == 1.0_f64.to_bits())
                .ok_or(ProblemIrError::MemoryEstimateOverflow)?;
            AffineExpression::try_new([AffineTerm::try_new(axis + 2, 1.0)?], 0.0)
        },
    );
    assert!(matches!(
        duplicate_local,
        Err(ThicknessCanonicalizationError::Ir(
            ProblemIrError::DuplicateObservationId { identifier }
        )) if identifier == ObservationId::new(400)
    ));
    Ok(())
}

#[test]
fn invalid_or_failed_linearization_is_source_indexed() -> TestResult {
    let failed = compile_level_problem::<2>()?.try_compose_local_normal_thickness(
        [local_constraint::<2>(400, 1.0)?],
        |_, _| -> Result<AffineExpression, io::Error> { Err(io::Error::other("linearizer")) },
    );
    assert!(matches!(
        failed,
        Err(ThicknessCanonicalizationError::Linearization {
            constraint_index: 0,
            axis: 0,
            observation_id,
            ..
        }) if observation_id == ObservationId::new(400)
    ));

    let invalid = compile_level_problem::<2>()?
        .try_compose_local_normal_thickness([local_constraint::<2>(400, 1.0)?], |_, _| {
            AffineExpression::try_new([AffineTerm::try_new(4, 1.0)?], 0.0)
        });
    assert!(matches!(
        invalid,
        Err(ThicknessCanonicalizationError::InvalidLinearization {
            constraint_index: 0,
            axis: 0,
            source: ProblemIrError::AffineVariableOutOfRange {
                variable: 4,
                variable_count: 4,
            },
            ..
        })
    ));
    Ok(())
}

#[test]
fn thickness_scaling_rejects_coefficient_and_constant_overflow_and_underflow() -> TestResult {
    let overflow = compile_level_problem::<1>()?
        .try_compose_local_normal_thickness([local_constraint::<1>(400, 2.0)?], |_, _| {
            AffineExpression::try_new([AffineTerm::try_new(2, f64::MAX)?], 0.0)
        });
    assert!(matches!(
        overflow,
        Err(
            ThicknessCanonicalizationError::ScaledGradientCoefficientNotRepresentable {
                constraint_index: 0,
                axis: 0,
                ..
            }
        )
    ));

    let underflow = compile_level_problem::<1>()?.try_compose_local_normal_thickness(
        [local_constraint::<1>(400, f64::from_bits(1))?],
        |_, _| AffineExpression::try_new([AffineTerm::try_new(2, f64::MIN_POSITIVE)?], 0.0),
    );
    assert!(matches!(
        underflow,
        Err(
            ThicknessCanonicalizationError::ScaledGradientCoefficientNotRepresentable {
                constraint_index: 0,
                axis: 0,
                ..
            }
        )
    ));

    let constant_overflow = compile_level_problem::<1>()?
        .try_compose_local_normal_thickness([local_constraint::<1>(400, 2.0)?], |_, _| {
            AffineExpression::try_new([], f64::MAX)
        });
    assert!(matches!(
        constant_overflow,
        Err(
            ThicknessCanonicalizationError::ScaledGradientConstantNotRepresentable {
                constraint_index: 0,
                axis: 0,
                ..
            }
        )
    ));

    let constant_underflow = compile_level_problem::<1>()?.try_compose_local_normal_thickness(
        [local_constraint::<1>(400, f64::from_bits(1))?],
        |_, _| AffineExpression::try_new([], f64::MIN_POSITIVE),
    );
    assert!(matches!(
        constant_underflow,
        Err(
            ThicknessCanonicalizationError::ScaledGradientConstantNotRepresentable {
                constraint_index: 0,
                axis: 0,
                ..
            }
        )
    ));

    let exact_zero_constant = compile_level_problem::<1>()?
        .try_compose_local_normal_thickness([local_constraint::<1>(400, 2.0)?], |_, _| {
            AffineExpression::try_new([AffineTerm::try_new(2, 1.0)?], 0.0)
        })?;
    assert_eq!(
        exact_zero_constant.canonical_problem().second_order_cones()[0].lhs()[0]
            .constant()
            .to_bits(),
        0.0_f64.to_bits()
    );
    Ok(())
}

#[test]
fn local_constraint_is_immutable_send_and_sync() -> TestResult {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<LocalNormalThickness<1>>();
    assert_send_sync::<LocalNormalThickness<2>>();
    assert_send_sync::<LocalNormalThickness<3>>();
    let constraint = local_constraint::<3>(400, 2.5)?;
    assert_eq!(constraint.lower(), LevelId::new(10));
    assert_eq!(constraint.upper(), LevelId::new(20));
    assert_eq!(constraint.point(), Point::try_new([0.5, 0.5, 0.5])?);
    assert_eq!(constraint.minimum_thickness().to_bits(), 2.5_f64.to_bits());
    assert_eq!(
        constraint.provenance().observation_id(),
        ObservationId::new(400)
    );
    Ok(())
}
