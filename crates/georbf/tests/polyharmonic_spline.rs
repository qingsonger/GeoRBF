//! Independent truth and property tests for REQ-KERNEL-002.

use std::error::Error;

use georbf::{
    Dim, KernelArgument, KernelCalculusError, KernelDefiniteness, KernelDerivativeCapability,
    KernelDerivativeOrder, KernelSupport, Point, PolyharmonicSpline,
    PolyharmonicSplineConstructionError, PolyharmonicSplineEvaluationError, RadialDerivativeOrder,
    RadialExpansionCoefficient, RadialSeparation, SpatialKernelJet, SupportedDimension,
    SurfaceSpline, SurfaceSplineConstructionError,
};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

const ORDERS: [KernelDerivativeOrder; 4] = [
    KernelDerivativeOrder::Value,
    KernelDerivativeOrder::First,
    KernelDerivativeOrder::Second,
    KernelDerivativeOrder::Third,
];

fn assert_close(actual: f64, expected: f64, relative: f64) {
    let scale = expected.abs().max(1.0);
    assert!(
        (actual - expected).abs() <= relative * scale,
        "actual={actual:.17e}, expected={expected:.17e}, error={:.17e}, tolerance={:.17e}",
        (actual - expected).abs(),
        relative * scale,
    );
}

fn assert_same_bits(actual: f64, expected: f64) {
    assert_eq!(actual.to_bits(), expected.to_bits());
}

#[test]
fn metadata_classifies_power_dimension_order_and_center_limits() -> TestResult {
    let expected_sign_and_order = [
        (1, -0.5, 1, KernelDerivativeOrder::Value),
        (2, 0.25 * 0.5_f64.ln(), 2, KernelDerivativeOrder::First),
        (3, 0.125, 2, KernelDerivativeOrder::Second),
        (4, -0.0625 * 0.5_f64.ln(), 3, KernelDerivativeOrder::Third),
        (5, -0.03125, 3, KernelDerivativeOrder::Third),
        (6, 0.015_625 * 0.5_f64.ln(), 4, KernelDerivativeOrder::Third),
    ];

    for (power, expected_value, cpd_order, center_order) in expected_sign_and_order {
        let kernel = PolyharmonicSpline::try_new(power)?;
        let metadata = kernel.metadata();
        assert_eq!(kernel.power(), power);
        assert_eq!(kernel.cpd_order().get(), cpd_order);
        assert_eq!(metadata.name(), "polyharmonic_spline");
        assert_eq!(metadata.dimensions().flags(), [true, true, true]);
        assert_eq!(metadata.support(), KernelSupport::Global);
        assert!(metadata.parameters().is_empty());
        assert_eq!(
            metadata.derivatives().maximum_center_order(),
            Some(center_order)
        );
        assert_eq!(
            metadata.derivatives().maximum_away_order(),
            KernelDerivativeOrder::Third
        );
        assert!(matches!(
            metadata.definiteness(),
            KernelDefiniteness::ConditionallyPositiveDefinite { order }
                if order.get() == cpd_order
        ));
        assert_close(kernel.radial_value(0.5)?, expected_value, 8.0e-15);
    }
    Ok(())
}

#[test]
fn surface_splines_bind_sobolev_order_to_dimension_and_power() -> TestResult {
    let one = SurfaceSpline::<1>::try_new(1)?;
    let plane = SurfaceSpline::<2>::try_new(2)?;
    let volume = SurfaceSpline::<3>::try_new(2)?;
    let smoother_volume = SurfaceSpline::<3>::try_new(3)?;

    for (order, power, dimensions, kernel) in [
        (1, 1, [true, false, false], one.metadata()),
        (2, 2, [false, true, false], plane.metadata()),
        (2, 1, [false, false, true], volume.metadata()),
        (3, 3, [false, false, true], smoother_volume.metadata()),
    ] {
        assert_eq!(kernel.name(), "surface_spline");
        assert_eq!(kernel.dimensions().flags(), dimensions);
        assert_eq!(kernel.support(), KernelSupport::Global);
        assert!(kernel.parameters().is_empty());
        assert!(matches!(
            kernel.definiteness(),
            KernelDefiniteness::ConditionallyPositiveDefinite { order: actual }
                if actual.get() == order
        ));
        let expected_center = match power {
            1 => KernelDerivativeOrder::Value,
            2 => KernelDerivativeOrder::First,
            _ => KernelDerivativeOrder::Second,
        };
        assert_eq!(
            kernel.derivatives().maximum_center_order(),
            Some(expected_center)
        );
    }
    assert_eq!((one.order(), one.power()), (1, 1));
    assert_eq!((plane.order(), plane.power()), (2, 2));
    assert_eq!((volume.order(), volume.power()), (2, 1));
    assert_eq!((smoother_volume.order(), smoother_volume.power()), (3, 3));
    Ok(())
}

