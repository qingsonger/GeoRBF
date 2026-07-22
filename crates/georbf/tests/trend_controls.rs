//! Independent truth, transition, reference-gradient, and error-path tests.

use std::error::Error;
use std::num::NonZeroUsize;

use georbf::{
    AffineNormalization, AngleUnit, AnisotropyConditionPolicy, AxisOrder, CenterRepresenter,
    CompiledTrendControls, ConditionPolicy, CoordinateMetadata, CrsMetadata, DenseFactorization,
    DenseSolveOptions, Enforcement, ExecutionOptions, FieldId, FieldProblem, FittedField,
    FittedFieldEvaluationError, FunctionalAtom, FunctionalExpr, FunctionalProvenance,
    FunctionalTerm, Gaussian, GeoProject, GlobalAnisotropy, Handedness, KernelDefinition,
    KernelDerivativeOrder, LengthUnit, LocalTrendBackground, LocalTrendControl, Matern,
    MaternSmoothness, Multiquadric, ObservationFunctional, ObservationId, OperationalDomain, Point,
    ProjectField, Regularization, ResolvedTrendDirectionSource, ResolvedTrendOrientation,
    SemanticConstraint, SemanticExpression, SemanticProblemIr, SemanticProvenance,
    SemanticRelation, SmoothRegion, SmoothSpatialWeight, SourceLocation,
    TrendControlCompilationError, TrendControlOrientation, TrendControlPolicy,
    TrendControlPolicyError, TrendDirectionSource, UnitDirection, VerticalDirection,
    try_compile_local_trend_controls,
};

const TEST_MEMORY_LIMIT_BYTES: usize = 64 * 1024 * 1024;

fn assert_send_sync<T: Send + Sync>() {}

fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "expected {expected:.17e}, got {actual:.17e}"
    );
}

fn point<const D: usize>(components: [f64; D]) -> Result<Point<D>, Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    Ok(Point::try_new(components)?)
}

fn direction<const D: usize>(components: [f64; D]) -> Result<UnitDirection<D>, Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    Ok(UnitDirection::try_new(components)?)
}

fn domain<const D: usize>(extent: f64) -> Result<OperationalDomain<D>, Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    Ok(OperationalDomain::try_new(
        point([-extent; D])?,
        point([extent; D])?,
    )?)
}

fn background<const D: usize>() -> Result<LocalTrendBackground<D>, Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    Ok(LocalTrendBackground::new(
        KernelDefinition::from(Gaussian::try_new(1.0)?),
        GlobalAnisotropy::try_isotropic(1.0)?,
        SmoothSpatialWeight::try_constant(0.5)?,
    ))
}

fn policy(minimum: f64, low: f64, jump: f64) -> Result<TrendControlPolicy, Box<dyn Error>> {
    Ok(TrendControlPolicy::try_new(
        AnisotropyConditionPolicy::Maximum(100.0),
        minimum,
        low,
        jump,
    )?)
}

fn spheroid<const D: usize>(
    location: [f64; D],
    axis: [f64; D],
    region: Option<SmoothRegion<D>>,
) -> Result<LocalTrendControl<D>, Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    Ok(LocalTrendControl::new(
        point(location)?,
        KernelDefinition::from(Gaussian::try_new(0.8)?),
        TrendControlOrientation::Spheroidal {
            principal_axis: TrendDirectionSource::Explicit(direction(axis)?),
            axial_length: 2.0,
            transverse_length: 0.75,
        },
        1.25,
        1.4,
        region,
    ))
}

