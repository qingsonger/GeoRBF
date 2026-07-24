//! Independent polynomial truth and error-path tests for one-dimensional level points.

use std::error::Error;
use std::num::{NonZeroU32, NonZeroUsize};
use std::sync::Mutex;

use georbf::{
    AffineNormalization, AngleUnit, AxisOrder, CancellationToken, CenterRepresenter,
    ConditionPolicy, CoordinateMetadata, CrsMetadata, DenseFactorization, DenseSolveOptions,
    Enforcement, ExecutionControl, ExecutionError, ExecutionOperation, ExecutionOptions,
    ExecutionStage, FieldProblem, FittedField, FunctionalAtom, FunctionalExpr,
    FunctionalProvenance, FunctionalTerm, Handedness, KernelDefinition, LengthUnit,
    LevelPointError, LevelPointKind, LevelPointRequest, LevelPointSettings, Matern,
    MaternSmoothness, ObservationFunctional, ObservationId, Point, PolyharmonicSpline,
    ProgressEvent, ProgressSink, Regularization, SemanticConstraint, SemanticExpression,
    SemanticProblemIr, SemanticProvenance, SemanticRelation, SourceLocation, VerticalDirection,
};

const TEST_MEMORY_LIMIT_BYTES: usize = 64 * 1024 * 1024;

fn metadata() -> Result<CoordinateMetadata<1>, Box<dyn Error>> {
    Ok(CoordinateMetadata::new(
        LengthUnit::try_new("m")?,
        CrsMetadata::unspecified(),
        AxisOrder::identity(),
        VerticalDirection::Up,
        Handedness::Right,
        AngleUnit::Radians,
    ))
}

fn value_expression(point: f64, identifier: u64) -> Result<FunctionalExpr<1>, Box<dyn Error>> {
    Ok(FunctionalExpr::try_new([FunctionalTerm::try_new(
        1.0,
        FunctionalAtom::value(
            Point::try_new([point])?,
            FunctionalProvenance::new(identifier),
        ),
    )?])?)
}

