//! Independent truth and property tests for REQ-KERNEL-003.
#![allow(clippy::excessive_precision, clippy::unreadable_literal)]

use georbf::{
    Gaussian, InverseMultiquadric, KernelArgument, KernelCalculusError, KernelDefiniteness,
    KernelDerivativeCapability, KernelDerivativeOrder, KernelParameterConstraint,
    KernelParameterUnit, KernelParameterValueError, KernelSupport, Matern, MaternSmoothness,
    Multiquadric, Point, RadialJet, RadialSeparation, SmoothKernelConstructionError,
    SmoothKernelEvaluationError, SmoothKernelFamily, SpatialKernelJet,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

const ORDERS: [KernelDerivativeOrder; 4] = [
    KernelDerivativeOrder::Value,
    KernelDerivativeOrder::First,
    KernelDerivativeOrder::Second,
    KernelDerivativeOrder::Third,
];

#[test]
fn metadata_declares_exact_parameters_classifications_and_center_limits() -> TestResult {
    let gaussian = Gaussian::try_new(1.7)?;
    let inverse = InverseMultiquadric::try_new(1.7)?;
    let multiquadric = Multiquadric::try_new(1.7)?;

    for metadata in [gaussian.metadata(), inverse.metadata()] {
        assert_eq!(
            metadata.definiteness(),
            KernelDefiniteness::StrictlyPositiveDefinite
        );
        assert_eq!(metadata.support(), KernelSupport::Global);
        assert_eq!(metadata.dimensions().flags(), [true, true, true]);
        assert_eq!(
            metadata.derivatives().maximum_away_order(),
            KernelDerivativeOrder::Third
        );
        assert_eq!(
            metadata.derivatives().maximum_center_order(),
            Some(KernelDerivativeOrder::Third)
        );
        assert_length_scale_parameter(metadata);
    }
    assert_eq!(gaussian.metadata().name(), "gaussian");
    assert_eq!(inverse.metadata().name(), "inverse_multiquadric");
    assert_eq!(multiquadric.metadata().name(), "multiquadric");
    assert!(matches!(
        multiquadric.metadata().definiteness(),
        KernelDefiniteness::ConditionallyPositiveDefinite { order }
            if order.get() == 1 && order.maximum_polynomial_degree() == 0
    ));
    assert_length_scale_parameter(multiquadric.metadata());

    for (smoothness, name, center) in [
        (
            MaternSmoothness::OneHalf,
            "matern_1_2",
            KernelDerivativeOrder::Value,
        ),
        (
            MaternSmoothness::ThreeHalves,
            "matern_3_2",
            KernelDerivativeOrder::Second,
        ),
        (
            MaternSmoothness::FiveHalves,
            "matern_5_2",
            KernelDerivativeOrder::Third,
        ),
    ] {
        let kernel = Matern::try_new(smoothness, 1.7)?;
        let metadata = kernel.metadata();
        assert_eq!(kernel.smoothness(), smoothness);
        assert_same_bits(kernel.length_scale(), 1.7);
        assert_eq!(metadata.name(), name);
        assert_eq!(
            metadata.definiteness(),
            KernelDefiniteness::StrictlyPositiveDefinite
        );
        assert_eq!(metadata.derivatives().maximum_center_order(), Some(center));
        assert_length_scale_parameter(metadata);
    }
    assert_same_bits(gaussian.length_scale(), 1.7);
    assert_same_bits(inverse.length_scale(), 1.7);
    assert_same_bits(multiquadric.length_scale(), 1.7);
    Ok(())
}

#[test]
fn radial_terms_match_embedded_ninety_digit_reference_values() -> TestResult {
    let radius = 0.73;
    let length_scale = 1.7;
    let separation =
        RadialSeparation::try_new(Point::try_new([radius, 0.0])?, Point::try_new([0.0, 0.0])?)?;

    let gaussian = Gaussian::try_new(length_scale)?;
    assert_reference(
        |order| gaussian.radial_derivative(radius, order),
        gaussian.radial_jet(separation)?,
        [
            9.1192527111318645291084487825529309505e-1,
            -2.3034790585211976146190891388455500325e-1,
            -2.5736031136371592631268213533549063760e-1,
            2.2441828339091769866161791922630386573e-1,
            -3.1554507650975309789302590943089726472e-1,
            7.9705157734297495315539416569050174134e-2,
        ],
    )?;
    let inverse = InverseMultiquadric::try_new(length_scale)?;
    assert_reference(
        |order| inverse.radial_derivative(radius, order),
        inverse.radial_jet(separation)?,
        [
            9.1886522493038886752800684003856025476e-1,
            -1.9596588103630952505052586789802477022e-1,
            -1.4306580544592918509665932085129159718e-1,
            3.8156350288436110485404481366657320125e-1,
            -2.6844641237850619869935050396989694550e-1,
            1.7175425607202330630505641523096623058e-1,
        ],
    )?;
    let multiquadric = Multiquadric::try_new(length_scale)?;
    assert_reference(
        |order| multiquadric.radial_derivative(radius, order),
        multiquadric.radial_jet(separation)?,
        [
            -1.0882988852644387732393130147986117287,
            -2.3210090456719165165932352706856366297e-1,
            -2.6844641237850619869935050396989694550e-1,
            1.7175425607202330630505641523096623058e-1,
            -3.1794644461259130364290894118981323694e-1,
            6.7808263334363157456929366054679851286e-2,
        ],
    )?;

    let matern_references = [
        (
            MaternSmoothness::OneHalf,
            [
                6.5089185969887425433063565945529648835e-1,
                -3.8287756452874956137096215262076264020e-1,
                2.2522209678161738904174244271809567071e-1,
                -1.3248358634212787590690731924593862983e-1,
                -5.2448981442294460461775637345309950713e-1,
                1.0270026180884410872047928988646509285,
            ],
        ),
        (
            MaternSmoothness::ThreeHalves,
            [
                8.2884876128148059630547410390237775379e-1,
                -3.6019203754497202092248345307225252046e-1,
                -1.2643086211475364981891551936835203463e-1,
                6.3153080063957672423544470706942354907e-1,
                -4.9341375006160550811299103160582537050e-1,
                5.0271628485870117574530892087325114502e-1,
            ],
        ),
        (
            MaternSmoothness::FiveHalves,
            [
                8.6804818433334082167577328302891468834e-1,
                -3.1591182964923604128631435107011369041e-1,
                -2.2921025640066105993895881656869490863e-1,
                5.6875850616355935346876147987300261361e-1,
                -4.3275593102635074148810185078097765810e-1,
                2.7882969126806805691663429344148321846e-1,
            ],
        ),
    ];
    for (smoothness, expected) in matern_references {
        let kernel = Matern::try_new(smoothness, length_scale)?;
        assert_reference(
            |order| kernel.radial_derivative(radius, order),
            kernel.radial_jet(separation)?,
            expected,
        )?;
    }
    Ok(())
}

#[test]
fn analytic_radial_derivatives_match_independent_finite_differences() -> TestResult {
    let gaussian = Gaussian::try_new(1.2)?;
    let inverse = InverseMultiquadric::try_new(1.2)?;
    let multiquadric = Multiquadric::try_new(1.2)?;
    for radius in [0.4, 0.9, 1.5] {
        assert_finite_differences(
            radius,
            |r| gaussian.radial_value(r),
            |r, order| gaussian.radial_derivative(r, order),
        )?;
        assert_finite_differences(
            radius,
            |r| inverse.radial_value(r),
            |r, order| inverse.radial_derivative(r, order),
        )?;
        assert_finite_differences(
            radius,
            |r| multiquadric.radial_value(r),
            |r, order| multiquadric.radial_derivative(r, order),
        )?;
        for smoothness in [
            MaternSmoothness::OneHalf,
            MaternSmoothness::ThreeHalves,
            MaternSmoothness::FiveHalves,
        ] {
            let matern = Matern::try_new(smoothness, 1.2)?;
            assert_finite_differences(
                radius,
                |r| matern.radial_value(r),
                |r, order| matern.radial_derivative(r, order),
            )?;
        }
    }
    Ok(())
}

#[test]
fn center_capabilities_and_limits_are_exact() -> TestResult {
    let center =
        RadialSeparation::try_new(Point::try_new([0.4, -0.7])?, Point::try_new([0.4, -0.7])?)?;
    for (family, center_second, jet) in [
        (
            SmoothKernelFamily::Gaussian,
            -0.25,
            Gaussian::try_new(2.0)?.radial_jet(center)?,
        ),
        (
            SmoothKernelFamily::InverseMultiquadric,
            -0.25,
            InverseMultiquadric::try_new(2.0)?.radial_jet(center)?,
        ),
        (
            SmoothKernelFamily::Multiquadric,
            -0.25,
            Multiquadric::try_new(2.0)?.radial_jet(center)?,
        ),
        (
            SmoothKernelFamily::Matern(MaternSmoothness::FiveHalves),
            -5.0 / 12.0,
            Matern::try_new(MaternSmoothness::FiveHalves, 2.0)?.radial_jet(center)?,
        ),
    ] {
        assert_close(jet.second_derivative(), center_second, 2.0e-15);
        assert_same_bits(jet.first_derivative(), 0.0);
        assert_same_bits(jet.third_derivative(), 0.0);
        let _ = family;
    }

    let one_half = Matern::try_new(MaternSmoothness::OneHalf, 2.0)?;
    assert_eq!(
        one_half.radial_derivative(0.0, KernelDerivativeOrder::Value)?,
        Some(1.0)
    );
    for order in ORDERS.into_iter().skip(1) {
        assert_eq!(one_half.radial_derivative(0.0, order)?, None);
        assert_eq!(
            one_half.metadata().derivatives().capability(order),
            KernelDerivativeCapability::SupportedAwayFromCenters
        );
    }
    assert!(matches!(
        one_half.radial_jet(center),
        Err(SmoothKernelEvaluationError::CenterJetUnsupported {
            family: SmoothKernelFamily::Matern(MaternSmoothness::OneHalf),
            maximum_center_order: KernelDerivativeOrder::Value
        })
    ));

    let three_halves = Matern::try_new(MaternSmoothness::ThreeHalves, 2.0)?;
    assert_eq!(
        three_halves.radial_derivative(0.0, KernelDerivativeOrder::First)?,
        Some(0.0)
    );
    assert_close(
        three_halves
            .radial_derivative(0.0, KernelDerivativeOrder::Second)?
            .ok_or("Matérn 3/2 center second derivative missing")?,
        -0.75,
        2.0e-15,
    );
    assert_eq!(
        three_halves.radial_derivative(0.0, KernelDerivativeOrder::Third)?,
        None
    );
    assert!(matches!(
        three_halves.radial_jet(center),
        Err(SmoothKernelEvaluationError::CenterJetUnsupported {
            family: SmoothKernelFamily::Matern(MaternSmoothness::ThreeHalves),
            maximum_center_order: KernelDerivativeOrder::Second
        })
    ));
    Ok(())
}

#[test]
fn deterministic_gram_matrices_match_spd_and_cpd_classifications() -> TestResult {
    let gaussian = Gaussian::try_new(1.1)?;
    let inverse = InverseMultiquadric::try_new(1.1)?;
    for seed in 1..=5 {
        assert_spd_all_dimensions(seed, |r| gaussian.radial_value(r))?;
        assert_spd_all_dimensions(seed + 10, |r| inverse.radial_value(r))?;
        for smoothness in [
            MaternSmoothness::OneHalf,
            MaternSmoothness::ThreeHalves,
            MaternSmoothness::FiveHalves,
        ] {
            let matern = Matern::try_new(smoothness, 1.1)?;
            assert_spd_all_dimensions(seed + 20, |r| matern.radial_value(r))?;
        }
        assert_projected_multiquadric::<1>(seed)?;
        assert_projected_multiquadric::<2>(seed)?;
        assert_projected_multiquadric::<3>(seed)?;
    }
    Ok(())
}

#[test]
fn coordinate_scaling_preserves_values_and_rescales_derivatives() -> TestResult {
    let radius = 0.8;
    let length_scale = 1.3;
    let coordinate_scale = 10.0;
    let scaled_radius = coordinate_scale * radius;
    let scaled_length_scale = coordinate_scale * length_scale;
    let separation =
        RadialSeparation::try_new(Point::try_new([radius, 0.0])?, Point::try_new([0.0, 0.0])?)?;
    let scaled_separation = RadialSeparation::try_new(
        Point::try_new([scaled_radius, 0.0])?,
        Point::try_new([0.0, 0.0])?,
    )?;

    let gaussian = Gaussian::try_new(length_scale)?;
    let scaled_gaussian = Gaussian::try_new(scaled_length_scale)?;
    assert_scale_covariance(
        |order| gaussian.radial_derivative(radius, order),
        gaussian.radial_jet(separation)?,
        |order| scaled_gaussian.radial_derivative(scaled_radius, order),
        scaled_gaussian.radial_jet(scaled_separation)?,
        coordinate_scale,
    )?;

    let inverse = InverseMultiquadric::try_new(length_scale)?;
    let scaled_inverse = InverseMultiquadric::try_new(scaled_length_scale)?;
    assert_scale_covariance(
        |order| inverse.radial_derivative(radius, order),
        inverse.radial_jet(separation)?,
        |order| scaled_inverse.radial_derivative(scaled_radius, order),
        scaled_inverse.radial_jet(scaled_separation)?,
        coordinate_scale,
    )?;

    let multiquadric = Multiquadric::try_new(length_scale)?;
    let scaled_multiquadric = Multiquadric::try_new(scaled_length_scale)?;
    assert_scale_covariance(
        |order| multiquadric.radial_derivative(radius, order),
        multiquadric.radial_jet(separation)?,
        |order| scaled_multiquadric.radial_derivative(scaled_radius, order),
        scaled_multiquadric.radial_jet(scaled_separation)?,
        coordinate_scale,
    )?;

    for smoothness in [
        MaternSmoothness::OneHalf,
        MaternSmoothness::ThreeHalves,
        MaternSmoothness::FiveHalves,
    ] {
        let matern = Matern::try_new(smoothness, length_scale)?;
        let scaled_matern = Matern::try_new(smoothness, scaled_length_scale)?;
        assert_scale_covariance(
            |order| matern.radial_derivative(radius, order),
            matern.radial_jet(separation)?,
            |order| scaled_matern.radial_derivative(scaled_radius, order),
            scaled_matern.radial_jet(scaled_separation)?,
            coordinate_scale,
        )?;
    }
    Ok(())
}

#[test]
fn extreme_paths_preserve_representable_exponential_and_rational_tails() -> TestResult {
    let gaussian = Gaussian::try_new(1.0e-100)?;
    assert_same_bits(gaussian.radial_value(4.0e-99)?, 0.0);
    let gaussian_third = gaussian
        .radial_derivative(4.0e-99, KernelDerivativeOrder::Third)?
        .ok_or("Gaussian away derivative missing")?;
    assert!(gaussian_third.is_finite() && gaussian_third != 0.0);

    let matern = Matern::try_new(MaternSmoothness::OneHalf, 1.0e-100)?;
    assert_same_bits(matern.radial_value(8.0e-98)?, 0.0);
    let matern_third = matern
        .radial_derivative(8.0e-98, KernelDerivativeOrder::Third)?
        .ok_or("Matérn away derivative missing")?;
    assert!(matern_third.is_finite() && matern_third != 0.0);

    let rational_tail = InverseMultiquadric::try_new(0.5)?.radial_value(f64::MAX)?;
    assert!(rational_tail > 0.0 && rational_tail < f64::MIN_POSITIVE);

    let multiquadric = Multiquadric::try_new(0.5)?;
    let multiquadric_first = multiquadric
        .radial_derivative(f64::MAX, KernelDerivativeOrder::First)?
        .ok_or("multiquadric away derivative missing")?;
    assert_same_bits(multiquadric_first, -2.0);

    let tiny_radius = 1.0e-200;
    let separation = RadialSeparation::try_new(
        Point::try_new([tiny_radius, 0.0])?,
        Point::try_new([0.0, 0.0])?,
    )?;
    let jet = Matern::try_new(MaternSmoothness::OneHalf, 1.0e200)?.radial_jet(separation)?;
    let expansion = jet.expansion_coefficients().ok_or("missing expansion")?;
    assert_close(expansion.first_over_radius(), -1.0, 3.0e-14);
    assert_close(expansion.second_remainder_over_radius(), 1.0e200, 5.0e-14);
    Ok(())
}

#[test]
#[allow(clippy::needless_range_loop)]
fn shared_cartesian_calculus_preserves_exchange_signs_and_tensor_symmetry() -> TestResult {
    let separation = RadialSeparation::try_new(
        Point::try_new([0.7, -0.4, 1.1])?,
        Point::try_new([-0.2, 0.3, 0.5])?,
    )?;
    for jet in [
        Gaussian::try_new(1.4)?.radial_jet(separation)?,
        InverseMultiquadric::try_new(1.4)?.radial_jet(separation)?,
        Multiquadric::try_new(1.4)?.radial_jet(separation)?,
        Matern::try_new(MaternSmoothness::OneHalf, 1.4)?.radial_jet(separation)?,
        Matern::try_new(MaternSmoothness::ThreeHalves, 1.4)?.radial_jet(separation)?,
        Matern::try_new(MaternSmoothness::FiveHalves, 1.4)?.radial_jet(separation)?,
    ] {
        let spatial = SpatialKernelJet::try_new(separation, jet)?;
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
fn construction_and_numeric_pathologies_return_structured_errors() -> TestResult {
    for length_scale in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(matches!(
            Gaussian::try_new(length_scale),
            Err(SmoothKernelConstructionError::InvalidLengthScale(
                KernelParameterValueError::NonFinite { .. }
                    | KernelParameterValueError::ViolatesConstraint { .. }
            ))
        ));
    }
    assert!(matches!(
        Gaussian::try_new(f64::MIN_POSITIVE),
        Err(SmoothKernelConstructionError::NonRepresentableDerivativeScale { .. })
    ));
    assert!(matches!(
        Matern::try_new(MaternSmoothness::FiveHalves, 1.0e-104),
        Err(SmoothKernelConstructionError::NonRepresentableDerivativeScale { .. })
    ));

    let gaussian = Gaussian::try_new(1.0)?;
    assert!(matches!(
        gaussian.radial_value(-1.0),
        Err(SmoothKernelEvaluationError::NegativeRadius { radius: -1.0 })
    ));
    for radius in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(matches!(
            gaussian.radial_value(radius),
            Err(SmoothKernelEvaluationError::NonFiniteRadius { .. })
        ));
    }
    assert!(matches!(
        Multiquadric::try_new(0.5)?.radial_value(f64::MAX),
        Err(SmoothKernelEvaluationError::KernelCalculus(
            KernelCalculusError::NonFiniteRadialDerivative { .. }
        ))
    ));
    Ok(())
}

#[test]
fn smooth_kernel_values_are_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<Gaussian>();
    assert_send_sync::<InverseMultiquadric>();
    assert_send_sync::<Multiquadric>();
    assert_send_sync::<Matern>();
    assert_send_sync::<MaternSmoothness>();
    assert_send_sync::<SmoothKernelFamily>();
    assert_send_sync::<SmoothKernelConstructionError>();
    assert_send_sync::<SmoothKernelEvaluationError>();
}

