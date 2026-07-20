//! Deterministic tangent-plus-gauge semantic compilation benchmark.

use std::error::Error;
use std::hint::black_box;
use std::num::NonZeroUsize;
use std::time::Instant;

use georbf::{
    AffineExpression, AffineTerm, DerivativeGaugeAnchor, Enforcement, ExecutionOptions,
    FunctionalAtom, ObservationId, Point, ProblemIrError, SemanticProvenance, SoftLoss,
    SourceLocation, TangentObservation, TangentProblem, UnitDirection, VariableBlock,
};

const TANGENT_COUNT: usize = 30;
const VARIABLE_COUNT: usize = 4;

fn provenance(identifier: u64, units: &str) -> Result<SemanticProvenance, Box<dyn Error>> {
    Ok(SemanticProvenance::try_new(
        ObservationId::new(identifier),
        SourceLocation::try_new("tangent-benchmark.csv".to_owned(), NonZeroUsize::MIN)?,
        units.to_owned(),
        format!("tangents[{identifier}]"),
        Some("benchmark".to_owned()),
    )?)
}

fn build() -> Result<TangentProblem<3>, Box<dyn Error>> {
    let point = Point::try_new([0.0, 0.0, 0.0])?;
    let directions = [
        UnitDirection::try_new([1.0, 0.0, 0.0])?,
        UnitDirection::try_new([0.0, 1.0, 0.0])?,
        UnitDirection::try_new([1.0, 2.0, 3.0])?,
    ];
    let losses = [
        SoftLoss::SquaredL2,
        SoftLoss::AbsoluteL1,
        SoftLoss::Huber { delta: 0.5 },
    ];
    let mut tangents = Vec::new();
    tangents.try_reserve_exact(TANGENT_COUNT)?;
    for index in 0..TANGENT_COUNT {
        let identifier = u64::try_from(index)?
            .checked_add(1)
            .ok_or("identifier overflow")?;
        tangents.push(TangentObservation::try_new(
            provenance(identifier, "1/m")?,
            point,
            directions[index % directions.len()],
            Enforcement::Soft {
                scale: 1.0,
                loss: losses[index % losses.len()],
            },
        )?);
    }
    let gauge = DerivativeGaugeAnchor::try_new(provenance(10_000, "m")?, point, 0.0)?;
    Ok(TangentProblem::try_new(
        tangents,
        Some(gauge),
        ExecutionOptions::default(),
    )?)
}

fn run(iterations: u32) -> Result<(), Box<dyn Error>> {
    let started = Instant::now();
    let mut checksum = 0_usize;
    for _ in 0..iterations {
        let semantic = black_box(build()?).into_semantic_problem();
        let canonical = semantic.try_compile(
            [VariableBlock::try_new(
                "gradient-and-value".to_owned(),
                NonZeroUsize::new(VARIABLE_COUNT).ok_or("variables")?,
            )?],
            |functional, _| -> Result<AffineExpression, ProblemIrError> {
                let mut coefficients = [0.0; VARIABLE_COUNT];
                for term in functional.expression().terms() {
                    match term.atom() {
                        FunctionalAtom::DirectionalDerivative { direction, .. } => {
                            for (axis, component) in
                                direction.components().iter().copied().enumerate()
                            {
                                coefficients[axis] += term.coefficient() * component;
                            }
                        }
                        FunctionalAtom::Value { .. } => {
                            coefficients[VARIABLE_COUNT - 1] += term.coefficient();
                        }
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
        checksum = checksum
            .checked_add(black_box(canonical.memory_estimate().numeric_bytes))
            .ok_or("checksum overflow")?;
    }
    let elapsed = started.elapsed().as_secs_f64() * 1.0e6 / f64::from(iterations);
    println!("D=3 tangents={TANGENT_COUNT}: {elapsed:.2} us/build+compile checksum={checksum}");
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let smoke = std::env::args().any(|argument| argument == "--smoke");
    let iterations = if smoke { 2 } else { 2_000 };
    println!("deterministic tangent-plus-gauge compilation benchmark");
    run(iterations)
}
