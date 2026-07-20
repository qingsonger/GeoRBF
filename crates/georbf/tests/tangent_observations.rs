//! Independent truth tests for tangent and derivative-gauge compilation.

use std::error::Error;
use std::num::NonZeroUsize;

use georbf::{
    AffineExpression, AffineTerm, CanonicalProblem, CanonicalSoftRelation, DerivativeGaugeAnchor,
    Dim, Enforcement, ErrorCode, ExecutionOptions, FunctionalAtom, ObservationId, Point,
    ProblemIrError, SemanticProblemIr, SemanticProvenance, SoftLoss, SourceLocation,
    SupportedDimension, TangentObservation, TangentObservationError, TangentProblem,
    TangentProblemError, UnitDirection, VariableBlock,
};

type TestResult = Result<(), Box<dyn Error>>;

fn provenance(identifier: u64, units: &str) -> Result<SemanticProvenance, ProblemIrError> {
    SemanticProvenance::try_new(
        ObservationId::new(identifier),
        SourceLocation::try_new(
            "tests/tangent_observations.csv".to_owned(),
            NonZeroUsize::MIN,
        )?,
        units.to_owned(),
        "fields.stratigraphy.tangents".to_owned(),
        Some("tangent".to_owned()),
    )
}

fn tangent<const D: usize>(
    identifier: u64,
    point: Point<D>,
    direction: [f64; D],
    enforcement: Enforcement,
) -> Result<TangentObservation<D>, Box<dyn Error>>
where
    Dim<D>: SupportedDimension,
{
    Ok(TangentObservation::try_new(
        provenance(identifier, "1/m")?,
        point,
        UnitDirection::try_new(direction)?,
        enforcement,
    )?)
}

fn gauge<const D: usize>(
    identifier: u64,
    point: Point<D>,
    value: f64,
) -> Result<DerivativeGaugeAnchor<D>, Box<dyn Error>>
where
    Dim<D>: SupportedDimension,
{
    Ok(DerivativeGaugeAnchor::try_new(
        provenance(identifier, "m")?,
        point,
        value,
    )?)
}

fn linearize<const D: usize>(
    functional: &georbf::ObservationFunctional<D>,
    _: &SemanticProvenance,
) -> Result<AffineExpression, ProblemIrError>
where
    Dim<D>: SupportedDimension,
{
    let mut coefficients = vec![0.0; D + 1];
    for term in functional.expression().terms() {
        match term.atom() {
            FunctionalAtom::DirectionalDerivative { direction, .. } => {
                for (axis, component) in direction.components().iter().copied().enumerate() {
                    coefficients[axis] += term.coefficient() * component;
                }
            }
            FunctionalAtom::Value { .. } => coefficients[D] += term.coefficient(),
        }
    }
    AffineExpression::try_new(
        coefficients
            .into_iter()
            .enumerate()
            .filter(|(_, coefficient)| *coefficient != 0.0)
            .map(|(axis, coefficient)| AffineTerm::try_new(axis, coefficient))
            .collect::<Result<Vec<_>, _>>()?,
        0.0,
    )
}

fn compile<const D: usize>(problem: TangentProblem<D>) -> Result<CanonicalProblem, Box<dyn Error>>
where
    Dim<D>: SupportedDimension,
{
    Ok(problem.into_semantic_problem().try_compile(
        [VariableBlock::try_new(
            "gradient-and-value".to_owned(),
            NonZeroUsize::new(D + 1).ok_or("zero variable count")?,
        )?],
        linearize::<D>,
    )?)
}

fn row(problem: &CanonicalProblem, index: usize) -> Vec<f64> {
    let mut row = vec![0.0; problem.variable_count()];
    for term in problem.equalities()[index].row().terms() {
        row[term.variable()] = term.coefficient();
    }
    row
}

fn close(left: f64, right: f64) -> bool {
    (left - right).abs() <= 2.0e-14 * left.abs().max(right.abs()).max(1.0)
}

fn assert_send_sync<T: Send + Sync>() {}

#[test]
fn analytic_tangent_lowers_to_the_independent_directional_derivative() -> TestResult {
    let point = Point::try_new([2.0, -1.0, 4.0])?;
    let direction = UnitDirection::try_new([1.0, 2.0, 2.0])?;
    let observation =
        TangentObservation::try_new(provenance(1, "1/m")?, point, direction, Enforcement::Hard)?;
    assert_eq!(observation.point(), point);
    assert_eq!(observation.tangent(), direction);

    let canonical = compile(TangentProblem::try_new(
        [observation],
        Some(gauge(2, Point::try_new([0.0, 0.0, 0.0])?, 3.5)?),
        ExecutionOptions::default(),
    )?)?;
    assert_eq!(canonical.equalities().len(), 2);
    for (actual, expected) in row(&canonical, 0)[..3]
        .iter()
        .copied()
        .zip(direction.components().iter().copied())
    {
        assert!(close(actual, expected));
    }
    assert_eq!(canonical.equalities()[0].rhs().to_bits(), 0.0_f64.to_bits());
    assert_eq!(row(&canonical, 1), vec![0.0, 0.0, 0.0, 1.0]);
    assert_eq!(canonical.equalities()[1].rhs().to_bits(), 3.5_f64.to_bits());
    Ok(())
}

