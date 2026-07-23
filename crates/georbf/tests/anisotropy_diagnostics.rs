//! Independent schema, confidence-region, direction-jump, and coverage tests.

use std::error::Error;
use std::num::NonZeroUsize;

use georbf::{
    AffineNormalization, AngleUnit, AnisotropyConditionPolicy, AnisotropyControlOrientation,
    AnisotropyDiagnosticExport, AnisotropyDiagnosticExportError, AxisOrder, CenterRepresenter,
    ConditionPolicy, CoordinateMetadata, CrsMetadata, DenseFactorization, DenseSolveOptions,
    Enforcement, ExecutionOptions, FieldId, FieldProblem, FittedField, FunctionalAtom,
    FunctionalExpr, FunctionalProvenance, FunctionalTerm, Gaussian, GeoProject, GlobalAnisotropy,
    Handedness, KernelDefinition, LengthUnit, LocalTrendBackground, LocalTrendControl,
    ObservationFunctional, ObservationId, OperationalDomain, Point, ProjectField, Regularization,
    ResolvedTrendDirectionSource, SemanticConstraint, SemanticExpression, SemanticProblemIr,
    SemanticProvenance, SemanticRelation, SmoothRegion, SmoothSpatialWeight, SourceLocation,
    TrendControlOrientation, TrendControlPolicy, TrendDirectionSource, UnitDirection,
    VerticalDirection, try_compile_local_trend_controls, try_export_anisotropy_diagnostics,
};

const TEST_MEMORY_LIMIT_BYTES: usize = 64 * 1024 * 1024;

fn assert_send_sync<T: Send + Sync>() {}

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
        GlobalAnisotropy::try_isotropic(2.0)?,
        SmoothSpatialWeight::try_constant(0.5)?,
    ))
}

fn policy(maximum_jump: f64) -> Result<TrendControlPolicy, Box<dyn Error>> {
    Ok(TrendControlPolicy::try_new(
        AnisotropyConditionPolicy::Maximum(100.0),
        f64::MIN_POSITIVE,
        10.0,
        maximum_jump,
    )?)
}

fn spheroid<const D: usize>(
    location: [f64; D],
    axis: [f64; D],
    strength: f64,
    region: Option<SmoothRegion<D>>,
) -> Result<LocalTrendControl<D>, Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    Ok(LocalTrendControl::new(
        point(location)?,
        KernelDefinition::from(Gaussian::try_new(1.0)?),
        TrendControlOrientation::Spheroidal {
            principal_axis: TrendDirectionSource::Explicit(direction(axis)?),
            axial_length: 3.0,
            transverse_length: 1.5,
        },
        1.0,
        strength,
        region,
    ))
}

