//! Independent truth and error-path tests for atomic functionals.

#![allow(clippy::float_cmp, clippy::unwrap_used)]

use std::error::Error;
use std::io;

use georbf::{
    CenterRepresenter, FunctionalAtom, FunctionalError, FunctionalExpr, FunctionalProvenance,
    FunctionalStorage, FunctionalTerm, Gaussian, KernelActionError, KernelDerivativeOrder, Matern,
    MaternSmoothness, ObservationFunctional, Point, PolynomialSpace, RadialSeparation,
    ScalarFieldSample, SpatialKernelJet, SpatialKernelJetPrefix, UnitDirection,
};

fn provenance(identifier: u64) -> FunctionalProvenance {
    FunctionalProvenance::new(identifier)
}

fn term<const D: usize>(coefficient: f64, atom: FunctionalAtom<D>) -> FunctionalTerm<D>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    FunctionalTerm::try_new(coefficient, atom).unwrap()
}

fn expression<const D: usize>(
    terms: impl IntoIterator<Item = FunctionalTerm<D>>,
) -> FunctionalExpr<D>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    FunctionalExpr::try_new(terms).unwrap()
}

fn gaussian_jet<const D: usize>(
    query: Point<D>,
    center: Point<D>,
    length_scale: f64,
) -> Result<SpatialKernelJetPrefix<D>, Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    let separation = RadialSeparation::try_new(query, center)?;
    let radial = Gaussian::try_new(length_scale)?.radial_jet(separation)?;
    Ok(SpatialKernelJet::try_new(separation, radial)?.into())
}

fn single_observation<const D: usize>(atom: FunctionalAtom<D>) -> ObservationFunctional<D>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    ObservationFunctional::new(expression([term(1.0, atom)]))
}

fn single_center<const D: usize>(atom: FunctionalAtom<D>) -> CenterRepresenter<D>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    CenterRepresenter::new(expression([term(1.0, atom)]))
}

fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "actual={actual:.17e}, expected={expected:.17e}, tolerance={tolerance:.3e}"
    );
}

#[test]
fn analytic_scalar_field_actions_cover_d1_d2_d3() {
    let x1 = Point::try_new([2.0]).unwrap();
    let d1 = UnitDirection::try_new([1.0]).unwrap();
    let expr1 = expression([
        term(2.0, FunctionalAtom::value(x1, provenance(1))),
        term(
            -3.0,
            FunctionalAtom::directional_derivative(x1, d1, provenance(2)),
        ),
    ]);
    let sample1 = ScalarFieldSample::try_new(4.0, [4.0]).unwrap();
    assert_eq!(expr1.try_apply_samples(&[sample1, sample1]).unwrap(), -4.0);

    let x2 = Point::try_new([3.0, -2.0]).unwrap();
    let d2 = UnitDirection::try_new([0.0, 1.0]).unwrap();
    let expr2 = expression([
        term(0.5, FunctionalAtom::value(x2, provenance(3))),
        term(
            4.0,
            FunctionalAtom::directional_derivative(x2, d2, provenance(4)),
        ),
    ]);
    let sample2 = ScalarFieldSample::try_new(5.0, [6.0, 2.0]).unwrap();
    assert_eq!(expr2.try_apply_samples(&[sample2, sample2]).unwrap(), 10.5);

    let x3 = Point::try_new([1.0, 2.0, -1.0]).unwrap();
    let d3 = UnitDirection::try_new([0.0, 0.0, -2.0]).unwrap();
    let expr3 = expression([
        term(1.0, FunctionalAtom::value(x3, provenance(5))),
        term(
            0.25,
            FunctionalAtom::directional_derivative(x3, d3, provenance(6)),
        ),
    ]);
    let sample3 = ScalarFieldSample::try_new(2.0, [1.0, 2.0, 3.0]).unwrap();
    assert_eq!(expr3.try_apply_samples(&[sample3, sample3]).unwrap(), 1.25);
}

