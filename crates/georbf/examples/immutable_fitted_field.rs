//! Fit and evaluate an immutable Gaussian scalar field.

use std::error::Error;
use std::num::NonZeroUsize;

use georbf::{
    AffineNormalization, AngleUnit, AxisOrder, CenterRepresenter, ConditionPolicy,
    CoordinateMetadata, CrsMetadata, DenseFactorization, DenseSolveOptions, Enforcement,
    ExecutionOptions, FieldProblem, FittedField, FunctionalAtom, FunctionalExpr,
    FunctionalProvenance, FunctionalTerm, Gaussian, Handedness, KernelDefinition, LengthUnit,
    ObservationFunctional, ObservationId, Point, Regularization, SemanticConstraint,
    SemanticExpression, SemanticProblemIr, SemanticProvenance, SemanticRelation, SourceLocation,
    VerticalDirection,
};

fn main() -> Result<(), Box<dyn Error>> {
    let normalized_center = Point::try_new([0.0])?;
    let expression = FunctionalExpr::try_new([FunctionalTerm::try_new(
        1.0,
        FunctionalAtom::value(normalized_center, FunctionalProvenance::new(1)),
    )?])?;
    let constraint = SemanticConstraint::try_new(
        SemanticProvenance::try_new(
            ObservationId::new(1),
            SourceLocation::try_new(
                "example/model.csv".to_owned(),
                NonZeroUsize::new(1).ok_or("line")?,
            )?,
            "m".to_owned(),
            "field.equalities[0]".to_owned(),
            Some("immutable fitted field example".to_owned()),
        )?,
        SemanticRelation::Equality {
            expression: SemanticExpression::try_new(
                ObservationFunctional::new(expression.clone()),
                0.0,
            )?,
            target: 1.0,
        },
        Enforcement::Hard,
    )?;
    let problem = FieldProblem::try_new(
        SemanticProblemIr::try_new([constraint], ExecutionOptions::default())?,
        [CenterRepresenter::new(expression)],
    )?;
    let metadata = CoordinateMetadata::new(
        LengthUnit::try_new("m")?,
        CrsMetadata::unspecified(),
        AxisOrder::identity(),
        VerticalDirection::Up,
        Handedness::Right,
        AngleUnit::Radians,
    );
    let normalization = AffineNormalization::try_new(Point::try_new([10.0])?, [[2.0]])?;
    let solve_options = DenseSolveOptions::try_new(
        DenseFactorization::Cholesky,
        Regularization::None,
        ConditionPolicy::default(),
        4,
        NonZeroUsize::new(64 * 1024 * 1024).ok_or("memory limit")?,
    )?;
    let model = FittedField::try_fit(
        problem,
        metadata,
        normalization,
        KernelDefinition::from(Gaussian::try_new(1.0)?),
        None,
        solve_options,
    )?;

    let evaluation = model.try_evaluate_with_hessian(Point::try_new([11.0])?)?;
    println!("value = {:.8}", evaluation.value());
    println!(
        "original-coordinate gradient = {:?}",
        evaluation.gradient().components()
    );
    println!("original-coordinate Hessian = {:?}", evaluation.hessian());
    println!("capabilities = {:?}", model.capabilities());
    Ok(())
}
