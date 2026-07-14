//! Independent truth and property tests for REQ-KERNEL-004.
#![allow(clippy::excessive_precision, clippy::unreadable_literal)]

use georbf::{
    KernelArgument, KernelDefiniteness, KernelDerivativeCapability, KernelDerivativeOrder,
    KernelParameterConstraint, KernelParameterUnit, KernelParameterValueError, KernelSupport,
    Point, RadialSeparation, SpatialKernelJet, Wendland, WendlandConstructionError,
    WendlandEvaluationError, WendlandSmoothness,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

const ORDERS: [KernelDerivativeOrder; 4] = [
    KernelDerivativeOrder::Value,
    KernelDerivativeOrder::First,
    KernelDerivativeOrder::Second,
    KernelDerivativeOrder::Third,
];

#[test]
fn metadata_declares_exact_support_classification_and_capabilities() -> TestResult {
    for (smoothness, name, center_order) in [
        (
            WendlandSmoothness::C2,
            "wendland_c2",
            KernelDerivativeOrder::Second,
        ),
        (
            WendlandSmoothness::C4,
            "wendland_c4",
            KernelDerivativeOrder::Third,
        ),
        (
            WendlandSmoothness::C6,
            "wendland_c6",
            KernelDerivativeOrder::Third,
        ),
    ] {
        let kernel = Wendland::try_new(smoothness, 2.5)?;
        let metadata = kernel.metadata();
        assert_eq!(kernel.smoothness(), smoothness);
        assert_same_bits(kernel.support_radius(), 2.5);
        assert_eq!(metadata.name(), name);
        assert_eq!(
            metadata.definiteness(),
            KernelDefiniteness::StrictlyPositiveDefinite
        );
        assert_eq!(metadata.dimensions().flags(), [true, true, true]);
        assert_eq!(
            metadata.support(),
            KernelSupport::Compact {
                radius_parameter: "support_radius"
            }
        );
        assert_eq!(
            metadata.derivatives().maximum_away_order(),
            KernelDerivativeOrder::Third
        );
        assert_eq!(
            metadata.derivatives().maximum_center_order(),
            Some(center_order)
        );
        assert_eq!(metadata.parameters().len(), 1);
        let parameter = metadata.parameters()[0];
        assert_eq!(parameter.name(), "support_radius");
        assert_eq!(parameter.unit(), KernelParameterUnit::CoordinateLength);
        assert_eq!(parameter.constraint(), KernelParameterConstraint::Positive);
        assert!(!parameter.description().is_empty());
    }
    Ok(())
}

#[test]
fn radial_terms_match_exact_rational_reference_values() -> TestResult {
    // q = 2/5, rho = 5/2. These references were derived independently with
    // exact rational arithmetic from the published factored polynomials.
    let radius = 1.0;
    let support_radius = 2.5;
    let separation =
        RadialSeparation::try_new(Point::try_new([radius, 0.0])?, Point::try_new([0.0, 0.0])?)?;
    let references = [
        (
            WendlandSmoothness::C2,
            [
                3.369599999999999817568152593594277278e-1,
                -6.912000000000000365929508916451595724e-1,
                6.912000000000000365929508916451595724e-1,
                9.215999999999999747757328805164434016e-1,
                -6.912000000000000365929508916451595724e-1,
                1.382400000000000073185901783290319145,
            ],
        ),
        (
            WendlandSmoothness::C4,
            [
                2.457216000000000122316379247422446497e-1,
                -6.967295999999999489560309484659228474e-1,
                1.161216000000000025949020709958858788,
                6.193151999999999546275830653030425310e-1,
                -6.967295999999999489560309484659228474e-1,
                1.857945600000000085927354120940435678,
            ],
        ),
        (
            WendlandSmoothness::C6,
            [
                1.721270476800000059913742234130040742e-1,
                -6.266983219199999677329060432384721935e-1,
                1.517477068800000106207903627364430577,
                -6.306398208000000282780206362076569349e-1,
                -6.266983219199999677329060432384721935e-1,
                2.144175390720000073940809670602902770,
            ],
        ),
    ];

    for (smoothness, expected) in references {
        let kernel = Wendland::try_new(smoothness, support_radius)?;
        let jet = kernel.radial_jet(separation)?;
        for (order, expected) in ORDERS.into_iter().zip(expected) {
            assert_close(
                kernel
                    .radial_derivative(radius, order)?
                    .ok_or("away derivative missing")?,
                expected,
                5.0e-15,
            );
        }
        let expansion = jet
            .expansion_coefficients()
            .ok_or("missing expansion coefficients")?;
        assert_close(expansion.first_over_radius(), expected[4], 5.0e-15);
        assert_close(
            expansion.second_remainder_over_radius(),
            expected[5],
            5.0e-15,
        );
    }
    Ok(())
}

#[test]
fn support_boundary_and_exterior_are_exact_positive_zero() -> TestResult {
    let support_radius = 1.75;
    for smoothness in [
        WendlandSmoothness::C2,
        WendlandSmoothness::C4,
        WendlandSmoothness::C6,
    ] {
        let kernel = Wendland::try_new(smoothness, support_radius)?;
        for radius in [support_radius, support_radius + 1.0, f64::MAX] {
            assert_same_bits(kernel.radial_value(radius)?, 0.0);
            for order in ORDERS {
                assert_same_bits(
                    kernel
                        .radial_derivative(radius, order)?
                        .ok_or("positive-radius derivative missing")?,
                    0.0,
                );
            }
        }

        let boundary = RadialSeparation::try_new(
            Point::try_new([support_radius, 0.0, 0.0])?,
            Point::try_new([0.0, 0.0, 0.0])?,
        )?;
        let jet = kernel.radial_jet(boundary)?;
        assert_same_bits(jet.value(), 0.0);
        assert_same_bits(jet.first_derivative(), 0.0);
        assert_same_bits(jet.second_derivative(), 0.0);
        assert_same_bits(jet.third_derivative(), 0.0);
        let expansion = jet.expansion_coefficients().ok_or("missing expansion")?;
        assert_same_bits(expansion.first_over_radius(), 0.0);
        assert_same_bits(expansion.second_remainder_over_radius(), 0.0);
    }
    Ok(())
}

#[test]
fn interior_limits_match_zero_extension_through_third_order() -> TestResult {
    let contracts = [
        (
            WendlandSmoothness::C2,
            [4, 3, 2, 1],
            [5.0, -20.0, 60.0, -120.0],
        ),
        (
            WendlandSmoothness::C4,
            [6, 5, 4, 3],
            [56.0 / 3.0, -112.0, 560.0, -2240.0],
        ),
        (
            WendlandSmoothness::C6,
            [8, 7, 6, 5],
            [66.0, -528.0, 3696.0, -22176.0],
        ),
    ];

    for (smoothness, boundary_powers, signed_limits) in contracts {
        let kernel = Wendland::try_new(smoothness, 1.0)?;
        let mut previous_magnitudes = [f64::INFINITY; 4];
        let mut previous_limit_errors = [f64::INFINITY; 4];
        for binary_exponent in [4, 8, 12] {
            let t = 2.0_f64.powi(-binary_exponent);
            let radius = 1.0 - t;
            for index in 0..ORDERS.len() {
                let actual = kernel
                    .radial_derivative(radius, ORDERS[index])?
                    .ok_or("just-inside derivative missing")?;
                assert!(actual != 0.0 && actual.abs() < previous_magnitudes[index]);
                previous_magnitudes[index] = actual.abs();

                let normalized = actual / t.powi(boundary_powers[index]);
                let error = (normalized - signed_limits[index]).abs();
                assert!(
                    error < previous_limit_errors[index],
                    "{smoothness:?} order {:?} boundary quotient did not converge: error {error:.17e}",
                    ORDERS[index]
                );
                previous_limit_errors[index] = error;
                if binary_exponent == 12 {
                    assert_close(normalized, signed_limits[index], 5.0e-3);
                }
            }
        }
    }
    Ok(())
}

#[test]
fn analytic_derivatives_match_independent_interior_finite_differences() -> TestResult {
    for smoothness in [
        WendlandSmoothness::C2,
        WendlandSmoothness::C4,
        WendlandSmoothness::C6,
    ] {
        let kernel = Wendland::try_new(smoothness, 2.3)?;
        for radius in [0.31, 0.87, 1.61] {
            assert_finite_differences(
                radius,
                |r| kernel.radial_value(r),
                |r, order| kernel.radial_derivative(r, order),
            )?;
        }
    }
    Ok(())
}

#[test]
fn center_limits_and_c2_capability_are_not_overpromised() -> TestResult {
    let center =
        RadialSeparation::try_new(Point::try_new([0.4, -0.7])?, Point::try_new([0.4, -0.7])?)?;
    let c2 = Wendland::try_new(WendlandSmoothness::C2, 2.0)?;
    assert_same_bits(c2.radial_value(0.0)?, 1.0);
    assert_same_bits(
        c2.radial_derivative(0.0, KernelDerivativeOrder::First)?
            .ok_or("C2 center first derivative missing")?,
        0.0,
    );
    assert_close(
        c2.radial_derivative(0.0, KernelDerivativeOrder::Second)?
            .ok_or("C2 center second derivative missing")?,
        -5.0,
        2.0e-15,
    );
    assert_eq!(
        c2.radial_derivative(0.0, KernelDerivativeOrder::Third)?,
        None
    );
    assert_eq!(
        c2.metadata()
            .derivatives()
            .capability(KernelDerivativeOrder::Third),
        KernelDerivativeCapability::SupportedAwayFromCenters
    );
    assert!(matches!(
        c2.radial_jet(center),
        Err(WendlandEvaluationError::CenterJetUnsupported {
            smoothness: WendlandSmoothness::C2,
            maximum_center_order: KernelDerivativeOrder::Second,
        })
    ));

    for (smoothness, center_second) in [
        (WendlandSmoothness::C4, -14.0 / 3.0),
        (WendlandSmoothness::C6, -5.5),
    ] {
        let kernel = Wendland::try_new(smoothness, 2.0)?;
        let jet = kernel.radial_jet(center)?;
        assert_same_bits(jet.value(), 1.0);
        assert_same_bits(jet.first_derivative(), 0.0);
        assert_close(jet.second_derivative(), center_second, 2.0e-15);
        assert_same_bits(jet.third_derivative(), 0.0);
    }
    Ok(())
}

#[test]
fn deterministic_gram_matrices_are_spd_in_every_supported_dimension() -> TestResult {
    for smoothness in [
        WendlandSmoothness::C2,
        WendlandSmoothness::C4,
        WendlandSmoothness::C6,
    ] {
        let kernel = Wendland::try_new(smoothness, 1.4)?;
        for seed in 1..=8 {
            assert_positive_energy::<1, _>(seed, |radius| kernel.radial_value(radius))?;
            assert_positive_energy::<2, _>(seed + 10, |radius| kernel.radial_value(radius))?;
            assert_positive_energy::<3, _>(seed + 20, |radius| kernel.radial_value(radius))?;
        }
    }
    Ok(())
}

#[test]
fn coordinate_scaling_preserves_values_and_rescales_derivatives() -> TestResult {
    let radius = 0.8;
    let support_radius = 1.3;
    let coordinate_scale = 10.0;
    for smoothness in [
        WendlandSmoothness::C2,
        WendlandSmoothness::C4,
        WendlandSmoothness::C6,
    ] {
        let kernel = Wendland::try_new(smoothness, support_radius)?;
        let scaled = Wendland::try_new(smoothness, support_radius * coordinate_scale)?;
        for (order, power) in ORDERS.into_iter().zip([0, 1, 2, 3]) {
            let original = kernel
                .radial_derivative(radius, order)?
                .ok_or("original derivative missing")?;
            let transformed = scaled
                .radial_derivative(radius * coordinate_scale, order)?
                .ok_or("scaled derivative missing")?;
            assert_close(
                transformed,
                original / coordinate_scale.powi(power),
                8.0e-15,
            );
        }
    }
    Ok(())
}

#[test]
#[allow(clippy::needless_range_loop)]
fn cartesian_calculus_preserves_exchange_signs_and_tensor_symmetry() -> TestResult {
    let separation = RadialSeparation::try_new(
        Point::try_new([0.7, -0.4, 1.1])?,
        Point::try_new([-0.2, 0.3, 0.5])?,
    )?;
    for smoothness in [
        WendlandSmoothness::C2,
        WendlandSmoothness::C4,
        WendlandSmoothness::C6,
    ] {
        let radial = Wendland::try_new(smoothness, 2.0)?.radial_jet(separation)?;
        let spatial = SpatialKernelJet::try_new(separation, radial)?;
        let query_gradient = spatial.first_derivative(KernelArgument::Query);
        let center_gradient = spatial.first_derivative(KernelArgument::Center);
        for axis in 0..3 {
            assert_same_bits(center_gradient[axis], -query_gradient[axis]);
        }
        let hessian = spatial.second_derivative([KernelArgument::Query; 2]);
        let mixed = spatial.second_derivative([KernelArgument::Query, KernelArgument::Center]);
        for row in 0..3 {
            for column in 0..3 {
                assert_same_bits(hessian[row][column], hessian[column][row]);
                assert_same_bits(mixed[row][column], -hessian[row][column]);
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
    }
    Ok(())
}

#[test]
fn extreme_and_invalid_inputs_follow_structured_paths() -> TestResult {
    for support_radius in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(matches!(
            Wendland::try_new(WendlandSmoothness::C2, support_radius),
            Err(WendlandConstructionError::InvalidSupportRadius(
                KernelParameterValueError::NonFinite { .. }
                    | KernelParameterValueError::ViolatesConstraint { .. }
            ))
        ));
    }
    assert!(matches!(
        Wendland::try_new(WendlandSmoothness::C6, f64::MIN_POSITIVE),
        Err(WendlandConstructionError::NonRepresentableDerivativeScale { .. })
    ));

    let kernel = Wendland::try_new(WendlandSmoothness::C6, 1.0)?;
    assert!(matches!(
        kernel.radial_value(-1.0),
        Err(WendlandEvaluationError::NegativeRadius { radius: -1.0 })
    ));
    for radius in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(matches!(
            kernel.radial_value(radius),
            Err(WendlandEvaluationError::NonFiniteRadius { .. })
        ));
    }

    let just_inside = f64::from_bits(1.0_f64.to_bits() - 1);
    assert_same_bits(kernel.radial_value(1.0)?, 0.0);
    let third = kernel
        .radial_derivative(just_inside, KernelDerivativeOrder::Third)?
        .ok_or("just-inside third derivative missing")?;
    assert!(third.is_finite() && third != 0.0);

    let huge = Wendland::try_new(WendlandSmoothness::C4, f64::MAX)?;
    assert_same_bits(huge.radial_value(1.0)?, 1.0);
    Ok(())
}

#[test]
fn combined_log_recovers_representable_subnormal_scale_products() -> TestResult {
    // Independent 200-digit decimal evaluation using the exact f64 radius,
    // followed by round-to-nearest in units of 2^-1074, gives four negative
    // subnormal units for every center Hessian coefficient below. The stored
    // inverse squared has already rounded to zero, so a direct scaled product
    // cannot produce this result.
    let second_order_radius = 1.0e162_f64;
    let second_inverse = second_order_radius.recip();
    assert!(second_inverse > 0.0);
    assert_same_bits(second_inverse * second_inverse, 0.0);
    for smoothness in [
        WendlandSmoothness::C2,
        WendlandSmoothness::C4,
        WendlandSmoothness::C6,
    ] {
        let kernel = Wendland::try_new(smoothness, second_order_radius)?;
        let second = kernel
            .radial_derivative(0.0, KernelDerivativeOrder::Second)?
            .ok_or("center second derivative missing")?;
        assert_eq!(second.to_bits(), 0x8000_0000_0000_0004);
    }

    // At q=1/4, independent exact-rational polynomial evaluation plus
    // 200-digit decimal scaling by the exact f64 radius gives the bit patterns
    // below for phi''' and b. The stored inverse cubed is zero, making these
    // nonzero subnormal results direct evidence for complete-product recovery.
    let third_order_radius = 1.0e108_f64;
    let third_inverse = third_order_radius.recip();
    let third_inverse_squared = third_inverse * third_inverse;
    assert!(third_inverse_squared > 0.0);
    assert_same_bits(third_inverse_squared * third_inverse, 0.0);
    let radius = third_order_radius * 0.25;
    let separation =
        RadialSeparation::try_new(Point::try_new([radius, 0.0])?, Point::try_new([0.0, 0.0])?)?;
    for (smoothness, third_bits, expansion_bits) in [
        (WendlandSmoothness::C2, 0x9, 0x7),
        (WendlandSmoothness::C4, 0xf, 0x9),
        (WendlandSmoothness::C6, 0x13, 0xc),
    ] {
        let kernel = Wendland::try_new(smoothness, third_order_radius)?;
        let third = kernel
            .radial_derivative(radius, KernelDerivativeOrder::Third)?
            .ok_or("interior third derivative missing")?;
        assert_eq!(third.to_bits(), third_bits);
        let expansion = kernel
            .radial_jet(separation)?
            .expansion_coefficients()
            .copied()
            .ok_or("stable expansion coefficients missing")?;
        assert_eq!(
            expansion.second_remainder_over_radius().to_bits(),
            expansion_bits
        );
    }
    Ok(())
}

#[test]
fn public_wendland_values_are_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<Wendland>();
    assert_send_sync::<WendlandSmoothness>();
    assert_send_sync::<WendlandConstructionError>();
    assert_send_sync::<WendlandEvaluationError>();
}

