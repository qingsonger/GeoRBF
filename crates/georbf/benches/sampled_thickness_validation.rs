//! Deterministic fitted-field sampled geometric thickness benchmark.

use std::error::Error;
use std::hint::black_box;
use std::num::{NonZeroU32, NonZeroUsize};
use std::time::Instant;

use georbf::{
    AffineNormalization, AngleUnit, AxisOrder, CenterRepresenter, ConditionPolicy,
    CoordinateMetadata, CrsMetadata, DenseFactorization, DenseSolveOptions, Enforcement,
    ExecutionOptions, FieldProblem, FittedField, FunctionalAtom, FunctionalExpr,
    FunctionalProvenance, FunctionalTerm, Handedness, KernelDefinition, LengthUnit, LevelId,
    ObservationFunctional, ObservationId, Point, PolyharmonicSpline, Regularization,
    SampledThicknessLocation, SampledThicknessRequest, SampledThicknessSettings,
    SemanticConstraint, SemanticExpression, SemanticProblemIr, SemanticProvenance,
    SemanticRelation, SourceLocation, VerticalDirection,
};

const LOCATION_COUNT: usize = 32;
const TEST_MEMORY_LIMIT_BYTES: usize = 64 * 1024 * 1024;

fn provenance(identifier: u64, path: String) -> Result<SemanticProvenance, Box<dyn Error>> {
    Ok(SemanticProvenance::try_new(
        ObservationId::new(identifier),
        SourceLocation::try_new(
            "sampled-thickness-benchmark.csv".to_owned(),
            NonZeroUsize::MIN,
        )?,
        "m".to_owned(),
        path,
        Some("benchmark".to_owned()),
    )?)
}

fn expression(x: f64, identifier: u64) -> Result<FunctionalExpr<1>, Box<dyn Error>> {
    Ok(FunctionalExpr::try_new([FunctionalTerm::try_new(
        1.0,
        FunctionalAtom::value(Point::try_new([x])?, FunctionalProvenance::new(identifier)),
    )?])?)
}

fn model() -> Result<FittedField<1>, Box<dyn Error>> {
    let mut constraints = Vec::new();
    let mut centers = Vec::new();
    for (index, x) in [-2.0_f64, -1.0, 1.0, 2.0].into_iter().enumerate() {
        let identifier = u64::try_from(index + 1)?;
        let functional = expression(x, identifier)?;
        centers.push(CenterRepresenter::new(functional.clone()));
        constraints.push(SemanticConstraint::try_new(
            provenance(identifier, format!("field.equalities[{index}]"))?,
            SemanticRelation::Equality {
                expression: SemanticExpression::try_new(
                    ObservationFunctional::new(functional),
                    0.0,
                )?,
                target: x,
            },
            Enforcement::Hard,
        )?);
    }
    let problem = FieldProblem::try_new(
        SemanticProblemIr::try_new(constraints, ExecutionOptions::default())?,
        centers,
    )?;
    let metadata = CoordinateMetadata::new(
        LengthUnit::try_new("m")?,
        CrsMetadata::unspecified(),
        AxisOrder::identity(),
        VerticalDirection::Up,
        Handedness::Right,
        AngleUnit::Radians,
    );
    Ok(FittedField::try_fit(
        problem,
        metadata,
        AffineNormalization::try_new(Point::try_new([0.0])?, [[1.0]])?,
        KernelDefinition::from(PolyharmonicSpline::try_new(4)?),
        None,
        DenseSolveOptions::try_new(
            DenseFactorization::PivotedLblt,
            Regularization::None,
            ConditionPolicy::default(),
            4,
            NonZeroUsize::new(TEST_MEMORY_LIMIT_BYTES).ok_or("memory limit")?,
        )?,
    )?)
}

fn request() -> Result<SampledThicknessRequest<1>, Box<dyn Error>> {
    let mut locations = Vec::new();
    locations.try_reserve_exact(LOCATION_COUNT)?;
    for index in 0..LOCATION_COUNT {
        let x = -0.5 + f64::from(u32::try_from(index)?) / 31.0;
        let identifier = 100_u64
            .checked_add(u64::try_from(index)?)
            .ok_or("identifier overflow")?;
        locations.push(SampledThicknessLocation::new(
            Point::try_new([x])?,
            provenance(identifier, format!("thickness.samples[{index}]"))?,
        ));
    }
    Ok(SampledThicknessRequest::try_new(
        LevelId::new(10),
        -1.0,
        LevelId::new(20),
        1.0,
        1.5,
        locations,
        vec![0.0, 0.5, 0.95, 1.0],
        false,
        SampledThicknessSettings::try_new(
            2.0,
            NonZeroU32::new(16).ok_or("search steps")?,
            NonZeroU32::new(48).ok_or("refinement iterations")?,
            1.0e-12,
            1.0e-12,
            1.0e-12,
        )?,
    )?)
}

fn run(iterations: u32) -> Result<(), Box<dyn Error>> {
    let model = model()?;
    let request = request()?;
    let started = Instant::now();
    let mut checksum = 0_usize;
    let mut distance_checksum = 0.0_f64;
    for _ in 0..iterations {
        let report = black_box(model.try_validate_sampled_thickness(black_box(&request))?);
        checksum = checksum
            .checked_add(report.measurements().len())
            .and_then(|value| value.checked_add(report.failures().len()))
            .ok_or("checksum overflow")?;
        distance_checksum += report.minimum().ok_or("benchmark measurement")?;
    }
    let elapsed = started.elapsed().as_secs_f64() * 1.0e6 / f64::from(iterations);
    println!(
        "D=1 locations={LOCATION_COUNT}: {elapsed:.2} us/validation checksum={checksum} distance_checksum={distance_checksum:.1}"
    );
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let smoke = std::env::args().any(|argument| argument == "--smoke");
    let iterations = if smoke { 1 } else { 500 };
    println!("fitted-field sampled geometric thickness benchmark");
    run(iterations)
}
