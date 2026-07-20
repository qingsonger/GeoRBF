//! Deterministic sampled local-thickness compilation benchmark.

use std::error::Error;
use std::hint::black_box;
use std::num::NonZeroUsize;
use std::time::Instant;

use georbf::{
    AffineExpression, AffineTerm, FunctionalAtom, FunctionalExpr, FunctionalProvenance,
    FunctionalTerm, LevelDefinition, LevelId, LevelMembership, LevelOrder, LevelProblem,
    LevelValue, LocalNormalThickness, ObservationFunctional, ObservationId, Point, ProblemIrError,
    SemanticProvenance, SourceLocation, VariableBlock,
};

const CONSTRAINT_COUNT: usize = 32;
const FIELD_VARIABLE_COUNT: usize = 5;

fn provenance(identifier: u64, path: &str) -> Result<SemanticProvenance, Box<dyn Error>> {
    Ok(SemanticProvenance::try_new(
        ObservationId::new(identifier),
        SourceLocation::try_new("thickness-benchmark.csv".to_owned(), NonZeroUsize::MIN)?,
        "m".to_owned(),
        path.to_owned(),
        Some("benchmark".to_owned()),
    )?)
}

fn value(identifier: u64, x: f64) -> Result<ObservationFunctional<3>, Box<dyn Error>> {
    Ok(ObservationFunctional::new(FunctionalExpr::try_new([
        FunctionalTerm::try_new(
            1.0,
            FunctionalAtom::value(
                Point::try_new([x, 0.0, 0.0])?,
                FunctionalProvenance::new(identifier),
            ),
        )?,
    ])?))
}

fn build() -> Result<georbf::CompiledLevelProblem, Box<dyn Error>> {
    let lower = LevelId::new(10);
    let upper = LevelId::new(20);
    let levels = LevelProblem::try_new(
        [
            LevelDefinition::new(
                lower,
                LevelValue::try_fixed(0.0)?,
                provenance(1, "levels.lower")?,
            ),
            LevelDefinition::new(
                upper,
                LevelValue::try_fixed(10.0)?,
                provenance(2, "levels.upper")?,
            ),
        ],
        [
            LevelMembership::new(lower, value(3, 0.0)?, provenance(3, "memberships.lower")?),
            LevelMembership::new(upper, value(4, 10.0)?, provenance(4, "memberships.upper")?),
        ],
        [LevelOrder::try_new(
            lower,
            upper,
            2.0,
            provenance(5, "levels.scalar_gap")?,
        )?],
    )?;
    let mut constraints = Vec::new();
    constraints.try_reserve_exact(CONSTRAINT_COUNT)?;
    for index in 0..CONSTRAINT_COUNT {
        let identifier = 100_u64
            .checked_add(u64::try_from(index)?)
            .ok_or("identifier overflow")?;
        constraints.push(LocalNormalThickness::try_new(
            lower,
            upper,
            Point::try_new([f64::from(u32::try_from(index)?), 0.5, -0.25])?,
            2.0,
            provenance(identifier, &format!("thickness.samples[{index}]"))?,
        )?);
    }
    let mut membership_variable = 0_usize;
    let compiled = levels.try_compile(
        [VariableBlock::try_new(
            "field".to_owned(),
            NonZeroUsize::new(FIELD_VARIABLE_COUNT).ok_or("field variables")?,
        )?],
        |_, _| {
            let variable = membership_variable;
            membership_variable += 1;
            AffineExpression::try_new([AffineTerm::try_new(variable, 1.0)?], 0.0)
        },
    )?;
    Ok(
        compiled.try_compose_local_normal_thickness(constraints, |functional, _| {
            let FunctionalAtom::DirectionalDerivative { direction, .. } =
                functional.expression().terms()[0].atom()
            else {
                return Err(ProblemIrError::MemoryEstimateOverflow);
            };
            let axis = direction
                .components()
                .iter()
                .position(|component| component.to_bits() == 1.0_f64.to_bits())
                .ok_or(ProblemIrError::MemoryEstimateOverflow)?;
            AffineExpression::try_new([AffineTerm::try_new(axis + 2, 1.0)?], 0.0)
        })?,
    )
}

fn run(iterations: u32) -> Result<(), Box<dyn Error>> {
    let started = Instant::now();
    let mut checksum = 0_usize;
    for _ in 0..iterations {
        let compiled = black_box(build()?);
        checksum = checksum
            .checked_add(black_box(
                compiled.canonical_problem().memory_estimate().numeric_bytes,
            ))
            .ok_or("checksum overflow")?;
    }
    let elapsed = started.elapsed().as_secs_f64() * 1.0e6 / f64::from(iterations);
    println!(
        "D=3 local_constraints={CONSTRAINT_COUNT}: {elapsed:.2} us/build+compile checksum={checksum}"
    );
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let smoke = std::env::args().any(|argument| argument == "--smoke");
    let iterations = if smoke { 2 } else { 2_000 };
    println!("scalar-gap plus sampled local-thickness compilation benchmark");
    run(iterations)
}
