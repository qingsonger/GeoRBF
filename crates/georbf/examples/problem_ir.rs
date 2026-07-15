//! Compile provenance-bearing semantic constraints to solver-neutral rows.

use std::error::Error;
use std::num::NonZeroUsize;

use georbf::{
    AffineExpression, AffineTerm, Enforcement, ExecutionOptions, FunctionalAtom, FunctionalExpr,
    FunctionalProvenance, FunctionalTerm, ObservationFunctional, ObservationId, Point,
    SemanticConstraint, SemanticExpression, SemanticProblemIr, SemanticProvenance,
    SemanticRelation, SourceLocation, VariableBlock,
};

fn expression(identifier: u64, x: f64) -> Result<SemanticExpression<1>, Box<dyn Error>> {
    let atom = FunctionalAtom::value(Point::try_new([x])?, FunctionalProvenance::new(identifier));
    let functional =
        ObservationFunctional::new(FunctionalExpr::try_new([FunctionalTerm::try_new(
            1.0, atom,
        )?])?);
    Ok(SemanticExpression::try_new(functional, 0.25)?)
}

fn provenance(identifier: u64) -> Result<SemanticProvenance, Box<dyn Error>> {
    Ok(SemanticProvenance::try_new(
        ObservationId::new(identifier),
        SourceLocation::try_new(
            "example/observations.csv".to_owned(),
            NonZeroUsize::new(2).ok_or("line must be positive")?,
        )?,
        "m".to_owned(),
        format!("field.constraints[{identifier}]"),
        Some("example".to_owned()),
    )?)
}

fn main() -> Result<(), Box<dyn Error>> {
    let equality = SemanticConstraint::try_new(
        provenance(1)?,
        SemanticRelation::Equality {
            expression: expression(10, 0.0)?,
            target: 2.0,
        },
        Enforcement::Hard,
    )?;
    let interval = SemanticConstraint::try_new(
        provenance(2)?,
        SemanticRelation::LinearBound {
            expression: expression(20, 1.0)?,
            lower: Some(-1.0),
            upper: Some(3.0),
        },
        Enforcement::Hard,
    )?;
    let semantic = SemanticProblemIr::try_new([equality, interval], ExecutionOptions::default())?;

    // A later basis/assembly layer supplies this explicit functional-to-row map.
    let canonical = semantic.try_compile(
        [VariableBlock::try_new(
            "field-coefficients".to_owned(),
            NonZeroUsize::new(2).ok_or("block length must be positive")?,
        )?],
        |functional, _| {
            let atom_id = functional.expression().terms()[0]
                .atom()
                .provenance()
                .identifier();
            let variable = usize::from(atom_id != 10);
            AffineExpression::try_new([AffineTerm::try_new(variable, 1.0)?], 0.5)
        },
    )?;

    println!(
        "variables={} equality_rhs={} interval=[{}, {}] source={}",
        canonical.variable_count(),
        canonical.equalities()[0].rhs(),
        canonical.linear_bounds()[0].lower().ok_or("lower bound")?,
        canonical.linear_bounds()[0].upper().ok_or("upper bound")?,
        canonical.equalities()[0].provenance().source().path(),
    );
    Ok(())
}