fn polynomial_model(
    polynomial: impl Fn(f64) -> f64,
    normalization: AffineNormalization<1>,
) -> Result<FittedField<1>, Box<dyn Error>> {
    let sites = [-2.0, -0.5, 1.0, 2.5];
    let mut constraints = Vec::new();
    let mut centers = Vec::new();
    for (index, site) in sites.into_iter().enumerate() {
        let identifier = u64::try_from(index + 1)?;
        let expression = value_expression(site, identifier)?;
        centers.push(CenterRepresenter::new(expression.clone()));
        constraints.push(SemanticConstraint::try_new(
            SemanticProvenance::try_new(
                ObservationId::new(identifier),
                SourceLocation::try_new(
                    "contour-test.csv".to_owned(),
                    NonZeroUsize::new(index + 1).ok_or("line")?,
                )?,
                "m".to_owned(),
                format!("field.equalities[{index}]"),
                Some("contour polynomial truth".to_owned()),
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
    let options = DenseSolveOptions::try_new(
        DenseFactorization::PivotedLblt,
        Regularization::None,
        ConditionPolicy::default(),
        4,
        NonZeroUsize::new(TEST_MEMORY_LIMIT_BYTES).ok_or("memory limit")?,
    )?;
    Ok(FittedField::try_fit(
        problem,
        metadata()?,
        normalization,
        KernelDefinition::from(PolyharmonicSpline::try_new(4)?),
        None,
        options,
    )?)
}

fn center_limited_model() -> Result<FittedField<1>, Box<dyn Error>> {
    let expression = value_expression(0.0, 1)?;
    let constraint = SemanticConstraint::try_new(
        SemanticProvenance::try_new(
            ObservationId::new(1),
            SourceLocation::try_new("contour-center-test.csv".to_owned(), NonZeroUsize::MIN)?,
            "m".to_owned(),
            "field.equalities[0]".to_owned(),
            Some("center capability".to_owned()),
        )?,
        SemanticRelation::Equality {
            expression: SemanticExpression::try_new(
                ObservationFunctional::new(expression.clone()),
                0.0,
            )?,
            target: 1.0,
        },
        Enforcement::Hard,
    )?;
    let problem = FieldProblem::try_new(
        SemanticProblemIr::try_new([constraint], ExecutionOptions::default())?,
        [CenterRepresenter::new(expression)],
    )?;
    let options = DenseSolveOptions::try_new(
        DenseFactorization::Cholesky,
        Regularization::None,
        ConditionPolicy::default(),
        4,
        NonZeroUsize::new(TEST_MEMORY_LIMIT_BYTES).ok_or("memory limit")?,
    )?;
    Ok(FittedField::try_fit(
        problem,
        metadata()?,
        identity_normalization()?,
        KernelDefinition::from(Matern::try_new(MaternSmoothness::OneHalf, 1.0)?),
        None,
        options,
    )?)
}

fn identity_normalization() -> Result<AffineNormalization<1>, Box<dyn Error>> {
    Ok(AffineNormalization::try_new(
        Point::try_new([0.0])?,
        [[1.0]],
    )?)
}

fn settings() -> Result<LevelPointSettings, Box<dyn Error>> {
    Ok(LevelPointSettings::try_new(
        NonZeroU32::new(8).ok_or("scan intervals")?,
        NonZeroU32::new(64).ok_or("refinement iterations")?,
        1.0e-11,
        1.0e-11,
        1.0e-10,
    )?)
}

fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    let scale = actual.abs().max(expected.abs()).max(1.0);
    assert!(
        (actual - expected).abs() <= tolerance * scale,
        "expected {expected:.17e}, got {actual:.17e}"
    );
}

#[derive(Default)]
struct RecordingProgress {
    events: Mutex<Vec<ProgressEvent>>,
}

impl RecordingProgress {
    fn events(&self) -> Vec<ProgressEvent> {
        self.events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

impl ProgressSink for RecordingProgress {
    fn on_progress(&self, event: ProgressEvent) {
        self.events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(event);
    }
}

#[test]
fn crossing_roots_and_stationary_evidence_use_original_coordinates() -> Result<(), Box<dyn Error>> {
    let polynomial = |x: f64| x * x + x - 2.0;
    let model = polynomial_model(
        polynomial,
        AffineNormalization::try_new(Point::try_new([5.0])?, [[2.0]])?,
    )?;
    let request = LevelPointRequest::try_new(0.0, -1.0, 9.0, settings()?)?;
    let report = model.try_level_points(&request)?;

    assert_eq!(report.points().len(), 2);
    assert_close(report.points()[0].point().components()[0], 1.0, 1.0e-10);
    assert_close(report.points()[1].point().components()[0], 7.0, 1.0e-10);
    assert!(
        report
            .points()
            .iter()
            .all(|point| point.kind() == LevelPointKind::Crossing)
    );
    assert_eq!(report.stationary_points().len(), 1);
    let stationary = report.stationary_points()[0];
    assert_close(stationary.point().components()[0], 4.0, 1.0e-10);
    assert_close(stationary.value(), -2.25, 1.0e-10);
    assert!(!stationary.is_at_level());
    assert!(report.degenerate_intervals().is_empty());
    assert_eq!(report.diagnostics().requested_scan_intervals(), 8);
    assert_eq!(report.diagnostics().examined_segments(), 16);
    assert_eq!(report.diagnostics().value_brackets().len(), 2);
    assert_eq!(report.diagnostics().stationary_brackets().len(), 1);
    Ok(())
}

#[test]
fn tangent_root_is_reported_once_as_stationary() -> Result<(), Box<dyn Error>> {
    let polynomial = |x: f64| (x - 0.47).powi(2);
    let model = polynomial_model(polynomial, identity_normalization()?)?;
    let request = LevelPointRequest::try_new(0.0, -1.0, 2.0, settings()?)?;
    let report = model.try_level_points(&request)?;

    assert_eq!(report.points().len(), 1);
    let point = report.points()[0];
    assert_close(point.point().components()[0], 0.47, 1.0e-10);
    assert_eq!(point.kind(), LevelPointKind::Stationary);
    assert_eq!(report.stationary_points().len(), 1);
    assert!(report.stationary_points()[0].is_at_level());
    assert!(point.residual().abs() <= request.settings().value_tolerance());
    Ok(())
}

#[test]
fn boundary_roots_are_deduplicated_and_classified() -> Result<(), Box<dyn Error>> {
    let polynomial = |x: f64| x * x - 1.0;
    let model = polynomial_model(polynomial, identity_normalization()?)?;
    let request = LevelPointRequest::try_new(0.0, -1.0, 1.0, settings()?)?;
    let report = model.try_level_points(&request)?;

    assert_eq!(report.points().len(), 2);
    assert_eq!(report.points()[0].kind(), LevelPointKind::Boundary);
    assert_eq!(report.points()[1].kind(), LevelPointKind::Boundary);
    assert_close(report.points()[0].point().components()[0], -1.0, 1.0e-12);
    assert_close(report.points()[1].point().components()[0], 1.0, 1.0e-12);
    Ok(())
}

#[test]
fn constant_level_reports_one_degenerate_interval_without_fake_points() -> Result<(), Box<dyn Error>>
{
    let model = polynomial_model(|_| 3.0, identity_normalization()?)?;
    let request = LevelPointRequest::try_new(3.0, -2.0, 2.0, settings()?)?;
    let report = model.try_level_points(&request)?;

    assert!(report.points().is_empty());
    assert!(report.stationary_points().is_empty());
    assert_eq!(report.degenerate_intervals().len(), 1);
    let interval = report.degenerate_intervals()[0];
    assert_close(interval.lower(), -2.0, 1.0e-12);
    assert_close(interval.upper(), 2.0, 1.0e-12);
    assert!(interval.maximum_value_residual() <= request.settings().value_tolerance());
    assert!(interval.maximum_absolute_derivative() <= request.settings().derivative_tolerance());
    assert!(report.has_non_isolated_level_set());
    Ok(())
}

#[test]
fn invalid_settings_refinement_exhaustion_and_cancellation_are_structured()
-> Result<(), Box<dyn Error>> {
    assert!(
        LevelPointSettings::try_new(NonZeroU32::MIN, NonZeroU32::MIN, 0.0, 1.0e-9, 1.0e-9,)
            .is_err()
    );
    assert!(LevelPointRequest::try_new(0.0, 1.0, 1.0, settings()?).is_err());

    let model = polynomial_model(|x| x * x - 0.3, identity_normalization()?)?;
    let insufficient =
        LevelPointSettings::try_new(NonZeroU32::MIN, NonZeroU32::MIN, 1.0e-15, 1.0e-15, 1.0e-15)?;
    let request = LevelPointRequest::try_new(0.0, -1.0, 1.0, insufficient)?;
    assert!(matches!(
        model.try_level_points(&request),
        Err(LevelPointError::RefinementLimitReached { .. })
    ));

    let token = CancellationToken::new();
    token.cancel();
    let cancelled = model.try_level_points_with_control(
        &LevelPointRequest::try_new(0.0, -1.0, 1.0, settings()?)?,
        ExecutionOptions::default(),
        ExecutionControl::with_cancellation(&token),
    );
    assert!(matches!(
        cancelled,
        Err(LevelPointError::Execution(ExecutionError::Cancelled {
            operation: ExecutionOperation::LevelPointExtraction,
            ..
        }))
    ));
    Ok(())
}

#[test]
fn execution_policy_and_progress_are_enforced_before_and_during_evaluation()
-> Result<(), Box<dyn Error>> {
    let model = polynomial_model(|x| x * x - 1.0, identity_normalization()?)?;
    let request = LevelPointRequest::try_new(0.0, -2.0, 2.0, settings()?)?;
    let rejected_sink = RecordingProgress::default();
    let requested_threads = NonZeroUsize::new(2).ok_or("thread count")?;
    let rejected = model.try_level_points_with_control(
        &request,
        ExecutionOptions::new(false, Some(requested_threads), None),
        ExecutionControl::with_progress(&rejected_sink),
    );
    assert!(matches!(
        rejected,
        Err(LevelPointError::Execution(
            ExecutionError::UnsupportedThreadCount {
                operation: ExecutionOperation::LevelPointExtraction,
                requested,
                maximum,
            }
        )) if requested == requested_threads && maximum == NonZeroUsize::MIN
    ));
    assert!(rejected_sink.events().is_empty());

    let sink = RecordingProgress::default();
    let report = model.try_level_points_with_control(
        &request,
        ExecutionOptions::new(false, Some(NonZeroUsize::MIN), None),
        ExecutionControl::with_progress(&sink),
    )?;
    let events = sink.events();
    assert_eq!(
        events.first().map(|event| event.stage()),
        Some(ExecutionStage::Started)
    );
    assert_eq!(
        events.last().map(|event| event.stage()),
        Some(ExecutionStage::Completed)
    );
    assert!(events.iter().all(|event| {
        event.operation() == ExecutionOperation::LevelPointExtraction
            && !event.deterministic()
            && event.thread_count() == NonZeroUsize::MIN
    }));
    assert_eq!(
        events
            .iter()
            .filter(|event| event.stage() == ExecutionStage::LevelPointEvaluation)
            .count(),
        report.diagnostics().evaluations()
    );
    assert!(events.windows(2).all(|pair| {
        pair[0].completed() <= pair[1].completed() && pair[0].total() == pair[1].total()
    }));
    Ok(())
}

#[test]
fn work_overflow_and_center_capability_failure_return_no_report() -> Result<(), Box<dyn Error>> {
    let model = polynomial_model(|x| x, identity_normalization()?)?;
    let unrepresentable =
        LevelPointSettings::try_new(NonZeroU32::MAX, NonZeroU32::MIN, 1.0e-9, 1.0e-9, 1.0e-9)?;
    let request = LevelPointRequest::try_new(0.0, -1.0, 1.0, unrepresentable)?;
    assert!(matches!(
        model.try_level_points(&request),
        Err(LevelPointError::WorkBudgetOverflow { .. })
    ));

    let center_limited = center_limited_model()?;
    let center_request = LevelPointRequest::try_new(0.5, -1.0, 1.0, settings()?)?;
    assert!(matches!(
        center_limited.try_level_points(&center_request),
        Err(LevelPointError::Evaluation { coordinate, .. })
            if coordinate.to_bits() == 0.0_f64.to_bits()
    ));
    Ok(())
}
