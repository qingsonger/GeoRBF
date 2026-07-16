//! Independent truth, capability, persistence-input, and thread-safety tests.

use std::error::Error;
use std::num::NonZeroUsize;
use std::sync::Arc;

use georbf::{
    AffineNormalization, AngleUnit, AnisotropyConditionPolicy, AxisOrder, CenterRepresenter,
    ConditionPolicy, CoordinateMetadata, CpdRankDecision, CrsMetadata, DenseFactorization,
    DenseSolveOptions, Enforcement, ExecutionOptions, FieldProblem, FittedField,
    FittedFieldEvaluationError, FittedFieldOutput, FunctionalAtom, FunctionalExpr,
    FunctionalProvenance, FunctionalTerm, Gaussian, GlobalAnisotropy, Handedness, KernelDefinition,
    KernelDerivativeCapability, LengthUnit, Matern, MaternSmoothness, ObservationFunctional,
    ObservationId, Point, PolyharmonicSpline, Regularization, SemanticConstraint,
    SemanticExpression, SemanticProblemIr, SemanticProvenance, SemanticRelation, SourceLocation,
    UnitDirection, VerticalDirection,
};

const TEST_MEMORY_LIMIT_BYTES: usize = 64 * 1024 * 1024;

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

fn identity_normalization<const D: usize>() -> Result<AffineNormalization<D>, Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    Ok(AffineNormalization::try_new(
        Point::try_new([0.0; D])?,
        std::array::from_fn(|row| std::array::from_fn(|column| f64::from(row == column))),
    )?)
}

fn options(factorization: DenseFactorization) -> Result<DenseSolveOptions, Box<dyn Error>> {
    Ok(DenseSolveOptions::try_new(
        factorization,
        Regularization::None,
        ConditionPolicy::default(),
        4,
        NonZeroUsize::new(TEST_MEMORY_LIMIT_BYTES).ok_or("memory limit")?,
    )?)
}

fn value_expression<const D: usize>(
    point: [f64; D],
    identifier: u64,
) -> Result<FunctionalExpr<D>, Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    Ok(FunctionalExpr::try_new([FunctionalTerm::try_new(
        1.0,
        FunctionalAtom::value(
            Point::try_new(point)?,
            FunctionalProvenance::new(identifier),
        ),
    )?])?)
}

fn derivative_expression(
    point: [f64; 1],
    identifier: u64,
) -> Result<FunctionalExpr<1>, Box<dyn Error>> {
    Ok(FunctionalExpr::try_new([FunctionalTerm::try_new(
        1.0,
        FunctionalAtom::directional_derivative(
            Point::try_new(point)?,
            UnitDirection::try_new([1.0])?,
            FunctionalProvenance::new(identifier),
        ),
    )?])?)
}

fn problem<const D: usize>(
    expressions_and_targets: impl IntoIterator<Item = (FunctionalExpr<D>, f64)>,
) -> Result<FieldProblem<D>, Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    let mut constraints = Vec::new();
    let mut centers = Vec::new();
    for (index, (expression, target)) in expressions_and_targets.into_iter().enumerate() {
        let line = index + 1;
        let identifier = u64::try_from(line)?;
        centers.push(CenterRepresenter::new(expression.clone()));
        constraints.push(SemanticConstraint::try_new(
            SemanticProvenance::try_new(
                ObservationId::new(identifier),
                SourceLocation::try_new(
                    "model-test.csv".to_owned(),
                    NonZeroUsize::new(line).ok_or("line")?,
                )?,
                "m".to_owned(),
                format!("field.equalities[{index}]"),
                Some("model test".to_owned()),
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
        SemanticProblemIr::try_new(constraints, ExecutionOptions::default())?,
        centers,
    )?)
}

fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    let scale = actual.abs().max(expected.abs()).max(1.0);
    assert!(
        (actual - expected).abs() <= tolerance * scale,
        "expected {expected:.17e}, got {actual:.17e}"
    );
}

