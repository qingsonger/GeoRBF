//! Deterministic production compact-sparse assembly, solve, and evaluation benchmark.

use std::error::Error;
use std::hint::black_box;
use std::num::NonZeroUsize;
use std::time::Instant;

use georbf::{
    AffineNormalization, AngleUnit, AxisOrder, CenterRepresenter, CoordinateMetadata, CrsMetadata,
    Enforcement, ExecutionOptions, FieldProblem, FittedField, FunctionalAtom, FunctionalExpr,
    FunctionalProvenance, FunctionalTerm, Handedness, LengthUnit, ObservationFunctional,
    ObservationId, Point, SemanticConstraint, SemanticExpression, SemanticProblemIr,
    SemanticProvenance, SemanticRelation, SourceLocation, SparseFactorization, SparseFitOptions,
    VerticalDirection, Wendland, WendlandSmoothness, try_solve_sparse_field,
};

const MEMORY_LIMIT: usize = 256 * 1024 * 1024;

fn options() -> Result<SparseFitOptions, Box<dyn Error>> {
    Ok(SparseFitOptions::new(
        SparseFactorization::FaerLlt,
        NonZeroUsize::new(MEMORY_LIMIT).ok_or("memory limit")?,
    ))
}

fn fixture(side: u32) -> Result<FieldProblem<3>, Box<dyn Error>> {
    let count = usize::try_from(side)?
        .checked_pow(3)
        .ok_or("fixture count overflow")?;
    let mut constraints = Vec::new();
    let mut centers = Vec::new();
    constraints.try_reserve_exact(count)?;
    centers.try_reserve_exact(count)?;
    let mut index = 0_usize;
    for x in 0..side {
        for y in 0..side {
            for z in 0..side {
                let point = Point::try_new([f64::from(x), f64::from(y), f64::from(z)])?;
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
                            "compact-sparse-benchmark.csv".to_owned(),
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
                        target: (ordinal * 0.03125).sin(),
                    },
                    Enforcement::Hard,
                )?);
                index += 1;
            }
        }
    }
    Ok(FieldProblem::try_new(
        SemanticProblemIr::try_new(
            constraints,
            ExecutionOptions::new(
                true,
                Some(NonZeroUsize::MIN),
                NonZeroUsize::new(MEMORY_LIMIT),
            ),
        )?,
        centers,
    )?)
}

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

fn main() -> Result<(), Box<dyn Error>> {
    let smoke = std::env::args().any(|argument| argument == "--smoke");
    let side = if smoke { 4 } else { 8 };
    let iterations = if smoke { 1 } else { 5 };
    let problem = fixture(side)?;
    let kernel = Wendland::try_new(WendlandSmoothness::C4, 1.01)?;
    let sparse_options = options()?;

    let mut assembly_checksum = 0.0_f64;
    let assembly_started = Instant::now();
    for _ in 0..iterations {
        let system = black_box(&problem).try_assemble_sparse(kernel, None, sparse_options)?;
        assembly_checksum += f64::from(u32::try_from(system.diagnostics().stored_nonzeros)?);
        assembly_checksum += system.diagnostics().maximum_absolute_entry;
        let _ = black_box(system);
    }
    let assembly_nanoseconds =
        assembly_started.elapsed().as_secs_f64() * 1.0e9 / f64::from(iterations);

    let system = problem.try_assemble_sparse(kernel, None, sparse_options)?;
    let mut solve_checksum = 0.0_f64;
    let solve_started = Instant::now();
    for _ in 0..iterations {
        let solution = try_solve_sparse_field(black_box(&system))?;
        solve_checksum += solution.values().iter().copied().sum::<f64>();
        solve_checksum += solution.diagnostics().residual.original_infinity;
    }
    let solve_nanoseconds = solve_started.elapsed().as_secs_f64() * 1.0e9 / f64::from(iterations);

    let model = FittedField::try_fit_sparse(
        fixture(side)?,
        metadata()?,
        normalization()?,
        kernel,
        None,
        sparse_options,
    )?;
    let query_count = 64_u32;
    let mut evaluation_checksum = 0.0_f64;
    let evaluation_started = Instant::now();
    for index in 0..query_count {
        let coordinate = f64::from(index % side) + 0.25;
        let evaluation = black_box(&model).try_evaluate(Point::try_new([
            coordinate,
            coordinate * 0.5,
            coordinate * 0.25,
        ])?)?;
        evaluation_checksum += evaluation.value();
        evaluation_checksum += evaluation
            .gradient()
            .components()
            .iter()
            .copied()
            .sum::<f64>();
        evaluation_checksum += f64::from(u32::try_from(evaluation.center_evaluations())?);
    }
    let evaluation_nanoseconds =
        evaluation_started.elapsed().as_secs_f64() * 1.0e9 / f64::from(query_count);

    println!(
        "phase,points,stored_nonzeros,nanoseconds,checksum\nassembly,{},{},{assembly_nanoseconds:.2},{assembly_checksum:.17e}",
        problem.centers().len(),
        system.diagnostics().stored_nonzeros
    );
    println!(
        "solve,{},{},{solve_nanoseconds:.2},{solve_checksum:.17e}",
        problem.centers().len(),
        system.diagnostics().stored_nonzeros
    );
    println!(
        "local-evaluation,{},{},{evaluation_nanoseconds:.2},{evaluation_checksum:.17e}",
        problem.centers().len(),
        system.diagnostics().stored_nonzeros
    );
    Ok(())
}