#[test]
fn radial_derivatives_match_embedded_eighty_digit_reference_values() -> TestResult {
    // Values were generated independently with 80-digit decimal arithmetic at
    // r=0.7. Each row is value, first, second, third, a=phi'/r, and
    // b=(phi''-a)/r for powers one through six.
    let reference = [
        [
            -0.7,
            -1.0,
            0.0,
            0.0,
            -1.428_571_428_571_428_6,
            2.040_816_326_530_612_3,
        ],
        [
            -0.174_770_722_529_978_87,
            0.200_655_078_485_774_67,
            2.286_650_112_122_535,
            2.857_142_857_142_857,
            0.286_650_112_122_535_26,
            2.857_142_857_142_857,
        ],
        [0.343, 1.47, 4.2, 6.0, 2.1, 3.0],
        [
            0.085_637_654_039_689_64,
            0.146_358_023_083_940_83,
            -1.332_751_329_640_253_6,
            -12.207_860_941_829_296,
            0.209_082_890_119_915_46,
            -2.202_620_313_943_098_7,
        ],
        [-0.168_07, -1.2005, -6.86, -29.4, -1.715, -7.35],
        [
            -0.041_962_450_479_447_93,
            -0.191_608_146_966_696_5,
            0.071_970_378_809_310_68,
            10.701_259_307_481_775,
            -0.273_725_924_238_137_87,
            0.493_851_861_496_355_06,
        ],
    ];
    let center = Point::try_new([0.0, 0.0])?;
    let separation = RadialSeparation::try_new(Point::try_new([0.7, 0.0])?, center)?;

    for (index, expected) in reference.iter().enumerate() {
        let kernel = PolyharmonicSpline::try_new(u16::try_from(index + 1)?)?;
        for (order, expected_derivative) in ORDERS.iter().copied().zip(expected[..4].iter()) {
            assert_close(
                kernel
                    .radial_derivative(0.7, order)?
                    .ok_or("away derivative missing")?,
                *expected_derivative,
                2.0e-14,
            );
        }
        let jet = kernel.radial_jet(separation)?;
        let expansion = jet
            .expansion_coefficients()
            .ok_or("D=2 expansion coefficients missing")?;
        assert_close(expansion.first_over_radius(), expected[4], 2.0e-14);
        assert_close(
            expansion.second_remainder_over_radius(),
            expected[5],
            2.0e-14,
        );
    }
    Ok(())
}

#[test]
fn derivative_scaling_preserves_representable_subnormal_results() -> TestResult {
    // For p=219 and r=2^-5, the odd-power second and third derivatives are
    // exact integer multiples of powers of two. Although r^(p-n) alone is
    // below the f64 subnormal range, multiplication by the derivative
    // coefficient brings the final result back into that range. The expected
    // bit patterns below follow by rounding those exact integer multiples to
    // units of 2^-1074; they do not use the production evaluator.
    let radius = 0.03125;
    let kernel = PolyharmonicSpline::try_new(219)?;
    assert_eq!(
        kernel
            .radial_derivative(radius, KernelDerivativeOrder::Second)?
            .ok_or("away second derivative missing")?
            .to_bits(),
        23
    );
    assert_eq!(
        kernel
            .radial_derivative(radius, KernelDerivativeOrder::Third)?
            .ok_or("away third derivative missing")?
            .to_bits(),
        0x27_853
    );

    let separation =
        RadialSeparation::try_new(Point::try_new([radius, 0.0])?, Point::try_new([0.0, 0.0])?)?;
    let jet = kernel.radial_jet(separation)?;
    let expansion = jet
        .expansion_coefficients()
        .ok_or("D=2 expansion coefficients missing")?;
    assert_eq!(expansion.first_over_radius().to_bits(), 0);
    assert_eq!(expansion.second_remainder_over_radius().to_bits(), 743);
    Ok(())
}

