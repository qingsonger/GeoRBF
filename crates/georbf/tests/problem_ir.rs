//! Independent semantic-to-canonical problem IR integration tests.

#![allow(clippy::expect_used, clippy::float_cmp, clippy::unwrap_used)]

use std::error::Error;
use std::fmt;
use std::num::NonZeroUsize;

use georbf::{
    AffineExpression, AffineTerm, CanonicalSoftRelation, CanonicalizationError, Enforcement,
    ExecutionOptions, FunctionalAtom, FunctionalExpr, FunctionalProvenance, FunctionalTerm,
    ObservationFunctional, ObservationId, Point, ProblemIrError, ProblemIrStorage,
    SemanticConstraint, SemanticExpression, SemanticMetadataField, SemanticProblemIr,
    SemanticProvenance, SemanticRelation, SoftLoss, SourceLocation, VariableBlock,
};

type TestResult = Result<(), Box<dyn Error>>;

fn functional<const D: usize>(atom_id: u64, point: [f64; D]) -> ObservationFunctional<D>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    let atom = FunctionalAtom::value(
        Point::try_new(point).expect("test point is finite"),
        FunctionalProvenance::new(atom_id),
    );
    let term = FunctionalTerm::try_new(1.0, atom).expect("test coefficient is finite");
    ObservationFunctional::new(FunctionalExpr::try_new([term]).expect("one term is nonempty"))
}

fn provenance(identifier: u64) -> Result<SemanticProvenance, ProblemIrError> {
    SemanticProvenance::try_new(
        ObservationId::new(identifier),
        SourceLocation::try_new(
            format!("observations/{identifier}.csv"),
            NonZeroUsize::new(1).expect("positive test line"),
        )?,
        "m".to_owned(),
        format!("fields.scalar.observations[{identifier}]"),
        Some("test-group".to_owned()),
    )
}

fn semantic_expression<const D: usize>(
    atom_id: u64,
    point: [f64; D],
    constant: f64,
) -> Result<SemanticExpression<D>, ProblemIrError>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    SemanticExpression::try_new(functional(atom_id, point), constant)
}

fn constraint<const D: usize>(
    identifier: u64,
    relation: SemanticRelation<D>,
) -> Result<SemanticConstraint<D>, ProblemIrError>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    SemanticConstraint::try_new(provenance(identifier)?, relation, Enforcement::Hard)
}

fn affine(terms: &[(usize, f64)], constant: f64) -> Result<AffineExpression, ProblemIrError> {
    AffineExpression::try_new(
        terms
            .iter()
            .map(|(variable, coefficient)| AffineTerm::try_new(*variable, *coefficient))
            .collect::<Result<Vec<_>, _>>()?,
        constant,
    )
}

fn assert_affine_terms(expression: &AffineExpression, expected: &[(usize, f64)]) {
    let actual = expression
        .terms()
        .iter()
        .map(|term| (term.variable(), term.coefficient()))
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}

#[test]
fn provenance_retains_every_semantic_source_field() -> TestResult {
    let provenance = SemanticProvenance::try_new(
        ObservationId::new(42),
        SourceLocation::try_new(
            "input/model.yaml".to_owned(),
            NonZeroUsize::new(17).unwrap(),
        )?,
        "feet".to_owned(),
        "fields.grade.constraints[3]".to_owned(),
        Some("drillholes-A".to_owned()),
    )?;

    assert_eq!(provenance.observation_id().identifier(), 42);
    assert_eq!(provenance.source().path(), "input/model.yaml");
    assert_eq!(provenance.source().line().get(), 17);
    assert_eq!(provenance.original_units(), "feet");
    assert_eq!(provenance.field_path(), "fields.grade.constraints[3]");
    assert_eq!(provenance.constraint_group(), Some("drillholes-A"));
    Ok(())
}