#[test]
fn explicit_spheroidal_and_ellipsoidal_controls_compile_in_order() -> Result<(), Box<dyn Error>> {
    let region = SmoothRegion::try_new(point([-2.0, -2.0])?, point([2.0, 2.0])?, 0.5)?;
    let first = spheroid([0.0, 0.0], [1.0, 0.0], Some(region))?;
    let second = LocalTrendControl::new(
        point([0.5, -0.25])?,
        KernelDefinition::from(Gaussian::try_new(0.9)?),
        TrendControlOrientation::Ellipsoidal {
            principal_axes: [
                TrendDirectionSource::Explicit(direction([0.0, 1.0])?),
                TrendDirectionSource::Explicit(direction([1.0, 0.0])?),
            ],
            axis_lengths: [3.0, 1.0],
            orthogonality_tolerance: 1.0e-12,
        },
        0.8,
        -0.7,
        None,
    );
    let compiled = try_compile_local_trend_controls(
        background()?,
        &[first, second],
        None,
        domain(4.0)?,
        0.25,
        policy(1.0e-12, 1.0e-6, std::f64::consts::FRAC_PI_4)?,
    )?;

    assert_eq!(compiled.mixture().components().len(), 3);
    assert_eq!(compiled.mixture().background_index(), 0);
    let diagnostics = compiled.diagnostics();
    assert_eq!(diagnostics.controls().len(), 2);
    assert_eq!(diagnostics.controls()[0].location(), point([0.0, 0.0])?);
    assert_eq!(
        diagnostics.controls()[1].direction_jump_from_previous_radians(),
        Some(std::f64::consts::FRAC_PI_2)
    );
    assert!(diagnostics.controls()[1].direction_jump_exceeds_policy());
    assert_eq!(diagnostics.jump_exceedance_count(), 1);
    assert_eq!(diagnostics.low_confidence_direction_count(), 0);
    assert_close(
        diagnostics.maximum_direction_jump_radians(),
        std::f64::consts::FRAC_PI_2,
        4.0 * f64::EPSILON,
    );
    assert!(matches!(
        diagnostics.controls()[1].orientation(),
        ResolvedTrendOrientation::Ellipsoidal { axis_lengths, .. }
            if axis_lengths.map(f64::to_bits) == [3.0_f64.to_bits(), 1.0_f64.to_bits()]
    ));

    let coverage = compiled.mixture().try_coverage(point([0.0, 0.0])?)?;
    assert!(coverage.squared_weight_sum() > coverage.background_squared_weight());
    Ok(())
}

#[test]
fn rotated_control_metrics_match_hand_formed_truth() -> Result<(), Box<dyn Error>> {
    let spheroidal_axis = direction([1.0, 1.0])?;
    let ellipsoidal_axes = [direction([1.0, 1.0])?, direction([-1.0, 1.0])?];
    let controls = [
        LocalTrendControl::new(
            point([0.0, 0.0])?,
            KernelDefinition::from(Gaussian::try_new(0.8)?),
            TrendControlOrientation::Spheroidal {
                principal_axis: TrendDirectionSource::Explicit(spheroidal_axis),
                axial_length: 2.0,
                transverse_length: 0.75,
            },
            1.0,
            1.0,
            None,
        ),
        LocalTrendControl::new(
            point([0.0, 0.0])?,
            KernelDefinition::from(Gaussian::try_new(0.8)?),
            TrendControlOrientation::Ellipsoidal {
                principal_axes: ellipsoidal_axes.map(TrendDirectionSource::Explicit),
                axis_lengths: [3.0, 1.0],
                orthogonality_tolerance: 8.0 * f64::EPSILON,
            },
            1.0,
            1.0,
            None,
        ),
    ];
    let compiled = try_compile_local_trend_controls(
        background()?,
        &controls,
        None,
        domain(2.0)?,
        0.25,
        policy(1.0e-12, 1.0e-6, 1.0)?,
    )?;

    let spheroidal_metric = compiled.mixture().components()[1].anisotropy().metric();
    let u = spheroidal_axis.components();
    for (row, actual_row) in spheroidal_metric.iter().enumerate() {
        for (column, actual) in actual_row.iter().enumerate() {
            let projector = u[row] * u[column];
            let expected = projector / 2.0_f64.powi(2)
                + (f64::from(row == column) - projector) / 0.75_f64.powi(2);
            assert_close(*actual, expected, 32.0 * f64::EPSILON);
        }
    }

    let ellipsoidal_metric = compiled.mixture().components()[2].anisotropy().metric();
    for (row, actual_row) in ellipsoidal_metric.iter().enumerate() {
        for (column, actual) in actual_row.iter().enumerate() {
            let expected = ellipsoidal_axes[0].components()[row]
                * ellipsoidal_axes[0].components()[column]
                / 3.0_f64.powi(2)
                + ellipsoidal_axes[1].components()[row] * ellipsoidal_axes[1].components()[column];
            assert_close(*actual, expected, 32.0 * f64::EPSILON);
        }
    }
    Ok(())
}

