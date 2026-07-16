//! Independent analytic and property tests for hard-equality field assembly.

use std::error::Error;
use std::io;
use std::num::NonZeroUsize;

use georbf::{
    CenterRepresenter, CpdRankDecision, DenseFieldSystem, Enforcement, ExecutionOptions,
    FieldAssemblyError, FieldProblem, FieldProblemError, FunctionalAtom, FunctionalExpr,
    FunctionalProvenance, FunctionalTerm, Gaussian, KernelArgument, KernelDerivativeCapability,
    KernelDerivativeOrder, Matern, MaternSmoothness, ObservationFunctional, ObservationId, Point,
    PolyharmonicSpline, RadialSeparation, SemanticConstraint, SemanticExpression,
    SemanticProblemIr, SemanticProvenance, SemanticRelation, SourceLocation, SpatialKernelJet,
    SpatialKernelJetPrefix, UnitDirection,
};

type TestResult = Result<(), Box<dyn Error>>;

fn provenance(identifier: u64) -> Result<SemanticProvenance, Box<dyn Error>> {
    Ok(SemanticProvenance::try_new(
        ObservationId::new(identifier),
        SourceLocation::try_new(
            "field-test.csv".to_owned(),
            NonZeroUsize::new(usize::try_from(identifier)?.checked_add(1).ok_or("line")?)
                .ok_or("line")?,
        )?,
        "m".to_owned(),
        format!("field.equalities[{identifier}]"),
        Some("field-test".to_owned()),
    )?)
}

fn mixed_expression<const D: usize>(
    point: [f64; D],
    direction: [f64; D],
    identifier: u64,
) -> Result<FunctionalExpr<D>, Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    let point = Point::try_new(point)?;
    let direction = UnitDirection::try_new(direction)?;
    Ok(FunctionalExpr::try_new([
        FunctionalTerm::try_new(
            1.0,
            FunctionalAtom::value(point, FunctionalProvenance::new(identifier)),
        )?,
        FunctionalTerm::try_new(
            0.25,
            FunctionalAtom::directional_derivative(
                point,
                direction,
                FunctionalProvenance::new(identifier + 1),
            ),
        )?,
    ])?)
}

fn analytic_linear_action<const D: usize>(expression: &FunctionalExpr<D>) -> f64
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    expression
        .terms()
        .iter()
        .map(|term| {
            let atom = term.atom();
            let action = match atom {
                FunctionalAtom::Value { point, .. } => {
                    2.0 + point
                        .components()
                        .iter()
                        .enumerate()
                        .map(|(axis, value)| axis_coefficient(axis) * value)
                        .sum::<f64>()
                }
                FunctionalAtom::DirectionalDerivative { direction, .. } => direction
                    .components()
                    .iter()
                    .enumerate()
                    .map(|(axis, value)| axis_coefficient(axis) * value)
                    .sum(),
            };
            term.coefficient() * action
        })
        .sum()
}

fn axis_coefficient(axis: usize) -> f64 {
    match axis {
        0 => 1.0,
        1 => 2.0,
        2 => 3.0,
        _ => 0.0,
    }
}

fn build_problem<const D: usize>(
    expressions: Vec<FunctionalExpr<D>>,
) -> Result<FieldProblem<D>, Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    let mut constraints = Vec::new();
    let mut centers = Vec::new();
    constraints.try_reserve_exact(expressions.len())?;
    centers.try_reserve_exact(expressions.len())?;
    for (index, expression) in expressions.into_iter().enumerate() {
        let expected = analytic_linear_action(&expression);
        let observation = ObservationFunctional::new(expression.clone());
        centers.push(CenterRepresenter::new(expression));
        constraints.push(SemanticConstraint::try_new(
            provenance(index as u64)?,
            SemanticRelation::Equality {
                expression: SemanticExpression::try_new(observation, 0.5)?,
                target: expected + 0.5,
            },
            Enforcement::Hard,
        )?);
    }
    let semantic = SemanticProblemIr::try_new(constraints, ExecutionOptions::default())?;
    Ok(FieldProblem::try_new(semantic, centers)?)
}

fn gaussian_evaluator<const D: usize>(
    query: Point<D>,
    center: Point<D>,
    length_scale: f64,
) -> Result<georbf::SpatialKernelJetPrefix<D>, io::Error>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    let separation = RadialSeparation::try_new(query, center)
        .map_err(|error| io::Error::other(error.to_string()))?;
    let radial = Gaussian::try_new(length_scale)
        .map_err(|error| io::Error::other(error.to_string()))?
        .radial_jet(separation)
        .map_err(|error| io::Error::other(error.to_string()))?;
    Ok(SpatialKernelJet::try_new(separation, radial)
        .map_err(|error| io::Error::other(error.to_string()))?
        .into())
}