#[test]
fn diagnostic_schema_preserves_controls_background_weights_and_coverage()
-> Result<(), Box<dyn Error>> {
    let controls = [
        spheroid([0.0, 0.0], [1.0, 0.0], 2.0, None)?,
        LocalTrendControl::new(
            point([1.0, 0.0])?,
            KernelDefinition::from(Gaussian::try_new(0.75)?),
            TrendControlOrientation::Ellipsoidal {
                principal_axes: [
                    TrendDirectionSource::Explicit(direction([-1.0, 0.0])?),
                    TrendDirectionSource::Explicit(direction([0.0, 1.0])?),
                ],
                axis_lengths: [4.0, 1.0],
                orthogonality_tolerance: 8.0 * f64::EPSILON,
            },
            1.0,
            -1.5,
            None,
        ),
    ];
    let compiled = try_compile_local_trend_controls(
        background()?,
        &controls,
        None,
        domain(3.0)?,
        0.25,
        policy(0.25)?,
    )?;
    let sample = point([0.0, 0.0])?;
    let export = try_export_anisotropy_diagnostics(&compiled, &[sample])?;

    assert_eq!(export.controls().len(), 2);
    assert_eq!(export.controls()[0].control_index(), 0);
    assert_eq!(export.controls()[0].component_index(), 1);
    assert_eq!(export.controls()[1].component_index(), 2);
    assert_eq!(export.controls()[0].position(), point([0.0, 0.0])?);
    assert_eq!(export.controls()[0].strength().to_bits(), 2.0_f64.to_bits());
    assert_eq!(
        export.controls()[1].strength().to_bits(),
        (-1.5_f64).to_bits()
    );
    assert!(export.controls()[0].condition_number() >= 1.0);
    assert!(matches!(
        export.controls()[0].orientation(),
        AnisotropyControlOrientation::Spheroidal {
            axial_length,
            transverse_length,
            ..
        } if axial_length.to_bits() == 3.0_f64.to_bits()
            && transverse_length.to_bits() == 1.5_f64.to_bits()
    ));
    assert!(matches!(
        export.controls()[1].orientation(),
        AnisotropyControlOrientation::Ellipsoidal { axis_lengths, .. }
            if axis_lengths.map(f64::to_bits) == [4.0_f64.to_bits(), 1.0_f64.to_bits()]
    ));

    let background = export.background();
    assert_eq!(background.component_index(), 0);
    assert_eq!(background.weight_magnitude().to_bits(), 0.5_f64.to_bits());
    assert_eq!(background.minimum_weight().to_bits(), 0.25_f64.to_bits());
    assert_eq!(background.policy_ratio().to_bits(), 2.0_f64.to_bits());
    assert_eq!(background.condition_number().to_bits(), 1.0_f64.to_bits());

    let weights = export.samples()[0].component_weights();
    let expected_second = -1.5 * (-0.5_f64).exp();
    assert_eq!(weights.len(), 3);
    assert_eq!(weights[0].to_bits(), 0.5_f64.to_bits());
    assert_eq!(weights[1].to_bits(), 2.0_f64.to_bits());
    assert!((weights[2] - expected_second).abs() <= 4.0 * f64::EPSILON);
    let coverage = export.samples()[0].coverage();
    let expected_coverage = 0.25 + 4.0 + expected_second * expected_second;
    assert!((coverage.squared_weight_sum() - expected_coverage).abs() <= 8.0 * f64::EPSILON);
    assert_eq!(
        coverage.background_squared_weight().to_bits(),
        0.25_f64.to_bits()
    );
    assert_eq!(coverage.active_components(), 3);
    assert!(coverage.inside_operational_domain());
    assert_eq!(export.samples()[0].position(), sample);

    assert_eq!(
        export.controls()[1]
            .direction_jump_from_previous_radians()
            .map(f64::to_bits),
        Some(0.0_f64.to_bits())
    );
    assert!(!export.controls()[1].direction_jump_exceeds_policy());
    assert_eq!(
        export.summary().maximum_direction_jump_radians().to_bits(),
        0.0_f64.to_bits()
    );
    Ok(())
}