#[test]
fn regional_gate_is_exactly_c2_zero_at_boundaries() -> Result<(), Box<dyn Error>> {
    let region = SmoothRegion::try_new(point([-1.0, -1.0])?, point([1.0, 1.0])?, 0.25)?;
    let control = spheroid([0.0, 0.0], [1.0, 0.0], Some(region))?;
    let compiled = try_compile_local_trend_controls(
        background()?,
        &[control],
        None,
        domain(2.0)?,
        0.25,
        policy(1.0e-12, 1.0e-6, 1.0)?,
    )?;
    let background_only = georbf::LocalTrendMixture::try_new(
        vec![georbf::LocalTrendComponent::new(
            KernelDefinition::from(Gaussian::try_new(1.0)?),
            GlobalAnisotropy::try_isotropic(1.0)?,
            SmoothSpatialWeight::try_constant(0.5)?,
        )],
        0,
        domain(2.0)?,
        0.25,
    )?;
    let center = point([0.2, -0.1])?;
    for query in [[-1.0, 0.0], [1.0, 0.0], [0.0, -1.0], [0.0, 1.0]] {
        let query = point(query)?;
        let actual =
            compiled
                .mixture()
                .try_evaluate(query, center, KernelDerivativeOrder::Second)?;
        let expected =
            background_only.try_evaluate(query, center, KernelDerivativeOrder::Second)?;
        assert_eq!(actual.value().to_bits(), expected.value().to_bits());
        assert_eq!(actual.gradient(), expected.gradient());
        assert_eq!(actual.hessian(), expected.hessian());
    }
    let outside = compiled.mixture().try_coverage(point([1.1, 0.0])?)?;
    assert_eq!(outside.active_components(), 1);
    assert_eq!(
        outside.squared_weight_sum().to_bits(),
        outside.background_squared_weight().to_bits()
    );
    Ok(())
}

fn mixture_value(
    compiled: &CompiledTrendControls<2>,
    query: [f64; 2],
    center: Point<2>,
) -> Result<f64, Box<dyn Error>> {
    Ok(compiled
        .mixture()
        .try_evaluate(point(query)?, center, KernelDerivativeOrder::Value)?
        .value())
}

#[test]
fn regional_gaussian_product_rules_match_finite_differences() -> Result<(), Box<dyn Error>> {
    let region = SmoothRegion::try_new(point([-1.0, -1.0])?, point([1.0, 1.0])?, 0.4)?;
    let control = spheroid([0.1, -0.2], [1.0, 0.0], Some(region))?;
    let compiled = try_compile_local_trend_controls(
        background()?,
        &[control],
        None,
        domain(2.0)?,
        0.25,
        policy(1.0e-12, 1.0e-6, 1.0)?,
    )?;
    let query = [-0.82, 0.15];
    let center = point([0.35, -0.3])?;
    let analytic =
        compiled
            .mixture()
            .try_evaluate(point(query)?, center, KernelDerivativeOrder::Second)?;
    let gradient = analytic.gradient().ok_or("missing gradient")?;
    let hessian = analytic.hessian().ok_or("missing Hessian")?;
    let gradient_step = 2.0e-6;
    for axis in 0..2 {
        let mut plus = query;
        let mut minus = query;
        plus[axis] += gradient_step;
        minus[axis] -= gradient_step;
        let finite_difference = (mixture_value(&compiled, plus, center)?
            - mixture_value(&compiled, minus, center)?)
            / (2.0 * gradient_step);
        assert!((gradient[axis] - finite_difference).abs() <= 2.0e-7);
    }
    let h = 1.0e-4;
    let base = mixture_value(&compiled, query, center)?;
    for axis in 0..2 {
        let mut plus = query;
        let mut minus = query;
        plus[axis] += h;
        minus[axis] -= h;
        let finite_difference = (mixture_value(&compiled, plus, center)? - 2.0 * base
            + mixture_value(&compiled, minus, center)?)
            / (h * h);
        assert!((hessian[axis][axis] - finite_difference).abs() <= 2.0e-5);
    }
    let mixed_finite_difference = (mixture_value(&compiled, [query[0] + h, query[1] + h], center)?
        - mixture_value(&compiled, [query[0] + h, query[1] - h], center)?
        - mixture_value(&compiled, [query[0] - h, query[1] + h], center)?
        + mixture_value(&compiled, [query[0] - h, query[1] - h], center)?)
        / (4.0 * h * h);
    assert!((hessian[0][1] - mixed_finite_difference).abs() <= 2.0e-5);
    assert!((hessian[1][0] - mixed_finite_difference).abs() <= 2.0e-5);
    Ok(())
}

