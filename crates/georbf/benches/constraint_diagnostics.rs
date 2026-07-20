//! Deterministic hard-affine duplicate-review benchmark.

use std::error::Error;
use std::hint::black_box;
use std::num::NonZeroUsize;
use std::time::Instant;

use georbf::{
    AffineExpression, AffineTerm, Enforcement, ExecutionOptions, FunctionalAtom, FunctionalExpr,
    FunctionalProvenance, FunctionalTerm, ObservationFunctional, ObservationId, Point,
    SemanticConstraint, SemanticExpression, SemanticProblemIr, SemanticProvenance,
    SemanticRelation, SourceLocation, VariableBlock, try_review_constraints,
};

const GROUP_COUNT: usize = 32;
const CONSTRAINT_COUNT: usize = GROUP_COUNT * 3;
const VARIABLE_COUNT: usize = GROUP_COUNT * 2;

fn constraint(identifier: u64, target: f64) -> Result<SemanticConstraint<1>, Box<dyn Error>> {
    let atom = FunctionalAtom::value(
        Point::try_new([0.0])?,
        FunctionalProvenance::new(identifier),
    );
    Ok(SemanticConstraint::try_new(
        SemanticProvenance::try_new(
            ObservationId::new(identifier),
            SourceLocation::try_new(
                "constraint-review-benchmark.csv".to_owned(),
                NonZeroUsize::new(usize::try_from(identifier)? + 1).ok_or("source line")?,
            )?,
            "m".to_owned(),
            format!("field.constraints[{identifier}]"),
            Some("benchmark".to_owned()),
        )?,
        SemanticRelation::Equality {
            expression: SemanticExpression::try_new(
                ObservationFunctional::new(FunctionalExpr::try_new([FunctionalTerm::try_new(
                    1.0, atom,
                )?])?),
                0.0,
            )?,
            target,
        },
        Enforcement::Hard,
    )?)
}

fn build() -> Result<georbf::CanonicalProblem, Box<dyn Error>> {
    let mut constraints = Vec::new();
    constraints.try_reserve_exact(CONSTRAINT_COUNT)?;
    for group in 0..GROUP_COUNT {
        let base = u64::try_from(group * 3)?;
        constraints.push(constraint(base, 3.0)?);
        constraints.push(constraint(base + 1, 6.0)?);
        constraints.push(constraint(base + 2, -3.0)?);
    }
    let semantic = SemanticProblemIr::try_new(constraints, ExecutionOptions::default())?;
    Ok(semantic.try_compile(
        [VariableBlock::try_new(
            "field".to_owned(),
            NonZeroUsize::new(VARIABLE_COUNT).ok_or("variable count")?,
        )?],
        |_, source| {
            let identifier = usize::try_from(source.observation_id().identifier())
                .map_err(|_| georbf::ProblemIrError::VariableCountOverflow)?;
            let group = identifier / 3;
            let variables = [group * 2, group * 2 + 1];
            let coefficients = match identifier % 3 {
                0 => [1.0, 2.0],
                1 => [2.0, 4.0],
                _ => [-1.0, -(2.0 + 1.0e-15)],
            };
            AffineExpression::try_new(
                [
                    AffineTerm::try_new(variables[0], coefficients[0])?,
                    AffineTerm::try_new(variables[1], coefficients[1])?,
                ],
                0.0,
            )
        },
    )?)
}

fn main() -> Result<(), Box<dyn Error>> {
    let smoke = std::env::args().any(|argument| argument == "--smoke");
    let iterations = if smoke { 8 } else { 5_000 };
    let problem = build()?;
    let started = Instant::now();
    let mut checksum = 0_usize;
    for _ in 0..iterations {
        checksum = checksum
            .checked_add(
                black_box(try_review_constraints(black_box(&problem))?)
                    .pairs
                    .len(),
            )
            .ok_or("checksum overflow")?;
    }
    let elapsed = started.elapsed().as_secs_f64() * 1.0e6 / f64::from(iterations);
    println!(
        "hard affine constraints={CONSTRAINT_COUNT}: {elapsed:.2} us/review checksum={checksum}"
    );
    Ok(())
}
