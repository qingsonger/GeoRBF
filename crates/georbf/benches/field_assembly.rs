//! Deterministic mixed-functional dense field-assembly benchmark.

use std::error::Error;
use std::hint::black_box;
use std::io;
use std::num::NonZeroUsize;
use std::time::Instant;

use georbf::{
    CenterRepresenter, Enforcement, ExecutionOptions, FieldProblem, FunctionalAtom, FunctionalExpr,
    FunctionalProvenance, FunctionalTerm, Gaussian, ObservationFunctional, ObservationId, Point,
    RadialSeparation, SemanticConstraint, SemanticExpression, SemanticProblemIr,
    SemanticProvenance, SemanticRelation, SourceLocation, SpatialKernelJet, UnitDirection,
};

fn build<const D: usize>(count: usize) -> Result<FieldProblem<D>, Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    let mut constraints = Vec::new();
    let mut centers = Vec::new();
    constraints.try_reserve_exact(count)?;
    centers.try_reserve_exact(count)?;
    for index in 0..count {
        let index_u32 = u32::try_from(index)?;
        let mut point = [0.0; D];
        for (axis, value) in point.iter_mut().enumerate() {
            let axis_u32 = u32::try_from(axis)?;
            *value = f64::from(index_u32) * 0.075 + f64::from(axis_u32) * 0.2;
        }
        let mut direction = [0.0; D];
        direction
            .first_mut()
            .ok_or("supported dimensions are nonzero")?
            .clone_from(&1.0);
        if let Some(second) = direction.get_mut(1) {
            *second = 0.5;
        }
        if let Some(third) = direction.get_mut(2) {
            *third = -0.25;
        }
        let point = Point::try_new(point)?;
        let identifier = u64::try_from(index)? * 2;
        let expression = FunctionalExpr::try_new([
            FunctionalTerm::try_new(
                1.0,
                FunctionalAtom::value(point, FunctionalProvenance::new(identifier)),
            )?,
            FunctionalTerm::try_new(
                0.125,
                FunctionalAtom::directional_derivative(
                    point,
                    UnitDirection::try_new(direction)?,
                    FunctionalProvenance::new(identifier + 1),
                ),
            )?,
        ])?;
        centers.push(CenterRepresenter::new(expression.clone()));
        let provenance = SemanticProvenance::try_new(
            ObservationId::new(u64::try_from(index)?),
            SourceLocation::try_new(
                "field-benchmark.csv".to_owned(),
                NonZeroUsize::new(index + 1).ok_or("line")?,
            )?,
            "m".to_owned(),
            format!("field.equalities[{index}]"),
            Some("benchmark".to_owned()),
        )?;
        constraints.push(SemanticConstraint::try_new(
            provenance,
            SemanticRelation::Equality {
                expression: SemanticExpression::try_new(
                    ObservationFunctional::new(expression),
                    0.0,
                )?,
                target: f64::from(index_u32) * 0.01,
            },
            Enforcement::Hard,
        )?);
    }
    let semantic = SemanticProblemIr::try_new(constraints, ExecutionOptions::default())?;
    Ok(FieldProblem::try_new(semantic, centers)?)
}

fn run<const D: usize>(label: &str, iterations: u32) -> Result<(), Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    let problem = build::<D>(32)?;
    let kernel = Gaussian::try_new(1.25)?;
    let mut checksum = 0.0;
    let started = Instant::now();
    for _ in 0..iterations {
        let system = black_box(&problem).try_assemble(kernel.metadata(), |query, center, _| {
            let separation = RadialSeparation::try_new(query, center)
                .map_err(|error| io::Error::other(error.to_string()))?;
            let radial = kernel
                .radial_jet(separation)
                .map_err(|error| io::Error::other(error.to_string()))?;
            Ok::<_, io::Error>(
                SpatialKernelJet::try_new(separation, radial)
                    .map_err(|error| io::Error::other(error.to_string()))?
                    .into(),
            )
        })?;
        checksum += black_box(system.matrix().values().iter().sum::<f64>());
        checksum += black_box(system.rhs().iter().sum::<f64>());
    }
    let elapsed = started.elapsed().as_secs_f64() * 1.0e6 / f64::from(iterations);
    println!("{label}: {elapsed:.2} us/assembly checksum={checksum:.17e}");
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let smoke = std::env::args().any(|argument| argument == "--smoke");
    let iterations = if smoke { 2 } else { 100 };
    println!("field-assembly deterministic single-thread benchmark");
    run::<1>("D=1", iterations)?;
    run::<2>("D=2", iterations)?;
    run::<3>("D=3", iterations)?;
    Ok(())
}
