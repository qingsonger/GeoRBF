//! Independent block, batch, thread, sparse-locality, and memory tests.

use std::error::Error;
use std::fmt;
use std::mem::size_of;
use std::num::NonZeroUsize;

use georbf::{
    AffineNormalization, AngleUnit, AxisOrder, BatchEvaluationError, BatchEvaluationOptions,
    CenterRepresenter, ConditionPolicy, CoordinateMetadata, CrsMetadata, DENSE_ASSEMBLY_BLOCK_SIZE,
    DenseFactorization, DenseSolveOptions, Enforcement, ExecutionOptions, FieldProblem,
    FittedField, FittedFieldAssemblyDiagnostics, FunctionalAtom, FunctionalExpr,
    FunctionalProvenance, FunctionalTerm, Gaussian, Handedness, KernelDefinition,
    KernelDerivativeOrder, LengthUnit, ObservationFunctional, ObservationId, Point,
    RadialSeparation, Regularization, SemanticConstraint, SemanticExpression, SemanticProblemIr,
    SemanticProvenance, SemanticRelation, SourceLocation, SparseFactorization, SparseFitOptions,
    SpatialKernelJet, SpatialKernelJetPrefix, VerticalDirection, Wendland, WendlandSmoothness,
};

const MEMORY_LIMIT: usize = 64 * 1024 * 1024;

type TestResult = Result<(), Box<dyn Error>>;

#[derive(Debug)]
struct EvaluatorError(String);

impl fmt::Display for EvaluatorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl Error for EvaluatorError {}

fn metadata<const D: usize>() -> Result<CoordinateMetadata<D>, Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    Ok(CoordinateMetadata::new(
        LengthUnit::try_new("m")?,
        CrsMetadata::unspecified(),
        AxisOrder::identity(),
        VerticalDirection::Up,
        Handedness::Right,
        AngleUnit::Radians,
    ))
}

fn normalization<const D: usize>() -> Result<AffineNormalization<D>, Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    Ok(AffineNormalization::try_new(
        Point::try_new([0.0; D])?,
        std::array::from_fn(|row| std::array::from_fn(|column| f64::from(row == column))),
    )?)
}

fn problem<const D: usize>(count: usize, spacing: f64) -> Result<FieldProblem<D>, Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    let mut constraints = Vec::new();
    let mut centers = Vec::new();
    constraints.try_reserve_exact(count)?;
    centers.try_reserve_exact(count)?;
    for index in 0..count {
        let ordinal = f64::from(u32::try_from(index)?);
        let point = Point::try_new(std::array::from_fn(|axis| {
            ordinal * spacing + f64::from(u32::try_from(axis).unwrap_or_default()) * 0.03125
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
                    "performance-test.csv".to_owned(),
                    NonZeroUsize::new(index + 1).ok_or("line")?,
                )?,
                "m".to_owned(),
                format!("field.equalities[{index}]"),
                Some("performance test".to_owned()),
            )?,
            SemanticRelation::Equality {
                expression: SemanticExpression::try_new(
                    ObservationFunctional::new(expression),
                    0.0,
                )?,
                target: (ordinal * 0.25).sin(),
            },
            Enforcement::Hard,
        )?);
    }
    Ok(FieldProblem::try_new(
        SemanticProblemIr::try_new(constraints, ExecutionOptions::default())?,
        centers,
    )?)
}

fn dense_options() -> Result<DenseSolveOptions, Box<dyn Error>> {
    Ok(DenseSolveOptions::try_new(
        DenseFactorization::Cholesky,
        Regularization::None,
        ConditionPolicy::default(),
        4,
        NonZeroUsize::new(MEMORY_LIMIT).ok_or("memory limit")?,
    )?)
}

fn sparse_options() -> Result<SparseFitOptions, Box<dyn Error>> {
    Ok(SparseFitOptions::new(
        SparseFactorization::FaerLlt,
        NonZeroUsize::new(MEMORY_LIMIT).ok_or("memory limit")?,
    ))
}

fn dense_model<const D: usize>() -> Result<FittedField<D>, Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    Ok(FittedField::try_fit(
        problem(9, 0.45)?,
        metadata()?,
        normalization()?,
        KernelDefinition::from(Gaussian::try_new(0.7)?),
        None,
        dense_options()?,
    )?)
}

fn sparse_model() -> Result<FittedField<3>, Box<dyn Error>> {
    Ok(FittedField::try_fit_sparse(
        problem(18, 0.5)?,
        metadata()?,
        normalization()?,
        Wendland::try_new(WendlandSmoothness::C4, 0.76)?,
        None,
        sparse_options()?,
    )?)
}