#[test]
fn logarithmic_derivative_scaling_preserves_representable_subnormal_results() -> TestResult {
    // At p=1090 and r=1/2, the bare powers in the third derivative and b
    // coefficient underflow even though the complete products are nonzero.
    // Independent 100-digit decimal evaluation followed by round-to-nearest,
    // ties-to-even in units of 2^-1074 gives the exact f64 bit patterns below.
    let radius = 0.5;
    let kernel = PolyharmonicSpline::try_new(1090)?;
    assert_eq!(
        kernel
            .radial_derivative(radius, KernelDerivativeOrder::Third)?
            .ok_or("away third derivative missing")?
            .to_bits(),
        0x8000_0000_0001_a928
    );

    let separation =
        RadialSeparation::try_new(Point::try_new([radius, 0.0])?, Point::try_new([0.0, 0.0])?)?;
    let jet = kernel.radial_jet(separation)?;
    let expansion = jet
        .expansion_coefficients()
        .ok_or("D=2 expansion coefficients missing")?;
    assert_eq!(
        expansion.second_remainder_over_radius().to_bits(),
        0x8000_0000_0000_0064
    );
    Ok(())
}

#[test]
fn even_power_reference_length_changes_only_the_cpd_polynomial_term() -> TestResult {
    // The second-difference stencil annihilates constants and linear terms.
    // For phi(r)=r^2 log(r), changing coordinate scale by c introduces a
    // c^2 log(c) r^2 polynomial term whose projected Gram energy is exactly
    // zero. The remaining energy must scale by c^2.
    let kernel = PolyharmonicSpline::try_new(2)?;
    let points = [0.0_f64, 1.0, 2.0];
    let weights = [1.0_f64, -2.0, 1.0];
    assert_same_bits(weights.iter().sum(), 0.0);
    assert_same_bits(
        points
            .iter()
            .zip(weights)
            .map(|(point, weight)| point * weight)
            .sum(),
        0.0,
    );

    let energy = |scale: f64| -> Result<f64, PolyharmonicSplineEvaluationError> {
        let mut result = 0.0;
        for (row, x) in points.iter().enumerate() {
            for (column, y) in points.iter().enumerate() {
                result +=
                    weights[row] * weights[column] * kernel.radial_value(scale * (x - y).abs())?;
            }
        }
        Ok(result)
    };
    let baseline = energy(1.0)?;
    let scale = 3.5;
    assert!(baseline > 0.0);
    assert_close(energy(scale)?, scale * scale * baseline, 2.0e-15);
    Ok(())
}

#[test]
fn analytic_radial_derivatives_match_independent_finite_differences() -> TestResult {
    for power in 1..=6 {
        let kernel = PolyharmonicSpline::try_new(power)?;
        for radius in [0.35_f64, 0.9, 1.7] {
            let h = 1.0e-3 * radius.max(1.0);
            let f = |r| kernel.radial_value(r).unwrap_or(f64::NAN);
            let fm2 = f(radius - 2.0 * h);
            let fm1 = f(radius - h);
            let f0 = f(radius);
            let fp1 = f(radius + h);
            let fp2 = f(radius + 2.0 * h);
            let first = (fm2 - 8.0 * fm1 + 8.0 * fp1 - fp2) / (12.0 * h);
            let second = (-fp2 + 16.0 * fp1 - 30.0 * f0 + 16.0 * fm1 - fm2) / (12.0 * h * h);
            let third = (-fm2 + 2.0 * fm1 - 2.0 * fp1 + fp2) / (2.0 * h.powi(3));

            assert_close(
                kernel
                    .radial_derivative(radius, KernelDerivativeOrder::First)?
                    .ok_or("first derivative missing")?,
                first,
                2.0e-8,
            );
            assert_close(
                kernel
                    .radial_derivative(radius, KernelDerivativeOrder::Second)?
                    .ok_or("second derivative missing")?,
                second,
                3.0e-7,
            );
            assert_close(
                kernel
                    .radial_derivative(radius, KernelDerivativeOrder::Third)?
                    .ok_or("third derivative missing")?,
                third,
                3.0e-5,
            );
        }
    }
    Ok(())
}

