//! Deterministic mixed linear-semantic compilation benchmark.

use std::error::Error;
use std::hint::black_box;
use std::num::NonZeroUsize;
use std::time::Instant;

use georbf::{
    AffineExpression, AffineTerm, Enforcement, ExecutionOptions, FunctionalAtom, FunctionalExpr,
    FunctionalProvenance, FunctionalTerm, InsideOrientation, LinearConstraint, MonotonicitySense,
    ObservationFunctional, ObservationId, Point, ProblemIrError, RegionSide, SemanticProblemIr,
    SemanticProvenance, SourceLocation, UnitDirection, VariableBlock,
};

const CONSTRAINT_COUNT: usize = 96;
const VARIABLE_COUNT: usize = CONSTRAINT_COUNT * 2;

fn provenance(identifier: u64) -> Result<SemanticProvenance, Box<dyn Error>> {
    Ok(SemanticProvenance::try_new(
        ObservationId::new(identifier),
        SourceLocation::try_new(
            "linear-benchmark.csv".to_owned(),
            NonZeroUsize::new(usize::try_from(identifier)? + 1).ok_or("line")?,
        )?,
        "m".to_owned(),
        format!("constraints[{identifier}]"),
        Some("benchmark".to_owned()),
    )?)
}

fn functional(
    variable: usize,
    derivative: bool,
) -> Result<ObservationFunctional<1>, Box<dyn Error>> {
    let provenance = FunctionalProvenance::new(u64::try_from(variable)?);
    let atom = if derivative {
        FunctionalAtom::directional_derivative(
            Point::try_new([0.0])?,
            UnitDirection::try_new([1.0])?,
            provenance,
        )
    } else {
        FunctionalAtom::value(Point::try_new([0.0])?, provenance)
    };
    Ok(ObservationFunctional::new(FunctionalExpr::try_new([
        FunctionalTerm::try_new(1.0, atom)?,
    ])?))
}

fn build() -> Result<SemanticProblemIr<1>, Box<dyn Error>> {
    let mut constraints = Vec::new();
    constraints.try_reserve_exact(CONSTRAINT_COUNT)?;
    for index in 0..CONSTRAINT_COUNT {
        let identifier = u64::try_from(index)?;
        let constraint = match index % 6 {
            0 => LinearConstraint::try_lower(
                provenance(identifier)?,
                functional(index, false)?,
                -1.0,
                Enforcement::Hard,
            )?,
            1 => LinearConstraint::try_upper(
                provenance(identifier)?,
                functional(index, false)?,
                1.0,
                Enforcement::Hard,
            )?,
            2 => LinearConstraint::try_interval(
                provenance(identifier)?,
                functional(index, false)?,
                -0.5,
                0.5,
                Enforcement::Hard,
            )?,
            3 => LinearConstraint::try_region(
                provenance(identifier)?,
                functional(index, false)?,
                0.0,
                RegionSide::Inside,
                InsideOrientation::InsideAtOrBelow,
                Enforcement::Hard,
            )?,
            4 => LinearConstraint::try_scalar_gap(
                provenance(identifier)?,
                functional(index, false)?,
                functional(index + CONSTRAINT_COUNT, false)?,
                0.25,
                Enforcement::Hard,
            )?,
            _ => LinearConstraint::try_monotonicity(
                provenance(identifier)?,
                functional(index, true)?,
                MonotonicitySense::Increasing,
                0.0,
                Enforcement::Hard,
            )?,
        };
        constraints.push(constraint.try_into_semantic_constraint()?);
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
                "field".to_owned(),
                NonZeroUsize::new(VARIABLE_COUNT).ok_or("variables")?,
            )?],
            |functional, _| -> Result<AffineExpression, ProblemIrError> {
                let mut terms = functional
                    .expression()
                    .terms()
                    .iter()
                    .map(|term| {
                        Ok((
                            usize::try_from(term.atom().provenance().identifier())
                                .map_err(|_| ProblemIrError::VariableCountOverflow)?,
                            term.coefficient(),
                        ))
                    })
                    .collect::<Result<Vec<_>, ProblemIrError>>()?;
                terms.sort_by_key(|(variable, _)| *variable);
                AffineExpression::try_new(
                    terms
                        .into_iter()
                        .map(|(variable, coefficient)| AffineTerm::try_new(variable, coefficient))
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
        "D=1 constraints={CONSTRAINT_COUNT}: {elapsed:.2} us/build+compile checksum={checksum}"
    );
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let smoke = std::env::args().any(|argument| argument == "--smoke");
    let iterations = if smoke { 2 } else { 2_000 };
    println!("mixed deterministic linear-semantic compilation benchmark");
    run(iterations)
}