fn queries<const D: usize>(count: usize) -> Result<Vec<Point<D>>, Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    (0..count)
        .map(|index| {
            let ordinal = f64::from(u32::try_from(index)?);
            Ok(Point::try_new(std::array::from_fn(|axis| {
                ordinal * 0.137 + f64::from(u32::try_from(axis).unwrap_or_default()) * 0.019
            }))?)
        })
        .collect()
}

fn batch_options(threads: usize, memory: usize) -> Result<BatchEvaluationOptions, Box<dyn Error>> {
    Ok(BatchEvaluationOptions::new(
        NonZeroUsize::new(threads).ok_or("threads")?,
        NonZeroUsize::new(memory).ok_or("memory")?,
    ))
}

fn verify_dense_dimension<const D: usize>() -> TestResult
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    let model = dense_model::<D>()?;
    let points = queries::<D>(37)?;
    let serial = model.try_evaluate_batch(&points, batch_options(1, MEMORY_LIMIT)?)?;
    let parallel = model.try_evaluate_batch(&points, batch_options(4, MEMORY_LIMIT)?)?;
    assert_eq!(serial.evaluations(), parallel.evaluations());
    assert_eq!(parallel.diagnostics().worker_count, 4);
    assert_eq!(
        parallel.diagnostics().center_evaluations,
        points.len() * model.centers().len()
    );
    for (point, evaluation) in points.iter().zip(parallel.evaluations()) {
        assert_eq!(*evaluation, model.try_evaluate(*point)?);
    }
    Ok(())
}

#[test]
fn dense_batch_is_bit_exact_in_d1_d2_d3_and_across_threads() -> TestResult {
    verify_dense_dimension::<1>()?;
    verify_dense_dimension::<2>()?;
    verify_dense_dimension::<3>()?;
    Ok(())
}

#[test]
fn upper_triangle_blocks_evaluate_and_reflect_each_pair_once() -> TestResult {
    let count = DENSE_ASSEMBLY_BLOCK_SIZE + 1;
    let field = problem::<1>(count, 0.25)?;
    let kernel = Gaussian::try_new(0.8)?;
    let system = field.try_assemble(kernel.metadata(), |query, center, demanded| {
        if demanded != KernelDerivativeOrder::Value {
            return Err(EvaluatorError("unexpected derivative demand".to_owned()));
        }
        let separation = RadialSeparation::try_new(query, center)
            .map_err(|error| EvaluatorError(error.to_string()))?;
        if separation.is_center() {
            return SpatialKernelJetPrefix::try_center_value(
                separation,
                kernel
                    .radial_value(0.0)
                    .map_err(|error| EvaluatorError(error.to_string()))?,
            )
            .map_err(|error| EvaluatorError(error.to_string()));
        }
        Ok(SpatialKernelJet::try_new(
            separation,
            kernel
                .radial_jet(separation)
                .map_err(|error| EvaluatorError(error.to_string()))?,
        )
        .map_err(|error| EvaluatorError(error.to_string()))?
        .into())
    })?;
    let diagnostics = system.diagnostics();
    assert_eq!(diagnostics.assembly_block_size, DENSE_ASSEMBLY_BLOCK_SIZE);
    assert_eq!(diagnostics.upper_triangle_blocks, 3);
    assert_eq!(
        diagnostics.kernel_entry_evaluations,
        count * (count + 1) / 2
    );
    assert_eq!(
        diagnostics.reflected_kernel_entries,
        count * (count - 1) / 2
    );
    assert_eq!(
        diagnostics.normalized_asymmetry.to_bits(),
        0.0_f64.to_bits()
    );
    for row in 0..count {
        for column in 0..count {
            assert_eq!(
                system.matrix().get(row, column).ok_or("entry")?.to_bits(),
                system
                    .matrix()
                    .get(column, row)
                    .ok_or("transpose")?
                    .to_bits()
            );
        }
    }
    Ok(())
}

