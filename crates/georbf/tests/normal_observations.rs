//! Independent truth tests for normal-observation semantic compilation.

use std::error::Error;

use georbf::{
    AffineExpression, AffineTerm, AngleUnit, CanonicalProblem, DiagnosticPath, Dim, Enforcement,
    ExecutionOptions, FunctionalAtom, GradientMagnitudePolicy, NormalConstraintRole, NormalMode,
    NormalObservation, NormalObservationError, ObservationId, Point, ProblemIrError,
    SemanticProblemIr, SemanticProvenance, SoftLoss, SourceLocation, SupportedDimension,
    UnitDirection, VariableBlock, Vector,
};

type TestResult = Result<(), Box<dyn Error>>;

fn provenance(identifier: u64) -> Result<SemanticProvenance, ProblemIrError> {
    SemanticProvenance::try_new(
        ObservationId::new(identifier),
        SourceLocation::try_new(
            "tests/normal_observations.csv".to_owned(),
            std::num::NonZeroUsize::MIN,
        )?,
        "gradient-unit".to_owned(),
        "fields.stratigraphy.normals".to_owned(),
        Some("normal".to_owned()),
    )
}

fn provenances(start: u64, count: usize) -> Result<Vec<SemanticProvenance>, ProblemIrError> {
    (0..count)
        .map(|offset| provenance(start + offset as u64))
        .collect()
}

fn gradient_linearizer<const D: usize>(
    functional: &georbf::ObservationFunctional<D>,
    _: &SemanticProvenance,
) -> Result<AffineExpression, ProblemIrError>
where
    Dim<D>: SupportedDimension,
{
    let mut coefficients = [0.0; D];
    for term in functional.expression().terms() {
        let FunctionalAtom::DirectionalDerivative { direction, .. } = term.atom() else {
            unreachable!("normal observations contain derivatives only");
        };
        for (axis, component) in direction.components().iter().copied().enumerate() {
            coefficients[axis] += term.coefficient() * component;
        }
    }
    let terms = coefficients
        .into_iter()
        .enumerate()
        .filter(|(_, coefficient)| *coefficient != 0.0)
        .map(|(axis, coefficient)| AffineTerm::try_new(axis, coefficient))
        .collect::<Result<Vec<_>, _>>()?;
    AffineExpression::try_new(terms, 0.0)
}

fn compile<const D: usize>(
    observation: NormalObservation<D>,
) -> Result<CanonicalProblem, Box<dyn Error>>
where
    Dim<D>: SupportedDimension,
{
    let semantic =
        SemanticProblemIr::try_new(observation.into_constraints(), ExecutionOptions::default())?;
    let dimension = std::num::NonZeroUsize::new(D)
        .ok_or_else(|| std::io::Error::other("supported dimensions are nonzero"))?;
    let block = VariableBlock::try_new("gradient".to_owned(), dimension)?;
    Ok(semantic.try_compile([block], gradient_linearizer::<D>)?)
}

fn row<const D: usize>(expression: &AffineExpression) -> [f64; D] {
    let mut values = [0.0; D];
    for term in expression.terms() {
        values[term.variable()] = term.coefficient();
    }
    values
}

fn row_bits<const D: usize>(expression: &AffineExpression) -> [u64; D] {
    row(expression).map(f64::to_bits)
}

fn dot<const D: usize>(left: [f64; D], right: [f64; D]) -> f64 {
    left.into_iter().zip(right).map(|(a, b)| a * b).sum()
}

fn norm<const D: usize>(value: [f64; D]) -> f64 {
    dot(value, value).sqrt()
}

fn close(left: f64, right: f64) -> bool {
    (left - right).abs() <= 2.0e-14 * left.abs().max(right.abs()).max(1.0)
}

fn assert_send_sync<T: Send + Sync>() {}

