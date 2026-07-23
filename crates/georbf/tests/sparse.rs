//! Independent truth and failure-path tests for compact sparse fields.

use std::error::Error;
use std::num::NonZeroUsize;

use georbf::{
    AffineNormalization, AngleUnit, AnisotropyConditionPolicy, AxisOrder, CancellationToken,
    CenterRepresenter, ConditionPolicy, CoordinateMetadata, CrsMetadata, DenseFactorization,
    DenseSolveOptions, Enforcement, ExecutionControl, ExecutionError, ExecutionOperation,
    ExecutionOptions, FieldProblem, FittedField, FunctionalAtom, FunctionalExpr,
    FunctionalProvenance, FunctionalTerm, GlobalAnisotropy, Handedness, KernelDefinition,
    LengthUnit, ObservationFunctional, ObservationId, Point, Regularization, SemanticConstraint,
    SemanticExpression, SemanticProblemIr, SemanticProvenance, SemanticRelation, SourceLocation,
    SparseFactorization, SparseFieldAssemblyError, SparseFitOptions, SparseSolveError,
    UnitDirection, VerticalDirection, Wendland, WendlandSmoothness, try_solve_sparse_field,
};

type TestResult = Result<(), Box<dyn Error>>;

const MEMORY_LIMIT: usize = 64 * 1024 * 1024;

