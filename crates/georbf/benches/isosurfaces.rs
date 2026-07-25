//! Deterministic three-dimensional isosurface extraction benchmark.

use std::error::Error;
use std::hint::black_box;
use std::num::{NonZeroU32, NonZeroUsize};
use std::time::Instant;

use georbf::{
    AffineNormalization, AngleUnit, AxisOrder, CenterRepresenter, ConditionPolicy,
    CoordinateMetadata, CrsMetadata, DenseFactorization, DenseSolveOptions, Enforcement,
    ExecutionOptions, FieldProblem, FittedField, FunctionalAtom, FunctionalExpr,
    FunctionalProvenance, FunctionalTerm, Handedness, IsosurfaceMethod, IsosurfaceRequest,
    IsosurfaceSettings, KernelDefinition, LengthUnit, ObservationFunctional, ObservationId, Point,
    PolyharmonicSpline, Regularization, SemanticConstraint, SemanticExpression, SemanticProblemIr,
    SemanticProvenance, SemanticRelation, SourceLocation, VerticalDirection,
};

fn build_model() -> Result<FittedField<3>, Box<dyn Error>> {
    let polynomial = |x: f64, y: f64, z: f64| x * x + y * y + z * z - 0.73 * 0.73;
    let coordinates = [-2.0, 0.0, 2.0];
    let mut constraints = Vec::new();
    let mut centers = Vec::new();
    let mut index = 0_usize;
    for z in coordinates {
        for y in coordinates {
            for x in coordinates {
                index += 1;
                let identifier = u64::try_from(index)?;
                let site = [x, y, z];
                let expression = FunctionalExpr::try_new([FunctionalTerm::try_new(
                    1.0,
                    FunctionalAtom::value(
                        Point::try_new(site)?,
                        FunctionalProvenance::new(identifier),
                    ),
                )?])?;
                centers.push(CenterRepresenter::new(expression.clone()));
                constraints.push(SemanticConstraint::try_new(
                    SemanticProvenance::try_new(
                        ObservationId::new(identifier),
                        SourceLocation::try_new(
                            "isosurface-benchmark.csv".to_owned(),
                            NonZeroUsize::new(index).ok_or("line")?,
                        )?,
                        "m".to_owned(),
                        format!("field.equalities[{}]", index - 1),
                        Some("isosurface benchmark".to_owned()),
                    )?,
                    SemanticRelation::Equality {
                        expression: SemanticExpression::try_new(
                            ObservationFunctional::new(expression),
                            0.0,
                        )?,
                        target: polynomial(x, y, z),
                    },
                    Enforcement::Hard,
                )?);
            }
        }
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
    let normalization = AffineNormalization::try_new(
        Point::try_new([0.0, 0.0, 0.0])?,
        [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
    )?;
    let options = DenseSolveOptions::try_new(
        DenseFactorization::PivotedLblt,
        Regularization::None,
        ConditionPolicy::default(),
        4,
        NonZeroUsize::new(128 * 1024 * 1024).ok_or("memory limit")?,
    )?;
    Ok(FittedField::try_fit(
        problem,
        metadata,
        normalization,
        KernelDefinition::from(PolyharmonicSpline::try_new(5)?),
        None,
        options,
    )?)
}

fn main() -> Result<(), Box<dyn Error>> {
    let smoke = std::env::args().any(|argument| argument == "--smoke");
    let iterations = if smoke { 1_u32 } else { 50_u32 };
    let model = build_model()?;
    let cells = NonZeroU32::new(24).ok_or("cells")?;
    let settings = IsosurfaceSettings::try_new(
        cells,
        cells,
        cells,
        NonZeroU32::new(64).ok_or("refinement iterations")?,
        1.0e-10,
        1.0e-10,
    )?;
    let request = IsosurfaceRequest::try_new(
        0.0,
        Point::try_new([-1.0, -1.0, -1.0])?,
        Point::try_new([1.0, 1.0, 1.0])?,
        IsosurfaceMethod::TopologyAwareMarchingCubes,
        settings,
    )?;

    let started = Instant::now();
    let mut checksum = 0.0;
    for _ in 0..iterations {
        let report = black_box(&model).try_isosurface(black_box(&request))?;
        if report.components().len() != 1 || !report.components()[0].is_closed() {
            return Err("unexpected isosurface benchmark topology".into());
        }
        for vertex in report.vertices() {
            checksum += vertex.point().components().iter().sum::<f64>();
            checksum += vertex.normal().components().iter().sum::<f64>();
            checksum += vertex.value() + vertex.residual();
        }
        checksum += f64::from(u32::try_from(report.diagnostics().evaluations())?);
        checksum += f64::from(u32::try_from(report.triangles().len())?);
    }
    let nanoseconds = started.elapsed().as_secs_f64() * 1.0e9 / f64::from(iterations);
    println!(
        "georbf.isosurfaces.v1: {nanoseconds:.2} ns/extraction iterations={iterations} checksum={checksum:.17e}"
    );
    Ok(())
}