#[test]
fn polynomial_action_is_analytic_and_linear_in_all_dimensions() {
    let d1 = UnitDirection::try_new([1.0]).unwrap();
    let point1 = Point::try_new([2.0]).unwrap();
    let expr1 = expression([
        term(2.0, FunctionalAtom::value(point1, provenance(10))),
        term(
            -1.0,
            FunctionalAtom::directional_derivative(point1, d1, provenance(11)),
        ),
    ]);
    assert_eq!(
        expr1
            .try_apply_polynomial(&PolynomialSpace::try_new(3).unwrap())
            .unwrap(),
        [2.0, 3.0, 4.0]
    );

    let d2 = UnitDirection::try_new([1.0, 0.0]).unwrap();
    let point2 = Point::try_new([2.0, 3.0]).unwrap();
    let expr2 = expression([
        term(2.0, FunctionalAtom::value(point2, provenance(12))),
        term(
            -1.0,
            FunctionalAtom::directional_derivative(point2, d2, provenance(13)),
        ),
    ]);
    assert_eq!(
        expr2
            .try_apply_polynomial(&PolynomialSpace::try_new(3).unwrap())
            .unwrap(),
        [2.0, 3.0, 6.0, 4.0, 9.0, 18.0]
    );

    let d3 = UnitDirection::try_new([1.0, 0.0, 0.0]).unwrap();
    let point3 = Point::try_new([2.0, 3.0, 4.0]).unwrap();
    let expr3 = expression([
        term(1.0, FunctionalAtom::value(point3, provenance(14))),
        term(
            2.0,
            FunctionalAtom::directional_derivative(point3, d3, provenance(15)),
        ),
    ]);
    assert_eq!(
        expr3
            .try_apply_polynomial(&PolynomialSpace::try_new(2).unwrap())
            .unwrap(),
        [1.0, 4.0, 3.0, 4.0]
    );
}

#[test]
fn kernel_actions_apply_query_and_center_signs_exactly_once() {
    let query = Point::try_new([2.0]).unwrap();
    let center = Point::try_new([0.5]).unwrap();
    let direction = UnitDirection::try_new([1.0]).unwrap();
    let scale: f64 = 2.0;
    let displacement: f64 = 1.5;
    let phi = (-0.5_f64 * (displacement / scale).powi(2)).exp();
    let query_first = -displacement / scale.powi(2) * phi;
    let center_first = -query_first;
    let mixed_second = (1.0 / scale.powi(2) - displacement.powi(2) / scale.powi(4)) * phi;

    let value_query = FunctionalAtom::value(query, provenance(20));
    let derivative_query = FunctionalAtom::directional_derivative(query, direction, provenance(21));
    let value_center = FunctionalAtom::value(center, provenance(22));
    let derivative_center =
        FunctionalAtom::directional_derivative(center, direction, provenance(23));
    let evaluate = |x, y, _| gaussian_jet(x, y, scale);

    assert_close(
        single_observation(value_query)
            .try_apply_kernel(&single_center(value_center), evaluate)
            .unwrap(),
        phi,
        2.0e-15,
    );
    assert_close(
        single_observation(derivative_query)
            .try_apply_kernel(&single_center(value_center), evaluate)
            .unwrap(),
        query_first,
        2.0e-15,
    );
    assert_close(
        single_observation(value_query)
            .try_apply_kernel(&single_center(derivative_center), evaluate)
            .unwrap(),
        center_first,
        2.0e-15,
    );
    assert_close(
        single_observation(derivative_query)
            .try_apply_kernel(&single_center(derivative_center), evaluate)
            .unwrap(),
        mixed_second,
        2.0e-15,
    );
}