#[test]
fn gaussian_value_gradient_hessian_use_original_coordinate_chain_rule() -> Result<(), Box<dyn Error>>
{
    let kernel = Gaussian::try_new(1.25)?;
    let normalization = AffineNormalization::try_new(Point::try_new([10.0])?, [[2.0]])?;
    let model = FittedField::try_fit(
        problem([
            (value_expression([-1.0], 10)?, 2.0),
            (value_expression([1.0], 20)?, -1.0),
        ])?,
        metadata()?,
        normalization,
        KernelDefinition::from(kernel),
        None,
        options(DenseFactorization::Cholesky)?,
    )?;

    let query_original = Point::try_new([11.0])?;
    let query_normalized = 0.5_f64;
    let mut expected_value = 0.0;
    let mut expected_gradient_normalized = 0.0;
    let mut expected_hessian_normalized = 0.0;
    let inverse_length_squared = 1.0 / 1.25_f64.powi(2);
    for (center, weight) in [-1.0_f64, 1.0_f64]
        .into_iter()
        .zip(model.center_weights().iter().copied())
    {
        let displacement = query_normalized - center;
        let value = (-0.5 * displacement.powi(2) * inverse_length_squared).exp();
        expected_value += weight * value;
        expected_gradient_normalized += weight * (-displacement * inverse_length_squared) * value;
        expected_hessian_normalized += weight
            * (displacement.powi(2) * inverse_length_squared.powi(2) - inverse_length_squared)
            * value;
    }

    let evaluated = model.try_evaluate_with_hessian(query_original)?;
    assert_close(evaluated.value(), expected_value, 256.0 * f64::EPSILON);
    assert_close(
        evaluated.gradient().components()[0],
        expected_gradient_normalized / 2.0,
        512.0 * f64::EPSILON,
    );
    assert_close(
        evaluated.hessian()[0][0],
        expected_hessian_normalized / 4.0,
        1024.0 * f64::EPSILON,
    );
    assert_eq!(
        model.capabilities().hessian(),
        KernelDerivativeCapability::SupportedEverywhere
    );
    Ok(())
}

#[test]
fn directional_center_uses_exact_center_argument_signs_through_third_order()
-> Result<(), Box<dyn Error>> {
    let length_scale = 1.5_f64;
    let model = FittedField::try_fit(
        problem([(derivative_expression([0.0], 7)?, 2.0)])?,
        metadata()?,
        identity_normalization()?,
        KernelDefinition::from(Gaussian::try_new(length_scale)?),
        None,
        options(DenseFactorization::Cholesky)?,
    )?;
    let query = 0.75_f64;
    let inverse_length_squared = 1.0 / length_scale.powi(2);
    let kernel_value = (-0.5 * query.powi(2) * inverse_length_squared).exp();
    let weight = model.center_weights()[0];
    let expected_value = weight * query * inverse_length_squared * kernel_value;
    let expected_gradient = weight
        * (inverse_length_squared - query.powi(2) * inverse_length_squared.powi(2))
        * kernel_value;
    let expected_hessian = weight
        * (-3.0 * query * inverse_length_squared.powi(2)
            + query.powi(3) * inverse_length_squared.powi(3))
        * kernel_value;

    let evaluated = model.try_evaluate_with_hessian(Point::try_new([query])?)?;
    assert_close(evaluated.value(), expected_value, 512.0 * f64::EPSILON);
    assert_close(
        evaluated.gradient().components()[0],
        expected_gradient,
        1024.0 * f64::EPSILON,
    );
    assert_close(
        evaluated.hessian()[0][0],
        expected_hessian,
        2048.0 * f64::EPSILON,
    );
    Ok(())
}

#[test]
fn anisotropy_and_normalization_apply_both_original_coordinate_chain_rules()
-> Result<(), Box<dyn Error>> {
    let anisotropy = GlobalAnisotropy::try_from_transform(
        [[0.5, 0.0], [0.0, 0.25]],
        AnisotropyConditionPolicy::Unbounded,
    )?;
    let model = FittedField::try_fit(
        problem([(value_expression([0.0, 0.0], 9)?, 1.0)])?,
        metadata()?,
        AffineNormalization::try_new(Point::try_new([10.0, -4.0])?, [[2.0, 0.0], [0.0, 4.0]])?,
        KernelDefinition::from(Gaussian::try_new(1.0)?),
        Some(anisotropy),
        options(DenseFactorization::Cholesky)?,
    )?;

    let value = (-0.25_f64).exp();
    let evaluated = model.try_evaluate_with_hessian(Point::try_new([12.0, 4.0])?)?;
    assert_close(evaluated.value(), value, 256.0 * f64::EPSILON);
    assert_close(
        evaluated.gradient().components()[0],
        -0.125 * value,
        512.0 * f64::EPSILON,
    );
    assert_close(
        evaluated.gradient().components()[1],
        -0.03125 * value,
        512.0 * f64::EPSILON,
    );
    assert_close(
        evaluated.hessian()[0][0],
        -0.046_875 * value,
        1024.0 * f64::EPSILON,
    );
    assert_close(
        evaluated.hessian()[0][1],
        0.003_906_25 * value,
        1024.0 * f64::EPSILON,
    );
    assert_close(
        evaluated.hessian()[1][0],
        0.003_906_25 * value,
        1024.0 * f64::EPSILON,
    );
    assert_close(
        evaluated.hessian()[1][1],
        -0.002_929_687_5 * value,
        1024.0 * f64::EPSILON,
    );
    Ok(())
}