#[test]
fn center_limits_are_exact_and_higher_center_jets_are_not_fabricated() -> TestResult {
    let center =
        RadialSeparation::try_new(Point::try_new([1.0, -2.0])?, Point::try_new([1.0, -2.0])?)?;

    for power in 1..=5 {
        let kernel = PolyharmonicSpline::try_new(power)?;
        let maximum = (power - 1).min(3) as usize;
        for (index, order) in ORDERS.iter().copied().enumerate() {
            let actual = kernel.radial_derivative(0.0, order)?;
            if index <= maximum {
                assert_eq!(actual, Some(0.0));
                assert_eq!(
                    kernel.metadata().derivatives().capability(order),
                    KernelDerivativeCapability::SupportedEverywhere
                );
            } else {
                assert_eq!(actual, None);
                assert_eq!(
                    kernel.metadata().derivatives().capability(order),
                    KernelDerivativeCapability::SupportedAwayFromCenters
                );
            }
        }

        if power < 4 {
            assert!(matches!(
                kernel.radial_jet(center),
                Err(PolyharmonicSplineEvaluationError::CenterJetUnsupported {
                    power: actual,
                    ..
                }) if actual == power
            ));
        } else {
            let spatial = SpatialKernelJet::try_new(center, kernel.radial_jet(center)?)?;
            assert_same_bits(spatial.value(), 0.0);
            for value in spatial.first_derivative(KernelArgument::Query) {
                assert_same_bits(value, 0.0);
            }
            for row in spatial.second_derivative([KernelArgument::Query; 2]) {
                for value in row {
                    assert_same_bits(value, 0.0);
                }
            }
            for matrix in spatial.third_derivative([KernelArgument::Query; 3]) {
                for row in matrix {
                    for value in row {
                        assert_same_bits(value, 0.0);
                    }
                }
            }
        }
    }
    Ok(())
}

#[test]
#[allow(clippy::needless_range_loop)]
fn shared_cartesian_calculus_preserves_exchange_signs_and_tensor_symmetry() -> TestResult {
    let query = Point::try_new([0.7, -0.4, 1.1])?;
    let center = Point::try_new([-0.2, 0.3, 0.5])?;
    let separation = RadialSeparation::try_new(query, center)?;
    let kernel = PolyharmonicSpline::try_new(5)?;
    let spatial = SpatialKernelJet::try_new(separation, kernel.radial_jet(separation)?)?;

    let query_gradient = spatial.first_derivative(KernelArgument::Query);
    let center_gradient = spatial.first_derivative(KernelArgument::Center);
    for axis in 0..3 {
        assert_same_bits(center_gradient[axis], -query_gradient[axis]);
    }

    let query_hessian = spatial.second_derivative([KernelArgument::Query; 2]);
    let mixed_hessian = spatial.second_derivative([KernelArgument::Query, KernelArgument::Center]);
    for row in 0..3 {
        for column in 0..3 {
            assert_same_bits(query_hessian[row][column], query_hessian[column][row]);
            assert_same_bits(mixed_hessian[row][column], -query_hessian[row][column]);
        }
    }

    let third = spatial.third_derivative([KernelArgument::Query; 3]);
    for i in 0..3 {
        for j in 0..3 {
            for k in 0..3 {
                assert_same_bits(third[i][j][k], third[j][i][k]);
                assert_same_bits(third[i][j][k], third[k][j][i]);
            }
        }
    }
    Ok(())
}

