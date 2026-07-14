//! Independent truth and failure-path tests for complete polynomial spaces.

#![allow(clippy::float_cmp)]

use std::error::Error;

use georbf::{
    GeometryError, MultiIndex, Point, PolynomialOutput, PolynomialSpace, PolynomialSpaceError,
};

#[test]
fn complete_spaces_have_exact_combinatorial_term_counts() -> Result<(), Box<dyn Error>> {
    for order in 1..=16 {
        assert_eq!(PolynomialSpace::<1>::try_new(order)?.term_count(), order);
        assert_eq!(
            PolynomialSpace::<2>::try_new(order)?.term_count(),
            order * (order + 1) / 2
        );
        assert_eq!(
            PolynomialSpace::<3>::try_new(order)?.term_count(),
            order * (order + 1) * (order + 2) / 6
        );
    }
    Ok(())
}

#[test]
fn graded_descending_lexicographic_order_is_complete_and_deterministic()
-> Result<(), Box<dyn Error>> {
    let space = PolynomialSpace::<3>::try_new(4)?;
    let actual: Vec<_> = space.terms().iter().map(|term| *term.exponents()).collect();
    let expected = vec![
        [0, 0, 0],
        [1, 0, 0],
        [0, 1, 0],
        [0, 0, 1],
        [2, 0, 0],
        [1, 1, 0],
        [1, 0, 1],
        [0, 2, 0],
        [0, 1, 1],
        [0, 0, 2],
        [3, 0, 0],
        [2, 1, 0],
        [2, 0, 1],
        [1, 2, 0],
        [1, 1, 1],
        [1, 0, 2],
        [0, 3, 0],
        [0, 2, 1],
        [0, 1, 2],
        [0, 0, 3],
    ];

    assert_eq!(actual, expected);
    for (term, expected_degree) in space.terms().iter().zip(
        expected
            .iter()
            .map(|exponents| exponents.iter().sum::<usize>()),
    ) {
        assert_eq!(term.total_degree(), expected_degree);
    }
    Ok(())
}

#[test]
fn values_and_gradients_match_independent_analytic_truth() -> Result<(), Box<dyn Error>> {
    let space = PolynomialSpace::<2>::try_new(3)?;
    let mut values = vec![0.0; space.term_count()];
    let mut gradients = vec![[0.0; 2]; space.term_count()];
    space.try_evaluate(Point::try_new([-2.0, 3.0])?, &mut values, &mut gradients)?;

    assert_eq!(values, [1.0, -2.0, 3.0, 4.0, -6.0, 9.0]);
    assert_eq!(
        gradients,
        [
            [0.0, 0.0],
            [1.0, 0.0],
            [0.0, 1.0],
            [-4.0, 0.0],
            [3.0, -2.0],
            [0.0, 6.0],
        ]
    );
    Ok(())
}

#[test]
fn derivatives_at_axes_and_origin_do_not_divide_by_coordinates() -> Result<(), Box<dyn Error>> {
    let space = PolynomialSpace::<3>::try_new(4)?;
    let mut gradients = vec![[f64::NAN; 3]; space.term_count()];
    space.try_evaluate_gradients(Point::try_new([0.0, 0.0, 0.0])?, &mut gradients)?;

    for (term, gradient) in space.terms().iter().zip(gradients) {
        let exponents = term.exponents();
        let expected = std::array::from_fn(|axis| {
            if exponents[axis] == 1
                && exponents
                    .iter()
                    .enumerate()
                    .all(|(other_axis, exponent)| other_axis == axis || *exponent == 0)
            {
                1.0
            } else {
                0.0
            }
        });
        assert_eq!(gradient, expected);
    }
    Ok(())
}

#[test]
fn scaled_products_preserve_representable_mixed_monomials() -> Result<(), Box<dyn Error>> {
    let space = PolynomialSpace::<2>::try_new(4)?;
    let mut values = vec![0.0; space.term_count()];
    space.try_evaluate_values(Point::try_new([1.0e-200, 1.0e100])?, &mut values)?;

    let mixed_index = space
        .terms()
        .iter()
        .position(|term| term.exponents() == &[2, 1])
        .ok_or("complete degree-three basis omitted [2, 1]")?;
    assert_close(values[mixed_index], 1.0e-300);
    assert_ne!(values[mixed_index], 0.0);
    Ok(())
}