fn independent_gaussian_action<const D: usize>(
    observation: &FunctionalExpr<D>,
    center: &FunctionalExpr<D>,
    length_scale: f64,
) -> f64
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    let inverse_square = 1.0 / length_scale.powi(2);
    let mut result = 0.0;
    for observation_term in observation.terms() {
        for center_term in center.terms() {
            let observation_atom = observation_term.atom();
            let center_atom = center_term.atom();
            let displacement = std::array::from_fn::<_, D, _>(|axis| {
                observation_atom.point().components()[axis] - center_atom.point().components()[axis]
            });
            let radius_square = displacement.iter().map(|value| value * value).sum::<f64>();
            let value = (-0.5 * radius_square * inverse_square).exp();
            let action = match (observation_atom.direction(), center_atom.direction()) {
                (None, None) => value,
                (Some(direction), None) => {
                    -inverse_square
                        * direction
                            .components()
                            .iter()
                            .zip(displacement)
                            .map(|(direction, displacement)| direction * displacement)
                            .sum::<f64>()
                        * value
                }
                (None, Some(direction)) => {
                    inverse_square
                        * direction
                            .components()
                            .iter()
                            .zip(displacement)
                            .map(|(direction, displacement)| direction * displacement)
                            .sum::<f64>()
                        * value
                }
                (Some(left), Some(right)) => {
                    let dot = left
                        .components()
                        .iter()
                        .zip(right.components())
                        .map(|(left, right)| left * right)
                        .sum::<f64>();
                    let left_displacement = left
                        .components()
                        .iter()
                        .zip(displacement)
                        .map(|(direction, displacement)| direction * displacement)
                        .sum::<f64>();
                    let right_displacement = right
                        .components()
                        .iter()
                        .zip(displacement)
                        .map(|(direction, displacement)| direction * displacement)
                        .sum::<f64>();
                    (inverse_square * dot
                        - inverse_square.powi(2) * left_displacement * right_displacement)
                        * value
                }
            };
            result += observation_term.coefficient() * center_term.coefficient() * action;
        }
    }
    result
}

fn verify_dimension<const D: usize>(
    first_point: [f64; D],
    first_direction: [f64; D],
    second_point: [f64; D],
    second_direction: [f64; D],
) -> TestResult
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    let expressions = vec![
        mixed_expression(first_point, first_direction, 10)?,
        mixed_expression(second_point, second_direction, 20)?,
    ];
    let problem = build_problem(expressions.clone())?;
    let kernel = Gaussian::try_new(1.25)?;
    let system = problem.try_assemble(kernel.metadata(), |query, center, _| {
        gaussian_evaluator(query, center, kernel.length_scale())
    })?;

    assert_eq!(system.center_count(), 2);
    assert_eq!(system.polynomial_count(), 0);
    assert_eq!(system.matrix().dimension(), 2);
    assert_eq!(system.diagnostics().kernel_entry_evaluations, 3);
    assert_eq!(
        system.diagnostics().normalized_asymmetry.to_bits(),
        0.0_f64.to_bits()
    );
    assert!(system.cpd().is_none());
    for row in 0..2 {
        assert!((system.rhs()[row] - analytic_linear_action(&expressions[row])).abs() < 2.0e-15);
        for column in 0..2 {
            let expected = independent_gaussian_action(
                &expressions[row],
                &expressions[column],
                kernel.length_scale(),
            );
            let actual = system.matrix().get(row, column).ok_or("matrix entry")?;
            assert!((actual - expected).abs() <= 8.0e-15 * expected.abs().max(1.0));
            assert_eq!(
                actual.to_bits(),
                system
                    .matrix()
                    .get(column, row)
                    .ok_or("transpose")?
                    .to_bits()
            );
        }
    }
    Ok(())
}

#[test]
fn analytic_mixed_field_assembly_is_symmetric_in_d1_d2_d3() -> TestResult {
    verify_dimension([-0.5], [1.0], [0.75], [-1.0])?;
    verify_dimension([-0.5, 0.25], [1.0, 2.0], [0.75, -1.0], [-2.0, 1.0])?;
    verify_dimension(
        [-0.5, 0.25, 1.0],
        [1.0, 2.0, -1.0],
        [0.75, -1.0, 0.5],
        [-2.0, 1.0, 3.0],
    )?;
    Ok(())
}