#[test]
fn direction_jump_export_is_antipodal_and_policy_explicit() -> Result<(), Box<dyn Error>> {
    let controls = [
        spheroid([0.0, 0.0], [1.0, 0.0], 1.0, None)?,
        spheroid([0.5, 0.0], [-1.0, 0.0], 1.0, None)?,
        spheroid([1.0, 0.0], [0.0, 1.0], 1.0, None)?,
    ];
    let compiled = try_compile_local_trend_controls(
        background()?,
        &controls,
        None,
        domain(3.0)?,
        0.25,
        policy(std::f64::consts::FRAC_PI_4)?,
    )?;
    let export = try_export_anisotropy_diagnostics(&compiled, &[])?;

    assert_eq!(
        export.controls()[1]
            .direction_jump_from_previous_radians()
            .map(f64::to_bits),
        Some(0.0_f64.to_bits())
    );
    assert_eq!(
        export.controls()[2]
            .direction_jump_from_previous_radians()
            .map(f64::to_bits),
        Some(std::f64::consts::FRAC_PI_2.to_bits())
    );
    assert!(export.controls()[2].direction_jump_exceeds_policy());
    assert_eq!(export.summary().jump_exceedance_count(), 1);
    assert_eq!(
        export.summary().maximum_direction_jump_radians().to_bits(),
        std::f64::consts::FRAC_PI_2.to_bits()
    );
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

fn fitted_axis<const D: usize>() -> Result<FittedField<D>, Box<dyn Error>>
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
                    "anisotropy-diagnostics-reference.csv".to_owned(),
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
                target: [-1.0, 1.0][index],
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
        AffineNormalization::try_new(
            point([0.0; D])?,
            std::array::from_fn(|row| std::array::from_fn(|column| f64::from(row == column))),
        )?,
        KernelDefinition::from(Gaussian::try_new(1.0)?),
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

#[test]
fn low_confidence_regions_preserve_reference_source_and_region() -> Result<(), Box<dyn Error>> {
    let field_id = FieldId::new(41);
    let project = GeoProject::try_new([ProjectField::new(field_id, fitted_axis::<2>()?)])?;
    let region = SmoothRegion::try_new(point([-1.0, -1.0])?, point([1.0, 1.0])?, 0.25)?;
    let reference = LocalTrendControl::new(
        point([0.0, 0.0])?,
        KernelDefinition::from(Gaussian::try_new(1.0)?),
        TrendControlOrientation::Spheroidal {
            principal_axis: TrendDirectionSource::ReferenceFieldGradient(field_id),
            axial_length: 2.0,
            transverse_length: 1.0,
        },
        1.0,
        1.0,
        Some(region),
    );
    let unbounded_reference = LocalTrendControl::new(
        point([0.25, 0.0])?,
        KernelDefinition::from(Gaussian::try_new(1.0)?),
        TrendControlOrientation::Ellipsoidal {
            principal_axes: [
                TrendDirectionSource::Explicit(direction([0.0, 1.0])?),
                TrendDirectionSource::ReferenceFieldGradient(field_id),
            ],
            axis_lengths: [2.0, 1.0],
            orthogonality_tolerance: 8.0 * f64::EPSILON,
        },
        1.0,
        1.0,
        None,
    );
    let explicit = spheroid([0.5, 0.0], [1.0, 0.0], 1.0, None)?;
    let compiled = try_compile_local_trend_controls(
        background()?,
        &[reference, unbounded_reference, explicit],
        Some(&project),
        domain(2.0)?,
        0.25,
        policy(1.0)?,
    )?;
    let export = try_export_anisotropy_diagnostics(&compiled, &[])?;

    assert_eq!(export.summary().low_confidence_direction_count(), 2);
    assert_eq!(export.low_confidence_regions().len(), 2);
    let low = export.low_confidence_regions()[0];
    assert_eq!(low.control_index(), 0);
    assert_eq!(low.axis_index(), 0);
    assert_eq!(low.position(), point([0.0, 0.0])?);
    assert_eq!(low.region(), Some(region));
    assert_eq!(low.field_id(), field_id);
    assert!(low.gradient_norm().is_finite() && low.gradient_norm() > 0.0);
    let unbounded = export.low_confidence_regions()[1];
    assert_eq!(unbounded.control_index(), 1);
    assert_eq!(unbounded.axis_index(), 1);
    assert_eq!(unbounded.region(), None);
    assert_eq!(unbounded.field_id(), field_id);
    assert!(matches!(
        export.controls()[0].orientation(),
        AnisotropyControlOrientation::Spheroidal { principal_axis, .. }
            if matches!(
                principal_axis.source(),
                ResolvedTrendDirectionSource::ReferenceFieldGradient {
                    field_id: observed,
                    low_confidence: true,
                    ..
                } if observed == field_id
            )
    ));
    assert!(matches!(
        export.controls()[2].orientation(),
        AnisotropyControlOrientation::Spheroidal { principal_axis, .. }
            if principal_axis.source() == ResolvedTrendDirectionSource::Explicit
    ));
    Ok(())
}

fn assert_dimension_export<const D: usize>(axis: [f64; D]) -> Result<(), Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    let control = spheroid([0.0; D], axis, 1.0, None)?;
    let compiled = try_compile_local_trend_controls(
        background()?,
        &[control],
        None,
        domain(1.0)?,
        0.25,
        policy(1.0)?,
    )?;
    let export = try_export_anisotropy_diagnostics(&compiled, &[point([0.0; D])?])?;
    assert_eq!(export.controls().len(), 1);
    assert_eq!(export.samples()[0].component_weights().len(), 2);
    Ok(())
}

#[test]
fn dimensions_auto_traits_and_evaluation_errors_are_explicit() -> Result<(), Box<dyn Error>> {
    assert_dimension_export([1.0])?;
    assert_dimension_export([0.0, 0.0, 1.0])?;
    assert_send_sync::<AnisotropyDiagnosticExport<1>>();
    assert_send_sync::<AnisotropyDiagnosticExport<2>>();
    assert_send_sync::<AnisotropyDiagnosticExport<3>>();

    let control = spheroid([-f64::MAX], [1.0], 1.0, None)?;
    let compiled = try_compile_local_trend_controls(
        background()?,
        &[control],
        None,
        domain(1.0)?,
        0.25,
        policy(1.0)?,
    )?;
    assert!(matches!(
        try_export_anisotropy_diagnostics(&compiled, &[point([f64::MAX])?]),
        Err(AnisotropyDiagnosticExportError::Evaluation { sample: 0, .. })
    ));
    Ok(())
}