#[test]
fn regional_derivatives_survive_gaussian_value_underflow() -> Result<(), Box<dyn Error>> {
    let region = SmoothRegion::try_new(point([-1.0e-153])?, point([5.0e-153])?, 2.0e-153)?;
    let control = LocalTrendControl::new(
        point([0.0])?,
        KernelDefinition::from(Gaussian::try_new(1.0)?),
        TrendControlOrientation::Spheroidal {
            principal_axis: TrendDirectionSource::Explicit(direction([1.0])?),
            axial_length: 1.0,
            transverse_length: 1.0,
        },
        1.0e-154,
        1.0,
        Some(region),
    );
    let tiny_background = LocalTrendBackground::new(
        KernelDefinition::from(Gaussian::try_new(1.0)?),
        GlobalAnisotropy::try_isotropic(1.0)?,
        SmoothSpatialWeight::try_constant(1.0e-160)?,
    );
    let compiled = try_compile_local_trend_controls(
        tiny_background,
        &[control],
        None,
        domain(1.0)?,
        1.0e-161,
        policy(1.0e-12, 1.0, 1.0)?,
    )?;
    let evaluation = compiled.mixture().try_evaluate(
        point([4.472_135_954_999_58e-153])?,
        point([0.0])?,
        KernelDerivativeOrder::Second,
    )?;
    assert!(evaluation.value().is_finite());
    let gradient = evaluation.gradient().ok_or("missing gradient")?[0];
    let hessian = evaluation.hessian().ok_or("missing Hessian")?[0][0];
    assert!(gradient.is_finite() && gradient != 0.0);
    assert!(hessian.is_finite() && hessian != 0.0);
    Ok(())
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

fn value_expression<const D: usize>(
    sample: [f64; D],
    identifier: u64,
) -> Result<FunctionalExpr<D>, Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    Ok(FunctionalExpr::try_new([FunctionalTerm::try_new(
        1.0,
        FunctionalAtom::value(point(sample)?, FunctionalProvenance::new(identifier)),
    )?])?)
}

fn fitted_two_point<const D: usize>(
    targets: [f64; 2],
    normalization: AffineNormalization<D>,
    kernel: KernelDefinition<D>,
) -> Result<FittedField<D>, Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    let mut constraints = Vec::new();
    let mut centers = Vec::new();
    for (index, coordinate) in [-1.0, 1.0].into_iter().enumerate() {
        let identifier = u64::try_from(index + 1)?;
        let sample = std::array::from_fn(|axis| if axis == 0 { coordinate } else { 0.0 });
        let expression = value_expression(sample, identifier)?;
        centers.push(CenterRepresenter::new(expression.clone()));
        constraints.push(SemanticConstraint::try_new(
            SemanticProvenance::try_new(
                ObservationId::new(identifier),
                SourceLocation::try_new(
                    "trend-reference.csv".to_owned(),
                    NonZeroUsize::new(index + 1).ok_or("source line")?,
                )?,
                "m".to_owned(),
                format!("reference[{index}]"),
                None,
            )?,
            SemanticRelation::Equality {
                expression: SemanticExpression::try_new(
                    ObservationFunctional::new(expression),
                    0.0,
                )?,
                target: targets[index],
            },
            Enforcement::Hard,
        )?);
    }
    let problem = FieldProblem::try_new(
        SemanticProblemIr::try_new(constraints, ExecutionOptions::default())?,
        centers,
    )?;
    Ok(FittedField::try_fit(
        problem,
        metadata()?,
        normalization,
        kernel,
        None,
        DenseSolveOptions::try_new(
            DenseFactorization::Cholesky,
            Regularization::None,
            ConditionPolicy::default(),
            4,
            NonZeroUsize::new(TEST_MEMORY_LIMIT_BYTES).ok_or("memory limit")?,
        )?,
    )?)
}

fn fitted_axis<const D: usize>() -> Result<FittedField<D>, Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    fitted_two_point(
        [-1.0, 1.0],
        AffineNormalization::try_new(
            point([0.0; D])?,
            std::array::from_fn(|row| std::array::from_fn(|column| f64::from(row == column))),
        )?,
        KernelDefinition::from(Gaussian::try_new(1.0)?),
    )
}