fn verify_cpd_augmented_matrix(system: &DenseFieldSystem<1>) {
    assert_eq!(system.center_count(), 3);
    assert_eq!(system.polynomial_count(), 2);
    assert_eq!(system.matrix().dimension(), 5);
    let expected_augmented = [
        0.0, 3.0, 8.0, 1.0, -1.0, 3.0, 0.0, -3.0, 0.0, 1.0, 8.0, -3.0, 0.0, 1.0, 1.0, 1.0, 0.0,
        1.0, 0.0, 0.0, -1.0, 1.0, 1.0, 0.0, 0.0,
    ];
    for (actual, expected) in system.matrix().values().iter().zip(expected_augmented) {
        assert!((actual - expected).abs() <= 8.0 * f64::EPSILON);
    }
    assert_eq!(system.rhs()[3..], [0.0, 0.0]);
    assert_eq!(
        system.diagnostics().normalized_asymmetry.to_bits(),
        0.0_f64.to_bits()
    );
}

fn verify_cpd_rank_and_null_space(system: &DenseFieldSystem<1>) -> TestResult {
    let cpd = system.cpd().ok_or("CPD evidence")?;
    assert_eq!(cpd.polynomial_space().term_count(), 2);
    assert_eq!(
        cpd.null_space().actions().values(),
        [1.0, -1.0, 0.0, 1.0, 1.0, 1.0]
    );
    let rank = cpd.null_space().diagnostics();
    assert_eq!(rank.rrqr_rank, 2);
    assert_eq!(rank.svd_rank, 2);
    assert_eq!(rank.decision, CpdRankDecision::FullRank);
    assert_eq!(
        (
            cpd.null_space().actions().rows(),
            cpd.null_space().actions().columns()
        ),
        (3, 2)
    );
    assert_eq!(
        (
            cpd.projected_energy().rows(),
            cpd.projected_energy().columns()
        ),
        (1, 1)
    );
    assert!(
        cpd.null_space().quality().side_condition_residual <= cpd.null_space().quality().tolerance
    );
    let basis = cpd.null_space().basis();
    assert_eq!((basis.rows(), basis.columns()), (3, 1));
    let expected_null = [
        1.0 / 6.0_f64.sqrt(),
        2.0 / 6.0_f64.sqrt(),
        -1.0 / 6.0_f64.sqrt(),
    ];
    let alignment = basis
        .values()
        .iter()
        .zip(expected_null)
        .map(|(actual, expected)| actual * expected)
        .sum::<f64>()
        .abs();
    assert!((alignment - 1.0).abs() <= 32.0 * f64::EPSILON);
    for polynomial in 0..2 {
        let mut side_action = 0.0;
        for row in 0..3 {
            side_action += cpd
                .null_space()
                .actions()
                .get(row, polynomial)
                .ok_or("side action")?
                * basis.get(row, 0).ok_or("basis value")?;
        }
        assert!(side_action.abs() <= cpd.null_space().quality().tolerance);
    }
    let orthonormality = basis
        .values()
        .iter()
        .map(|value| value * value)
        .sum::<f64>();
    assert!((orthonormality - 1.0).abs() <= cpd.null_space().quality().tolerance);
    assert!(
        (cpd.projected_energy().get(0, 0).ok_or("projected scalar")? - 4.0 / 3.0).abs()
            <= 64.0 * f64::EPSILON
    );
    Ok(())
}

#[test]
fn cpd_assembly_adds_complete_polynomial_rows_and_null_space() -> TestResult {
    let points = [-1.0, 0.0, 1.0];
    let mut expressions = Vec::new();
    for (index, point) in points.into_iter().enumerate() {
        let atom = if index == 1 {
            FunctionalAtom::directional_derivative(
                Point::try_new([point])?,
                UnitDirection::try_new([1.0])?,
                FunctionalProvenance::new(index as u64),
            )
        } else {
            FunctionalAtom::value(
                Point::try_new([point])?,
                FunctionalProvenance::new(index as u64),
            )
        };
        expressions.push(FunctionalExpr::try_new([FunctionalTerm::try_new(
            1.0, atom,
        )?])?);
    }
    let problem = build_problem(expressions)?;
    let kernel = PolyharmonicSpline::try_new(3)?;
    let system = problem.try_assemble(kernel.metadata(), |query, center, demand| {
        let separation = RadialSeparation::try_new(query, center)
            .map_err(|error| io::Error::other(error.to_string()))?;
        if separation.is_center() {
            let value = kernel
                .radial_value(0.0)
                .map_err(|error| io::Error::other(error.to_string()))?;
            return match demand {
                KernelDerivativeOrder::Value => {
                    SpatialKernelJetPrefix::try_center_value(separation, value)
                        .map_err(|error| io::Error::other(error.to_string()))
                }
                KernelDerivativeOrder::First => {
                    SpatialKernelJetPrefix::try_center_through_first(separation, value)
                        .map_err(|error| io::Error::other(error.to_string()))
                }
                KernelDerivativeOrder::Second => {
                    let second = kernel
                        .radial_derivative(0.0, KernelDerivativeOrder::Second)
                        .map_err(|error| io::Error::other(error.to_string()))?
                        .ok_or_else(|| io::Error::other("missing center Hessian"))?;
                    SpatialKernelJetPrefix::try_center_through_second(separation, value, second)
                        .map_err(|error| io::Error::other(error.to_string()))
                }
                KernelDerivativeOrder::Third => Err(io::Error::other(
                    "atomic matrix action requested third order",
                )),
            };
        }
        let radial = kernel
            .radial_jet(separation)
            .map_err(|error| io::Error::other(error.to_string()))?;
        Ok::<_, io::Error>(
            SpatialKernelJet::try_new(separation, radial)
                .map_err(|error| io::Error::other(error.to_string()))?
                .into(),
        )
    })?;

    verify_cpd_augmented_matrix(&system);
    verify_cpd_rank_and_null_space(&system)?;
    Ok(())
}

