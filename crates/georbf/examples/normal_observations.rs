//! Compile an angular normal observation into solver-neutral scalar relations.

use std::error::Error;
use std::num::NonZeroUsize;

use georbf::{
    AffineExpression, AffineTerm, AngleUnit, Enforcement, ExecutionOptions, FunctionalAtom,
    NormalObservation, ObservationId, Point, ProblemIrError, SemanticProblemIr, SemanticProvenance,
    SourceLocation, UnitDirection, VariableBlock,
};

fn provenance(identifier: u64) -> Result<SemanticProvenance, ProblemIrError> {
    SemanticProvenance::try_new(
        ObservationId::new(identifier),
        SourceLocation::try_new("example/normals.csv".to_owned(), NonZeroUsize::MIN)?,
        "1/m".to_owned(),
        "fields.stratigraphy.normals".to_owned(),
        Some("angular-normal".to_owned()),
    )
}

fn main() -> Result<(), Box<dyn Error>> {
    let observation = NormalObservation::try_angular_cone(
        [provenance(1)?, provenance(2)?],
        Point::try_new([10.0, 20.0, -5.0])?,
        UnitDirection::try_new([0.0, 0.0, 1.0])?,
        15.0,
        AngleUnit::Degrees,
        0.05,
        Enforcement::Hard,
    )?;
    let semantic =
        SemanticProblemIr::try_new(observation.into_constraints(), ExecutionOptions::default())?;
    let canonical = semantic.try_compile(
        [VariableBlock::try_new(
            "local-gradient".to_owned(),
            NonZeroUsize::new(3).ok_or("gradient block")?,
        )?],
        |functional, _| -> Result<AffineExpression, ProblemIrError> {
            let mut coefficients = [0.0; 3];
            for term in functional.expression().terms() {
                let FunctionalAtom::DirectionalDerivative { direction, .. } = term.atom() else {
                    return Err(ProblemIrError::VariableCountOverflow);
                };
                for (axis, component) in direction.components().iter().copied().enumerate() {
                    coefficients[axis] += term.coefficient() * component;
                }
            }
            AffineExpression::try_new(
                coefficients
                    .into_iter()
                    .enumerate()
                    .filter(|(_, coefficient)| *coefficient != 0.0)
                    .map(|(axis, coefficient)| AffineTerm::try_new(axis, coefficient))
                    .collect::<Result<Vec<_>, _>>()?,
                0.0,
            )
        },
    )?;

    println!(
        "cones={}, projection lower={:?}, geological terms in solver=none",
        canonical.second_order_cones().len(),
        canonical.linear_bounds()[0].lower()
    );
    Ok(())
}
