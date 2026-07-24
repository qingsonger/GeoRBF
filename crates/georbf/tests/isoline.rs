//! Independent polynomial truth and error-path tests for two-dimensional isolines.

use std::error::Error;
use std::num::{NonZeroU32, NonZeroUsize};
use std::sync::Mutex;

use georbf::{
    AffineNormalization, AngleUnit, AxisOrder, CancellationToken, CenterRepresenter,
    ConditionPolicy, CoordinateMetadata, CrsMetadata, DenseFactorization, DenseSolveOptions,
    Enforcement, ExecutionControl, ExecutionError, ExecutionOperation, ExecutionOptions,
    ExecutionStage, FieldProblem, FittedField, FunctionalAtom, FunctionalExpr,
    FunctionalProvenance, FunctionalTerm, Handedness, IsolineAmbiguityDecider, IsolineBoundarySide,
    IsolineCellPairing, IsolineError, IsolineMethod, IsolineRequest, IsolineRequestError,
    IsolineSettings, IsolineSettingsError, IsolineTolerance, KernelDefinition, LengthUnit,
    ObservationFunctional, ObservationId, Point, PolyharmonicSpline, ProgressEvent, ProgressSink,
    Regularization, SemanticConstraint, SemanticExpression, SemanticProblemIr, SemanticProvenance,
    SemanticRelation, SourceLocation, VerticalDirection,
};

const TEST_MEMORY_LIMIT_BYTES: usize = 64 * 1024 * 1024;

fn metadata() -> Result<CoordinateMetadata<2>, Box<dyn Error>> {
    Ok(CoordinateMetadata::new(
        LengthUnit::try_new("m")?,
        CrsMetadata::unspecified(),
        AxisOrder::identity(),
        VerticalDirection::Up,
        Handedness::Right,
        AngleUnit::Radians,
    ))
}

fn value_expression(point: [f64; 2], identifier: u64) -> Result<FunctionalExpr<2>, Box<dyn Error>> {
    Ok(FunctionalExpr::try_new([FunctionalTerm::try_new(
        1.0,
        FunctionalAtom::value(
            Point::try_new(point)?,
            FunctionalProvenance::new(identifier),
        ),
    )?])?)
}

fn polynomial_model(
    polynomial: impl Fn(f64, f64) -> f64,
    normalization: AffineNormalization<2>,
) -> Result<FittedField<2>, Box<dyn Error>> {
    let sites = [
        [-2.0, -2.0],
        [0.0, -2.0],
        [2.0, -2.0],
        [-2.0, 0.0],
        [0.0, 0.0],
        [2.0, 0.0],
        [-2.0, 2.0],
        [0.0, 2.0],
        [2.0, 2.0],
    ];
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
                    "isoline-test.csv".to_owned(),
                    NonZeroUsize::new(index + 1).ok_or("line")?,
                )?,
                "m".to_owned(),
                format!("field.equalities[{index}]"),
                Some("isoline polynomial truth".to_owned()),
            )?,
            SemanticRelation::Equality {
                expression: SemanticExpression::try_new(
                    ObservationFunctional::new(expression),
                    0.0,
                )?,
                target: polynomial(site[0], site[1]),
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
        KernelDefinition::from(PolyharmonicSpline::try_new(5)?),
        None,
        options,
    )?)
}

fn identity_normalization() -> Result<AffineNormalization<2>, Box<dyn Error>> {
    Ok(AffineNormalization::try_new(
        Point::try_new([0.0, 0.0])?,
        [[1.0, 0.0], [0.0, 1.0]],
    )?)
}

fn settings(cells: u32) -> Result<IsolineSettings, Box<dyn Error>> {
    Ok(IsolineSettings::try_new(
        NonZeroU32::new(cells).ok_or("x cells")?,
        NonZeroU32::new(cells).ok_or("y cells")?,
        NonZeroU32::new(64).ok_or("refinement iterations")?,
        1.0e-11,
        1.0e-11,
    )?)
}