#[test]
fn kernel_exchange_identity_and_center_limit_hold() {
    let left = Point::try_new([-1.0, 0.5]).unwrap();
    let right = Point::try_new([2.0, -0.25]).unwrap();
    let direction = UnitDirection::try_new([2.0, -1.0]).unwrap();
    let left_derivative = FunctionalAtom::directional_derivative(left, direction, provenance(30));
    let right_derivative = FunctionalAtom::directional_derivative(right, direction, provenance(31));
    let left_value = FunctionalAtom::value(left, provenance(32));
    let right_value = FunctionalAtom::value(right, provenance(33));
    let evaluate = |x, y, _| gaussian_jet(x, y, 1.5);

    let left_right = single_observation(left_derivative)
        .try_apply_kernel(&single_center(right_value), evaluate)
        .unwrap();
    let exchanged = single_observation(right_value)
        .try_apply_kernel(&single_center(left_derivative), evaluate)
        .unwrap();
    assert_close(left_right, exchanged, 3.0e-15);

    let derivative_derivative = single_observation(left_derivative)
        .try_apply_kernel(&single_center(right_derivative), evaluate)
        .unwrap();
    let derivative_derivative_exchanged = single_observation(right_derivative)
        .try_apply_kernel(&single_center(left_derivative), evaluate)
        .unwrap();
    assert_close(
        derivative_derivative,
        derivative_derivative_exchanged,
        3.0e-15,
    );

    let center_action = single_observation(left_derivative)
        .try_apply_kernel(&single_center(left_derivative), evaluate)
        .unwrap();
    assert_close(center_action, 1.0 / 1.5_f64.powi(2), 3.0e-15);

    let value_exchange = single_observation(left_value)
        .try_apply_kernel(&single_center(right_value), evaluate)
        .unwrap();
    let value_exchange_reversed = single_observation(right_value)
        .try_apply_kernel(&single_center(left_value), evaluate)
        .unwrap();
    assert_close(value_exchange, value_exchange_reversed, 3.0e-15);
}

#[test]
fn coincident_matern_actions_request_only_the_exact_derivative_demand() -> Result<(), Box<dyn Error>>
{
    let point = Point::try_new([0.0, 0.0])?;
    let value = FunctionalAtom::value(point, provenance(34));
    let one_half = Matern::try_new(MaternSmoothness::OneHalf, 2.0)?;
    let value_action = single_observation(value).try_apply_kernel(
        &single_center(value),
        |query, center, demand| -> Result<SpatialKernelJetPrefix<2>, io::Error> {
            assert_eq!(demand, KernelDerivativeOrder::Value);
            let separation = RadialSeparation::try_new(query, center)
                .map_err(|error| io::Error::other(error.to_string()))?;
            SpatialKernelJetPrefix::try_center_value(
                separation,
                one_half
                    .radial_value(separation.radius())
                    .map_err(|error| io::Error::other(error.to_string()))?,
            )
            .map_err(|error| io::Error::other(error.to_string()))
        },
    )?;
    assert_eq!(value_action, 1.0);

    let direction = UnitDirection::try_new([1.0, 0.0])?;
    let derivative = FunctionalAtom::directional_derivative(point, direction, provenance(35));
    let three_halves = Matern::try_new(MaternSmoothness::ThreeHalves, 2.0)?;
    let derivative_action = single_observation(derivative).try_apply_kernel(
        &single_center(derivative),
        |query, center, demand| -> Result<SpatialKernelJetPrefix<2>, io::Error> {
            assert_eq!(demand, KernelDerivativeOrder::Second);
            let separation = RadialSeparation::try_new(query, center)
                .map_err(|error| io::Error::other(error.to_string()))?;
            let second = three_halves
                .radial_derivative(separation.radius(), KernelDerivativeOrder::Second)
                .map_err(|error| io::Error::other(error.to_string()))?
                .ok_or_else(|| io::Error::other("Matérn 3/2 center second derivative missing"))?;
            SpatialKernelJetPrefix::try_center_through_second(
                separation,
                three_halves
                    .radial_value(separation.radius())
                    .map_err(|error| io::Error::other(error.to_string()))?,
                second,
            )
            .map_err(|error| io::Error::other(error.to_string()))
        },
    )?;
    assert_close(derivative_action, 0.75, 2.0e-15);
    Ok(())
}