#[test]
fn projected_random_gram_energies_are_positive_in_every_dimension() -> TestResult {
    for seed in 1..=4 {
        for power in 1..=6 {
            let kernel = PolyharmonicSpline::try_new(power)?;
            let order = kernel.cpd_order().get();
            assert_projected_positive::<1, _>(order, seed, |radius| kernel.radial_value(radius))?;
            assert_projected_positive::<2, _>(order, seed, |radius| kernel.radial_value(radius))?;
            assert_projected_positive::<3, _>(order, seed, |radius| kernel.radial_value(radius))?;
        }
    }

    for seed in 20..=23 {
        let line = SurfaceSpline::<1>::try_new(2)?;
        let plane = SurfaceSpline::<2>::try_new(2)?;
        let volume = SurfaceSpline::<3>::try_new(2)?;
        assert_projected_positive::<1, _>(2, seed, |radius| line.radial_value(radius))?;
        assert_projected_positive::<2, _>(2, seed, |radius| plane.radial_value(radius))?;
        assert_projected_positive::<3, _>(2, seed, |radius| volume.radial_value(radius))?;
    }
    Ok(())
}

#[test]
fn construction_and_numeric_pathologies_return_structured_errors() -> TestResult {
    assert!(matches!(
        PolyharmonicSpline::try_new(0),
        Err(PolyharmonicSplineConstructionError::ZeroPower)
    ));
    assert!(matches!(
        SurfaceSpline::<1>::try_new(0),
        Err(SurfaceSplineConstructionError::OrderTooLow {
            order: 0,
            dimension: 1
        })
    ));
    assert!(matches!(
        SurfaceSpline::<2>::try_new(1),
        Err(SurfaceSplineConstructionError::OrderTooLow {
            order: 1,
            dimension: 2
        })
    ));
    assert!(matches!(
        SurfaceSpline::<3>::try_new(1),
        Err(SurfaceSplineConstructionError::OrderTooLow {
            order: 1,
            dimension: 3
        })
    ));
    assert!(matches!(
        SurfaceSpline::<3>::try_new(u16::MAX),
        Err(SurfaceSplineConstructionError::OrderOverflow { order: u16::MAX })
    ));

    let kernel = PolyharmonicSpline::try_new(5)?;
    assert!(matches!(
        kernel.radial_value(-1.0),
        Err(PolyharmonicSplineEvaluationError::NegativeRadius { radius: -1.0 })
    ));
    for radius in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(matches!(
            kernel.radial_value(radius),
            Err(PolyharmonicSplineEvaluationError::NonFiniteRadius { .. })
        ));
    }
    assert!(matches!(
        kernel.radial_value(f64::MAX),
        Err(PolyharmonicSplineEvaluationError::KernelCalculus(
            KernelCalculusError::NonFiniteRadialDerivative {
                order: RadialDerivativeOrder::Value,
                ..
            }
        ))
    ));

    let smallest = 1.0e-200;
    let separation = RadialSeparation::try_new(
        Point::try_new([smallest, 0.0])?,
        Point::try_new([0.0, 0.0])?,
    )?;
    assert!(matches!(
        PolyharmonicSpline::try_new(1)?.radial_jet(separation),
        Err(PolyharmonicSplineEvaluationError::KernelCalculus(
            KernelCalculusError::NonFiniteRadialExpansionCoefficient {
                coefficient: RadialExpansionCoefficient::SecondRemainderOverRadius,
                ..
            }
        ))
    ));
    Ok(())
}

#[test]
fn kernel_values_are_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<PolyharmonicSpline>();
    assert_send_sync::<SurfaceSpline<1>>();
    assert_send_sync::<SurfaceSpline<2>>();
    assert_send_sync::<SurfaceSpline<3>>();
    assert_send_sync::<PolyharmonicSplineConstructionError>();
    assert_send_sync::<SurfaceSplineConstructionError>();
    assert_send_sync::<PolyharmonicSplineEvaluationError>();
}