fn assert_length_scale_parameter(metadata: georbf::KernelMetadata<'_>) {
    assert_eq!(metadata.parameters().len(), 1);
    let parameter = metadata.parameters()[0];
    assert_eq!(parameter.name(), "length_scale");
    assert_eq!(parameter.unit(), KernelParameterUnit::CoordinateLength);
    assert_eq!(parameter.constraint(), KernelParameterConstraint::Positive);
    assert!(!parameter.description().is_empty());
}

fn assert_reference<F>(mut derivative: F, jet: RadialJet, expected: [f64; 6]) -> TestResult
where
    F: FnMut(KernelDerivativeOrder) -> Result<Option<f64>, SmoothKernelEvaluationError>,
{
    for (order, expected) in ORDERS.into_iter().zip(expected) {
        assert_close(
            derivative(order)?.ok_or("away derivative missing")?,
            expected,
            4.0e-15,
        );
    }
    let expansion = jet
        .expansion_coefficients()
        .ok_or("missing expansion coefficients")?;
    assert_close(expansion.first_over_radius(), expected[4], 4.0e-15);
    assert_close(
        expansion.second_remainder_over_radius(),
        expected[5],
        4.0e-15,
    );
    Ok(())
}

fn assert_scale_covariance<F, G>(
    mut derivative: F,
    jet: RadialJet,
    mut scaled_derivative: G,
    scaled_jet: RadialJet,
    coordinate_scale: f64,
) -> TestResult
where
    F: FnMut(KernelDerivativeOrder) -> Result<Option<f64>, SmoothKernelEvaluationError>,
    G: FnMut(KernelDerivativeOrder) -> Result<Option<f64>, SmoothKernelEvaluationError>,
{
    let expansion = jet
        .expansion_coefficients()
        .ok_or("missing expansion coefficients")?;
    let scaled_expansion = scaled_jet
        .expansion_coefficients()
        .ok_or("missing scaled expansion coefficients")?;
    let terms = [
        derivative(KernelDerivativeOrder::Value)?.ok_or("value missing")?,
        derivative(KernelDerivativeOrder::First)?.ok_or("first derivative missing")?,
        derivative(KernelDerivativeOrder::Second)?.ok_or("second derivative missing")?,
        derivative(KernelDerivativeOrder::Third)?.ok_or("third derivative missing")?,
        expansion.first_over_radius(),
        expansion.second_remainder_over_radius(),
    ];
    let scaled_terms = [
        scaled_derivative(KernelDerivativeOrder::Value)?.ok_or("scaled value missing")?,
        scaled_derivative(KernelDerivativeOrder::First)?
            .ok_or("scaled first derivative missing")?,
        scaled_derivative(KernelDerivativeOrder::Second)?
            .ok_or("scaled second derivative missing")?,
        scaled_derivative(KernelDerivativeOrder::Third)?
            .ok_or("scaled third derivative missing")?,
        scaled_expansion.first_over_radius(),
        scaled_expansion.second_remainder_over_radius(),
    ];
    let powers = [0, 1, 2, 3, 2, 3];
    for ((term, scaled_term), power) in terms.into_iter().zip(scaled_terms).zip(powers) {
        assert_close(scaled_term, term / coordinate_scale.powi(power), 8.0e-15);
    }
    Ok(())
}

