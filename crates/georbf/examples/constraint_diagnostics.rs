//! Review duplicate and near-duplicate hard functionals without changing them.

use std::error::Error;
use std::num::NonZeroUsize;

use georbf::{
    AffineExpression, AffineTerm, Enforcement, ExecutionOptions, FunctionalAtom, FunctionalExpr,
    FunctionalProvenance, FunctionalTerm, ObservationFunctional, ObservationId, Point,
    SemanticConstraint, SemanticExpression, SemanticProblemIr, SemanticProvenance,
    SemanticRelation, SourceLocation, VariableBlock, try_review_constraints,
};

fn constraint(identifier: u64, target: f64) -> Result<SemanticConstraint<1>, Box<dyn Error>> {
    let atom = FunctionalAtom::value(
        Point::try_new([0.0])?,
        FunctionalProvenance::new(identifier),
    );
    let expression = SemanticExpression::try_new(
        ObservationFunctional::new(FunctionalExpr::try_new([FunctionalTerm::try_new(
            1.0, atom,
        )?])?),
        0.0,
    )?;
    Ok(SemanticConstraint::try_new(
        SemanticProvenance::try_new(
            ObservationId::new(identifier),
            SourceLocation::try_new(
                "example/constraints.csv".to_owned(),
                NonZeroUsize::new(usize::try_from(identifier)?).ok_or("source line")?,
            )?,
            "m".to_owned(),
            format!("field.constraints[{identifier}]"),
            Some("example".to_owned()),
        )?,
        SemanticRelation::Equality { expression, target },
        Enforcement::Hard,
    )?)
}

fn main() -> Result<(), Box<dyn Error>> {
    let semantic = SemanticProblemIr::try_new(
        [
            constraint(1, 3.0)?,
            constraint(2, 6.0)?,
            constraint(3, -3.0)?,
        ],
        ExecutionOptions::default(),
    )?;
    let canonical = semantic.try_compile(
        [VariableBlock::try_new(
            "field".to_owned(),
            NonZeroUsize::new(2).ok_or("variable count")?,
        )?],
        |_, source| {
            let coefficients = match source.observation_id().identifier() {
                1 => [1.0, 2.0],
                2 => [2.0, 4.0],
                3 => [-1.0, -(2.0 + 1.0e-15)],
                _ => unreachable!("known example source"),
            };
            AffineExpression::try_new(
                [
                    AffineTerm::try_new(0, coefficients[0])?,
                    AffineTerm::try_new(1, coefficients[1])?,
                ],
                0.0,
            )
        },
    )?;

    let review = try_review_constraints(&canonical)?;
    for pair in review.pairs {
        println!(
            "{:?}: observations {} and {}, orientation={:?}, normalized distance={:.3e}",
            pair.similarity,
            pair.first_provenance.observation_id().identifier(),
            pair.second_provenance.observation_id().identifier(),
            pair.orientation,
            pair.normalized_row_distance,
        );
    }
    Ok(())
}
