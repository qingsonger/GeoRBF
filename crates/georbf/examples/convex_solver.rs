//! Solve a bounded canonical soft-fit problem with explicit convex policy.

use std::error::Error;
use std::num::{NonZeroU32, NonZeroUsize};

use georbf::{
    AffineExpression, AffineTerm, ConvexSolveOptions, Enforcement, ExecutionOptions,
    FunctionalAtom, FunctionalExpr, FunctionalProvenance, FunctionalTerm, ObservationFunctional,
    ObservationId, Point, SemanticConstraint, SemanticExpression, SemanticProblemIr,
    SemanticProvenance, SemanticRelation, SoftLoss, SourceLocation, VariableBlock,
    try_solve_canonical,
};

fn expression() -> Result<SemanticExpression<1>, Box<dyn Error>> {
    Ok(SemanticExpression::try_new(
        ObservationFunctional::new(FunctionalExpr::try_new([FunctionalTerm::try_new(
            1.0,
            FunctionalAtom::value(Point::try_new([0.0])?, FunctionalProvenance::new(0)),
        )?])?),
        0.0,
    )?)
}

fn provenance(identifier: u64) -> Result<SemanticProvenance, Box<dyn Error>> {
    Ok(SemanticProvenance::try_new(
        ObservationId::new(identifier),
        SourceLocation::try_new(
            "example/convex.csv".to_owned(),
            NonZeroUsize::new(usize::try_from(identifier)? + 1).ok_or("line")?,
        )?,
        "m".to_owned(),
        format!("field.constraints[{identifier}]"),
        Some("example".to_owned()),
    )?)
}

fn main() -> Result<(), Box<dyn Error>> {
    let semantic = SemanticProblemIr::try_new(
        [
            SemanticConstraint::try_new(
                provenance(1)?,
                SemanticRelation::LinearBound {
                    expression: expression()?,
                    lower: Some(0.0),
                    upper: Some(2.0),
                },
                Enforcement::Hard,
            )?,
            SemanticConstraint::try_new(
                provenance(2)?,
                SemanticRelation::Equality {
                    expression: expression()?,
                    target: 1.25,
                },
                Enforcement::Soft {
                    scale: 0.5,
                    loss: SoftLoss::Huber { delta: 1.0 },
                },
            )?,
        ],
        ExecutionOptions::default(),
    )?;
    let canonical = semantic.try_compile(
        [VariableBlock::try_new(
            "field".to_owned(),
            NonZeroUsize::MIN,
        )?],
        |_, _| AffineExpression::try_new([AffineTerm::try_new(0, 1.0)?], 0.0),
    )?;
    let options = ConvexSolveOptions::try_new(
        1.0e-9,
        NonZeroU32::new(300).ok_or("iterations")?,
        Some(10.0),
        NonZeroUsize::new(64 * 1024 * 1024).ok_or("memory")?,
    )?;
    let solution = try_solve_canonical(&canonical, options)?;
    println!(
        "value={:.8}, objective={:.8}, backend={}, hard_violation={:.3e}",
        solution.values()[0],
        solution.diagnostics().kkt.original_objective,
        solution.diagnostics().backend,
        solution.diagnostics().constraints[0].original_residual,
    );
    Ok(())
}