fn assert_finite_differences<V, D>(radius: f64, value: V, derivative: D) -> TestResult
where
    V: Fn(f64) -> Result<f64, WendlandEvaluationError>,
    D: Fn(f64, KernelDerivativeOrder) -> Result<Option<f64>, WendlandEvaluationError>,
{
    let h = 5.0e-4;
    let fm2 = value(radius - 2.0 * h)?;
    let fm1 = value(radius - h)?;
    let f0 = value(radius)?;
    let fp1 = value(radius + h)?;
    let fp2 = value(radius + 2.0 * h)?;
    let approximations = [
        f0,
        (fm2 - 8.0 * fm1 + 8.0 * fp1 - fp2) / (12.0 * h),
        (-fp2 + 16.0 * fp1 - 30.0 * f0 + 16.0 * fm1 - fm2) / (12.0 * h * h),
        (-fm2 + 2.0 * fm1 - 2.0 * fp1 + fp2) / (2.0 * h.powi(3)),
    ];
    let tolerances = [2.0e-15, 2.0e-9, 8.0e-8, 8.0e-5];
    for ((order, approximation), tolerance) in
        ORDERS.into_iter().zip(approximations).zip(tolerances)
    {
        let actual = derivative(radius, order)?.ok_or("away derivative missing")?;
        let scale = actual.abs().max(approximation.abs()).max(1.0);
        assert!(
            (actual - approximation).abs() <= tolerance * scale,
            "finite-difference mismatch at r={radius}: actual {actual:.17e}, approximation {approximation:.17e}, tolerance {tolerance:.3e}"
        );
    }
    Ok(())
}