#[test]
fn gradient_vector_compiles_to_independent_analytic_component_equalities() -> TestResult {
    let observation = NormalObservation::try_gradient_vector(
        provenances(10, 3)?,
        Point::try_new([2.0, -1.0, 4.0])?,
        Vector::try_new([1.25, -2.5, 0.75])?,
        Enforcement::Hard,
    )?;
    assert_eq!(observation.mode(), NormalMode::GradientVector);
    assert_eq!(
        observation.roles(),
        &[
            NormalConstraintRole::GradientComponent(0),
            NormalConstraintRole::GradientComponent(1),
            NormalConstraintRole::GradientComponent(2),
        ]
    );

    let canonical = compile(observation)?;
    assert_eq!(canonical.equalities().len(), 3);
    assert_eq!(
        row_bits(canonical.equalities()[0].row()),
        [1.0_f64.to_bits(), 0.0_f64.to_bits(), 0.0_f64.to_bits()]
    );
    assert_eq!(
        row_bits(canonical.equalities()[1].row()),
        [0.0_f64.to_bits(), 1.0_f64.to_bits(), 0.0_f64.to_bits()]
    );
    assert_eq!(
        row_bits(canonical.equalities()[2].row()),
        [0.0_f64.to_bits(), 0.0_f64.to_bits(), 1.0_f64.to_bits()]
    );
    assert_eq!(
        canonical
            .equalities()
            .iter()
            .map(georbf::CanonicalEquality::rhs)
            .map(f64::to_bits)
            .collect::<Vec<_>>(),
        [1.25_f64.to_bits(), (-2.5_f64).to_bits(), 0.75_f64.to_bits()]
    );
    Ok(())
}

#[test]
fn direction_with_polarity_uses_an_orthonormal_complement_and_oriented_bound() -> TestResult {
    let normal = UnitDirection::try_new([1.0, 2.0, 2.0])?;
    let observation = NormalObservation::try_direction_with_polarity(
        provenances(20, 3)?,
        Point::try_new([0.0, 0.0, 0.0])?,
        normal,
        0.4,
        Enforcement::Hard,
    )?;
    assert_eq!(
        observation.roles(),
        &[
            NormalConstraintRole::OrthogonalComplement(0),
            NormalConstraintRole::OrthogonalComplement(1),
            NormalConstraintRole::PolarProjection,
        ]
    );

    let canonical = compile(observation)?;
    let first = row::<3>(canonical.equalities()[0].row());
    let second = row::<3>(canonical.equalities()[1].row());
    let n = *normal.components();
    assert!(close(norm(first), 1.0));
    assert!(close(norm(second), 1.0));
    assert!(close(dot(first, second), 0.0));
    assert!(close(dot(first, n), 0.0));
    assert!(close(dot(second, n), 0.0));

    let bound = &canonical.linear_bounds()[0];
    for (actual, expected) in row::<3>(bound.row()).into_iter().zip(n) {
        assert!(close(actual, expected));
    }
    assert_eq!(bound.lower(), Some(0.4));
    assert_eq!(bound.upper(), None);
    Ok(())
}

#[test]
fn direction_only_and_axial_rows_are_exactly_sign_invariant() -> TestResult {
    let point = Point::try_new([0.0, 0.0, 0.0])?;
    let positive = UnitDirection::try_new([1.0, -2.0, 3.0])?;
    let negative = UnitDirection::try_new([-1.0, 2.0, -3.0])?;

    let direction_positive = compile(NormalObservation::try_direction_only(
        provenances(30, 2)?,
        point,
        positive,
        Enforcement::Hard,
    )?)?;
    let direction_negative = compile(NormalObservation::try_direction_only(
        provenances(40, 2)?,
        point,
        negative,
        Enforcement::Hard,
    )?)?;
    let axial_positive = compile(NormalObservation::try_axial_direction(
        provenances(50, 2)?,
        point,
        positive,
        Enforcement::Hard,
    )?)?;
    let axial_negative = compile(NormalObservation::try_axial_direction(
        provenances(60, 2)?,
        point,
        negative,
        Enforcement::Hard,
    )?)?;

    let rows = |problem: &CanonicalProblem| {
        problem
            .equalities()
            .iter()
            .map(|equality| row_bits::<3>(equality.row()))
            .collect::<Vec<_>>()
    };
    assert_eq!(rows(&direction_positive), rows(&direction_negative));
    assert_eq!(rows(&direction_positive), rows(&axial_positive));
    assert_eq!(rows(&axial_positive), rows(&axial_negative));
    Ok(())
}

