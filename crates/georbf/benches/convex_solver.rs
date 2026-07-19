//! Deterministic sparse canonical QP production-adapter benchmark.

use std::error::Error;
use std::hint::black_box;
use std::num::{NonZeroU32, NonZeroUsize};
use std::time::Instant;

use georbf::{
    AffineExpression, AffineTerm, ConvexSolveOptions, Enforcement, ExecutionOptions,
    FunctionalAtom, FunctionalExpr, FunctionalProvenance, FunctionalTerm, ObservationFunctional,
    ObservationId, Point, SemanticConstraint, SemanticExpression, SemanticProblemIr,
    SemanticProvenance, SemanticRelation, SoftLoss, SourceLocation, VariableBlock,
    try_solve_canonical,
};

fn expression(identifier: u64) -> Result<SemanticExpression<1>, Box<dyn Error>> {
    Ok(SemanticExpression::try_new(
        ObservationFunctional::new(FunctionalExpr::try_new([FunctionalTerm::try_new(
            1.0,
            FunctionalAtom::value(
                Point::try_new([0.0])?,
                FunctionalProvenance::new(identifier),
            ),
        )?])?),
        0.0,
    )?)
}

fn provenance(identifier: u64) -> Result<SemanticProvenance, Box<dyn Error>> {
    Ok(SemanticProvenance::try_new(
        ObservationId::new(identifier),
        SourceLocation::try_new(
            "bench/convex.csv".to_owned(),
            NonZeroUsize::new(usize::try_from(identifier)? + 1).ok_or("line")?,
        )?,
        "1".to_owned(),
        format!("benchmark[{identifier}]"),
        Some("convex-benchmark".to_owned()),
    )?)
}

fn problem(size: usize) -> Result<georbf::CanonicalProblem, Box<dyn Error>> {
    let mut constraints = Vec::new();
    constraints.try_reserve_exact(size * 2)?;
    for variable in 0..size {
        let identifier = u64::try_from(variable)?;
        let denominator = f64::from(u32::try_from(size + 1)?);
        let target = f64::from(u32::try_from(variable + 1)?) / denominator;
        constraints.push(SemanticConstraint::try_new(
            provenance(identifier * 2)?,
            SemanticRelation::LinearBound {
                expression: expression(identifier)?,
                lower: Some(0.0),
                upper: Some(1.0),
            },
            Enforcement::Hard,
        )?);
        constraints.push(SemanticConstraint::try_new(
            provenance(identifier * 2 + 1)?,
            SemanticRelation::Equality {
                expression: expression(identifier)?,
                target,
            },
            Enforcement::Soft {
                scale: 1.0,
                loss: SoftLoss::SquaredL2,
            },
        )?);
    }
    let semantic = SemanticProblemIr::try_new(constraints, ExecutionOptions::default())?;
    Ok(semantic.try_compile(
        [VariableBlock::try_new(
            "z".to_owned(),
            NonZeroUsize::new(size).ok_or("size")?,
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
    )?)
}

fn run(size: usize, iterations: u32) -> Result<(), Box<dyn Error>> {
    let problem = problem(size)?;
    let options = ConvexSolveOptions::try_new(
        1.0e-9,
        NonZeroU32::new(300).ok_or("iterations")?,
        Some(10.0),
        NonZeroUsize::new(256 * 1024 * 1024).ok_or("memory")?,
    )?;
    let started = Instant::now();
    let mut checksum = 0.0;
    for _ in 0..iterations {
        let solution = try_solve_canonical(black_box(&problem), black_box(options))?;
        checksum += black_box(solution.values().iter().sum::<f64>());
    }
    let elapsed = started.elapsed().as_secs_f64() * 1.0e3 / f64::from(iterations);
    println!("clarabel-0.11.1,qp,{size},{iterations},{elapsed:.6},{checksum:.17e}");
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let smoke = std::env::args().any(|argument| argument == "--smoke");
    let sizes: &[usize] = if smoke { &[8, 16] } else { &[16, 32, 64] };
    let iterations = if smoke { 1 } else { 3 };
    println!("backend,problem,size,iterations,milliseconds_per_solve,checksum");
    for size in sizes {
        run(*size, iterations)?;
    }
    Ok(())
}