#[test]
fn complete_bases_reproduce_polynomials_in_every_supported_dimension() -> Result<(), Box<dyn Error>>
{
    let one = PolynomialSpace::<1>::try_new(5)?;
    let mut one_values = vec![0.0; one.term_count()];
    let x = 1.25;
    one.try_evaluate_values(Point::try_new([x])?, &mut one_values)?;
    let one_coefficients = [2.0, -3.0, 0.5, 1.0, -0.25];
    let reproduced_one = dot(&one_coefficients, &one_values);
    let analytic_one = 2.0 - 3.0 * x + 0.5 * x * x + x * x * x - 0.25 * x * x * x * x;
    assert_close(reproduced_one, analytic_one);

    let two = PolynomialSpace::<2>::try_new(3)?;
    let mut two_values = vec![0.0; two.term_count()];
    let [x, y] = [-0.75, 1.5];
    two.try_evaluate_values(Point::try_new([x, y])?, &mut two_values)?;
    let two_coefficients = [1.0, 2.0, -3.0, 4.0, 5.0, -6.0];
    let reproduced_two = dot(&two_coefficients, &two_values);
    let analytic_two = 1.0 + 2.0 * x - 3.0 * y + 4.0 * x * x + 5.0 * x * y - 6.0 * y * y;
    assert_close(reproduced_two, analytic_two);

    let three = PolynomialSpace::<3>::try_new(3)?;
    let mut three_values = vec![0.0; three.term_count()];
    let [x, y, z] = [0.5, -1.25, 2.0];
    three.try_evaluate_values(Point::try_new([x, y, z])?, &mut three_values)?;
    let three_coefficients = [1.0, -2.0, 3.0, -4.0, 5.0, 6.0, -7.0, 8.0, 9.0, -10.0];
    let reproduced_three = dot(&three_coefficients, &three_values);
    let analytic_three = 1.0 - 2.0 * x + 3.0 * y - 4.0 * z + 5.0 * x * x + 6.0 * x * y
        - 7.0 * x * z
        + 8.0 * y * y
        + 9.0 * y * z
        - 10.0 * z * z;
    assert_close(reproduced_three, analytic_three);
    Ok(())
}

#[test]
fn invalid_orders_and_multi_indices_fail_structurally() {
    assert_eq!(
        PolynomialSpace::<1>::try_new(0),
        Err(PolynomialSpaceError::ZeroOrder)
    );
    assert_eq!(
        MultiIndex::<2>::try_new([usize::MAX, 1]),
        Err(PolynomialSpaceError::DegreeOverflow)
    );
    assert_eq!(
        PolynomialSpace::<2>::try_new(usize::MAX),
        Err(PolynomialSpaceError::TermCountOverflow {
            order: usize::MAX,
            dimension: 2,
        })
    );
    assert_eq!(
        PolynomialSpace::<1>::try_new(usize::MAX),
        Err(PolynomialSpaceError::AllocationFailed {
            term_count: usize::MAX,
        })
    );
}

#[test]
fn output_mismatch_and_nonfinite_results_leave_storage_unchanged() -> Result<(), Box<dyn Error>> {
    let space = PolynomialSpace::<1>::try_new(3)?;
    let point = Point::try_new([2.0])?;
    let mut short_values = [7.0; 2];
    assert_eq!(
        space.try_evaluate_values(point, &mut short_values),
        Err(PolynomialSpaceError::OutputLengthMismatch {
            output: PolynomialOutput::Values,
            expected: 3,
            actual: 2,
        })
    );
    assert_eq!(short_values, [7.0; 2]);

    let mut values = [7.0; 3];
    assert_eq!(
        space.try_evaluate_values(Point::try_new([f64::MAX])?, &mut values),
        Err(PolynomialSpaceError::NonFiniteEvaluation {
            term_index: 2,
            derivative_axis: None,
        })
    );
    assert_eq!(values, [7.0; 3]);

    let mut gradients = [[11.0]; 3];
    assert_eq!(
        space.try_evaluate_gradients(Point::try_new([f64::MAX])?, &mut gradients),
        Err(PolynomialSpaceError::NonFiniteEvaluation {
            term_index: 2,
            derivative_axis: Some(0),
        })
    );
    assert_eq!(gradients, [[11.0]; 3]);
    Ok(())
}

#[test]
fn nonfinite_coordinates_are_rejected_before_polynomial_evaluation() {
    assert!(matches!(
        Point::<3>::try_new([0.0, f64::NAN, 1.0]),
        Err(GeometryError::NonFiniteComponent { index: 1, value }) if value.is_nan()
    ));
}

#[test]
fn public_polynomial_values_are_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<MultiIndex<1>>();
    assert_send_sync::<PolynomialSpace<1>>();
    assert_send_sync::<PolynomialSpace<2>>();
    assert_send_sync::<PolynomialSpace<3>>();
}

fn dot(coefficients: &[f64], values: &[f64]) -> f64 {
    coefficients
        .iter()
        .zip(values)
        .map(|(coefficient, value)| coefficient * value)
        .sum()
}

fn assert_close(actual: f64, expected: f64) {
    let scale = expected.abs().max(1.0);
    assert!(
        (actual - expected).abs() <= 8.0 * f64::EPSILON * scale,
        "actual={actual:.17e}, expected={expected:.17e}"
    );
}