fn sparse_options() -> SparseFitOptions {
    SparseFitOptions::new(
        SparseFactorization::FaerLlt,
        NonZeroUsize::new(MEMORY_LIMIT).unwrap_or(NonZeroUsize::MIN),
    )
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

fn value_problem<const D: usize>(
    points: &[[f64; D]],
    targets: &[f64],
) -> Result<FieldProblem<D>, Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    if points.len() != targets.len() {
        return Err("point and target counts differ".into());
    }
    let mut constraints = Vec::new();
    let mut centers = Vec::new();
    constraints.try_reserve_exact(points.len())?;
    centers.try_reserve_exact(points.len())?;
    for (index, (components, target)) in points.iter().zip(targets).enumerate() {
        let identifier = u64::try_from(index + 1)?;
        let point = Point::try_new(*components)?;
        let expression = FunctionalExpr::try_new([FunctionalTerm::try_new(
            1.0,
            FunctionalAtom::value(point, FunctionalProvenance::new(identifier)),
        )?])?;
        centers.push(CenterRepresenter::new(expression.clone()));
        constraints.push(SemanticConstraint::try_new(
            SemanticProvenance::try_new(
                ObservationId::new(identifier),
                SourceLocation::try_new(
                    "sparse-truth.csv".to_owned(),
                    NonZeroUsize::new(index + 1).ok_or("line")?,
                )?,
                "m".to_owned(),
                format!("field.equalities[{index}]"),
                Some("sparse".to_owned()),
            )?,
            SemanticRelation::Equality {
                expression: SemanticExpression::try_new(
                    ObservationFunctional::new(expression),
                    0.0,
                )?,
                target: *target,
            },
            Enforcement::Hard,
        )?);
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

fn mixed_problem_2d() -> Result<FieldProblem<2>, Box<dyn Error>> {
    let points = [[0.0, 0.0], [0.6, 0.1], [1.2, -0.1]];
    let directions = [[1.0, 0.0], [0.0, 1.0], [1.0, 1.0]];
    let targets = [0.25, -0.5, 1.25];
    let mut constraints = Vec::new();
    let mut centers = Vec::new();
    for (index, ((components, direction), target)) in
        points.into_iter().zip(directions).zip(targets).enumerate()
    {
        let identifier = u64::try_from(index + 1)?;
        let point = Point::try_new(components)?;
        let expression = FunctionalExpr::try_new([
            FunctionalTerm::try_new(
                1.0,
                FunctionalAtom::value(point, FunctionalProvenance::new(identifier)),
            )?,
            FunctionalTerm::try_new(
                0.25,
                FunctionalAtom::directional_derivative(
                    point,
                    UnitDirection::try_new(direction)?,
                    FunctionalProvenance::new(identifier + 100),
                ),
            )?,
        ])?;
        centers.push(CenterRepresenter::new(expression.clone()));
        constraints.push(SemanticConstraint::try_new(
            SemanticProvenance::try_new(
                ObservationId::new(identifier),
                SourceLocation::try_new(
                    "sparse-mixed-truth.csv".to_owned(),
                    NonZeroUsize::new(index + 1).ok_or("line")?,
                )?,
                "m".to_owned(),
                format!("field.equalities[{index}]"),
                Some("sparse-mixed".to_owned()),
            )?,
            SemanticRelation::Equality {
                expression: SemanticExpression::try_new(
                    ObservationFunctional::new(expression),
                    0.0,
                )?,
                target,
            },
            Enforcement::Hard,
        )?);
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

fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "actual={actual:.17e}, expected={expected:.17e}, tolerance={tolerance:.17e}"
    );
}

#[test]
fn hand_derived_csc_and_solution_match_independent_truth() -> TestResult {
    let points = [[0.0], [1.0], [2.0]];
    let rhs = [11.0 / 8.0, 11.0 / 4.0, 27.0 / 8.0];
    let kernel = Wendland::try_new(WendlandSmoothness::C2, 2.0)?;
    let problem = value_problem(&points, &rhs)?;
    let system = problem.try_assemble_sparse(kernel, None, sparse_options())?;

    assert_eq!(system.matrix().column_pointers(), &[0, 2, 5, 7]);
    assert_eq!(system.matrix().row_indices(), &[0, 1, 0, 1, 2, 1, 2]);
    assert_eq!(
        system.matrix().values(),
        &[
            1.0,
            3.0 / 16.0,
            3.0 / 16.0,
            1.0,
            3.0 / 16.0,
            3.0 / 16.0,
            1.0
        ]
    );
    assert_eq!(system.matrix().get(0, 2), Some(0.0));
    assert_eq!(system.diagnostics().neighborhood.supported_pairs, 5);
    assert_eq!(system.diagnostics().stored_nonzeros, 7);

    let solution = try_solve_sparse_field(&system)?;
    for (actual, expected) in solution.values().iter().zip([1.0, 2.0, 3.0]) {
        assert_close(*actual, expected, 64.0 * f64::EPSILON);
    }
    assert!(
        solution.diagnostics().residual.original_backward_error
            <= solution.diagnostics().residual_tolerance
    );
    Ok(())
}

fn parity<const D: usize>(points: &[[f64; D]], query: [f64; D]) -> TestResult
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    let targets = [0.25, -0.5, 1.25, 0.75];
    let kernel = Wendland::try_new(WendlandSmoothness::C4, 1.6)?;
    let dense = FittedField::try_fit(
        value_problem(points, &targets)?,
        metadata()?,
        normalization()?,
        KernelDefinition::from(kernel),
        None,
        dense_options()?,
    )?;
    let sparse = FittedField::try_fit_sparse(
        value_problem(points, &targets)?,
        metadata()?,
        normalization()?,
        kernel,
        None,
        sparse_options(),
    )?;
    for (actual, expected) in sparse.center_weights().iter().zip(dense.center_weights()) {
        assert_close(*actual, *expected, 4096.0 * f64::EPSILON);
    }
    let query = Point::try_new(query)?;
    let dense_evaluation = dense.try_evaluate_with_hessian(query)?;
    let sparse_evaluation = sparse.try_evaluate_with_hessian(query)?;
    assert_close(
        sparse_evaluation.value(),
        dense_evaluation.value(),
        4096.0 * f64::EPSILON,
    );
    for (actual, expected) in sparse_evaluation
        .gradient()
        .components()
        .iter()
        .zip(dense_evaluation.gradient().components())
    {
        assert_close(*actual, *expected, 8192.0 * f64::EPSILON);
    }
    for (actual, expected) in sparse_evaluation
        .hessian()
        .iter()
        .flatten()
        .zip(dense_evaluation.hessian().iter().flatten())
    {
        assert_close(*actual, *expected, 16384.0 * f64::EPSILON);
    }
    assert!(sparse_evaluation.center_evaluations() < sparse_evaluation.total_centers());
    assert_eq!(
        dense_evaluation.center_evaluations(),
        dense_evaluation.total_centers()
    );
    Ok(())
}

#[test]
fn dense_sparse_fit_and_local_evaluation_agree_in_all_dimensions() -> TestResult {
    parity::<1>(&[[0.0], [0.7], [1.4], [3.0]], [0.2])?;
    parity::<2>(
        &[[0.0, 0.0], [0.7, 0.1], [1.4, -0.1], [3.0, 0.0]],
        [0.2, 0.05],
    )?;
    parity::<3>(
        &[
            [0.0, 0.0, 0.0],
            [0.7, 0.1, -0.1],
            [1.4, -0.1, 0.1],
            [3.0, 0.0, 0.0],
        ],
        [0.2, 0.05, -0.03],
    )
}

#[test]
fn mixed_value_derivative_representers_preserve_dense_sparse_parity() -> TestResult {
    let kernel = Wendland::try_new(WendlandSmoothness::C4, 2.0)?;
    let dense = FittedField::try_fit(
        mixed_problem_2d()?,
        metadata()?,
        normalization()?,
        KernelDefinition::from(kernel),
        None,
        dense_options()?,
    )?;
    let sparse = FittedField::try_fit_sparse(
        mixed_problem_2d()?,
        metadata()?,
        normalization()?,
        kernel,
        None,
        sparse_options(),
    )?;
    for (actual, expected) in sparse.center_weights().iter().zip(dense.center_weights()) {
        assert_close(*actual, *expected, 8192.0 * f64::EPSILON);
    }
    let query = Point::try_new([0.3, 0.05])?;
    let dense_evaluation = dense.try_evaluate_with_hessian(query)?;
    let sparse_evaluation = sparse.try_evaluate_with_hessian(query)?;
    assert_close(
        sparse_evaluation.value(),
        dense_evaluation.value(),
        16384.0 * f64::EPSILON,
    );
    for (actual, expected) in sparse_evaluation
        .gradient()
        .components()
        .iter()
        .zip(dense_evaluation.gradient().components())
    {
        assert_close(*actual, *expected, 32768.0 * f64::EPSILON);
    }
    Ok(())
}

#[test]
fn anisotropy_uses_conservative_candidates_and_exact_support() -> TestResult {
    let points = [[0.0, 0.0], [0.6, 0.0], [0.0, 0.6]];
    let kernel = Wendland::try_new(WendlandSmoothness::C2, 1.0)?;
    let anisotropy = GlobalAnisotropy::try_from_transform(
        [[2.0, 0.0], [0.0, 1.0]],
        AnisotropyConditionPolicy::Unbounded,
    )?;
    let system = value_problem(&points, &[1.0, 2.0, 3.0])?.try_assemble_sparse(
        kernel,
        Some(anisotropy),
        sparse_options(),
    )?;
    assert_eq!(system.matrix().get(0, 1), Some(0.0));
    assert!(system.matrix().get(0, 2).is_some_and(|value| value > 0.0));
    assert!(
        system.diagnostics().neighborhood.candidate_term_hits
            > system.diagnostics().neighborhood.supported_pairs
    );
    Ok(())
}

#[test]
fn fixed_grid_storage_scales_with_support_not_dense_area() -> TestResult {
    let mut points = Vec::new();
    for x in 0_u32..8 {
        for y in 0_u32..8 {
            for z in 0_u32..8 {
                points.push([f64::from(x), f64::from(y), f64::from(z)]);
            }
        }
    }
    let targets = vec![0.0; points.len()];
    let system = value_problem(&points, &targets)?.try_assemble_sparse(
        Wendland::try_new(WendlandSmoothness::C2, 1.01)?,
        None,
        sparse_options(),
    )?;
    assert!(system.diagnostics().stored_nonzeros <= 7 * points.len());
    assert!(system.diagnostics().density < 0.02);
    assert_eq!(system.diagnostics().neighborhood.isolated_centers, 0);
    Ok(())
}

#[test]
fn cancellation_memory_and_singular_failures_are_explicit() -> TestResult {
    let points = [[0.0], [1.0], [2.0]];
    let kernel = Wendland::try_new(WendlandSmoothness::C2, 2.0)?;
    let token = CancellationToken::new();
    token.cancel();
    let cancelled = value_problem(&points, &[1.0, 2.0, 3.0])?.try_assemble_sparse_with_control(
        kernel,
        None,
        sparse_options(),
        ExecutionControl::with_cancellation(&token),
    );
    assert!(matches!(
        cancelled,
        Err(SparseFieldAssemblyError::Execution(
            ExecutionError::Cancelled {
                operation: ExecutionOperation::SparseFieldAssembly,
                ..
            }
        ))
    ));

    let tiny = SparseFitOptions::new(SparseFactorization::FaerLlt, NonZeroUsize::MIN);
    assert!(matches!(
        value_problem(&points, &[1.0, 2.0, 3.0])?.try_assemble_sparse(kernel, None, tiny),
        Err(SparseFieldAssemblyError::MemoryLimitExceeded { .. })
    ));

    let duplicate = [[0.0], [0.0]];
    let singular = value_problem(&duplicate, &[1.0, 1.0])?.try_assemble_sparse(
        kernel,
        None,
        sparse_options(),
    )?;
    assert_eq!(
        try_solve_sparse_field(&singular),
        Err(SparseSolveError::FactorizationRejected)
    );
    Ok(())
}
