//! Deterministic one-dimensional level-point extraction benchmark.

use std::error::Error;
use std::hint::black_box;
use std::num::{NonZeroU32, NonZeroUsize};
use std::time::Instant;

use georbf::{
    AffineNormalization, AngleUnit, AxisOrder, CenterRepresenter, ConditionPolicy,
    CoordinateMetadata, CrsMetadata, DenseFactorization, DenseSolveOptions, Enforcement,
    ExecutionOptions, FieldProblem, FittedField, FunctionalAtom, FunctionalExpr,
    FunctionalProvenance, FunctionalTerm, Handedness, KernelDefinition, LengthUnit,
    LevelPointRequest, LevelPointSettings, ObservationFunctional, ObservationId, Point,
    PolyharmonicSpline, Regularization, SemanticConstraint, SemanticExpression, SemanticProblemIr,
    SemanticProvenance, SemanticRelation, SourceLocation, VerticalDirection,
};

fn build_model() -> Result<FittedField<1>, Box<dyn Error>> {
    let polynomial = |x: f64| x * x + x - 2.0;
    let sites = [-2.0, -0.5, 1.0, 2.5];
    let mut constraints = Vec::new();
    let mut centers = Vec::new();
    for (index, site) in sites.into_iter().enumerate() {
        let identifier = u64::try_from(index + 1)?;
        let expression = FunctionalExpr::try_new([FunctionalTerm::try_new(
            1.0,
            FunctionalAtom::value(
                Point::try_new([site])?,
                FunctionalProvenance::new(identifier),
            ),
        )?])?;
        centers.push(CenterRepresenter::new(expression.clone()));
        constraints.push(SemanticConstraint::try_new(
            SemanticProvenance::try_new(
                ObservationId::new(identifier),
                SourceLocation::try_new(
                    "level-point-benchmark.csv".to_owned(),
                    NonZeroUsize::new(index + 1).ok_or("line")?,
                )?,
                "m".to_owned(),
                format!("field.equalities[{index}]"),
                Some("level-point benchmark".to_owned()),
            )?,
            SemanticRelation::Equality {
                expression: SemanticExpression::try_new(
                    ObservationFunctional::new(expression),
                    0.0,
                )?,
                target: polynomial(site),
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
    let normalization = AffineNormalization::try_new(Point::try_new([0.0])?, [[1.0]])?;
    let options = DenseSolveOptions::try_new(
        DenseFactorization::PivotedLblt,
        Regularization::None,
        ConditionPolicy::default(),
        4,
        NonZeroUsize::new(64 * 1024 * 1024).ok_or("memory limit")?,
    )?;
    Ok(FittedField::try_fit(
        problem,
        metadata,
        normalization,
        KernelDefinition::from(PolyharmonicSpline::try_new(4)?),
        None,
        options,
    )?)
}

fn main() -> Result<(), Box<dyn Error>> {
    let smoke = std::env::args().any(|argument| argument == "--smoke");
    let iterations = if smoke { 2_u32 } else { 2_000_u32 };
    let model = build_model()?;
    let settings = LevelPointSettings::try_new(
        NonZeroU32::new(64).ok_or("scan intervals")?,
        NonZeroU32::new(64).ok_or("refinement iterations")?,
        1.0e-12,
        1.0e-12,
        1.0e-11,
    )?;
    let request = LevelPointRequest::try_new(0.0, -4.0, 4.0, settings)?;

    let started = Instant::now();
    let mut checksum = 0.0;
    for _ in 0..iterations {
        let report = black_box(&model).try_level_points(black_box(&request))?;
        if report.points().len() != 2 || report.stationary_points().len() != 1 {
            return Err("unexpected level-point benchmark topology".into());
        }
        checksum += report
            .points()
            .iter()
            .map(|point| point.point().components()[0] + point.residual() + point.derivative())
            .sum::<f64>();
        checksum += report
            .stationary_points()
            .iter()
            .map(|point| point.point().components()[0] + point.value() + point.derivative())
            .sum::<f64>();
        checksum += f64::from(u32::try_from(report.diagnostics().evaluations())?);
    }
    let nanoseconds = started.elapsed().as_secs_f64() * 1.0e9 / f64::from(iterations);
    println!(
        "georbf.level_points.v1: {nanoseconds:.2} ns/extraction iterations={iterations} checksum={checksum:.17e}"
    );
    Ok(())
}
