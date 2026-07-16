//! Deterministic immutable fitted-field evaluation benchmark.

use std::error::Error;
use std::hint::black_box;
use std::num::NonZeroUsize;
use std::time::Instant;

use georbf::{
    AffineNormalization, AngleUnit, AxisOrder, CenterRepresenter, ConditionPolicy,
    CoordinateMetadata, CrsMetadata, DenseFactorization, DenseSolveOptions, Enforcement,
    ExecutionOptions, FieldProblem, FittedField, FunctionalAtom, FunctionalExpr,
    FunctionalProvenance, FunctionalTerm, Gaussian, Handedness, KernelDefinition, LengthUnit,
    ObservationFunctional, ObservationId, Point, Regularization, SemanticConstraint,
    SemanticExpression, SemanticProblemIr, SemanticProvenance, SemanticRelation, SourceLocation,
    VerticalDirection,
};

fn build_model<const D: usize>() -> Result<FittedField<D>, Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    let count = 12;
    let mut constraints = Vec::new();
    let mut centers = Vec::new();
    constraints.try_reserve_exact(count)?;
    centers.try_reserve_exact(count)?;
    for index in 0..count {
        let index_u32 = u32::try_from(index)?;
        let point = Point::try_new(std::array::from_fn(|axis| {
            let axis_u32 = u32::try_from(axis).unwrap_or_default();
            f64::from(index_u32) * (0.7 + 0.05 * f64::from(axis_u32)) + 0.1 * f64::from(axis_u32)
        }))?;
        let identifier = u64::try_from(index + 1)?;
        let expression = FunctionalExpr::try_new([FunctionalTerm::try_new(
            1.0,
            FunctionalAtom::value(point, FunctionalProvenance::new(identifier)),
        )?])?;
        centers.push(CenterRepresenter::new(expression.clone()));
        constraints.push(SemanticConstraint::try_new(
            SemanticProvenance::try_new(
                ObservationId::new(identifier),
                SourceLocation::try_new(
                    "fitted-field-benchmark.csv".to_owned(),
                    NonZeroUsize::new(index + 1).ok_or("line")?,
                )?,
                "m".to_owned(),
                format!("field.equalities[{index}]"),
                Some("benchmark".to_owned()),
            )?,
            SemanticRelation::Equality {
                expression: SemanticExpression::try_new(
                    ObservationFunctional::new(expression),
                    0.0,
                )?,
                target: (f64::from(index_u32) * 0.2).sin(),
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
    let normalization = AffineNormalization::try_new(
        Point::try_new([0.0; D])?,
        std::array::from_fn(|row| std::array::from_fn(|column| f64::from(row == column))),
    )?;
    let options = DenseSolveOptions::try_new(
        DenseFactorization::Cholesky,
        Regularization::None,
        ConditionPolicy::default(),
        4,
        NonZeroUsize::new(64 * 1024 * 1024).ok_or("memory limit")?,
    )?;
    Ok(FittedField::try_fit(
        problem,
        metadata,
        normalization,
        KernelDefinition::from(Gaussian::try_new(0.6)?),
        None,
        options,
    )?)
}

fn queries<const D: usize>() -> Result<Vec<Point<D>>, Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    (0..128)
        .map(|index| {
            let index_u32 = u32::try_from(index)?;
            Ok(Point::try_new(std::array::from_fn(|axis| {
                let axis_u32 = u32::try_from(axis).unwrap_or_default();
                f64::from(index_u32) * 0.061 + f64::from(axis_u32) * 0.17
            }))?)
        })
        .collect()
}

fn run<const D: usize>(label: &str, iterations: u32) -> Result<(), Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    let model = build_model::<D>()?;
    let queries = queries::<D>()?;
    let mut checksum = 0.0;
    let started = Instant::now();
    for _ in 0..iterations {
        for query in &queries {
            let evaluation = black_box(&model).try_evaluate_with_hessian(black_box(*query))?;
            checksum += black_box(evaluation.value());
            checksum += black_box(
                evaluation
                    .gradient()
                    .components()
                    .iter()
                    .copied()
                    .sum::<f64>(),
            );
            checksum += black_box(evaluation.hessian().iter().flatten().copied().sum::<f64>());
        }
    }
    let evaluations = f64::from(iterations) * f64::from(u32::try_from(queries.len())?);
    let nanoseconds = started.elapsed().as_secs_f64() * 1.0e9 / evaluations;
    println!("{label}: {nanoseconds:.2} ns/evaluation checksum={checksum:.17e}");
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let smoke = std::env::args().any(|argument| argument == "--smoke");
    let iterations = if smoke { 2 } else { 100 };
    println!("immutable fitted-field deterministic single-thread benchmark");
    run::<1>("D=1", iterations)?;
    run::<2>("D=2", iterations)?;
    run::<3>("D=3", iterations)?;
    Ok(())
}