fn cone_margin(problem: &CanonicalProblem, gradient: [f64; 3]) -> f64 {
    let cone = &problem.second_order_cones()[0];
    let lhs = cone
        .lhs()
        .iter()
        .map(|expression| dot(row::<3>(expression), gradient).powi(2))
        .sum::<f64>()
        .sqrt();
    dot(row::<3>(cone.rhs()), gradient) - lhs
}

#[test]
fn angular_cone_has_convex_sign_and_rotation_invariant_analytic_margin() -> TestResult {
    let point = Point::try_new([0.0, 0.0, 0.0])?;
    let first = compile(NormalObservation::try_angular_cone(
        provenances(70, 2)?,
        point,
        UnitDirection::try_new([0.0, 0.0, 1.0])?,
        45.0,
        AngleUnit::Degrees,
        0.25,
        Enforcement::Hard,
    )?)?;
    let rotated = compile(NormalObservation::try_angular_cone(
        provenances(80, 2)?,
        point,
        UnitDirection::try_new([1.0, 0.0, 0.0])?,
        std::f64::consts::FRAC_PI_4,
        AngleUnit::Radians,
        0.25,
        Enforcement::Hard,
    )?)?;

    assert_eq!(first.second_order_cones().len(), 1);
    assert_eq!(first.linear_bounds()[0].lower(), Some(0.25));
    let gradient = [0.2, -0.1, 1.0];
    let cyclic_rotation = [gradient[2], gradient[0], gradient[1]];
    assert!(close(
        cone_margin(&first, gradient),
        cone_margin(&rotated, cyclic_rotation)
    ));
    assert!(cone_margin(&first, gradient) > 0.0);
    Ok(())
}

#[test]
fn invalid_angles_minimums_provenance_and_directions_are_rejected() -> TestResult {
    let point = Point::try_new([0.0, 0.0])?;
    let normal = UnitDirection::try_new([0.0, 1.0])?;
    for (angle, unit) in [
        (-1.0, AngleUnit::Degrees),
        (90.0, AngleUnit::Degrees),
        (f64::INFINITY, AngleUnit::Radians),
        (std::f64::consts::FRAC_PI_2, AngleUnit::Radians),
    ] {
        assert!(matches!(
            NormalObservation::try_angular_cone(
                provenances(90, 2)?,
                point,
                normal,
                angle,
                unit,
                0.0,
                Enforcement::Hard,
            ),
            Err(NormalObservationError::InvalidAngularConeAngle { .. })
        ));
    }
    assert!(matches!(
        NormalObservation::try_direction_with_polarity(
            provenances(100, 2)?,
            point,
            normal,
            -0.1,
            Enforcement::Hard,
        ),
        Err(NormalObservationError::InvalidMinimumGradient { .. })
    ));
    assert!(
        NormalObservation::try_direction_only([provenance(110)?], point, normal, Enforcement::Hard)
            .is_ok()
    );
    assert!(matches!(
        NormalObservation::try_direction_with_polarity(
            [provenance(111)?, provenance(111)?],
            point,
            normal,
            0.0,
            Enforcement::Hard,
        ),
        Err(NormalObservationError::DuplicateObservationId { identifier: 111 })
    ));
    assert!(matches!(
        NormalObservation::try_gradient_vector(
            [provenance(112)?],
            point,
            Vector::try_new([1.0, 2.0])?,
            Enforcement::Hard,
        ),
        Err(NormalObservationError::ProvenanceCountMismatch {
            expected: 2,
            actual: 1
        })
    ));
    assert!(UnitDirection::<2>::try_new([0.0, 0.0]).is_err());
    Ok(())
}

#[test]
fn soft_normal_relations_remain_explicit_objectives() -> TestResult {
    let enforcement = Enforcement::Soft {
        scale: 2.5,
        loss: SoftLoss::SquaredL2,
    };
    let gradient = compile(NormalObservation::try_gradient_vector(
        provenances(115, 2)?,
        Point::try_new([0.0, 0.0])?,
        Vector::try_new([1.0, -1.0])?,
        enforcement,
    )?)?;
    assert!(gradient.equalities().is_empty());
    assert_eq!(gradient.soft_objectives().len(), 2);
    assert!(
        gradient
            .soft_objectives()
            .iter()
            .all(|objective| objective.scale().to_bits() == 2.5_f64.to_bits())
    );

    let cone = compile(NormalObservation::try_angular_cone(
        provenances(117, 2)?,
        Point::try_new([0.0, 0.0])?,
        UnitDirection::try_new([0.0, 1.0])?,
        10.0,
        AngleUnit::Degrees,
        0.0,
        enforcement,
    )?)?;
    assert!(cone.second_order_cones().is_empty());
    assert!(cone.linear_bounds().is_empty());
    assert_eq!(cone.soft_objectives().len(), 2);
    Ok(())
}

