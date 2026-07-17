//! Independent behavior tests for deterministic execution controls.

use std::error::Error;
use std::io;
use std::num::NonZeroUsize;
use std::sync::Mutex;
use std::thread;

use georbf::{
    AffineNormalization, AngleUnit, AxisOrder, CancellationToken, CenterRepresenter,
    ConditionPolicy, CoordinateMetadata, CrsMetadata, DenseEqualitySystem, DenseFactorization,
    DenseSolveError, DenseSolveOptions, Enforcement, ExecutionControl, ExecutionError,
    ExecutionOperation, ExecutionOptions, ExecutionStage, FieldAssemblyError, FieldProblem,
    FittedField, FunctionalAtom, FunctionalExpr, FunctionalProvenance, FunctionalTerm, Gaussian,
    Handedness, KernelDefinition, LengthUnit, ObservationFunctional, ObservationId, Point,
    ProgressEvent, ProgressSink, RadialSeparation, Regularization, SemanticConstraint,
    SemanticExpression, SemanticProblemIr, SemanticProvenance, SemanticRelation, SourceLocation,
    SpatialKernelJet, VerticalDirection,
};

type TestResult = Result<(), Box<dyn Error>>;

const TEST_MEMORY_LIMIT_BYTES: usize = 64 * 1024 * 1024;

#[derive(Default)]
struct RecordingSink {
    events: Mutex<Vec<ProgressEvent>>,
}