fn reference_control<const D: usize>(
    location: [f64; D],
    field_id: FieldId,
) -> Result<LocalTrendControl<D>, Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    Ok(LocalTrendControl::new(
        point(location)?,
        KernelDefinition::from(Gaussian::try_new(1.0)?),
        TrendControlOrientation::Spheroidal {
            principal_axis: TrendDirectionSource::ReferenceFieldGradient(field_id),
            axial_length: 1.0,
            transverse_length: 1.0,
        },
        1.0,
        1.0,
        None,
    ))
}

#[test]
fn reference_gradient_is_normalized_with_provenance_and_confidence() -> Result<(), Box<dyn Error>> {
    let field_id = FieldId::new(7);
    let project = GeoProject::try_new([ProjectField::new(field_id, fitted_axis::<2>()?)])?;
    let control = LocalTrendControl::new(
        point([0.0, 0.0])?,
        KernelDefinition::from(Gaussian::try_new(0.7)?),
        TrendControlOrientation::Spheroidal {
            principal_axis: TrendDirectionSource::ReferenceFieldGradient(field_id),
            axial_length: 2.0,
            transverse_length: 1.0,
        },
        1.0,
        1.0,
        None,
    );
    let compiled = try_compile_local_trend_controls(
        background()?,
        &[control],
        Some(&project),
        domain(2.0)?,
        0.25,
        policy(1.0e-12, 10.0, 1.0)?,
    )?;
    let ResolvedTrendOrientation::Spheroidal { principal_axis, .. } =
        compiled.diagnostics().controls()[0].orientation()
    else {
        return Err("expected spheroidal diagnostics".into());
    };
    assert!(principal_axis.direction().components()[0] > 0.999_999);
    assert!(principal_axis.direction().components()[1].abs() <= 1.0e-12);
    assert!(matches!(
        principal_axis.source(),
        ResolvedTrendDirectionSource::ReferenceFieldGradient {
            field_id: observed,
            gradient_norm,
            low_confidence: true,
        } if observed == field_id && gradient_norm > 0.0
    ));
    assert_eq!(compiled.diagnostics().low_confidence_direction_count(), 1);

    assert!(matches!(
        try_compile_local_trend_controls(
            background()?,
            &[control],
            None,
            domain(2.0)?,
            0.25,
            policy(1.0e-12, 1.0, 1.0)?,
        ),
        Err(TrendControlCompilationError::MissingReferenceProject { field_id: observed, .. })
            if observed == field_id
    ));
    assert!(matches!(
        try_compile_local_trend_controls(
            background()?,
            &[control],
            Some(&project),
            domain(2.0)?,
            0.25,
            policy(1000.0, 1000.0, 1.0)?,
        ),
        Err(TrendControlCompilationError::ReferenceGradientBelowMinimum { .. })
    ));
    Ok(())
}

