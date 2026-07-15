//! Dependency-free deterministic semantic-to-canonical IR benchmark.

use std::error::Error;
use std::hint::black_box;
use std::num::NonZeroUsize;
use std::time::Instant;

use georbf::{
    AffineExpression, AffineTerm, Enforcement, ExecutionOptions, FunctionalAtom, FunctionalExpr,
    FunctionalProvenance, FunctionalTerm, ObservationFunctional, ObservationId, Point,
    SemanticConstraint, SemanticExpression, SemanticProblemIr, SemanticProvenance,
    SemanticRelation, SourceLocation, VariableBlock,
};

fn expression<const D: usize>(
    identifier: u64,
    point: [f64; D],
) -> Result<SemanticExpression<D>, Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    let atom = FunctionalAtom::value(
        Point::try_new(point)?,
        FunctionalProvenance::new(identifier),
    );
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
            "benchmark.csv".to_owned(),
            NonZeroUsize::new(1).ok_or("line must be positive")?,
        )?,
        "m".to_owned(),
        format!("field.constraints[{identifier}]"),
        Some("benchmark".to_owned()),
    )?)
}

fn build<const D: usize>(point: [f64; D]) -> Result<SemanticProblemIr<D>, Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    let mut constraints = Vec::new();
    constraints.try_reserve_exact(96)?;
    for index in 0_u64..96 {
        let relation = match index % 3 {
            0 => SemanticRelation::Equality {
                expression: expression(index * 4, point)?,
                target: 2.0,
            },
            1 => SemanticRelation::LinearBound {
                expression: expression(index * 4, point)?,
                lower: Some(-1.0),
                upper: Some(3.0),
            },
            _ => SemanticRelation::SecondOrderCone {
                lhs: vec![
                    expression(index * 4, point)?,
                    expression(index * 4 + 1, point)?,
                ],
                rhs: expression(index * 4 + 2, point)?,
            },
        };
        constraints.push(SemanticConstraint::try_new(
            provenance(index)?,
            relation,
            Enforcement::Hard,
        )?);
    }
    Ok(SemanticProblemIr::try_new(
        constraints,
        ExecutionOptions::default(),
    )?)
}

fn run<const D: usize>(label: &str, point: [f64; D], iterations: u32) -> Result<(), Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    let problem = build(point)?;
    let mut checksum = 0_usize;
    let started = Instant::now();
    for _ in 0..iterations {
        let canonical = black_box(&problem).try_compile(
            [VariableBlock::try_new(
                "z".to_owned(),
                NonZeroUsize::new(D).ok_or("supported dimension is nonzero")?,
            )?],
            |_, _| AffineExpression::try_new([AffineTerm::try_new(0, 1.0)?], 0.5),
        )?;
        checksum = checksum.wrapping_add(black_box(canonical.memory_estimate().numeric_bytes));
    }
    let elapsed = started.elapsed().as_secs_f64() * 1.0e6 / f64::from(iterations);
    println!("{label}: {elapsed:.2} us/compile checksum={checksum}");
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let smoke = std::env::args().any(|argument| argument == "--smoke");
    let iterations = if smoke { 10 } else { 1_000 };
    println!("problem-ir deterministic single-thread benchmark");
    run("D=1", [0.5], iterations)?;
    run("D=2", [0.5, -1.0], iterations)?;
    run("D=3", [0.5, -1.0, 2.0], iterations)?;
    Ok(())
}