impl RecordingSink {
    fn events(&self) -> Vec<ProgressEvent> {
        self.events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

impl ProgressSink for RecordingSink {
    fn on_progress(&self, event: ProgressEvent) {
        self.events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(event);
    }
}

struct CancellingSink {
    events: Mutex<Vec<ProgressEvent>>,
    token: CancellationToken,
    stage: ExecutionStage,
}

impl CancellingSink {
    fn new(token: CancellationToken, stage: ExecutionStage) -> Self {
        Self {
            events: Mutex::new(Vec::new()),
            token,
            stage,
        }
    }

    fn events(&self) -> Vec<ProgressEvent> {
        self.events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

impl ProgressSink for CancellingSink {
    fn on_progress(&self, event: ProgressEvent) {
        self.events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(event);
        if event.stage() == self.stage {
            self.token.cancel();
        }
    }
}

fn solve_options() -> Result<DenseSolveOptions, Box<dyn Error>> {
    solve_options_with(Regularization::None, 2)
}

fn solve_options_with(
    regularization: Regularization,
    maximum_refinement_steps: usize,
) -> Result<DenseSolveOptions, Box<dyn Error>> {
    Ok(DenseSolveOptions::try_new(
        DenseFactorization::Cholesky,
        regularization,
        ConditionPolicy::default(),
        maximum_refinement_steps,
        NonZeroUsize::new(TEST_MEMORY_LIMIT_BYTES).ok_or("memory limit")?,
    )?)
}

fn dense_system() -> Result<DenseEqualitySystem, Box<dyn Error>> {
    Ok(DenseEqualitySystem::try_from_row_major(
        2,
        vec![4.0, 1.0, 1.0, 3.0],
        vec![6.0, 7.0],
    )?)
}

fn field_problem(execution: ExecutionOptions) -> Result<FieldProblem<1>, Box<dyn Error>> {
    let point = Point::try_new([0.0])?;
    let expression = FunctionalExpr::try_new([FunctionalTerm::try_new(
        1.0,
        FunctionalAtom::value(point, FunctionalProvenance::new(1)),
    )?])?;
    let provenance = SemanticProvenance::try_new(
        ObservationId::new(1),
        SourceLocation::try_new(
            "execution-test.csv".to_owned(),
            NonZeroUsize::new(1).ok_or("source line")?,
        )?,
        "m".to_owned(),
        "field.equalities[0]".to_owned(),
        Some("execution controls".to_owned()),
    )?;
    let constraint = SemanticConstraint::try_new(
        provenance,
        SemanticRelation::Equality {
            expression: SemanticExpression::try_new(
                ObservationFunctional::new(expression.clone()),
                0.0,
            )?,
            target: 3.0,
        },
        Enforcement::Hard,
    )?;
    Ok(FieldProblem::try_new(
        SemanticProblemIr::try_new([constraint], execution)?,
        [CenterRepresenter::new(expression)],
    )?)
}

fn assemble_with_control(
    problem: &FieldProblem<1>,
    kernel: &Gaussian,
    control: ExecutionControl<'_>,
) -> Result<georbf::DenseFieldSystem<1>, FieldAssemblyError<io::Error>> {
    problem.try_assemble_with_control(
        kernel.metadata(),
        |query, center, _| {
            let separation = RadialSeparation::try_new(query, center)
                .map_err(|error| io::Error::other(error.to_string()))?;
            let radial = kernel
                .radial_jet(separation)
                .map_err(|error| io::Error::other(error.to_string()))?;
            Ok(SpatialKernelJet::try_new(separation, radial)
                .map_err(|error| io::Error::other(error.to_string()))?
                .into())
        },
        control,
    )
}

fn assert_monotonic(events: &[ProgressEvent], operation: ExecutionOperation) {
    assert!(!events.is_empty());
    let total = events[0].total();
    assert!(total > 0);
    for event in events {
        assert_eq!(event.operation(), operation);
        assert_eq!(event.total(), total);
        assert!(event.completed() <= total);
    }
    for pair in events.windows(2) {
        assert!(pair[0].completed() <= pair[1].completed());
    }
}

fn progress_signature(events: &[ProgressEvent]) -> Vec<(ExecutionStage, usize, usize)> {
    events
        .iter()
        .map(|event| (event.stage(), event.completed(), event.total()))
        .collect()
}

#[test]
fn cancellation_token_is_sticky_cloneable_and_thread_safe() {
    let token = CancellationToken::new();
    let other_thread = token.clone();
    assert!(!token.is_cancelled());
    assert!(
        thread::spawn(move || other_thread.cancel()).join().is_ok(),
        "cancellation thread panicked"
    );
    assert!(token.is_cancelled());
    token.cancel();
    assert!(token.is_cancelled());
}

#[test]
fn pre_start_and_in_flight_cancellation_are_structured() -> TestResult {
    let system = dense_system()?;
    let pre_cancelled = CancellationToken::new();
    pre_cancelled.cancel();
    let sink = RecordingSink::default();
    let result = system.try_solve_with_control(
        solve_options()?,
        ExecutionOptions::default(),
        ExecutionControl::new(Some(&pre_cancelled), Some(&sink)),
    );
    assert!(matches!(
        result,
        Err(DenseSolveError::Execution(ExecutionError::Cancelled {
            operation: ExecutionOperation::DenseSolve,
            stage: ExecutionStage::Started,
        }))
    ));
    assert!(sink.events().is_empty());

    let token = CancellationToken::new();
    let cancelling = CancellingSink::new(token.clone(), ExecutionStage::RankReview);
    let result = system.try_solve_with_control(
        solve_options()?,
        ExecutionOptions::default(),
        ExecutionControl::new(Some(&token), Some(&cancelling)),
    );
    assert!(matches!(
        result,
        Err(DenseSolveError::Execution(ExecutionError::Cancelled {
            operation: ExecutionOperation::DenseSolve,
            stage: ExecutionStage::RankReview,
        }))
    ));
    let events = cancelling.events();
    assert_eq!(
        events.last().map(|event| event.stage()),
        Some(ExecutionStage::RankReview)
    );
    assert!(
        !events
            .iter()
            .any(|event| event.stage() == ExecutionStage::Completed)
    );
    Ok(())
}

#[test]
fn cancellation_from_completed_callback_is_post_completion() -> TestResult {
    let token = CancellationToken::new();
    let sink = CancellingSink::new(token.clone(), ExecutionStage::Completed);
    let solution = dense_system()?.try_solve_with_control(
        solve_options()?,
        ExecutionOptions::default(),
        ExecutionControl::new(Some(&token), Some(&sink)),
    )?;

    assert_eq!(solution.values(), &[1.0, 2.0]);
    let events = sink.events();
    assert_eq!(
        events
            .iter()
            .filter(|event| event.stage() == ExecutionStage::Completed)
            .count(),
        1
    );
    assert_eq!(
        events.last().map(|event| event.stage()),
        Some(ExecutionStage::Completed)
    );
    assert!(token.is_cancelled());
    Ok(())
}

#[test]
fn dense_progress_reports_exact_work_and_maximum_budget() -> TestResult {
    let exact_sink = RecordingSink::default();
    let _ = dense_system()?.try_solve_with_control(
        solve_options_with(Regularization::None, 2)?,
        ExecutionOptions::default(),
        ExecutionControl::with_progress(&exact_sink),
    )?;
    assert_eq!(
        progress_signature(&exact_sink.events()),
        vec![
            (ExecutionStage::Started, 0, 7),
            (ExecutionStage::MemoryReview, 1, 7),
            (ExecutionStage::RankReview, 2, 7),
            (ExecutionStage::Factorization, 3, 7),
            (ExecutionStage::ResidualReview, 4, 7),
            (ExecutionStage::ResidualReview, 5, 7),
            (ExecutionStage::Completed, 5, 7),
        ]
    );

    let first_candidate_sink = RecordingSink::default();
    let _ = DenseEqualitySystem::try_from_row_major(1, vec![10.0], vec![1.0])?
        .try_solve_with_control(
            solve_options_with(Regularization::None, 2)?,
            ExecutionOptions::default(),
            ExecutionControl::with_progress(&first_candidate_sink),
        )?;
    assert_eq!(
        progress_signature(&first_candidate_sink.events()),
        vec![
            (ExecutionStage::Started, 0, 7),
            (ExecutionStage::MemoryReview, 1, 7),
            (ExecutionStage::RankReview, 2, 7),
            (ExecutionStage::Factorization, 3, 7),
            (ExecutionStage::ResidualReview, 4, 7),
            (ExecutionStage::Refinement, 5, 7),
            (ExecutionStage::ResidualReview, 6, 7),
            (ExecutionStage::Completed, 6, 7),
        ]
    );

    let regularized_sink = RecordingSink::default();
    let _ = DenseEqualitySystem::try_from_row_major(2, vec![1.0, 1.0, 1.0, 1.0], vec![2.0, 2.0])?
        .try_solve_with_control(
        solve_options_with(Regularization::Explicit(1.0), 0)?,
        ExecutionOptions::default(),
        ExecutionControl::with_progress(&regularized_sink),
    )?;
    assert_eq!(
        progress_signature(&regularized_sink.events()),
        vec![
            (ExecutionStage::Started, 0, 6),
            (ExecutionStage::MemoryReview, 1, 6),
            (ExecutionStage::RankReview, 2, 6),
            (ExecutionStage::RankReview, 3, 6),
            (ExecutionStage::Factorization, 4, 6),
            (ExecutionStage::ResidualReview, 5, 6),
            (ExecutionStage::ResidualReview, 6, 6),
            (ExecutionStage::Completed, 6, 6),
        ]
    );

    let full_refinement_sink = RecordingSink::default();
    let _ = DenseEqualitySystem::try_from_row_major(1, vec![10.0], vec![1.0])?
        .try_solve_with_control(
            solve_options_with(Regularization::None, 1)?,
            ExecutionOptions::default(),
            ExecutionControl::with_progress(&full_refinement_sink),
        )?;
    assert_eq!(
        progress_signature(&full_refinement_sink.events()),
        vec![
            (ExecutionStage::Started, 0, 6),
            (ExecutionStage::MemoryReview, 1, 6),
            (ExecutionStage::RankReview, 2, 6),
            (ExecutionStage::Factorization, 3, 6),
            (ExecutionStage::ResidualReview, 4, 6),
            (ExecutionStage::Refinement, 5, 6),
            (ExecutionStage::ResidualReview, 6, 6),
            (ExecutionStage::Completed, 6, 6),
        ]
    );
    Ok(())
}

#[test]
fn dense_progress_is_monotonic_typed_and_deterministic() -> TestResult {
    let system = dense_system()?;
    let first_sink = RecordingSink::default();
    let execution = ExecutionOptions::new(
        true,
        NonZeroUsize::new(1),
        NonZeroUsize::new(TEST_MEMORY_LIMIT_BYTES),
    );
    let first = system.try_solve_with_control(
        solve_options()?,
        execution,
        ExecutionControl::with_progress(&first_sink),
    )?;
    let first_events = first_sink.events();
    assert_monotonic(&first_events, ExecutionOperation::DenseSolve);
    assert_eq!(
        first_events.first().map(|event| event.stage()),
        Some(ExecutionStage::Started)
    );
    assert_eq!(
        first_events.last().map(|event| event.stage()),
        Some(ExecutionStage::Completed)
    );
    assert!(first_events.iter().all(|event| event.deterministic()));
    assert!(
        first_events
            .iter()
            .all(|event| event.thread_count() == NonZeroUsize::MIN)
    );

    let second_sink = RecordingSink::default();
    let second = system.try_solve_with_control(
        solve_options()?,
        execution,
        ExecutionControl::with_progress(&second_sink),
    )?;
    assert_eq!(first.values(), second.values());
    assert_eq!(first.diagnostics(), second.diagnostics());
    assert_eq!(first_events, second_sink.events());
    Ok(())
}

#[test]
fn explicit_thread_count_is_enforced_before_progress_or_numerical_work() -> TestResult {
    let sink = RecordingSink::default();
    let requested = NonZeroUsize::new(2).ok_or("thread count")?;
    let result = dense_system()?.try_solve_with_control(
        solve_options()?,
        ExecutionOptions::new(false, Some(requested), None),
        ExecutionControl::with_progress(&sink),
    );
    assert!(matches!(
        result,
        Err(DenseSolveError::Execution(
            ExecutionError::UnsupportedThreadCount {
                operation: ExecutionOperation::DenseSolve,
                requested: value,
                maximum,
            }
        )) if value == requested && maximum == NonZeroUsize::MIN
    ));
    assert!(sink.events().is_empty());
    Ok(())
}

#[test]
fn field_assembly_propagates_execution_options_and_cancellation() -> TestResult {
    let execution = ExecutionOptions::new(false, NonZeroUsize::new(1), None);
    let problem = field_problem(execution)?;
    let kernel = Gaussian::try_new(1.0)?;
    let sink = RecordingSink::default();
    let system = assemble_with_control(&problem, &kernel, ExecutionControl::with_progress(&sink))?;
    assert_eq!(system.execution_options(), execution);
    let events = sink.events();
    assert_monotonic(&events, ExecutionOperation::FieldAssembly);
    assert_eq!(
        events.iter().map(|event| event.stage()).collect::<Vec<_>>(),
        vec![
            ExecutionStage::Started,
            ExecutionStage::KernelAssembly,
            ExecutionStage::Canonicalization,
            ExecutionStage::SymmetryReview,
            ExecutionStage::Completed,
        ]
    );
    assert!(events.iter().all(|event| !event.deterministic()));

    let token = CancellationToken::new();
    let cancelling = CancellingSink::new(token.clone(), ExecutionStage::KernelAssembly);
    let result = assemble_with_control(
        &problem,
        &kernel,
        ExecutionControl::new(Some(&token), Some(&cancelling)),
    );
    assert!(matches!(
        result,
        Err(FieldAssemblyError::Execution(ExecutionError::Cancelled {
            operation: ExecutionOperation::FieldAssembly,
            stage: ExecutionStage::KernelAssembly,
        }))
    ));
    assert_eq!(
        cancelling.events().last().map(|event| event.stage()),
        Some(ExecutionStage::KernelAssembly)
    );
    Ok(())
}

#[test]
fn fitted_field_propagates_one_control_without_retaining_it() -> TestResult {
    let sink = RecordingSink::default();
    let metadata = CoordinateMetadata::new(
        LengthUnit::try_new("m")?,
        CrsMetadata::unspecified(),
        AxisOrder::identity(),
        VerticalDirection::Up,
        Handedness::Right,
        AngleUnit::Radians,
    );
    let model = FittedField::try_fit_with_control(
        field_problem(ExecutionOptions::default())?,
        metadata,
        AffineNormalization::try_new(Point::try_new([0.0])?, [[1.0]])?,
        KernelDefinition::from(Gaussian::try_new(1.0)?),
        None,
        solve_options()?,
        ExecutionControl::with_progress(&sink),
    )?;
    let value = model.try_value(Point::try_new([0.0])?)?;
    assert!((value - 3.0).abs() <= f64::EPSILON);
    let events = sink.events();
    let operations = events
        .iter()
        .map(|event| event.operation())
        .collect::<Vec<_>>();
    let first_solve = operations
        .iter()
        .position(|operation| *operation == ExecutionOperation::DenseSolve)
        .ok_or("missing dense solve progress")?;
    assert!(
        operations[..first_solve]
            .iter()
            .all(|operation| *operation == ExecutionOperation::FieldAssembly)
    );
    assert!(
        operations[first_solve..]
            .iter()
            .all(|operation| *operation == ExecutionOperation::DenseSolve)
    );
    assert_eq!(
        events.last().map(|event| event.stage()),
        Some(ExecutionStage::Completed)
    );
    Ok(())
}
