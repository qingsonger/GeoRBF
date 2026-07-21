//! Independent project ownership, identifier, and reference-input tests.

use std::error::Error;
use std::num::NonZeroUsize;

use georbf::{
    AffineNormalization, AngleUnit, AxisOrder, CenterRepresenter, ConditionPolicy,
    CoordinateMetadata, CrsMetadata, DenseFactorization, DenseSolveOptions, Enforcement,
    ExecutionOptions, FieldId, FieldProblem, FittedField, FunctionalAtom, FunctionalExpr,
    FunctionalProvenance, FunctionalTerm, Gaussian, GeoProject, GeoProjectError, Handedness,
    KernelDefinition, LengthUnit, ObservationFunctional, ObservationId, Point, ProjectField,
    Regularization, SemanticConstraint, SemanticExpression, SemanticProblemIr, SemanticProvenance,
    SemanticRelation, SourceLocation, VerticalDirection,
};

const TEST_MEMORY_LIMIT_BYTES: usize = 64 * 1024 * 1024;

fn metadata<const D: usize>() -> Result<CoordinateMetadata<D>, Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    Ok(CoordinateMetadata::new(
        LengthUnit::try_new("m")?,
        CrsMetadata::unspecified(),
        AxisOrder::identity(),
        VerticalDirection::Up,
        Handedness::Right,
        AngleUnit::Radians,
    ))
}

fn value_expression<const D: usize>(
    point: [f64; D],
    identifier: u64,
) -> Result<FunctionalExpr<D>, Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    Ok(FunctionalExpr::try_new([FunctionalTerm::try_new(
        1.0,
        FunctionalAtom::value(
            Point::try_new(point)?,
            FunctionalProvenance::new(identifier),
        ),
    )?])?)
}

fn fitted_axis<const D: usize>(
    offset: f64,
    source_seed: u64,
) -> Result<FittedField<D>, Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    let mut constraints = Vec::new();
    let mut centers = Vec::new();
    for (index, (coordinate, target)) in [(-1.0, offset - 1.0), (1.0, offset + 1.0)]
        .into_iter()
        .enumerate()
    {
        let identifier = source_seed + u64::try_from(index)?;
        let point = std::array::from_fn(|axis| if axis == 0 { coordinate } else { 0.0 });
        let expression = value_expression(point, identifier)?;
        centers.push(CenterRepresenter::new(expression.clone()));
        constraints.push(SemanticConstraint::try_new(
            SemanticProvenance::try_new(
                ObservationId::new(identifier),
                SourceLocation::try_new(
                    "project-test.csv".to_owned(),
                    NonZeroUsize::new(index + 1).ok_or("source line")?,
                )?,
                "m".to_owned(),
                format!("fields[{source_seed}].equalities[{index}]"),
                None,
            )?,
            SemanticRelation::Equality {
                expression: SemanticExpression::try_new(
                    ObservationFunctional::new(expression),
                    0.0,
                )?,
                target,
            },
            Enforcement::Hard,
        )?);
    }
    let problem = FieldProblem::try_new(
        SemanticProblemIr::try_new(constraints, ExecutionOptions::default())?,
        centers,
    )?;
    let normalization = AffineNormalization::try_new(
        Point::try_new([0.0; D])?,
        std::array::from_fn(|row| std::array::from_fn(|column| f64::from(row == column))),
    )?;
    let options = DenseSolveOptions::try_new(
        DenseFactorization::Cholesky,
        Regularization::None,
        ConditionPolicy::default(),
        4,
        NonZeroUsize::new(TEST_MEMORY_LIMIT_BYTES).ok_or("memory limit")?,
    )?;
    Ok(FittedField::try_fit(
        problem,
        metadata()?,
        normalization,
        KernelDefinition::from(Gaussian::try_new(1.0)?),
        None,
        options,
    )?)
}

fn fitted_line(offset: f64, source_seed: u64) -> Result<FittedField<1>, Box<dyn Error>> {
    fitted_axis::<1>(offset, source_seed)
}

fn assert_close(actual: f64, expected: f64) {
    let scale = actual.abs().max(expected.abs()).max(1.0);
    assert!(
        (actual - expected).abs() <= 2048.0 * f64::EPSILON * scale,
        "expected {expected:.17e}, got {actual:.17e}"
    );
}