#[test]
fn equality_bounds_and_cones_map_with_exact_constant_shifts() -> TestResult {
    let constraints = [
        constraint(
            1,
            SemanticRelation::Equality {
                expression: semantic_expression(11, [0.0, 0.0], 1.5)?,
                target: 10.0,
            },
        )?,
        constraint(
            2,
            SemanticRelation::LinearBound {
                expression: semantic_expression(21, [1.0, 0.0], -1.0)?,
                lower: Some(2.0),
                upper: Some(7.0),
            },
        )?,
        constraint(
            3,
            SemanticRelation::SecondOrderCone {
                lhs: vec![
                    semantic_expression(31, [0.0, 1.0], 0.25)?,
                    semantic_expression(32, [1.0, 1.0], -0.5)?,
                ],
                rhs: semantic_expression(33, [2.0, 1.0], 1.0)?,
            },
        )?,
    ];
    let problem = SemanticProblemIr::try_new(constraints, ExecutionOptions::default())?;
    let canonical = problem.try_compile(
        [
            VariableBlock::try_new("field".to_owned(), NonZeroUsize::new(2).unwrap())?,
            VariableBlock::try_new("level".to_owned(), NonZeroUsize::new(1).unwrap())?,
        ],
        |expression, _| {
            let atom_id = expression.expression().terms()[0]
                .atom()
                .provenance()
                .identifier();
            match atom_id {
                11 => affine(&[(0, 2.0), (2, -1.0)], 0.5),
                21 => affine(&[(1, 3.0)], 3.0),
                31 => affine(&[(0, 1.0)], 0.75),
                32 => affine(&[(1, -2.0)], 1.5),
                33 => affine(&[(0, 0.5), (2, 4.0)], 2.0),
                _ => unreachable!("known atom ids"),
            }
        },
    )?;

    assert_eq!(canonical.variable_count(), 3);
    assert_eq!(
        canonical.variable_blocks().collect::<Vec<_>>(),
        vec![("field", 0, 2), ("level", 2, 1)]
    );
    assert_eq!(canonical.equalities().len(), 1);
    let equality = &canonical.equalities()[0];
    assert_affine_terms(equality.row(), &[(0, 2.0), (2, -1.0)]);
    assert_eq!(equality.rhs(), 8.0);
    assert_eq!(equality.row().constant(), 0.0);
    assert_eq!(
        equality.provenance().observation_id(),
        ObservationId::new(1)
    );

    let linear_bound = &canonical.linear_bounds()[0];
    assert_affine_terms(linear_bound.row(), &[(1, 3.0)]);
    assert_eq!(linear_bound.lower(), Some(0.0));
    assert_eq!(linear_bound.upper(), Some(5.0));
    assert_eq!(linear_bound.row().constant(), 0.0);

    let cone = &canonical.second_order_cones()[0];
    assert_affine_terms(&cone.lhs()[0], &[(0, 1.0)]);
    assert_affine_terms(&cone.lhs()[1], &[(1, -2.0)]);
    assert_affine_terms(cone.rhs(), &[(0, 0.5), (2, 4.0)]);
    assert_eq!(cone.lhs()[0].constant(), 1.0);
    assert_eq!(cone.lhs()[1].constant(), 1.0);
    assert_eq!(cone.rhs().constant(), 3.0);
    assert_eq!(cone.provenance().observation_id(), ObservationId::new(3));
    assert_eq!(cone.provenance().source().path(), "observations/3.csv");
    assert_eq!(cone.provenance().source().line().get(), 1);
    assert_eq!(cone.provenance().original_units(), "m");
    assert_eq!(
        cone.provenance().field_path(),
        "fields.scalar.observations[3]"
    );
    assert_eq!(cone.provenance().constraint_group(), Some("test-group"));

    assert_eq!(canonical.scaling().variable(), &[1.0, 1.0, 1.0]);
    assert_eq!(canonical.scaling().equality(), &[1.0]);
    assert_eq!(canonical.scaling().linear_bound(), &[1.0]);
    assert_eq!(canonical.scaling().cone(), &[1.0]);
    assert!(canonical.scaling().soft_objective().is_empty());
    assert!(canonical.capabilities().has_equalities);
    assert!(canonical.capabilities().has_linear_bounds);
    assert!(canonical.capabilities().has_second_order_cones);
    assert!(!canonical.capabilities().soft_objectives.has_any());
    assert_eq!(canonical.memory_estimate().coefficient_count, 7);
    assert!(canonical.memory_estimate().numeric_bytes > 0);
    Ok(())
}