#[test]
fn multiple_independent_tangents_share_a_point_and_preserve_order() -> TestResult {
    let point = Point::try_new([1.0, 2.0, 3.0])?;
    let first = tangent(10, point, [1.0, 0.0, 0.0], Enforcement::Hard)?;
    let second = tangent(11, point, [0.0, 1.0, 0.0], Enforcement::Hard)?;
    let problem = TangentProblem::try_new(
        [first, second],
        Some(gauge(12, point, -4.0)?),
        ExecutionOptions::default(),
    )?;
    assert_eq!(problem.tangent_count(), 2);
    assert_eq!(problem.gauge_point(), point);
    assert_eq!(problem.gauge_value().to_bits(), (-4.0_f64).to_bits());
    assert_eq!(problem.gauge_observation_id(), ObservationId::new(12));
    assert_eq!(
        problem
            .semantic_problem()
            .constraints()
            .iter()
            .map(|constraint| constraint.provenance().observation_id())
            .collect::<Vec<_>>(),
        [
            ObservationId::new(10),
            ObservationId::new(11),
            ObservationId::new(12),
        ]
    );
    let canonical = compile(problem)?;
    assert_eq!(row(&canonical, 0), vec![1.0, 0.0, 0.0, 0.0]);
    assert_eq!(row(&canonical, 1), vec![0.0, 1.0, 0.0, 0.0]);
    Ok(())
}

#[test]
fn every_scalar_soft_loss_remains_explicit_beside_the_hard_gauge() -> TestResult {
    let point = Point::try_new([0.0, 0.0])?;
    let tangents = [
        tangent(
            20,
            point,
            [1.0, 0.0],
            Enforcement::Soft {
                scale: 2.0,
                loss: SoftLoss::SquaredL2,
            },
        )?,
        tangent(
            21,
            point,
            [0.0, 1.0],
            Enforcement::Soft {
                scale: 3.0,
                loss: SoftLoss::AbsoluteL1,
            },
        )?,
        tangent(
            22,
            point,
            [1.0, 1.0],
            Enforcement::Soft {
                scale: 4.0,
                loss: SoftLoss::Huber { delta: 0.25 },
            },
        )?,
    ];
    let canonical = compile(TangentProblem::try_new(
        tangents,
        Some(gauge(23, point, 0.0)?),
        ExecutionOptions::default(),
    )?)?;
    assert_eq!(canonical.equalities().len(), 1);
    assert_eq!(canonical.soft_objectives().len(), 3);
    assert!(
        canonical
            .soft_objectives()
            .iter()
            .all(|objective| matches!(objective.relation(), CanonicalSoftRelation::Equality(_)))
    );
    assert_eq!(
        canonical
            .soft_objectives()
            .iter()
            .map(|objective| (objective.scale(), objective.loss()))
            .collect::<Vec<_>>(),
        [
            (2.0, SoftLoss::SquaredL2),
            (3.0, SoftLoss::AbsoluteL1),
            (4.0, SoftLoss::Huber { delta: 0.25 }),
        ]
    );
    Ok(())
}

#[test]
fn missing_gauge_is_rejected_with_the_first_tangent_source() -> TestResult {
    let Err(error) = TangentProblem::try_new(
        [tangent(
            30,
            Point::try_new([0.0])?,
            [1.0],
            Enforcement::Hard,
        )?],
        None,
        ExecutionOptions::default(),
    ) else {
        return Err("a derivative-only problem selected an implicit anchor".into());
    };
    let diagnostic = error
        .gauge_diagnostic()
        .ok_or("missing public gauge diagnostic")?;
    assert_eq!(diagnostic.code(), ErrorCode::MissingGauge);
    assert_eq!(
        diagnostic
            .primary_source()
            .and_then(georbf::DiagnosticPath::observation_id),
        Some(ObservationId::new(30))
    );
    Ok(())
}

#[test]
fn invalid_metadata_and_duplicate_identifiers_fail_structurally() -> TestResult {
    let Err(invalid) = TangentObservation::try_new(
        provenance(40, "1/m")?,
        Point::try_new([0.0, 0.0])?,
        UnitDirection::try_new([1.0, 0.0])?,
        Enforcement::Soft {
            scale: f64::NAN,
            loss: SoftLoss::SquaredL2,
        },
    ) else {
        return Err("non-finite soft scale was accepted".into());
    };
    assert!(matches!(
        invalid,
        TangentObservationError::Ir(ProblemIrError::InvalidSoftScale { .. })
    ));

    let Err(duplicate) = TangentProblem::try_new(
        [tangent(
            41,
            Point::try_new([0.0, 0.0])?,
            [1.0, 0.0],
            Enforcement::Hard,
        )?],
        Some(gauge(41, Point::try_new([0.0, 0.0])?, 0.0)?),
        ExecutionOptions::default(),
    ) else {
        return Err("duplicate tangent/gauge identifier was accepted".into());
    };
    assert!(matches!(
        duplicate,
        TangentProblemError::Ir(ProblemIrError::DuplicateObservationId { .. })
    ));
    Ok(())
}

#[test]
fn tangent_types_are_dimension_bounded_immutable_send_sync_values() -> TestResult {
    assert_send_sync::<TangentObservation<1>>();
    assert_send_sync::<TangentObservation<2>>();
    assert_send_sync::<TangentObservation<3>>();
    assert_send_sync::<DerivativeGaugeAnchor<1>>();
    assert_send_sync::<TangentProblem<3>>();

    let point = Point::try_new([0.0, 0.0, 0.0])?;
    let tangent = tangent(50, point, [1.0, -2.0, 3.0], Enforcement::Hard)?;
    assert_eq!(tangent.clone(), tangent);
    Ok(())
}

#[allow(dead_code)]
fn semantic_problem_type_check<const D: usize>(_: &SemanticProblemIr<D>)
where
    Dim<D>: SupportedDimension,
{
}