#[test]
fn construction_rejects_nonmatching_observation_and_center_roles() -> TestResult {
    let observation_expression = mixed_expression([0.0], [1.0], 1)?;
    let center_expression = mixed_expression([1.0], [1.0], 3)?;
    let observation = ObservationFunctional::new(observation_expression);
    let constraint = SemanticConstraint::try_new(
        provenance(1)?,
        SemanticRelation::Equality {
            expression: SemanticExpression::try_new(observation, 0.0)?,
            target: 0.0,
        },
        Enforcement::Hard,
    )?;
    let semantic = SemanticProblemIr::try_new([constraint], ExecutionOptions::default())?;
    assert!(matches!(
        FieldProblem::try_new(semantic, [CenterRepresenter::new(center_expression)]),
        Err(FieldProblemError::ObservationCenterExpressionMismatch { index: 0 })
    ));
    Ok(())
}

fn verify_nonsmooth_center_rejection<const D: usize>(
    value_point: [f64; D],
    derivative_point: [f64; D],
    direction: [f64; D],
) -> TestResult
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    let expression = FunctionalExpr::try_new([
        FunctionalTerm::try_new(
            1.0,
            FunctionalAtom::value(Point::try_new(value_point)?, FunctionalProvenance::new(10)),
        )?,
        FunctionalTerm::try_new(
            1.0,
            FunctionalAtom::directional_derivative(
                Point::try_new(derivative_point)?,
                UnitDirection::try_new(direction)?,
                FunctionalProvenance::new(11),
            ),
        )?,
    ])?;
    let problem = build_problem(vec![expression])?;
    let kernel = Matern::try_new(MaternSmoothness::OneHalf, 1.0)?;
    let mut evaluations = 0;
    let result = problem.try_assemble(kernel.metadata(), |_, _, _| {
        evaluations += 1;
        Err::<georbf::SpatialKernelJetPrefix<D>, _>(io::Error::other("must not evaluate"))
    });
    assert_eq!(evaluations, 0);
    assert!(matches!(
        result,
        Err(FieldAssemblyError::UnsupportedDerivativeCapability {
            observation_index: 0,
            observation_term_index: 1,
            center_index: 0,
            center_term_index: 1,
            observation_order: KernelDerivativeOrder::First,
            center_order: KernelDerivativeOrder::First,
            capability: KernelDerivativeCapability::SupportedAwayFromCenters,
            coincident: true,
        })
    ));
    Ok(())
}

#[test]
fn nonsmooth_center_derivative_demand_fails_before_evaluation_in_d1_d2_d3() -> TestResult {
    verify_nonsmooth_center_rejection([0.0], [1.0], [1.0])?;
    verify_nonsmooth_center_rejection([0.0, 0.0], [1.0, 0.0], [1.0, 2.0])?;
    verify_nonsmooth_center_rejection([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 2.0, -1.0])?;
    Ok(())
}

#[test]
fn evaluator_signs_remain_query_center_signs() -> TestResult {
    let query = Point::try_new([0.5])?;
    let center = Point::try_new([-0.25])?;
    let jet = gaussian_evaluator(query, center, 1.0)?;
    let complete = SpatialKernelJet::try_new(
        RadialSeparation::try_new(query, center)?,
        Gaussian::try_new(1.0)?.radial_jet(RadialSeparation::try_new(query, center)?)?,
    )?;
    assert_eq!(jet.value().to_bits(), complete.value().to_bits());
    assert_eq!(
        complete.first_derivative(KernelArgument::Center)[0].to_bits(),
        (-complete.first_derivative(KernelArgument::Query)[0]).to_bits()
    );
    Ok(())
}
