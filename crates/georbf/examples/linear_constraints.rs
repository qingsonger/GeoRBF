//! Compile explicit linear semantics into solver-neutral bound rows.

use std::error::Error;
use std::num::NonZeroUsize;

use georbf::{
    AffineExpression, AffineTerm, Enforcement, ExecutionOptions, FunctionalAtom, FunctionalExpr,
    FunctionalProvenance, FunctionalTerm, InsideOrientation, LinearConstraint, MonotonicitySense,
    ObservationFunctional, ObservationId, Point, RegionSide, SemanticProblemIr, SemanticProvenance,
    SourceLocation, UnitDirection, VariableBlock,
};

fn provenance(identifier: u64) -> Result<SemanticProvenance, Box<dyn Error>> {
    Ok(SemanticProvenance::try_new(
        ObservationId::new(identifier),
        SourceLocation::try_new(
            "example/linear.csv".to_owned(),
            NonZeroUsize::new(usize::try_from(identifier)? + 1).ok_or("line")?,
        )?,
        "m".to_owned(),
        format!("field.constraints[{identifier}]"),
        Some("example".to_owned()),
    )?)
}

fn value(variable: u64, point: [f64; 2]) -> Result<ObservationFunctional<2>, Box<dyn Error>> {
    Ok(ObservationFunctional::new(FunctionalExpr::try_new([
        FunctionalTerm::try_new(
            1.0,
            FunctionalAtom::value(Point::try_new(point)?, FunctionalProvenance::new(variable)),
        )?,
    ])?))
}

fn derivative(variable: u64) -> Result<ObservationFunctional<2>, Box<dyn Error>> {
    Ok(ObservationFunctional::new(FunctionalExpr::try_new([
        FunctionalTerm::try_new(
            1.0,
            FunctionalAtom::directional_derivative(
                Point::try_new([0.0, 0.0])?,
                UnitDirection::try_new([1.0, 0.0])?,
                FunctionalProvenance::new(variable),
            ),
        )?,
    ])?))
}

fn main() -> Result<(), Box<dyn Error>> {
    let inside = LinearConstraint::try_region(
        provenance(1)?,
        value(0, [0.0, 0.0])?,
        0.0,
        RegionSide::Inside,
        InsideOrientation::InsideAtOrBelow,
        Enforcement::Hard,
    )?
    .try_into_semantic_constraint()?;
    let increasing = LinearConstraint::try_monotonicity(
        provenance(2)?,
        derivative(1)?,
        MonotonicitySense::Increasing,
        0.1,
        Enforcement::Hard,
    )?
    .try_into_semantic_constraint()?;
    let semantic = SemanticProblemIr::try_new([inside, increasing], ExecutionOptions::default())?;

    let canonical = semantic.try_compile(
        [VariableBlock::try_new(
            "field-coefficients".to_owned(),
            NonZeroUsize::new(2).ok_or("block length")?,
        )?],
        |functional, _| {
            let variable = usize::try_from(
                functional.expression().terms()[0]
                    .atom()
                    .provenance()
                    .identifier(),
            )
            .map_err(|_| georbf::ProblemIrError::VariableCountOverflow)?;
            AffineExpression::try_new([AffineTerm::try_new(variable, 1.0)?], 0.0)
        },
    )?;

    println!(
        "inside upper={:?}, monotonic lower={:?}, canonical geological terms=none",
        canonical.linear_bounds()[0].upper(),
        canonical.linear_bounds()[1].lower(),
    );
    Ok(())
}
