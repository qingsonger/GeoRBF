//! Deterministic explicit-level validation and canonicalization benchmark.

use std::error::Error;
use std::hint::black_box;
use std::num::NonZeroUsize;
use std::time::Instant;

use georbf::{
    AffineExpression, AffineTerm, FunctionalAtom, FunctionalExpr, FunctionalProvenance,
    FunctionalTerm, LevelDefinition, LevelId, LevelMembership, LevelOrder, LevelPrior,
    LevelProblem, LevelValue, ObservationFunctional, ObservationId, Point, SemanticProvenance,
    SoftLoss, SourceLocation, VariableBlock,
};

fn provenance(identifier: u64, field_path: String) -> Result<SemanticProvenance, Box<dyn Error>> {
    Ok(SemanticProvenance::try_new(
        ObservationId::new(identifier),
        SourceLocation::try_new(
            "level-benchmark.csv".to_owned(),
            NonZeroUsize::new(usize::try_from(identifier)? + 1).ok_or("line")?,
        )?,
        "m".to_owned(),
        field_path,
        Some("benchmark".to_owned()),
    )?)
}

fn membership(index: usize, count: usize) -> Result<LevelMembership<1>, Box<dyn Error>> {
    let stable_id = u64::try_from(index)?;
    let coordinate = f64::from(u32::try_from(index)?);
    let expression = FunctionalExpr::try_new([FunctionalTerm::try_new(
        1.0,
        FunctionalAtom::value(
            Point::try_new([coordinate])?,
            FunctionalProvenance::new(stable_id),
        ),
    )?])?;
    let provenance_id = u64::try_from(count)?
        .checked_add(stable_id)
        .ok_or("membership identifier")?;
    Ok(LevelMembership::new(
        LevelId::new(stable_id),
        ObservationFunctional::new(expression),
        provenance(provenance_id, format!("memberships[{index}]"))?,
    ))
}

fn build(count: usize) -> Result<LevelProblem<1>, Box<dyn Error>> {
    let mut levels = Vec::new();
    let mut memberships = Vec::new();
    let mut orders = Vec::new();
    levels.try_reserve_exact(count)?;
    memberships.try_reserve_exact(count)?;
    orders.try_reserve_exact(count.saturating_sub(1))?;
    for index in 0..count {
        let stable_id = u64::try_from(index)?;
        let prior_mean = f64::from(u32::try_from(index)?);
        let value = if index == 0 {
            LevelValue::try_fixed(0.0)?
        } else if index + 1 == count {
            LevelValue::Prior(LevelPrior::try_new(prior_mean, 1.0, SoftLoss::SquaredL2)?)
        } else {
            LevelValue::unknown()
        };
        levels.push(LevelDefinition::new(
            LevelId::new(stable_id),
            value,
            provenance(stable_id, format!("levels[{index}]"))?,
        ));
        memberships.push(membership(index, count)?);
        if index > 0 {
            let order_id = u64::try_from(count)?
                .checked_mul(2)
                .and_then(|base| base.checked_add(stable_id))
                .ok_or("order identifier")?;
            orders.push(LevelOrder::try_new(
                LevelId::new(stable_id - 1),
                LevelId::new(stable_id),
                0.5,
                provenance(order_id, format!("orders[{}]", index - 1))?,
            )?);
        }
    }
    Ok(LevelProblem::try_new(levels, memberships, orders)?)
}

fn run(iterations: u32) -> Result<(), Box<dyn Error>> {
    let count = 64_usize;
    let started = Instant::now();
    let mut checksum = 0_usize;
    for _ in 0..iterations {
        let problem = black_box(build(count)?);
        let mut field_variable = 0_usize;
        let compiled = problem.try_compile(
            [VariableBlock::try_new(
                "field".to_owned(),
                NonZeroUsize::new(count).ok_or("field count")?,
            )?],
            |_, _| {
                let variable = field_variable;
                field_variable += 1;
                AffineExpression::try_new([AffineTerm::try_new(variable, 1.0)?], 0.0)
            },
        )?;
        checksum = checksum
            .checked_add(black_box(
                compiled.canonical_problem().memory_estimate().numeric_bytes,
            ))
            .ok_or("checksum overflow")?;
    }
    let elapsed = started.elapsed().as_secs_f64() * 1.0e6 / f64::from(iterations);
    println!("D=1 levels={count}: {elapsed:.2} us/validate+compile checksum={checksum}");
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let smoke = std::env::args().any(|argument| argument == "--smoke");
    let iterations = if smoke { 2 } else { 1_000 };
    println!("explicit-level deterministic validation and compilation benchmark");
    run(iterations)
}