fn assert_finite_differences<V, D>(radius: f64, value: V, derivative: D) -> TestResult
where
    V: Fn(f64) -> Result<f64, SmoothKernelEvaluationError>,
    D: Fn(f64, KernelDerivativeOrder) -> Result<Option<f64>, SmoothKernelEvaluationError>,
{
    let h = 1.0e-3 * radius.max(1.0);
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
    let tolerances = [2.0e-15, 2.0e-8, 4.0e-7, 4.0e-5];
    for ((order, approximation), tolerance) in
        ORDERS.into_iter().zip(approximations).zip(tolerances)
    {
        let actual = derivative(radius, order)?.ok_or("away derivative missing")?;
        let scale = actual.abs().max(approximation.abs()).max(1.0);
        assert!(
            (actual - approximation).abs() <= tolerance * scale,
            "finite-difference mismatch: actual {actual:.17e}, approximation {approximation:.17e}, tolerance {tolerance:.3e}"
        );
    }
    Ok(())
}

fn assert_spd_all_dimensions<F>(seed: u64, radial_value: F) -> TestResult
where
    F: Copy + Fn(f64) -> Result<f64, SmoothKernelEvaluationError>,
{
    assert_positive_energy::<1, _>(seed, false, radial_value)?;
    assert_positive_energy::<2, _>(seed, false, radial_value)?;
    assert_positive_energy::<3, _>(seed, false, radial_value)?;
    Ok(())
}