#[test]
fn expression_kernel_action_is_bilinear() {
    let query = Point::try_new([1.0, -2.0, 0.5]).unwrap();
    let center = Point::try_new([-0.5, 0.25, 1.0]).unwrap();
    let direction = UnitDirection::try_new([1.0, 2.0, -1.0]).unwrap();
    let query_value = FunctionalAtom::value(query, provenance(40));
    let query_derivative = FunctionalAtom::directional_derivative(query, direction, provenance(41));
    let center_value = FunctionalAtom::value(center, provenance(42));
    let evaluate = |x, y, _| gaussian_jet(x, y, 3.0);

    let combined = ObservationFunctional::new(expression([
        term(2.0, query_value),
        term(-0.75, query_derivative),
    ]))
    .try_apply_kernel(&single_center(center_value), evaluate)
    .unwrap();
    let value_action = single_observation(query_value)
        .try_apply_kernel(&single_center(center_value), evaluate)
        .unwrap();
    let derivative_action = single_observation(query_derivative)
        .try_apply_kernel(&single_center(center_value), evaluate)
        .unwrap();
    assert_close(
        combined,
        2.0 * value_action - 0.75 * derivative_action,
        3.0e-15,
    );
}

#[test]
fn construction_and_action_errors_are_structured() {
    let point = Point::try_new([0.0]).unwrap();
    let atom = FunctionalAtom::value(point, provenance(50));
    assert!(matches!(
        FunctionalTerm::try_new(f64::NAN, atom),
        Err(FunctionalError::NonFiniteCoefficient { value }) if value.is_nan()
    ));
    assert!(matches!(
        FunctionalExpr::<1>::try_new([]),
        Err(FunctionalError::EmptyExpression)
    ));
    assert!(matches!(
        ScalarFieldSample::<1>::try_new(f64::INFINITY, [0.0]),
        Err(FunctionalError::NonFiniteSampleValue { value }) if value == f64::INFINITY
    ));
    assert!(matches!(
        ScalarFieldSample::<1>::try_new(0.0, [f64::NEG_INFINITY]),
        Err(FunctionalError::NonFiniteSampleGradient { axis: 0, value })
            if value == f64::NEG_INFINITY
    ));

    let expr = expression([term(1.0, atom)]);
    assert_eq!(
        expr.try_apply_samples(&[]),
        Err(FunctionalError::SampleLengthMismatch {
            expected: 1,
            actual: 0,
        })
    );
    let maximum = ScalarFieldSample::try_new(f64::MAX, [0.0]).unwrap();
    let overflowing = expression([term(2.0, atom)]);
    assert_eq!(
        overflowing.try_apply_samples(&[maximum]),
        Err(FunctionalError::NonFiniteAction {
            term_index: 0,
            provenance: provenance(50),
        })
    );

    let extreme_point = Point::try_new([f64::MAX]).unwrap();
    let polynomial_expression = expression([term(
        1.0,
        FunctionalAtom::value(extreme_point, provenance(51)),
    )]);
    assert!(matches!(
        polynomial_expression.try_apply_polynomial(&PolynomialSpace::try_new(3).unwrap()),
        Err(FunctionalError::PolynomialEvaluation {
            term_index: 0,
            provenance: source_provenance,
            ..
        }) if source_provenance == provenance(51)
    ));
}

#[test]
fn allocation_failure_is_reported_without_partial_expression() {
    struct ImpossibleHint {
        term: Option<FunctionalTerm<1>>,
    }

    impl Iterator for ImpossibleHint {
        type Item = FunctionalTerm<1>;

        fn next(&mut self) -> Option<Self::Item> {
            self.term.take()
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (usize::MAX, Some(usize::MAX))
        }
    }

    let point = Point::try_new([0.0]).unwrap();
    let iterator = ImpossibleHint {
        term: Some(term(1.0, FunctionalAtom::value(point, provenance(60)))),
    };
    assert_eq!(
        FunctionalExpr::try_new(iterator),
        Err(FunctionalError::AllocationFailed {
            storage: FunctionalStorage::Terms,
            requested: usize::MAX,
        })
    );
}