fn request(
    method: IsolineMethod,
    lower: [f64; 2],
    upper: [f64; 2],
    settings: IsolineSettings,
) -> Result<IsolineRequest, Box<dyn Error>> {
    Ok(IsolineRequest::try_new(
        0.0,
        Point::try_new(lower)?,
        Point::try_new(upper)?,
        method,
        settings,
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
fn transformed_line_is_one_open_boundary_polyline() -> Result<(), Box<dyn Error>> {
    let model = polynomial_model(
        |x, _| x - 0.2,
        AffineNormalization::try_new(Point::try_new([4.0, -3.0])?, [[2.0, 0.0], [0.0, 0.5]])?,
    )?;
    let report = model.try_isolines(&request(
        IsolineMethod::DisambiguatedMarchingSquares,
        [3.0, -4.0],
        [5.0, -2.0],
        settings(8)?,
    )?)?;

    assert_eq!(report.polylines().len(), 1);
    assert!(!report.polylines()[0].is_closed());
    assert_eq!(report.diagnostics().open_polylines(), 1);
    assert_eq!(report.diagnostics().closed_polylines(), 0);
    assert_eq!(report.diagnostics().boundary_endpoints().len(), 2);
    let sides = report
        .diagnostics()
        .boundary_endpoints()
        .iter()
        .flat_map(|endpoint| endpoint.sides().iter().copied())
        .collect::<Vec<_>>();
    assert_eq!(
        sides,
        [IsolineBoundarySide::Bottom, IsolineBoundarySide::Top]
    );
    for vertex in report.vertices() {
        assert_close(vertex.point().components()[0], 4.4, 1.0e-9);
        assert!(vertex.residual().abs() <= 1.0e-10);
    }
    Ok(())
}

#[test]
fn circle_is_one_closed_deduplicated_component() -> Result<(), Box<dyn Error>> {
    let model = polynomial_model(|x, y| x * x + y * y - 0.81, identity_normalization()?)?;
    let report = model.try_isolines(&request(
        IsolineMethod::DisambiguatedMarchingSquares,
        [-1.5, -1.5],
        [1.5, 1.5],
        settings(24)?,
    )?)?;

    assert_eq!(report.polylines().len(), 1);
    assert!(report.polylines()[0].is_closed());
    assert!(report.polylines()[0].vertex_indices().len() > 32);
    assert!(report.diagnostics().boundary_endpoints().is_empty());
    assert_eq!(report.diagnostics().closed_polylines(), 1);
    assert_eq!(
        report.diagnostics().deduplicated_vertices(),
        report.diagnostics().unique_segments()
    );
    for vertex in report.vertices() {
        let [x, y] = *vertex.point().components();
        assert_close(x * x + y * y, 0.81, 2.0e-9);
        assert!(vertex.residual().abs() <= 1.0e-10);
    }
    Ok(())
}

#[test]
fn saddle_tie_is_disambiguated_deterministically() -> Result<(), Box<dyn Error>> {
    let model = polynomial_model(|x, y| x * y, identity_normalization()?)?;
    let report = model.try_isolines(&request(
        IsolineMethod::DisambiguatedMarchingSquares,
        [-1.0, -1.0],
        [1.0, 1.0],
        settings(1)?,
    )?)?;

    assert_eq!(report.polylines().len(), 2);
    assert!(report.polylines().iter().all(|line| !line.is_closed()));
    assert_eq!(report.diagnostics().boundary_endpoints().len(), 4);
    assert_eq!(report.diagnostics().ambiguous_cells().len(), 1);
    let ambiguous = report.diagnostics().ambiguous_cells()[0];
    assert_eq!(
        ambiguous.decider(),
        IsolineAmbiguityDecider::PositiveConnectivityTie
    );
    assert_eq!(
        ambiguous.pairing(),
        IsolineCellPairing::BottomRightAndTopLeft
    );
    assert_eq!(ambiguous.normalized_decider().to_bits(), 0.0_f64.to_bits());
    Ok(())
}

#[test]
fn nonzero_asymptotic_decider_selects_the_positive_connection() -> Result<(), Box<dyn Error>> {
    let model = polynomial_model(|x, y| x * y + 0.2, identity_normalization()?)?;
    let report = model.try_isolines(&request(
        IsolineMethod::DisambiguatedMarchingSquares,
        [-1.0, -1.0],
        [1.0, 1.0],
        settings(1)?,
    )?)?;

    let ambiguous = report
        .diagnostics()
        .ambiguous_cells()
        .first()
        .ok_or("ambiguous cell")?;
    assert_eq!(ambiguous.decider(), IsolineAmbiguityDecider::BilinearSaddle);
    assert!(ambiguous.normalized_decider() > 0.0);
    assert_eq!(
        ambiguous.pairing(),
        IsolineCellPairing::BottomRightAndTopLeft
    );
    Ok(())
}

#[test]
fn marching_simplices_reference_preserves_circle_topology() -> Result<(), Box<dyn Error>> {
    let model = polynomial_model(|x, y| x * x + y * y - 0.81, identity_normalization()?)?;
    let report = model.try_isolines(&request(
        IsolineMethod::MarchingSimplices,
        [-1.5, -1.5],
        [1.5, 1.5],
        settings(24)?,
    )?)?;

    assert_eq!(report.polylines().len(), 1);
    assert!(report.polylines()[0].is_closed());
    assert_eq!(
        report.diagnostics().method(),
        IsolineMethod::MarchingSimplices
    );
    assert!(report.diagnostics().ambiguous_cells().is_empty());
    assert_eq!(report.diagnostics().duplicate_segments(), 0);
    Ok(())
}

#[test]
fn invalid_input_degenerate_edges_and_refinement_exhaustion_are_structured()
-> Result<(), Box<dyn Error>> {
    assert!(matches!(
        IsolineSettings::try_new(
            NonZeroU32::MIN,
            NonZeroU32::MIN,
            NonZeroU32::MIN,
            0.0,
            1.0e-9,
        ),
        Err(IsolineSettingsError::InvalidTolerance {
            tolerance: IsolineTolerance::Value,
            ..
        })
    ));
    let valid_settings = settings(4)?;
    assert!(matches!(
        IsolineRequest::try_new(
            0.0,
            Point::try_new([1.0, -1.0])?,
            Point::try_new([1.0, 1.0])?,
            IsolineMethod::DisambiguatedMarchingSquares,
            valid_settings,
        ),
        Err(IsolineRequestError::InvalidDomain { .. })
    ));

    let constant = polynomial_model(|_, _| 0.0, identity_normalization()?)?;
    let degenerate = request(
        IsolineMethod::DisambiguatedMarchingSquares,
        [-1.0, -1.0],
        [1.0, 1.0],
        settings(2)?,
    )?;
    assert!(matches!(
        constant.try_isolines(&degenerate),
        Err(IsolineError::DegenerateGridEdge { .. })
    ));

    let line = polynomial_model(|x, _| x - 0.123, identity_normalization()?)?;
    let insufficient = IsolineSettings::try_new(
        NonZeroU32::MIN,
        NonZeroU32::MIN,
        NonZeroU32::MIN,
        1.0e-16,
        1.0e-16,
    )?;
    assert!(matches!(
        line.try_isolines(&request(
            IsolineMethod::DisambiguatedMarchingSquares,
            [-1.0, -1.0],
            [1.0, 1.0],
            insufficient,
        )?),
        Err(IsolineError::RefinementLimitReached { .. })
    ));
    Ok(())
}

#[test]
fn work_overflow_cancellation_serial_policy_and_progress_are_enforced() -> Result<(), Box<dyn Error>>
{
    let model = polynomial_model(|x, _| x - 0.2, identity_normalization()?)?;
    let overflow = IsolineSettings::try_new(
        NonZeroU32::MAX,
        NonZeroU32::MAX,
        NonZeroU32::MIN,
        1.0e-9,
        1.0e-9,
    )?;
    assert!(matches!(
        model.try_isolines(&request(
            IsolineMethod::DisambiguatedMarchingSquares,
            [-1.0, -1.0],
            [1.0, 1.0],
            overflow,
        )?),
        Err(IsolineError::WorkBudgetOverflow { .. })
    ));

    let extraction = request(
        IsolineMethod::DisambiguatedMarchingSquares,
        [-1.0, -1.0],
        [1.0, 1.0],
        settings(8)?,
    )?;
    let rejected_sink = RecordingProgress::default();
    let requested_threads = NonZeroUsize::new(2).ok_or("thread count")?;
    let rejected = model.try_isolines_with_control(
        &extraction,
        ExecutionOptions::new(false, Some(requested_threads), None),
        ExecutionControl::with_progress(&rejected_sink),
    );
    assert!(matches!(
        rejected,
        Err(IsolineError::Execution(
            ExecutionError::UnsupportedThreadCount {
                operation: ExecutionOperation::IsolineExtraction,
                requested,
                maximum,
            }
        )) if requested == requested_threads && maximum == NonZeroUsize::MIN
    ));
    assert!(rejected_sink.events().is_empty());

    let token = CancellationToken::new();
    token.cancel();
    assert!(matches!(
        model.try_isolines_with_control(
            &extraction,
            ExecutionOptions::default(),
            ExecutionControl::with_cancellation(&token),
        ),
        Err(IsolineError::Execution(ExecutionError::Cancelled {
            operation: ExecutionOperation::IsolineExtraction,
            ..
        }))
    ));

    let sink = RecordingProgress::default();
    let report = model.try_isolines_with_control(
        &extraction,
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
    assert_eq!(
        events
            .iter()
            .filter(|event| event.stage() == ExecutionStage::IsolineEvaluation)
            .count(),
        report.diagnostics().evaluations()
    );
    assert!(events.iter().all(|event| {
        event.operation() == ExecutionOperation::IsolineExtraction
            && !event.deterministic()
            && event.thread_count() == NonZeroUsize::MIN
    }));
    assert!(events.windows(2).all(|pair| {
        pair[0].completed() <= pair[1].completed() && pair[0].total() == pair[1].total()
    }));
    Ok(())
}