fn assert_projected_positive<const D: usize, F>(
    order: usize,
    seed: u64,
    radial_value: F,
) -> TestResult
where
    Dim<D>: SupportedDimension,
    F: Fn(f64) -> Result<f64, PolyharmonicSplineEvaluationError>,
{
    let exponents = complete_monomial_exponents::<D>(order - 1);
    let point_count = exponents.len() + 1;
    let points = deterministic_points::<D>(point_count, seed);
    let mut system = vec![vec![0.0; exponents.len() + 1]; exponents.len()];
    for (row, exponent) in exponents.iter().enumerate() {
        for column in 0..exponents.len() {
            system[row][column] = monomial(points[column], *exponent);
        }
        system[row][exponents.len()] = -monomial(points[point_count - 1], *exponent);
    }
    let mut weights = solve_square_system(system);
    weights.push(1.0);

    for exponent in &exponents {
        let moment = points
            .iter()
            .zip(weights.iter())
            .map(|(point, weight)| weight * monomial(*point, *exponent))
            .sum::<f64>();
        assert!(moment.abs() <= 2.0e-8, "polynomial moment {moment:.17e}");
    }

    let mut energy = 0.0;
    let mut absolute_scale = 0.0;
    for row in 0..point_count {
        for column in 0..point_count {
            let radius = euclidean_distance(points[row], points[column]);
            let term = weights[row] * weights[column] * radial_value(radius)?;
            energy += term;
            absolute_scale += term.abs();
        }
    }
    assert!(
        energy > 1.0e-10 * absolute_scale.max(1.0),
        "non-positive projected energy {energy:.17e} at scale {absolute_scale:.17e} for D={D}, order={order}, seed={seed}"
    );
    Ok(())
}

fn complete_monomial_exponents<const D: usize>(maximum_degree: usize) -> Vec<[u16; D]> {
    fn recurse<const D: usize>(
        axis: usize,
        remaining: u16,
        current: &mut [u16; D],
        output: &mut Vec<[u16; D]>,
    ) {
        if axis == D {
            output.push(*current);
            return;
        }
        for exponent in 0..=remaining {
            current[axis] = exponent;
            recurse(axis + 1, remaining - exponent, current, output);
        }
    }

    let mut output = Vec::new();
    let mut current = [0; D];
    recurse(
        0,
        u16::try_from(maximum_degree).unwrap_or(0),
        &mut current,
        &mut output,
    );
    output
}

fn deterministic_points<const D: usize>(count: usize, seed: u64) -> Vec<[f64; D]> {
    let mut state = seed ^ ((D as u64) << 48);
    (0..count)
        .map(|point_index| {
            std::array::from_fn(|axis| {
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                let sample = u32::try_from(state >> 32).unwrap_or(0);
                let unit = f64::from(sample) / 4_294_967_296.0;
                let ordinal = u32::try_from(point_index + axis).map_or(0.0, f64::from);
                1.8 * unit - 0.9 + 0.013 * ordinal
            })
        })
        .collect()
}

fn monomial<const D: usize>(point: [f64; D], exponent: [u16; D]) -> f64 {
    point
        .iter()
        .zip(exponent.iter())
        .map(|(coordinate, power)| coordinate.powi(i32::from(*power)))
        .product()
}

fn euclidean_distance<const D: usize>(first: [f64; D], second: [f64; D]) -> f64 {
    first
        .iter()
        .zip(second.iter())
        .map(|(left, right)| (left - right).powi(2))
        .sum::<f64>()
        .sqrt()
}

fn solve_square_system(mut augmented: Vec<Vec<f64>>) -> Vec<f64> {
    let size = augmented.len();
    for pivot in 0..size {
        let row = (pivot..size)
            .max_by(|left, right| {
                augmented[*left][pivot]
                    .abs()
                    .total_cmp(&augmented[*right][pivot].abs())
            })
            .unwrap_or(pivot);
        augmented.swap(pivot, row);
        assert!(augmented[pivot][pivot].abs() > 1.0e-12);
        let divisor = augmented[pivot][pivot];
        for value in &mut augmented[pivot][pivot..=size] {
            *value /= divisor;
        }
        let pivot_row = augmented[pivot].clone();
        for (row, values) in augmented.iter_mut().enumerate() {
            if row == pivot {
                continue;
            }
            let factor = values[pivot];
            for (value, pivot_value) in values[pivot..=size]
                .iter_mut()
                .zip(&pivot_row[pivot..=size])
            {
                *value -= factor * pivot_value;
            }
        }
    }
    augmented.into_iter().map(|row| row[size]).collect()
}