#[test]
fn dimension_boundaries_reject_only_vacuous_one_dimensional_modes() -> TestResult {
    let point = Point::try_new([0.0])?;
    let direction = UnitDirection::try_new([1.0])?;
    assert!(
        NormalObservation::try_gradient_vector(
            [provenance(120)?],
            point,
            Vector::try_new([2.0])?,
            Enforcement::Hard,
        )
        .is_ok()
    );
    assert!(
        NormalObservation::try_direction_with_polarity(
            [provenance(121)?],
            point,
            direction,
            0.0,
            Enforcement::Hard,
        )
        .is_ok()
    );
    for result in [
        NormalObservation::try_direction_only(
            Vec::<SemanticProvenance>::new(),
            point,
            direction,
            Enforcement::Hard,
        ),
        NormalObservation::try_axial_direction(
            Vec::<SemanticProvenance>::new(),
            point,
            direction,
            Enforcement::Hard,
        ),
        NormalObservation::try_angular_cone(
            provenances(122, 2)?,
            point,
            direction,
            0.2,
            AngleUnit::Radians,
            0.0,
            Enforcement::Hard,
        ),
    ] {
        assert!(matches!(
            result,
            Err(NormalObservationError::UnsupportedModeInOneDimension { .. })
        ));
    }

    let two = NormalObservation::try_direction_only(
        provenances(130, 1)?,
        Point::try_new([0.0, 0.0])?,
        UnitDirection::try_new([1.0, 1.0])?,
        Enforcement::Hard,
    )?;
    let three = NormalObservation::try_direction_only(
        provenances(140, 2)?,
        Point::try_new([0.0, 0.0, 0.0])?,
        UnitDirection::try_new([1.0, 1.0, 1.0])?,
        Enforcement::Hard,
    )?;
    assert_eq!(two.constraints().len(), 1);
    assert_eq!(three.constraints().len(), 2);
    Ok(())
}

#[test]
fn fitted_small_gradient_review_is_explicit_source_aware_and_non_enforcing() -> TestResult {
    let observation = NormalObservation::try_axial_direction(
        provenances(150, 2)?,
        Point::try_new([0.0, 0.0, 0.0])?,
        UnitDirection::try_new([0.0, 0.0, 1.0])?,
        Enforcement::Hard,
    )?;
    let policy = GradientMagnitudePolicy::try_new(1.0e-299, 0.6)?;
    let diagnostic =
        observation.try_review_gradient(Vector::try_new([3.0e-300, 4.0e-300, 0.0])?, policy)?;
    assert!(close(diagnostic.magnitude() / 1.0e-300, 5.0));
    assert!(diagnostic.is_near_zero());
    assert_eq!(diagnostic.policy(), policy);
    assert_eq!(
        diagnostic.source().observation_id(),
        Some(ObservationId::new(150))
    );
    assert_eq!(
        diagnostic.source(),
        &DiagnosticPath::try_observation(observation.constraints()[0].provenance())?
    );

    assert!(matches!(
        GradientMagnitudePolicy::try_new(0.0, 1.0e-6),
        Err(NormalObservationError::InvalidGradientReferenceScale { .. })
    ));
    assert!(matches!(
        GradientMagnitudePolicy::try_new(1.0, -1.0),
        Err(NormalObservationError::InvalidRelativeGradientThreshold { .. })
    ));
    assert!(matches!(
        GradientMagnitudePolicy::try_new(f64::MAX, 2.0),
        Err(NormalObservationError::GradientThresholdOverflow { .. })
    ));

    assert_send_sync::<NormalObservation<1>>();
    assert_send_sync::<NormalObservation<2>>();
    assert_send_sync::<NormalObservation<3>>();
    assert_send_sync::<georbf::NormalGradientDiagnostics>();
    Ok(())
}