fn assert_projected_multiquadric<const D: usize>(seed: u64) -> TestResult {
    let kernel = Multiquadric::try_new(1.1)?;
    assert_positive_energy::<D, _>(seed + 40, true, |radius| kernel.radial_value(radius))
}

fn assert_positive_energy<const D: usize, F>(
    seed: u64,
    project_constants: bool,
    radial_value: F,
) -> TestResult
where
    F: Fn(f64) -> Result<f64, SmoothKernelEvaluationError>,
{
    let points = deterministic_points::<D>(6, seed);
    let mut gram = vec![vec![0.0; points.len()]; points.len()];
    for row in 0..points.len() {
        for column in 0..points.len() {
            gram[row][column] = radial_value(distance(points[row], points[column]))?;
        }
    }

    let tested_matrix = if project_constants {
        // The columns `z_i = e_i - e_last` span the complete constant-zero
        // subspace, so these entries are the independently assembled `Z^T K Z`.
        let last = points.len() - 1;
        (0..last)
            .map(|row| {
                (0..last)
                    .map(|column| {
                        gram[row][column] - gram[row][last] - gram[last][column] + gram[last][last]
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    } else {
        gram.clone()
    };
    assert_strictly_positive_definite(&tested_matrix, D, project_constants, seed);

    let mut weights = deterministic_weights(seed);
    if project_constants {
        weights[5] = -weights[..5].iter().sum::<f64>();
        assert!(weights.iter().sum::<f64>().abs() <= 2.0e-15);
    }
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
        "non-positive energy {energy:.17e} at scale {scale:.17e}, D={D}, projected={project_constants}, seed={seed}"
    );
    Ok(())
}

fn assert_strictly_positive_definite(
    matrix: &[Vec<f64>],
    dimension: usize,
    projected: bool,
    seed: u64,
) {
    let size = matrix.len();
    assert!(size > 0 && matrix.iter().all(|row| row.len() == size));
    let scale = matrix
        .iter()
        .flatten()
        .fold(0.0_f64, |maximum, value| maximum.max(value.abs()));
    let symmetry_tolerance = 32.0 * f64::EPSILON * scale.max(f64::MIN_POSITIVE);
    for (row, row_values) in matrix.iter().enumerate() {
        for (column, column_values) in matrix.iter().take(row).enumerate() {
            assert!(
                (row_values[column] - column_values[row]).abs() <= symmetry_tolerance,
                "asymmetric Gram entry at D={dimension}, projected={projected}, seed={seed}, row={row}, column={column}"
            );
        }
    }
    let pivot_floor = 256.0 * f64::EPSILON * scale.max(f64::MIN_POSITIVE);
    let mut lower = vec![vec![0.0; size]; size];

    for row in 0..size {
        for column in 0..=row {
            let mut residual = matrix[row][column];
            for (row_term, column_term) in lower[row][..column].iter().zip(&lower[column][..column])
            {
                residual -= row_term * column_term;
            }
            if row == column {
                assert!(
                    residual.is_finite() && residual > pivot_floor,
                    "non-positive Cholesky pivot {residual:.17e} at scale {scale:.17e}, D={dimension}, projected={projected}, seed={seed}, row={row}"
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
                1.8 * unit - 0.9 + 0.017 * ordinal
            })
        })
        .collect()
}

fn deterministic_weights(seed: u64) -> [f64; 6] {
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
