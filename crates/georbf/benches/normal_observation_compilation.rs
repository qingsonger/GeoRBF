//! Deterministic mixed normal-observation compilation benchmark.

use std::error::Error;
use std::hint::black_box;
use std::num::NonZeroUsize;
use std::time::Instant;

use georbf::{
    AffineExpression, AffineTerm, AngleUnit, Enforcement, ExecutionOptions, FunctionalAtom,
    NormalObservation, ObservationId, Point, ProblemIrError, SemanticConstraint, SemanticProblemIr,
    SemanticProvenance, SoftLoss, SourceLocation, UnitDirection, VariableBlock, Vector,
};

const OBSERVATION_COUNT: usize = 30;
const VARIABLE_COUNT: usize = 3;

fn provenance(identifier: u64) -> Result<SemanticProvenance, Box<dyn Error>> {
    Ok(SemanticProvenance::try_new(
        ObservationId::new(identifier),
        SourceLocation::try_new("normal-benchmark.csv".to_owned(), NonZeroUsize::MIN)?,
        "1/m".to_owned(),
        format!("normals[{identifier}]"),
        Some("benchmark".to_owned()),
    )?)
}

fn next_provenances(
    next_identifier: &mut u64,
    count: usize,
) -> Result<Vec<SemanticProvenance>, Box<dyn Error>> {
    let mut values = Vec::new();
    values.try_reserve_exact(count)?;
    for _ in 0..count {
        values.push(provenance(*next_identifier)?);
        *next_identifier = next_identifier
            .checked_add(1)
            .ok_or("identifier overflow")?;
    }
    Ok(values)
}

fn build() -> Result<SemanticProblemIr<3>, Box<dyn Error>> {
    let point = Point::try_new([0.0, 0.0, 0.0])?;
    let normal = UnitDirection::try_new([1.0, 2.0, 3.0])?;
    let enforcement = Enforcement::Soft {
        scale: 1.0,
        loss: SoftLoss::SquaredL2,
    };
    let mut next_identifier = 1_u64;
    let mut constraints = Vec::<SemanticConstraint<3>>::new();
    constraints.try_reserve_exact(OBSERVATION_COUNT * 3)?;
    for index in 0..OBSERVATION_COUNT {
        let observation = match index % 5 {
            0 => NormalObservation::try_gradient_vector(
                next_provenances(&mut next_identifier, 3)?,
                point,
                Vector::try_new([0.1, -0.2, 0.3])?,
                enforcement,
            )?,
            1 => NormalObservation::try_direction_only(
                next_provenances(&mut next_identifier, 2)?,
                point,
                normal,
                enforcement,
            )?,
            2 => NormalObservation::try_direction_with_polarity(
                next_provenances(&mut next_identifier, 3)?,
                point,
                normal,
                0.0,
                enforcement,
            )?,
            3 => NormalObservation::try_angular_cone(
                next_provenances(&mut next_identifier, 2)?,
                point,
                normal,
                20.0,
                AngleUnit::Degrees,
                0.0,
                enforcement,
            )?,
            _ => NormalObservation::try_axial_direction(
                next_provenances(&mut next_identifier, 2)?,
                point,
                normal,
                enforcement,
            )?,
        };
        constraints.extend(observation.into_constraints());
    }
    Ok(SemanticProblemIr::try_new(
        constraints,
        ExecutionOptions::default(),
    )?)
}

fn run(iterations: u32) -> Result<(), Box<dyn Error>> {
    let started = Instant::now();
    let mut checksum = 0_usize;
    for _ in 0..iterations {
        let semantic = black_box(build()?);
        let canonical = semantic.try_compile(
            [VariableBlock::try_new(
                "gradient".to_owned(),
                NonZeroUsize::new(VARIABLE_COUNT).ok_or("variables")?,
            )?],
            |functional, _| -> Result<AffineExpression, ProblemIrError> {
                let mut coefficients = [0.0; VARIABLE_COUNT];
                for term in functional.expression().terms() {
                    let FunctionalAtom::DirectionalDerivative { direction, .. } = term.atom()
                    else {
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
        checksum = checksum
            .checked_add(black_box(canonical.memory_estimate().numeric_bytes))
            .ok_or("checksum overflow")?;
    }
    let elapsed = started.elapsed().as_secs_f64() * 1.0e6 / f64::from(iterations);
    println!(
        "D=3 observations={OBSERVATION_COUNT}: {elapsed:.2} us/build+compile checksum={checksum}"
    );
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let smoke = std::env::args().any(|argument| argument == "--smoke");
    let iterations = if smoke { 2 } else { 2_000 };
    println!("mixed deterministic normal-observation compilation benchmark");
    run(iterations)
}