#[test]
fn cpd_quadratic_truth_and_record_order_are_deterministic() -> Result<(), Box<dyn Error>> {
    let quadratic = |x: f64| 1.0 + 2.0 * x + 3.0 * x * x;
    let fit = || -> Result<FittedField<1>, Box<dyn Error>> {
        let fitted_problem = problem([
            (value_expression([-1.0], 1)?, quadratic(-1.0)),
            (value_expression([0.0], 2)?, quadratic(0.0)),
            (value_expression([2.0], 3)?, quadratic(2.0)),
            (value_expression([3.0], 4)?, quadratic(3.0)),
        ])?;
        Ok(FittedField::try_fit(
            fitted_problem,
            metadata()?,
            AffineNormalization::try_new(Point::try_new([5.0])?, [[2.0]])?,
            KernelDefinition::from(PolyharmonicSpline::try_new(4)?),
            None,
            options(DenseFactorization::PivotedLblt)?,
        )?)
    };
    let model = fit()?;
    let repeated = fit()?;

    for weight in model.center_weights() {
        assert_close(*weight, 0.0, 4096.0 * f64::EPSILON);
    }
    let record = model.record();
    assert_eq!(record.build_version(), env!("CARGO_PKG_VERSION"));
    assert_eq!(record.centers().len(), 4);
    let cpd = record.cpd_assembly().ok_or("CPD assembly evidence")?;
    assert_eq!(
        cpd,
        repeated
            .record()
            .cpd_assembly()
            .ok_or("repeated CPD assembly evidence")?
    );
    assert_eq!(record.polynomial_space(), Some(cpd.polynomial_space()));
    assert_eq!(
        cpd.polynomial_space()
            .terms()
            .iter()
            .map(|term| *term.exponents())
            .collect::<Vec<_>>(),
        [[0], [1], [2]]
    );
    let null_space = cpd.null_space();
    assert_eq!(
        null_space.actions().values(),
        [1.0, -1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 2.0, 4.0, 1.0, 3.0, 9.0]
    );
    let rank = null_space.diagnostics();
    assert_eq!(rank.decision, CpdRankDecision::FullRank);
    assert_eq!((rank.rows, rank.columns), (4, 3));
    assert_eq!((rank.rrqr_rank, rank.svd_rank), (3, 3));
    assert_eq!(rank.rrqr_diagonal.len(), 3);
    assert_eq!(rank.singular_values.len(), 3);
    assert!(!rank.threshold_adjacent);
    assert!(!rank.rank_disagreement);
    assert_eq!(
        (null_space.basis().rows(), null_space.basis().columns()),
        (4, 1)
    );
    let quality = null_space.quality();
    assert!(quality.side_condition_residual <= quality.tolerance);
    assert!(quality.original_side_condition_residual.is_finite());
    assert!(quality.orthonormality_residual <= quality.tolerance);
    assert_eq!(
        (
            cpd.projected_energy().rows(),
            cpd.projected_energy().columns()
        ),
        (1, 1)
    );
    assert_eq!(cpd.projected_energy().values().len(), 1);
    assert!(cpd.projected_energy().values()[0].is_finite());
    assert!(cpd.projected_energy().values()[0] > 0.0);
    for (actual, expected) in record
        .polynomial_coefficients()
        .iter()
        .copied()
        .zip([1.0, 2.0, 3.0])
    {
        assert_close(actual, expected, 4096.0 * f64::EPSILON);
    }

    let query = Point::try_new([8.0])?;
    let normalized = 1.5;
    let evaluated = model.try_evaluate_with_hessian(query)?;
    assert_close(
        evaluated.value(),
        quadratic(normalized),
        4096.0 * f64::EPSILON,
    );
    assert_close(
        evaluated.gradient().components()[0],
        1.0 + 3.0 * normalized,
        4096.0 * f64::EPSILON,
    );
    assert_close(evaluated.hessian()[0][0], 6.0 / 4.0, 4096.0 * f64::EPSILON);
    Ok(())
}