#[test]
fn kernel_evaluator_errors_retain_both_term_provenances() {
    let first = Point::try_new([0.0]).unwrap();
    let second = Point::try_new([1.0]).unwrap();
    let center = Point::try_new([2.0]).unwrap();
    let observation = ObservationFunctional::new(expression([
        term(1.0, FunctionalAtom::value(first, provenance(70))),
        term(1.0, FunctionalAtom::value(second, provenance(71))),
    ]));
    let center = single_center(FunctionalAtom::value(center, provenance(72)));
    let first_jet =
        gaussian_jet(first, center.expression().terms()[0].atom().point(), 1.0).unwrap();
    let mut calls = 0;
    let error = observation
        .try_apply_kernel(&center, |_, _, _| {
            calls += 1;
            if calls == 1 {
                Ok(first_jet)
            } else {
                Err(io::Error::other("synthetic evaluator failure"))
            }
        })
        .unwrap_err();
    assert!(matches!(
        error,
        KernelActionError::Evaluation {
            observation_term_index: 1,
            observation_provenance,
            center_term_index: 0,
            center_provenance,
            ..
        } if observation_provenance == provenance(71) && center_provenance == provenance(72)
    ));

    let coincident = Point::try_new([0.0]).unwrap();
    let direction = UnitDirection::try_new([1.0]).unwrap();
    let derivative = FunctionalAtom::directional_derivative(coincident, direction, provenance(73));
    let value_only = SpatialKernelJetPrefix::try_center_value(
        RadialSeparation::try_new(coincident, coincident).unwrap(),
        1.0,
    )
    .unwrap();
    assert!(matches!(
        single_observation(derivative)
            .try_apply_kernel(&single_center(derivative), |_, _, _| Ok::<_, io::Error>(
                value_only
            ),),
        Err(KernelActionError::InsufficientDerivativeOrder {
            demanded: KernelDerivativeOrder::Second,
            available_through: KernelDerivativeOrder::Value,
            ..
        })
    ));

    let finite_jet = gaussian_jet(coincident, coincident, 1.0).unwrap();
    let overflowing_observation = ObservationFunctional::new(expression([term(
        f64::MAX,
        FunctionalAtom::value(coincident, provenance(74)),
    )]));
    let weighted_center = CenterRepresenter::new(expression([term(
        2.0,
        FunctionalAtom::value(coincident, provenance(75)),
    )]));
    assert!(matches!(
        overflowing_observation.try_apply_kernel(&weighted_center, |_, _, _| {
            Ok::<_, io::Error>(finite_jet)
        }),
        Err(KernelActionError::NonFiniteAction {
            observation_term_index: 0,
            observation_provenance,
            center_term_index: 0,
            center_provenance,
        }) if observation_provenance == provenance(74) && center_provenance == provenance(75)
    ));
}

#[test]
fn observation_and_center_types_preserve_order_and_are_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<FunctionalExpr<1>>();
    assert_send_sync::<FunctionalExpr<2>>();
    assert_send_sync::<FunctionalExpr<3>>();
    assert_send_sync::<ObservationFunctional<3>>();
    assert_send_sync::<CenterRepresenter<3>>();
    assert_send_sync::<SpatialKernelJetPrefix<3>>();

    let point = Point::try_new([0.0, 1.0]).unwrap();
    let first = FunctionalAtom::value(point, provenance(80));
    let second = FunctionalAtom::value(point, provenance(81));
    let expr = expression([term(3.0, first), term(-2.0, second)]);
    assert_eq!(expr.terms()[0].atom().provenance(), provenance(80));
    assert_eq!(expr.terms()[1].atom().provenance(), provenance(81));
    assert_eq!(expr.terms()[0].coefficient(), 3.0);
    assert_eq!(expr.terms()[1].coefficient(), -2.0);
    assert_eq!(
        expr.maximum_derivative_order(),
        georbf::KernelDerivativeOrder::Value
    );

    let derivative = FunctionalAtom::directional_derivative(
        point,
        UnitDirection::try_new([1.0, 0.0]).unwrap(),
        provenance(82),
    );
    let mixed = expression([term(1.0, first), term(1.0, derivative)]);
    assert_eq!(
        mixed.maximum_derivative_order(),
        georbf::KernelDerivativeOrder::First
    );

    let observation = ObservationFunctional::new(expr.clone());
    let center = CenterRepresenter::new(expr.clone());
    assert_eq!(observation.expression(), &expr);
    assert_eq!(center.expression(), &expr);
    assert_ne!(
        std::any::type_name::<ObservationFunctional<2>>(),
        std::any::type_name::<CenterRepresenter<2>>()
    );
}