fn compile_dimension<const D: usize>(point: [f64; D]) -> TestResult
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    let problem = SemanticProblemIr::try_new(
        [constraint(
            D as u64,
            SemanticRelation::Equality {
                expression: semantic_expression(100 + D as u64, point, 0.0)?,
                target: 1.0,
            },
        )?],
        ExecutionOptions::new(true, NonZeroUsize::new(2), NonZeroUsize::new(4096)),
    )?;
    assert!(problem.execution_options().deterministic());
    let canonical = problem.try_compile(
        [VariableBlock::try_new(
            "coefficients".to_owned(),
            NonZeroUsize::new(D).unwrap(),
        )?],
        |_, _| affine(&[(0, 1.0)], 0.0),
    )?;
    assert_eq!(canonical.equalities()[0].rhs(), 1.0);
    Ok(())
}

#[test]
fn compilation_is_available_in_d1_d2_and_d3() -> TestResult {
    compile_dimension([0.0])?;
    compile_dimension([0.0, 1.0])?;
    compile_dimension([0.0, 1.0, 2.0])?;
    Ok(())
}

#[test]
fn repeated_compilation_is_deterministic_and_preserves_class_order() -> TestResult {
    let constraints = [
        constraint(
            10,
            SemanticRelation::LinearBound {
                expression: semantic_expression(1, [0.0], 0.0)?,
                lower: Some(-1.0),
                upper: None,
            },
        )?,
        constraint(
            11,
            SemanticRelation::Equality {
                expression: semantic_expression(2, [1.0], 0.0)?,
                target: 2.0,
            },
        )?,
        constraint(
            12,
            SemanticRelation::LinearBound {
                expression: semantic_expression(3, [2.0], 0.0)?,
                lower: None,
                upper: Some(4.0),
            },
        )?,
    ];
    let problem = SemanticProblemIr::try_new(constraints, ExecutionOptions::default())?;
    let compile = || {
        problem.try_compile(
            [VariableBlock::try_new("z".to_owned(), NonZeroUsize::new(1).unwrap()).unwrap()],
            |_, provenance| match provenance.observation_id().identifier() {
                10 => affine(&[(0, 10.0)], 0.0),
                11 => affine(&[(0, 11.0)], 0.0),
                12 => affine(&[(0, 12.0)], 0.0),
                _ => unreachable!("known observation identifiers"),
            },
        )
    };
    let first = compile()?;
    let second = compile()?;
    assert_eq!(first, second);
    assert_eq!(
        first
            .linear_bounds()
            .iter()
            .map(|row| row.provenance().observation_id().identifier())
            .collect::<Vec<_>>(),
        vec![10, 12]
    );
    Ok(())
}

#[test]
fn invalid_metadata_relations_and_enforcement_are_rejected() -> TestResult {
    assert_eq!(
        SourceLocation::try_new("  ".to_owned(), NonZeroUsize::new(1).unwrap()),
        Err(ProblemIrError::EmptyMetadata {
            field: SemanticMetadataField::SourcePath
        })
    );
    assert!(matches!(
        SemanticProvenance::try_new(
            ObservationId::new(1),
            SourceLocation::try_new("a".to_owned(), NonZeroUsize::new(1).unwrap())?,
            "m".to_owned(),
            "field".to_owned(),
            Some(String::new()),
        ),
        Err(ProblemIrError::EmptyMetadata {
            field: SemanticMetadataField::ConstraintGroup
        })
    ));

    let expression = semantic_expression(1, [0.0], 0.0)?;
    assert!(matches!(
        SemanticConstraint::try_new(
            provenance(1)?,
            SemanticRelation::LinearBound {
                expression: expression.clone(),
                lower: None,
                upper: None,
            },
            Enforcement::Hard,
        ),
        Err(ProblemIrError::MissingLinearBound)
    ));
    assert!(matches!(
        SemanticConstraint::try_new(
            provenance(2)?,
            SemanticRelation::LinearBound {
                expression: expression.clone(),
                lower: Some(2.0),
                upper: Some(1.0),
            },
            Enforcement::Hard,
        ),
        Err(ProblemIrError::ReversedLinearBounds { .. })
    ));
    assert!(matches!(
        SemanticConstraint::try_new(
            provenance(3)?,
            SemanticRelation::SecondOrderCone {
                lhs: Vec::new(),
                rhs: expression.clone(),
            },
            Enforcement::Hard,
        ),
        Err(ProblemIrError::EmptyConeLeftHandSide)
    ));
    assert!(matches!(
        SemanticConstraint::try_new(
            provenance(4)?,
            SemanticRelation::Equality {
                expression,
                target: 0.0,
            },
            Enforcement::Soft {
                scale: 0.0,
                loss: SoftLoss::SquaredL2,
            },
        ),
        Err(ProblemIrError::InvalidSoftScale { value: 0.0 })
    ));
    Ok(())
}