fn assert_positive_energy<const D: usize, F>(seed: u64, radial_value: F) -> TestResult
where
    F: Fn(f64) -> Result<f64, WendlandEvaluationError>,
{
    let points = deterministic_points::<D>(7, seed);
    let mut gram = vec![vec![0.0; points.len()]; points.len()];
    for row in 0..points.len() {
        for column in 0..points.len() {
            gram[row][column] = radial_value(distance(points[row], points[column]))?;
        }
    }
    assert_strictly_positive_definite(&gram, D, seed);

    let weights = deterministic_weights(seed);
    let mut energy = 0.0;
    let mut scale = 0.0;
    for row in 0..points.len() {
        for column in 0..points.len() {
            let term = weights[row] * weights[column] * gram[row][column];
            energy += term;
            scale += term.abs();
        }
    }
    assert!(
        energy > 1.0e-11 * scale.max(1.0),
        "non-positive energy {energy:.17e} at scale {scale:.17e}, D={D}, seed={seed}"
    );
    Ok(())
}

fn assert_strictly_positive_definite(matrix: &[Vec<f64>], dimension: usize, seed: u64) {
    let size = matrix.len();
    assert!(size > 0 && matrix.iter().all(|row| row.len() == size));
    let scale = matrix
        .iter()
        .flatten()
        .fold(0.0_f64, |maximum, value| maximum.max(value.abs()));
    let symmetry_tolerance = 32.0 * f64::EPSILON * scale.max(f64::MIN_POSITIVE);
    let pivot_floor = 256.0 * f64::EPSILON * scale.max(f64::MIN_POSITIVE);
    let mut lower = vec![vec![0.0; size]; size];
    for row in 0..size {
        for column in 0..=row {
            assert!(
                (matrix[row][column] - matrix[column][row]).abs() <= symmetry_tolerance,
                "asymmetric Gram entry at D={dimension}, seed={seed}, row={row}, column={column}"
            );
            let mut residual = matrix[row][column];
            for (row_term, column_term) in lower[row][..column].iter().zip(&lower[column][..column])
            {
                residual -= row_term * column_term;
            }
            if row == column {
                assert!(
                    residual.is_finite() && residual > pivot_floor,
                    "non-positive Cholesky pivot {residual:.17e} at D={dimension}, seed={seed}, row={row}"
                );
                lower[row][column] = residual.sqrt();
            } else {
                lower[row][column] = residual / lower[column][column];
                assert!(lower[row][column].is_finite());
            }
        }
    }
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
                1.6 * unit - 0.8 + 0.013 * ordinal
            })
        })
        .collect()
}

fn deterministic_weights(seed: u64) -> [f64; 7] {
    std::array::from_fn(|index| {
        let phase = f64::from(u32::try_from(seed + index as u64).unwrap_or(0));
        let ordinal = u32::try_from(index).map_or(0.0, f64::from);
        let one_based = u32::try_from(index + 1).map_or(0.0, f64::from);
        (0.73 * phase + 0.29 * ordinal).sin() + 0.13 * one_based
    })
}

fn distance<const D: usize>(first: [f64; D], second: [f64; D]) -> f64 {
    first
        .iter()
        .zip(second)
        .map(|(left, right)| (left - right).powi(2))
        .sum::<f64>()
        .sqrt()
}

fn assert_close(actual: f64, expected: f64, relative_tolerance: f64) {
    let scale = actual.abs().max(expected.abs()).max(f64::MIN_POSITIVE);
    assert!(
        (actual - expected).abs() <= relative_tolerance * scale,
        "actual {actual:.17e}, expected {expected:.17e}, relative error {:.3e}",
        (actual - expected).abs() / scale
    );
}

fn assert_same_bits(actual: f64, expected: f64) {
    assert_eq!(actual.to_bits(), expected.to_bits());
}