#[test]
fn sparse_batch_keeps_exact_locality_and_thread_determinism() -> TestResult {
    let model = sparse_model()?;
    let points = queries::<3>(64)?;
    let serial = model.try_evaluate_batch(&points, batch_options(1, MEMORY_LIMIT)?)?;
    let parallel = model.try_evaluate_batch(&points, batch_options(4, MEMORY_LIMIT)?)?;
    assert_eq!(serial.evaluations(), parallel.evaluations());
    assert!(
        parallel.diagnostics().center_evaluations < points.len() * model.centers().len(),
        "compact support should visit fewer than all centers"
    );
    let FittedFieldAssemblyDiagnostics::Sparse(assembly) = model.diagnostics().assembly() else {
        return Err("expected sparse assembly".into());
    };
    assert!(assembly.density < 1.0);
    assert_eq!(
        parallel.diagnostics().memory.workspace_bytes_per_worker,
        model.centers().len() * size_of::<usize>()
    );
    Ok(())
}

#[test]
fn logical_memory_is_exact_and_rejected_before_batch_allocation() -> TestResult {
    let dense = dense_model::<2>()?;
    let options = batch_options(4, MEMORY_LIMIT)?;
    let memory = dense.try_batch_memory_diagnostics(17, options)?;
    assert_eq!(memory.worker_count, 4);
    assert_eq!(
        memory.output_bytes,
        17 * size_of::<georbf::FittedFieldEvaluation<2>>()
    );
    assert_eq!(memory.workspace_bytes_per_worker, 0);
    assert_eq!(memory.estimated_peak_bytes, memory.output_bytes);

    let limited = batch_options(4, memory.estimated_peak_bytes.saturating_sub(1).max(1))?;
    assert!(matches!(
        dense.try_evaluate_batch(&queries::<2>(17)?, limited),
        Err(BatchEvaluationError::MemoryLimitExceeded {
            estimated_peak_bytes,
            limit_bytes,
        }) if estimated_peak_bytes == memory.estimated_peak_bytes
            && limit_bytes == limited.memory_limit_bytes().get()
    ));
    let empty = dense.try_evaluate_batch(&[], batch_options(8, 1)?)?;
    assert_eq!(empty.diagnostics().worker_count, 0);
    assert_eq!(empty.diagnostics().memory.estimated_peak_bytes, 0);
    Ok(())
}

#[test]
fn caller_workspace_reuses_capacity_and_clears_partial_failures() -> TestResult {
    let dense = dense_model::<1>()?;
    let sparse = sparse_model()?;
    let points = queries::<1>(23)?;
    let mut workspace = dense.try_evaluation_workspace()?;
    let mut output = Vec::new();
    let center_evaluations = dense.try_evaluate_batch_into(&points, &mut workspace, &mut output)?;
    assert_eq!(center_evaluations, points.len() * dense.centers().len());
    let capacity = output.capacity();
    let repeated = dense.try_evaluate_batch_into(&points, &mut workspace, &mut output)?;
    assert_eq!(repeated, center_evaluations);
    assert_eq!(output.capacity(), capacity);
    let dense_three = dense_model::<3>()?;
    let mut incompatible = dense_three.try_evaluation_workspace()?;
    assert!(matches!(
        sparse.try_evaluate_batch_into(&queries::<3>(1)?, &mut incompatible, &mut Vec::new()),
        Err(BatchEvaluationError::IncompatibleWorkspace { .. })
    ));
    Ok(())
}

fn convenience_batch_allocation_count(query_count: usize) -> Result<u64, Box<dyn Error>> {
    let model = dense_model::<1>()?;
    let points = queries::<1>(query_count)?;
    let options = batch_options(1, MEMORY_LIMIT)?;
    let mut batch = None;
    let allocation = allocation_counter::measure(|| {
        batch = Some(model.try_evaluate_batch(&points, options));
    });
    let completed = batch.ok_or("batch measurement did not run")??;
    assert_eq!(completed.evaluations().len(), query_count);
    Ok(allocation.count_total)
}

#[test]
fn warmed_workspace_has_zero_allocations_and_batch_count_is_length_independent() -> TestResult {
    let model = dense_model::<1>()?;
    let points = queries::<1>(128)?;
    let mut workspace = model.try_evaluation_workspace()?;
    let mut output = Vec::new();
    model.try_evaluate_batch_into(&points, &mut workspace, &mut output)?;
    let mut result = None;
    let warmed = allocation_counter::measure(|| {
        result = Some(model.try_evaluate_batch_into(&points, &mut workspace, &mut output));
    });
    assert_eq!(result.ok_or("workspace measurement did not run")??, 128 * 9);
    assert_eq!(warmed.count_total, 0);

    let small = convenience_batch_allocation_count(4)?;
    let large = convenience_batch_allocation_count(256)?;
    assert_eq!(small, 2);
    assert_eq!(large, small);
    Ok(())
}