#[test]
fn empty_and_duplicate_semantic_problems_are_rejected() -> TestResult {
    assert!(matches!(
        SemanticProblemIr::<1>::try_new([], ExecutionOptions::default()),
        Err(ProblemIrError::EmptySemanticProblem)
    ));
    let relation = SemanticRelation::Equality {
        expression: semantic_expression(1, [0.0], 0.0)?,
        target: 0.0,
    };
    let duplicated = [constraint(7, relation.clone())?, constraint(7, relation)?];
    assert!(matches!(
        SemanticProblemIr::try_new(duplicated, ExecutionOptions::default()),
        Err(ProblemIrError::DuplicateObservationId {
            identifier
        }) if identifier == ObservationId::new(7)
    ));
    Ok(())
}

#[test]
fn affine_validation_rejects_nonfinite_zero_unsorted_and_duplicate_terms() -> TestResult {
    assert!(matches!(
        AffineTerm::try_new(0, f64::NAN),
        Err(ProblemIrError::NonFiniteAffineCoefficient { .. })
    ));
    assert_eq!(
        AffineTerm::try_new(3, -0.0),
        Err(ProblemIrError::ZeroAffineCoefficient { variable: 3 })
    );
    let first = AffineTerm::try_new(2, 1.0)?;
    let second = AffineTerm::try_new(1, 1.0)?;
    assert!(matches!(
        AffineExpression::try_new([first, second], 0.0),
        Err(ProblemIrError::NonIncreasingAffineIndices {
            previous: 2,
            current: 1
        })
    ));
    assert!(matches!(
        AffineExpression::try_new([first, first], 0.0),
        Err(ProblemIrError::NonIncreasingAffineIndices {
            previous: 2,
            current: 2
        })
    ));
    assert!(matches!(
        AffineExpression::try_new([], f64::INFINITY),
        Err(ProblemIrError::NonFiniteAffineConstant { .. })
    ));
    Ok(())
}

#[test]
fn canonicalization_rejects_variable_space_and_linearizer_output_paths() -> TestResult {
    let problem = SemanticProblemIr::try_new(
        [constraint(
            8,
            SemanticRelation::Equality {
                expression: semantic_expression(1, [0.0], 0.0)?,
                target: 0.0,
            },
        )?],
        ExecutionOptions::default(),
    )?;

    assert!(matches!(
        problem.try_compile::<ProblemIrError>([], |_, _| unreachable!()),
        Err(CanonicalizationError::Ir(
            ProblemIrError::EmptyVariableSpace
        ))
    ));
    assert!(matches!(
        problem.try_compile(
            [VariableBlock::try_new("z".to_owned(), NonZeroUsize::new(1).unwrap())?],
            |_, _| affine(&[(1, 1.0)], 0.0),
        ),
        Err(CanonicalizationError::InvalidLinearization {
            constraint_index: 0,
            expression_index: 0,
            observation_id,
            source: ProblemIrError::AffineVariableOutOfRange {
                variable: 1,
                variable_count: 1
            }
        }) if observation_id == ObservationId::new(8)
    ));
    let overflow_problem = SemanticProblemIr::try_new(
        [constraint(
            9,
            SemanticRelation::Equality {
                expression: semantic_expression(2, [0.0], 0.0)?,
                target: f64::MAX,
            },
        )?],
        ExecutionOptions::default(),
    )?;
    assert!(matches!(
        overflow_problem.try_compile(
            [VariableBlock::try_new(
                "z".to_owned(),
                NonZeroUsize::new(1).unwrap()
            )?],
            |_, _| affine(&[(0, 1.0)], -f64::MAX),
        ),
        Err(CanonicalizationError::NonFiniteShiftedScalar {
            constraint_index: 0,
            observation_id,
            value,
        }) if observation_id == ObservationId::new(9) && value.is_infinite()
    ));
    Ok(())
}

#[derive(Debug)]
struct ForcedLinearizerError;

impl fmt::Display for ForcedLinearizerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("forced linearizer error")
    }
}