#[test]
fn exact_center_capabilities_reject_only_unavailable_outputs() -> Result<(), Box<dyn Error>> {
    let center = Point::try_new([0.0])?;
    let value_model = FittedField::try_fit(
        problem([(value_expression([0.0], 1)?, 1.0)])?,
        metadata()?,
        identity_normalization()?,
        KernelDefinition::from(Matern::try_new(MaternSmoothness::OneHalf, 1.0)?),
        None,
        options(DenseFactorization::Cholesky)?,
    )?;
    assert_eq!(
        value_model.capabilities().value(),
        KernelDerivativeCapability::SupportedEverywhere
    );
    assert_eq!(
        value_model.capabilities().gradient(),
        KernelDerivativeCapability::SupportedAwayFromCenters
    );
    assert_eq!(
        value_model.capabilities().hessian(),
        KernelDerivativeCapability::SupportedAwayFromCenters
    );
    assert_close(value_model.try_value(center)?, 1.0, 64.0 * f64::EPSILON);
    assert!(matches!(
        value_model.try_gradient(center),
        Err(FittedFieldEvaluationError::UnavailableAtCenter {
            output: FittedFieldOutput::Gradient,
            center_index: 0,
            term_index: 0,
            ..
        })
    ));
    assert!(matches!(
        value_model.try_hessian(center),
        Err(FittedFieldEvaluationError::UnavailableAtCenter {
            output: FittedFieldOutput::Hessian,
            center_index: 0,
            term_index: 0,
            ..
        })
    ));
    assert!(
        value_model
            .try_evaluate_with_hessian(Point::try_new([0.25])?)
            .is_ok()
    );

    let derivative_model = FittedField::try_fit(
        problem([(derivative_expression([0.0], 2)?, 1.0)])?,
        metadata()?,
        identity_normalization()?,
        KernelDefinition::from(Matern::try_new(MaternSmoothness::ThreeHalves, 1.0)?),
        None,
        options(DenseFactorization::Cholesky)?,
    )?;
    assert_eq!(
        derivative_model.capabilities().gradient(),
        KernelDerivativeCapability::SupportedEverywhere
    );
    assert_eq!(
        derivative_model.capabilities().hessian(),
        KernelDerivativeCapability::SupportedAwayFromCenters
    );
    assert!(derivative_model.try_evaluate(center).is_ok());
    assert!(matches!(
        derivative_model.try_hessian(center),
        Err(FittedFieldEvaluationError::UnavailableAtCenter {
            output: FittedFieldOutput::Hessian,
            ..
        })
    ));
    Ok(())
}

#[test]
fn fitted_fields_are_send_sync_and_support_deterministic_multithread_reads()
-> Result<(), Box<dyn Error>> {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<FittedField<1>>();

    let model = Arc::new(FittedField::try_fit(
        problem([
            (value_expression([-1.0], 1)?, -0.5),
            (value_expression([1.0], 2)?, 1.5),
        ])?,
        metadata()?,
        identity_normalization()?,
        KernelDefinition::from(Gaussian::try_new(0.75)?),
        None,
        options(DenseFactorization::Cholesky)?,
    )?);
    let query = Point::try_new([0.125])?;
    let expected = model.try_evaluate_with_hessian(query)?;
    let expected_bits = (
        expected.value().to_bits(),
        expected.gradient().components()[0].to_bits(),
        expected.hessian()[0][0].to_bits(),
    );

    let mut threads = Vec::new();
    for _ in 0..8 {
        let model = Arc::clone(&model);
        threads.push(std::thread::spawn(move || {
            let mut observed = None;
            for _ in 0..100 {
                let value = model
                    .try_evaluate_with_hessian(query)
                    .map_err(|error| error.to_string())?;
                let bits = (
                    value.value().to_bits(),
                    value.gradient().components()[0].to_bits(),
                    value.hessian()[0][0].to_bits(),
                );
                if let Some(previous) = observed
                    && bits != previous
                {
                    return Err("nondeterministic thread-local result".to_owned());
                }
                observed = Some(bits);
            }
            observed.ok_or_else(|| "missing evaluation".to_owned())
        }));
    }
    for thread in threads {
        assert_eq!(
            thread.join().map_err(|_| "thread panicked")??,
            expected_bits
        );
    }
    Ok(())
}
