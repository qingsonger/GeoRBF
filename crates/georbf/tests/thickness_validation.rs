//! Independent truth and error-path tests for sampled geometric thickness validation.

use std::error::Error;
use std::num::{NonZeroU32, NonZeroUsize};
use std::sync::Mutex;

use georbf::{
    AffineNormalization, AngleUnit, AxisOrder, CancellationToken, CenterRepresenter,
    ConditionPolicy, CoordinateMetadata, CrsMetadata, DenseFactorization, DenseSolveOptions,
    Enforcement, ExecutionControl, ExecutionError, ExecutionOperation, ExecutionOptions,
    ExecutionStage, FieldProblem, FittedField, FunctionalAtom, FunctionalExpr,
    FunctionalProvenance, FunctionalTerm, Handedness, KernelDefinition, LengthUnit, LevelId,
    ObservationFunctional, ObservationId, Point, PolyharmonicSpline, ProgressEvent, ProgressSink,
    Regularization, SampledThicknessFailureReason, SampledThicknessLocation,
    SampledThicknessRequest, SampledThicknessSettings, SampledThicknessValidationError,
    SemanticConstraint, SemanticExpression, SemanticProblemIr, SemanticProvenance,
    SemanticRelation, SourceLocation, ThicknessDiagnosticKind, ThicknessGuarantee,
    VerticalDirection,
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

fn provenance(identifier: u64) -> Result<SemanticProvenance, Box<dyn Error>> {
    Ok(SemanticProvenance::try_new(
        ObservationId::new(identifier),
        SourceLocation::try_new(
            "sampled-thickness.csv".to_owned(),
            NonZeroUsize::new(usize::try_from(identifier)?).ok_or("positive identifier")?,
        )?,
        "m".to_owned(),
        format!("thickness.samples[{identifier}]"),
        Some("sampled validation test".to_owned()),
    )?)
}

fn value_expression(x: f64, identifier: u64) -> Result<FunctionalExpr<1>, Box<dyn Error>> {
    Ok(FunctionalExpr::try_new([FunctionalTerm::try_new(
        1.0,
        FunctionalAtom::value(Point::try_new([x])?, FunctionalProvenance::new(identifier)),
    )?])?)
}

fn linear_model() -> Result<FittedField<1>, Box<dyn Error>> {
    let mut constraints = Vec::new();
    let mut centers = Vec::new();
    for (index, x) in [-2.0_f64, -1.0, 1.0, 2.0].into_iter().enumerate() {
        let expression = value_expression(x, u64::try_from(index + 1)?)?;
        centers.push(CenterRepresenter::new(expression.clone()));
        constraints.push(SemanticConstraint::try_new(
            provenance(u64::try_from(index + 1)?)?,
            SemanticRelation::Equality {
                expression: SemanticExpression::try_new(
                    ObservationFunctional::new(expression),
                    0.0,
                )?,
                target: x,
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
        AffineNormalization::try_new(Point::try_new([0.0])?, [[1.0]])?,
        KernelDefinition::from(PolyharmonicSpline::try_new(4)?),
        None,
        DenseSolveOptions::try_new(
            DenseFactorization::PivotedLblt,
            Regularization::None,
            ConditionPolicy::default(),
            4,
            NonZeroUsize::new(TEST_MEMORY_LIMIT_BYTES).ok_or("memory limit")?,
        )?,
    )?)
}

fn settings(maximum_search_distance: f64) -> Result<SampledThicknessSettings, Box<dyn Error>> {
    Ok(SampledThicknessSettings::try_new(
        maximum_search_distance,
        NonZeroU32::new(32).ok_or("search steps")?,
        NonZeroU32::new(64).ok_or("refinement iterations")?,
        1.0e-12,
        1.0e-12,
        1.0e-12,
    )?)
}

fn locations() -> Result<Vec<SampledThicknessLocation<1>>, Box<dyn Error>> {
    [-0.5_f64, 0.0, 0.5]
        .into_iter()
        .enumerate()
        .map(|(index, x)| {
            Ok(SampledThicknessLocation::new(
                Point::try_new([x])?,
                provenance(u64::try_from(index + 10)?)?,
            ))
        })
        .collect()
}

fn assert_provenance(actual: &SemanticProvenance, expected: &SemanticProvenance) {
    assert_eq!(actual.observation_id(), expected.observation_id());
    assert_eq!(actual.source().path(), expected.source().path());
    assert_eq!(actual.source().line(), expected.source().line());
    assert_eq!(actual.original_units(), expected.original_units());
    assert_eq!(actual.field_path(), expected.field_path());
    assert_eq!(actual.constraint_group(), expected.constraint_group());
}

struct CancelAfterEvaluations {
    token: CancellationToken,
    completed: usize,
    events: Mutex<Vec<ProgressEvent>>,
}

impl CancelAfterEvaluations {
    fn new(token: CancellationToken, completed: usize) -> Self {
        Self {
            token,
            completed,
            events: Mutex::new(Vec::new()),
        }
    }

    fn events(&self) -> Vec<ProgressEvent> {
        self.events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

impl ProgressSink for CancelAfterEvaluations {
    fn on_progress(&self, event: ProgressEvent) {
        self.events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(event);
        if event.stage() == ExecutionStage::SampledThicknessEvaluation
            && event.completed() == self.completed
        {
            self.token.cancel();
        }
    }
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
fn fitted_parallel_levels_measure_quantiles_violations_and_proposals_deterministically()
-> Result<(), Box<dyn Error>> {
    let model = linear_model()?;
    let selected_locations = locations()?;
    let request = SampledThicknessRequest::try_new(
        LevelId::new(100),
        -1.0,
        LevelId::new(200),
        1.0,
        2.5,
        selected_locations.clone(),
        vec![0.0, 0.5, 1.0],
        true,
        settings(2.0)?,
    )?;
    let report = model.try_validate_sampled_thickness(&request)?;
    let repeated = model.try_validate_sampled_thickness(&request)?;

    assert_eq!(report, repeated);
    assert_eq!(report.measurements().len(), 3);
    assert!(report.failures().is_empty());
    assert_eq!(report.minimum(), Some(2.0));
    assert!(
        report
            .quantiles()
            .iter()
            .all(|quantile| quantile.distance() == Some(2.0))
    );
    assert_eq!(report.violations().len(), 3);
    assert_eq!(report.proposed_constraints().len(), 3);
    for (index, measurement) in report.measurements().iter().enumerate() {
        assert_eq!(measurement.sample_index(), index);
        assert!((measurement.lower_intersection().components()[0] + 1.0).abs() <= 1.0e-12);
        assert!((measurement.upper_intersection().components()[0] - 1.0).abs() <= 1.0e-12);
        assert!((measurement.distance() - 2.0).abs() <= 1.0e-12);
        assert_provenance(
            measurement.provenance(),
            selected_locations[index].provenance(),
        );
        assert_provenance(
            report.violations()[index].provenance(),
            selected_locations[index].provenance(),
        );
        assert_provenance(
            report.proposed_constraints()[index].provenance(),
            selected_locations[index].provenance(),
        );
    }
    assert_eq!(
        report.diagnostics().kind(),
        ThicknessDiagnosticKind::SampledGeometricValidation
    );
    assert_eq!(
        report.diagnostics().guarantee(),
        ThicknessGuarantee::SampledGeometricEvidence
    );
    Ok(())
}

#[test]
fn controlled_validation_cancels_at_an_exact_evaluation_boundary_without_a_report()
-> Result<(), Box<dyn Error>> {
    let model = linear_model()?;
    let request = SampledThicknessRequest::try_new(
        LevelId::new(1),
        -1.0,
        LevelId::new(2),
        1.0,
        2.5,
        locations()?,
        vec![0.5],
        false,
        settings(2.0)?,
    )?;
    let token = CancellationToken::new();
    let sink = CancelAfterEvaluations::new(token.clone(), 3);

    let result = model.try_validate_sampled_thickness_with_control(
        &request,
        ExecutionOptions::default(),
        ExecutionControl::new(Some(&token), Some(&sink)),
    );
    assert!(matches!(
        result,
        Err(SampledThicknessValidationError::Execution(
            ExecutionError::Cancelled {
                operation: ExecutionOperation::SampledThicknessValidation,
                stage: ExecutionStage::SampledThicknessEvaluation,
            }
        ))
    ));
    let events = sink.events();
    let evaluation_events = events
        .iter()
        .filter(|event| event.stage() == ExecutionStage::SampledThicknessEvaluation)
        .count();
    assert_eq!(evaluation_events, 3);
    assert_eq!(events.last().map(|event| event.completed()), Some(3));
    assert!(
        !events
            .iter()
            .any(|event| event.stage() == ExecutionStage::Completed)
    );
    Ok(())
}

#[test]
fn controlled_validation_rejects_two_threads_before_fitted_field_evaluation()
-> Result<(), Box<dyn Error>> {
    let model = linear_model()?;
    let request = SampledThicknessRequest::try_new(
        LevelId::new(1),
        -1.0,
        LevelId::new(2),
        1.0,
        2.5,
        locations()?,
        vec![0.5],
        false,
        settings(2.0)?,
    )?;
    let sink = RecordingProgress::default();
    let requested = NonZeroUsize::new(2).ok_or("thread count")?;

    let result = model.try_validate_sampled_thickness_with_control(
        &request,
        ExecutionOptions::new(false, Some(requested), None),
        ExecutionControl::with_progress(&sink),
    );

    assert!(matches!(
        result,
        Err(SampledThicknessValidationError::Execution(
            ExecutionError::UnsupportedThreadCount {
                operation: ExecutionOperation::SampledThicknessValidation,
                requested: actual,
                maximum,
            }
        )) if actual == requested && maximum == NonZeroUsize::MIN
    ));
    assert!(
        sink.events().is_empty(),
        "no progress boundary, including a fitted-field evaluation, may run"
    );
    Ok(())
}

#[test]
fn controlled_validation_preserves_one_thread_and_false_determinism_in_all_progress()
-> Result<(), Box<dyn Error>> {
    let model = linear_model()?;
    let request = SampledThicknessRequest::try_new(
        LevelId::new(1),
        -1.0,
        LevelId::new(2),
        1.0,
        2.5,
        locations()?,
        vec![0.5],
        false,
        settings(2.0)?,
    )?;
    let sink = RecordingProgress::default();
    let requested = NonZeroUsize::MIN;

    let report = model.try_validate_sampled_thickness_with_control(
        &request,
        ExecutionOptions::new(false, Some(requested), None),
        ExecutionControl::with_progress(&sink),
    )?;

    assert_eq!(report, model.try_validate_sampled_thickness(&request)?);
    let events = sink.events();
    assert!(!events.is_empty());
    assert_eq!(
        events.last().map(|event| event.stage()),
        Some(ExecutionStage::Completed)
    );
    assert!(events.iter().all(|event| {
        event.thread_count() == requested
            && !event.deterministic()
            && event.operation() == ExecutionOperation::SampledThicknessValidation
    }));
    Ok(())
}

#[test]
fn no_intersection_is_reported_without_a_measurement_or_proposal() -> Result<(), Box<dyn Error>> {
    let model = linear_model()?;
    let request = SampledThicknessRequest::try_new(
        LevelId::new(1),
        -1.0,
        LevelId::new(2),
        10.0,
        1.0,
        vec![SampledThicknessLocation::new(
            Point::try_new([0.0])?,
            provenance(20)?,
        )],
        vec![0.5],
        true,
        settings(2.0)?,
    )?;
    let report = model.try_validate_sampled_thickness(&request)?;

    assert!(report.measurements().is_empty());
    assert_eq!(report.failures().len(), 1);
    assert!(matches!(
        report.failures()[0].reason(),
        SampledThicknessFailureReason::Intersections {
            lower: None,
            upper: Some(_)
        }
    ));
    assert_eq!(report.minimum(), None);
    assert_eq!(report.quantiles()[0].distance(), None);
    assert!(report.violations().is_empty());
    assert!(report.proposed_constraints().is_empty());
    Ok(())
}

#[test]
fn invalid_requests_are_rejected_before_field_evaluation() -> Result<(), Box<dyn Error>> {
    let one = vec![SampledThicknessLocation::new(
        Point::try_new([0.0])?,
        provenance(30)?,
    )];
    assert!(
        SampledThicknessRequest::try_new(
            LevelId::new(1),
            0.0,
            LevelId::new(1),
            1.0,
            1.0,
            one.clone(),
            vec![0.5],
            false,
            settings(1.0)?,
        )
        .is_err()
    );
    assert!(
        SampledThicknessRequest::try_new(
            LevelId::new(1),
            1.0,
            LevelId::new(2),
            0.0,
            1.0,
            one.clone(),
            vec![0.5],
            false,
            settings(1.0)?,
        )
        .is_err()
    );
    assert!(
        SampledThicknessRequest::try_new(
            LevelId::new(1),
            0.0,
            LevelId::new(2),
            1.0,
            1.0,
            one,
            vec![f64::NAN],
            false,
            settings(1.0)?,
        )
        .is_err()
    );
    assert!(
        SampledThicknessSettings::try_new(
            0.0,
            NonZeroU32::MIN,
            NonZeroU32::MIN,
            1.0e-6,
            1.0e-6,
            1.0e-6
        )
        .is_err()
    );
    Ok(())
}

fn assert_send_sync<T: Send + Sync>() {}

#[test]
fn public_validation_types_are_send_sync() {
    assert_send_sync::<SampledThicknessRequest<1>>();
    assert_send_sync::<SampledThicknessRequest<2>>();
    assert_send_sync::<SampledThicknessRequest<3>>();
}
