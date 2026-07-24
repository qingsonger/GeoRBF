//! Versioned deterministic dense/sparse batch performance baseline.

use std::error::Error;
use std::hint::black_box;
use std::num::NonZeroUsize;
use std::time::Instant;

use georbf::{
    AffineNormalization, AngleUnit, AxisOrder, BatchEvaluationOptions, CenterRepresenter,
    ConditionPolicy, CoordinateMetadata, CrsMetadata, DenseFactorization, DenseSolveOptions,
    Enforcement, ExecutionOptions, FieldProblem, FittedField, FunctionalAtom, FunctionalExpr,
    FunctionalProvenance, FunctionalTerm, Gaussian, Handedness, KernelDefinition, LengthUnit,
    ObservationFunctional, ObservationId, Point, Regularization, SemanticConstraint,
    SemanticExpression, SemanticProblemIr, SemanticProvenance, SemanticRelation, SourceLocation,
    SparseFactorization, SparseFitOptions, VerticalDirection, Wendland, WendlandSmoothness,
};

const SCHEMA: &str = "georbf.performance.v1";
const MEMORY_LIMIT: usize = 512 * 1024 * 1024;

fn metadata() -> Result<CoordinateMetadata<3>, Box<dyn Error>> {
    Ok(CoordinateMetadata::new(
        LengthUnit::try_new("m")?,
        CrsMetadata::unspecified(),
        AxisOrder::identity(),
        VerticalDirection::Up,
        Handedness::Right,
        AngleUnit::Radians,
    ))
}

fn normalization() -> Result<AffineNormalization<3>, Box<dyn Error>> {
    Ok(AffineNormalization::try_new(
        Point::try_new([0.0; 3])?,
        [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
    )?)
}

fn grid_problem(side: u32, spacing: f64) -> Result<FieldProblem<3>, Box<dyn Error>> {
    let count = usize::try_from(side)?
        .checked_pow(3)
        .ok_or("grid count overflow")?;
    let mut constraints = Vec::new();
    let mut centers = Vec::new();
    constraints.try_reserve_exact(count)?;
    centers.try_reserve_exact(count)?;
    let mut index = 0_usize;
    for x in 0..side {
        for y in 0..side {
            for z in 0..side {
                let point = Point::try_new([
                    f64::from(x) * spacing,
                    f64::from(y) * spacing,
                    f64::from(z) * spacing,
                ])?;
                let identifier = u64::try_from(index + 1)?;
                let expression = FunctionalExpr::try_new([FunctionalTerm::try_new(
                    1.0,
                    FunctionalAtom::value(point, FunctionalProvenance::new(identifier)),
                )?])?;
                centers.push(CenterRepresenter::new(expression.clone()));
                let ordinal = f64::from(u32::try_from(index + 1)?);
                constraints.push(SemanticConstraint::try_new(
                    SemanticProvenance::try_new(
                        ObservationId::new(identifier),
                        SourceLocation::try_new(
                            "performance-baseline.csv".to_owned(),
                            NonZeroUsize::new(index + 1).ok_or("line")?,
                        )?,
                        "m".to_owned(),
                        format!("field.equalities[{index}]"),
                        Some("performance baseline".to_owned()),
                    )?,
                    SemanticRelation::Equality {
                        expression: SemanticExpression::try_new(
                            ObservationFunctional::new(expression),
                            0.0,
                        )?,
                        target: (ordinal * 0.03125).sin(),
                    },
                    Enforcement::Hard,
                )?);
                index += 1;
            }
        }
    }
    Ok(FieldProblem::try_new(
        SemanticProblemIr::try_new(constraints, ExecutionOptions::default())?,
        centers,
    )?)
}

fn dense_model(side: u32) -> Result<FittedField<3>, Box<dyn Error>> {
    Ok(FittedField::try_fit(
        grid_problem(side, 0.75)?,
        metadata()?,
        normalization()?,
        KernelDefinition::from(Gaussian::try_new(0.9)?),
        None,
        DenseSolveOptions::try_new(
            DenseFactorization::Cholesky,
            Regularization::None,
            ConditionPolicy::default(),
            4,
            NonZeroUsize::new(MEMORY_LIMIT).ok_or("memory limit")?,
        )?,
    )?)
}

fn sparse_model(side: u32) -> Result<FittedField<3>, Box<dyn Error>> {
    Ok(FittedField::try_fit_sparse(
        grid_problem(side, 1.0)?,
        metadata()?,
        normalization()?,
        Wendland::try_new(WendlandSmoothness::C4, 1.01)?,
        None,
        SparseFitOptions::new(
            SparseFactorization::FaerLlt,
            NonZeroUsize::new(MEMORY_LIMIT).ok_or("memory limit")?,
        ),
    )?)
}

fn queries(side: u32, count: u32) -> Result<Vec<Point<3>>, Box<dyn Error>> {
    (0..count)
        .map(|index| {
            let x = index % side;
            let y = (index / side) % side;
            let z = (index / side.saturating_mul(side)) % side;
            Ok(Point::try_new([
                f64::from(x) + 0.173,
                f64::from(y) + 0.271,
                f64::from(z) + 0.389,
            ])?)
        })
        .collect()
}

fn run(
    workload: &str,
    model: &FittedField<3>,
    points: &[Point<3>],
    threads: usize,
    iterations: u32,
) -> Result<(), Box<dyn Error>> {
    let options = BatchEvaluationOptions::new(
        NonZeroUsize::new(threads).ok_or("threads")?,
        NonZeroUsize::new(MEMORY_LIMIT).ok_or("memory limit")?,
    );
    let memory = model.try_batch_memory_diagnostics(points.len(), options)?;
    let mut checksum = 0.0_f64;
    let mut center_evaluations = 0_usize;
    let started = Instant::now();
    for _ in 0..iterations {
        let batch = black_box(model).try_evaluate_batch(black_box(points), options)?;
        center_evaluations = batch.diagnostics().center_evaluations;
        for evaluation in batch.evaluations() {
            checksum += black_box(evaluation.value());
            checksum += black_box(
                evaluation
                    .gradient()
                    .components()
                    .iter()
                    .copied()
                    .sum::<f64>(),
            );
        }
    }
    let samples = f64::from(iterations) * f64::from(u32::try_from(points.len())?);
    let nanoseconds_per_query = started.elapsed().as_secs_f64() * 1.0e9 / samples;
    println!(
        "{SCHEMA},{workload},{},{},{threads},{iterations},{center_evaluations},{},{nanoseconds_per_query:.2},{checksum:.17e}",
        model.centers().len(),
        points.len(),
        memory.estimated_peak_bytes
    );
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let smoke = std::env::args().any(|argument| argument == "--smoke");
    let dense_side = if smoke { 3 } else { 4 };
    let sparse_side = if smoke { 4 } else { 6 };
    let query_count = if smoke { 32 } else { 512 };
    let iterations = if smoke { 1 } else { 20 };
    let dense = dense_model(dense_side)?;
    let sparse = sparse_model(sparse_side)?;
    let dense_queries = queries(dense_side, query_count)?;
    let sparse_queries = queries(sparse_side, query_count)?;

    println!(
        "schema,workload,centers,queries,threads,iterations,center_evaluations,estimated_peak_bytes,nanoseconds_per_query,checksum"
    );
    for threads in [1, 2, 4] {
        run(
            "dense-value-gradient",
            &dense,
            &dense_queries,
            threads,
            iterations,
        )?;
    }
    for threads in [1, 2, 4] {
        run(
            "sparse-value-gradient",
            &sparse,
            &sparse_queries,
            threads,
            iterations,
        )?;
    }
    Ok(())
}