#[test]
#[allow(clippy::too_many_lines)]
fn reference_gradient_failures_are_structured_for_all_required_cases() -> Result<(), Box<dyn Error>>
{
    let known = FieldId::new(20);
    let unknown = FieldId::new(21);
    let ordinary_project = GeoProject::try_new([ProjectField::new(known, fitted_axis::<1>()?)])?;
    let unknown_control = reference_control([0.0], unknown)?;
    let unknown_result = try_compile_local_trend_controls(
        background()?,
        &[unknown_control],
        Some(&ordinary_project),
        domain(2.0)?,
        0.25,
        policy(f64::MIN_POSITIVE, 1.0, 1.0)?,
    );

    let unavailable_field = fitted_two_point(
        [-1.0, 1.0],
        AffineNormalization::try_new(point([0.0])?, [[1.0]])?,
        KernelDefinition::from(Matern::try_new(MaternSmoothness::OneHalf, 1.0)?),
    )?;
    let unavailable_project = GeoProject::try_new([ProjectField::new(known, unavailable_field)])?;
    let unavailable_control = reference_control([-1.0], known)?;
    let unavailable_result = try_compile_local_trend_controls(
        background()?,
        &[unavailable_control],
        Some(&unavailable_project),
        domain(2.0)?,
        0.25,
        policy(f64::MIN_POSITIVE, 1.0, 1.0)?,
    );

    let zero_field = fitted_two_point(
        [1.0, 1.0],
        AffineNormalization::try_new(point([0.0])?, [[1.0]])?,
        KernelDefinition::from(Gaussian::try_new(1.0)?),
    )?;
    let zero_project = GeoProject::try_new([ProjectField::new(known, zero_field)])?;
    let zero_control = reference_control([0.0], known)?;
    let zero_result = try_compile_local_trend_controls(
        background()?,
        &[zero_control],
        Some(&zero_project),
        domain(2.0)?,
        0.25,
        policy(f64::MIN_POSITIVE, 1.0, 1.0)?,
    );

    let normalized_gradient = fitted_axis::<2>()?.try_gradient(point([0.0, 0.0])?)?;
    let normalized_axis_gradient = normalized_gradient.components()[0].abs();
    let desired_component = 0.75 * f64::MAX;
    let inverse_scale = desired_component / normalized_axis_gradient;
    let unrepresentable_field = fitted_two_point(
        [-1.0, 1.0],
        AffineNormalization::try_new(
            point([0.0, 0.0])?,
            [[inverse_scale.recip(), -1.0], [0.0, 1.0]],
        )?,
        KernelDefinition::from(Gaussian::try_new(1.0)?),
    )?;
    let unrepresentable_project =
        GeoProject::try_new([ProjectField::new(known, unrepresentable_field)])?;
    let unrepresentable_control = reference_control([0.0, 0.0], known)?;
    let unrepresentable_result = try_compile_local_trend_controls(
        background()?,
        &[unrepresentable_control],
        Some(&unrepresentable_project),
        domain(2.0)?,
        0.25,
        policy(f64::MIN_POSITIVE, 1.0, 1.0)?,
    );

    let cases = [
        (
            "unknown field",
            matches!(
                unknown_result,
                Err(TrendControlCompilationError::UnknownReferenceField {
                    field_id,
                    ..
                }) if field_id == unknown
            ),
        ),
        (
            "unavailable evaluation",
            matches!(
                unavailable_result,
                Err(TrendControlCompilationError::ReferenceEvaluation { source, .. })
                    if matches!(
                        source.as_ref(),
                        FittedFieldEvaluationError::UnavailableAtCenter { .. }
                    )
            ),
        ),
        (
            "zero gradient",
            matches!(
                zero_result,
                Err(TrendControlCompilationError::ReferenceGradientBelowMinimum {
                    norm,
                    ..
                }) if norm == 0.0
            ),
        ),
        (
            "unrepresentable norm",
            matches!(
                unrepresentable_result,
                Err(TrendControlCompilationError::NonRepresentableReferenceGradientNorm { .. })
            ),
        ),
    ];
    for (case, matched) in cases {
        assert!(
            matched,
            "missing structured reference-gradient case: {case}"
        );
    }
    Ok(())
}

#[test]
fn compiler_rejects_excessive_control_condition_number() -> Result<(), Box<dyn Error>> {
    let control = LocalTrendControl::new(
        point([0.0, 0.0])?,
        KernelDefinition::from(Gaussian::try_new(1.0)?),
        TrendControlOrientation::Spheroidal {
            principal_axis: TrendDirectionSource::Explicit(direction([1.0, 0.0])?),
            axial_length: 100.0,
            transverse_length: 1.0,
        },
        1.0,
        1.0,
        None,
    );
    let explicit_policy =
        TrendControlPolicy::try_new(AnisotropyConditionPolicy::Maximum(10.0), 1.0e-12, 1.0, 1.0)?;

    assert!(matches!(
        try_compile_local_trend_controls(
            background()?,
            &[control],
            None,
            domain(2.0)?,
            0.25,
            explicit_policy,
        ),
        Err(TrendControlCompilationError::Anisotropy {
            source: georbf::AnisotropyError::ConditionNumberExceeded { maximum, .. },
            ..
        }) if maximum.to_bits() == 10.0_f64.to_bits()
    ));
    Ok(())
}