#[test]
fn multiple_fields_retain_independent_models_and_deterministic_identifiers()
-> Result<(), Box<dyn Error>> {
    let first_id = FieldId::new(20);
    let second_id = FieldId::new(10);
    let project = GeoProject::try_new([
        ProjectField::new(first_id, fitted_line(5.0, 100)?),
        ProjectField::new(second_id, fitted_line(-7.0, 200)?),
    ])?;

    assert_eq!(project.len(), 2);
    assert!(!project.is_empty());
    assert_eq!(
        project
            .fields()
            .iter()
            .map(ProjectField::id)
            .collect::<Vec<_>>(),
        [first_id, second_id]
    );
    assert_eq!(first_id.identifier(), 20);
    assert_close(
        project
            .field(first_id)
            .ok_or("missing first field")?
            .try_value(Point::try_new([-1.0])?)?,
        4.0,
    );
    assert_close(
        project
            .field(second_id)
            .ok_or("missing second field")?
            .try_value(Point::try_new([-1.0])?)?,
        -8.0,
    );
    assert_close(
        project
            .field(first_id)
            .ok_or("missing first field")?
            .try_value(Point::try_new([1.0])?)?,
        6.0,
    );
    assert_close(
        project
            .field(second_id)
            .ok_or("missing second field")?
            .try_value(Point::try_new([1.0])?)?,
        -6.0,
    );
    Ok(())
}

#[test]
fn identifiers_are_unique_and_empty_projects_are_rejected() -> Result<(), Box<dyn Error>> {
    let empty = GeoProject::<1>::try_new(Vec::<ProjectField<1>>::new());
    assert!(matches!(empty, Err(GeoProjectError::EmptyFields)));

    let field_id = FieldId::new(42);
    let duplicate = GeoProject::try_new([
        ProjectField::new(field_id, fitted_line(0.0, 300)?),
        ProjectField::new(field_id, fitted_line(1.0, 400)?),
    ]);
    assert!(matches!(
        duplicate,
        Err(GeoProjectError::DuplicateFieldId {
            field_id: duplicate_id,
            first_index: 0,
            duplicate_index: 1,
        }) if duplicate_id == field_id
    ));
    Ok(())
}

#[test]
fn reference_input_resolves_one_field_and_delegates_outputs() -> Result<(), Box<dyn Error>> {
    let reference_id = FieldId::new(7);
    let other_id = FieldId::new(8);
    let project = GeoProject::try_new([
        ProjectField::new(reference_id, fitted_line(2.0, 500)?),
        ProjectField::new(other_id, fitted_line(100.0, 600)?),
    ])?;
    let reference = project.try_reference_input(reference_id)?;
    let point = Point::try_new([-1.0])?;

    assert_eq!(reference.field_id(), reference_id);
    assert!(std::ptr::eq(
        reference.field(),
        project
            .field(reference_id)
            .ok_or("missing reference field")?
    ));
    assert_close(reference.try_value(point)?, 1.0);
    assert_close(reference.try_evaluate(point)?.value(), 1.0);
    assert_close(reference.try_evaluate_with_hessian(point)?.value(), 1.0);
    let _gradient = reference.try_gradient(point)?;
    let _hessian = reference.try_hessian(point)?;

    let missing = FieldId::new(999);
    assert!(matches!(
        project.try_reference_input(missing),
        Err(GeoProjectError::UnknownReferenceField { field_id }) if field_id == missing
    ));
    Ok(())
}

#[test]
fn project_owns_fields_independently_of_source_values() -> Result<(), Box<dyn Error>> {
    let source = fitted_line(3.0, 700)?;
    let expected_weights = source.center_weights().to_vec();
    let project = GeoProject::try_new([ProjectField::new(FieldId::new(1), source.clone())])?;
    drop(source);

    let retained = project.field(FieldId::new(1)).ok_or("missing field")?;
    assert_eq!(retained.center_weights(), expected_weights);
    assert_close(retained.try_value(Point::try_new([1.0])?)?, 4.0);
    Ok(())
}

#[test]
fn project_public_types_are_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<FieldId>();
    assert_send_sync::<ProjectField<1>>();
    assert_send_sync::<ProjectField<2>>();
    assert_send_sync::<ProjectField<3>>();
    assert_send_sync::<GeoProject<1>>();
    assert_send_sync::<GeoProject<2>>();
    assert_send_sync::<GeoProject<3>>();
    assert_send_sync::<georbf::ReferenceFieldInput<'static, 1>>();
}

#[test]
fn projects_construct_in_every_supported_dimension() -> Result<(), Box<dyn Error>> {
    let project_1 = GeoProject::try_new([ProjectField::new(
        FieldId::new(1),
        fitted_axis::<1>(0.0, 800)?,
    )])?;
    let project_2 = GeoProject::try_new([ProjectField::new(
        FieldId::new(2),
        fitted_axis::<2>(0.0, 900)?,
    )])?;
    let project_3 = GeoProject::try_new([ProjectField::new(
        FieldId::new(3),
        fitted_axis::<3>(0.0, 1000)?,
    )])?;

    assert_eq!(project_1.len(), 1);
    assert_eq!(project_2.len(), 1);
    assert_eq!(project_3.len(), 1);
    Ok(())
}