impl Error for ForcedLinearizerError {}

#[test]
fn linearizer_errors_and_soft_objectives_retain_source_indices() -> TestResult {
    let hard = SemanticProblemIr::try_new(
        [constraint(
            91,
            SemanticRelation::Equality {
                expression: semantic_expression(1, [0.0], 0.0)?,
                target: 0.0,
            },
        )?],
        ExecutionOptions::default(),
    )?;
    let block = || VariableBlock::try_new("z".to_owned(), NonZeroUsize::new(1).unwrap());
    assert!(matches!(
        hard.try_compile([block()?], |_, _| Err(ForcedLinearizerError)),
        Err(CanonicalizationError::Linearization {
            constraint_index: 0,
            expression_index: 0,
            observation_id,
            ..
        }) if observation_id == ObservationId::new(91)
    ));

    let soft_constraint = SemanticConstraint::try_new(
        provenance(92)?,
        SemanticRelation::Equality {
            expression: semantic_expression(2, [0.0], 0.0)?,
            target: 0.0,
        },
        Enforcement::Soft {
            scale: 2.0,
            loss: SoftLoss::Huber { delta: 0.5 },
        },
    )?;
    let soft = SemanticProblemIr::try_new([soft_constraint], ExecutionOptions::default())?;
    let canonical = soft.try_compile([block()?], |_, _| affine(&[(0, 1.0)], 0.0))?;
    assert!(canonical.equalities().is_empty());
    let [objective] = canonical.soft_objectives() else {
        return Err(std::io::Error::other("expected one soft objective").into());
    };
    assert_eq!(objective.scale(), 2.0);
    assert_eq!(objective.loss(), SoftLoss::Huber { delta: 0.5 });
    assert_eq!(
        objective.provenance().observation_id(),
        ObservationId::new(92)
    );
    let CanonicalSoftRelation::Equality(relation) = objective.relation() else {
        return Err(std::io::Error::other("expected soft equality").into());
    };
    assert_eq!(relation.rhs(), 0.0);
    assert_affine_terms(relation.row(), &[(0, 1.0)]);
    Ok(())
}

struct ImpossibleSizeHint<T> {
    value: Option<T>,
}

impl<T> Iterator for ImpossibleSizeHint<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.value.take()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::MAX, Some(usize::MAX))
    }
}

#[test]
fn allocation_and_count_overflow_paths_fail_without_partial_success() -> TestResult {
    let one = constraint(
        100,
        SemanticRelation::Equality {
            expression: semantic_expression(1, [0.0], 0.0)?,
            target: 0.0,
        },
    )?;
    assert!(matches!(
        SemanticProblemIr::try_new(
            ImpossibleSizeHint { value: Some(one) },
            ExecutionOptions::default()
        ),
        Err(ProblemIrError::AllocationFailed {
            storage: ProblemIrStorage::SemanticConstraints,
            requested: usize::MAX
        })
    ));

    let term = AffineTerm::try_new(0, 1.0)?;
    assert!(matches!(
        AffineExpression::try_new(ImpossibleSizeHint { value: Some(term) }, 0.0),
        Err(ProblemIrError::AllocationFailed {
            storage: ProblemIrStorage::AffineTerms,
            requested: usize::MAX
        })
    ));

    let problem = SemanticProblemIr::try_new(
        [constraint(
            101,
            SemanticRelation::Equality {
                expression: semantic_expression(2, [0.0], 0.0)?,
                target: 0.0,
            },
        )?],
        ExecutionOptions::default(),
    )?;
    let huge = VariableBlock::try_new("huge".to_owned(), NonZeroUsize::new(usize::MAX).unwrap())?;
    let one = VariableBlock::try_new("one".to_owned(), NonZeroUsize::new(1).unwrap())?;
    assert!(matches!(
        problem.try_compile::<ProblemIrError>([huge, one], |_, _| unreachable!()),
        Err(CanonicalizationError::Ir(
            ProblemIrError::VariableCountOverflow
        ))
    ));
    Ok(())
}

#[test]
fn public_ir_values_are_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<SemanticProblemIr<1>>();
    assert_send_sync::<SemanticProblemIr<2>>();
    assert_send_sync::<SemanticProblemIr<3>>();
    assert_send_sync::<georbf::CanonicalProblem>();
}