#[test]
fn validation_cpd_rejection_dimensions_and_auto_traits() -> Result<(), Box<dyn Error>> {
    assert!(matches!(
        TrendControlPolicy::try_new(AnisotropyConditionPolicy::Unbounded, 1.0, 0.5, 0.2,),
        Err(TrendControlPolicyError::LowConfidenceBelowMinimum { .. })
    ));
    assert!(SmoothRegion::try_new(point([-1.0])?, point([1.0])?, 1.1).is_err());

    let cpd = LocalTrendControl::new(
        point([0.0])?,
        KernelDefinition::from(Multiquadric::try_new(1.0)?),
        TrendControlOrientation::Spheroidal {
            principal_axis: TrendDirectionSource::Explicit(direction([1.0])?),
            axial_length: 1.0,
            transverse_length: 1.0,
        },
        1.0,
        1.0,
        None,
    );
    assert!(matches!(
        try_compile_local_trend_controls(
            background()?,
            &[cpd],
            None,
            domain(2.0)?,
            0.25,
            policy(1.0e-12, 1.0, 1.0)?,
        ),
        Err(TrendControlCompilationError::Mixture(
            georbf::LocalTrendConstructionError::ConditionallyPositiveDefiniteComponent {
                component: 1,
                ..
            }
        ))
    ));

    let one = spheroid([0.0], [1.0], None)?;
    let three = spheroid([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], None)?;
    assert_eq!(
        try_compile_local_trend_controls(
            background()?,
            &[one],
            None,
            domain(2.0)?,
            0.25,
            policy(1.0e-12, 1.0, 1.0)?,
        )?
        .mixture()
        .components()
        .len(),
        2
    );
    assert_eq!(
        try_compile_local_trend_controls(
            background()?,
            &[three],
            None,
            domain(2.0)?,
            0.25,
            policy(1.0e-12, 1.0, 1.0)?,
        )?
        .mixture()
        .components()
        .len(),
        2
    );
    assert_send_sync::<CompiledTrendControls<1>>();
    assert_send_sync::<CompiledTrendControls<2>>();
    assert_send_sync::<CompiledTrendControls<3>>();
    Ok(())
}

#[test]
fn compact_control_skips_overflowing_fixed_kernel_when_query_factor_is_zero()
-> Result<(), Box<dyn Error>> {
    let region = SmoothRegion::try_new(point([-1.0])?, point([1.0])?, 0.25)?;
    let control = LocalTrendControl::new(
        point([0.0])?,
        KernelDefinition::from(Gaussian::try_new(1.0)?),
        TrendControlOrientation::Spheroidal {
            principal_axis: TrendDirectionSource::Explicit(direction([1.0])?),
            axial_length: 0.5,
            transverse_length: 0.5,
        },
        1.0,
        1.0,
        Some(region),
    );
    let compiled = try_compile_local_trend_controls(
        background()?,
        &[control],
        None,
        domain(2.0)?,
        0.25,
        policy(1.0e-12, 1.0, 1.0)?,
    )?;

    let evaluation = compiled.mixture().try_evaluate(
        point([f64::MAX])?,
        point([0.0])?,
        KernelDerivativeOrder::Second,
    )?;
    let reversed = compiled.mixture().try_evaluate(
        point([0.0])?,
        point([f64::MAX])?,
        KernelDerivativeOrder::Second,
    )?;

    for result in [evaluation, reversed] {
        assert_eq!(result.value().to_bits(), 0.0_f64.to_bits());
        assert_eq!(
            result.gradient().unwrap_or([f64::NAN]).map(f64::to_bits),
            [0.0_f64.to_bits()]
        );
        assert_eq!(
            result
                .hessian()
                .unwrap_or([[f64::NAN]])
                .map(|row| row.map(f64::to_bits)),
            [[0.0_f64.to_bits()]]
        );
    }
    Ok(())
}

#[test]
fn gaussian_weight_underflow_does_not_erase_representable_mixture_value()
-> Result<(), Box<dyn Error>> {
    let strength = 1.0e154;
    let separation = 47.0;
    let fixed_kernel_length = 100.0;
    let control = LocalTrendControl::new(
        point([0.0])?,
        KernelDefinition::from(Gaussian::try_new(fixed_kernel_length)?),
        TrendControlOrientation::Spheroidal {
            principal_axis: TrendDirectionSource::Explicit(direction([1.0])?),
            axial_length: 1.0,
            transverse_length: 1.0,
        },
        1.0,
        strength,
        None,
    );
    let compiled = try_compile_local_trend_controls(
        background()?,
        &[control],
        None,
        domain(50.0)?,
        0.25,
        policy(1.0e-12, 1.0, 1.0)?,
    )?;

    let expected = (2.0 * strength.ln()
        - 0.5 * separation * separation
        - 0.5 * (separation / fixed_kernel_length).powi(2))
    .exp();
    assert!(expected.is_finite() && expected != 0.0);

    for (query, center) in [(0.0, separation), (separation, 0.0)] {
        let actual = compiled
            .mixture()
            .try_evaluate(
                point([query])?,
                point([center])?,
                KernelDerivativeOrder::Value,
            )?
            .value();
        assert_close(actual, expected, expected * 32.0 * f64::EPSILON);
    }
    Ok(())
}
