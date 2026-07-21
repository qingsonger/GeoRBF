//! Independent property and error-path tests for local positive-definite mixtures.

use std::error::Error;

use georbf::{
    AnisotropyConditionPolicy, Gaussian, GlobalAnisotropy, KernelDefinition,
    KernelDerivativeCapability, KernelDerivativeOrder, LocalTrendComponent,
    LocalTrendConstructionError, LocalTrendEvaluationError, LocalTrendMixture, Matern,
    MaternSmoothness, Multiquadric, OperationalDomain, Point, SmoothSpatialWeight,
};
use nalgebra::DMatrix;

fn assert_send_sync<T: Send + Sync>() {}

fn domain<const D: usize>(extent: f64) -> Result<OperationalDomain<D>, Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    Ok(OperationalDomain::try_new(
        Point::try_new([-extent; D])?,
        Point::try_new([extent; D])?,
    )?)
}

fn background<const D: usize>(weight: f64) -> Result<LocalTrendComponent<D>, Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    Ok(LocalTrendComponent::new(
        KernelDefinition::from(Gaussian::try_new(1.1)?),
        GlobalAnisotropy::try_isotropic(1.0)?,
        SmoothSpatialWeight::try_constant(weight)?,
    ))
}

fn varying_component<const D: usize>(
    center: [f64; D],
    transform: [[f64; D]; D],
) -> Result<LocalTrendComponent<D>, Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    Ok(LocalTrendComponent::new(
        KernelDefinition::from(Gaussian::try_new(0.8)?),
        GlobalAnisotropy::try_from_transform(transform, AnisotropyConditionPolicy::Unbounded)?,
        SmoothSpatialWeight::try_gaussian(Point::try_new(center)?, 1.7, 1.25)?,
    ))
}

#[test]
fn deterministic_random_spd_gram_matrices_are_strictly_positive() -> Result<(), Box<dyn Error>> {
    let mut state = 0x9e37_79b9_7f4a_7c15_u64;
    for _case in 0..24 {
        let mut next = || {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let Ok(upper) = u32::try_from(state >> 32) else {
                return 0.0;
            };
            let unit = f64::from(upper) / f64::from(u32::MAX);
            2.0 * unit - 1.0
        };
        let transform = [
            [0.75 + 0.2 * next().abs(), 0.12 * next(), 0.08 * next()],
            [0.0, 0.9 + 0.2 * next().abs(), 0.1 * next()],
            [0.0, 0.0, 1.1 + 0.2 * next().abs()],
        ];
        let mixture = LocalTrendMixture::try_new(
            vec![
                background::<3>(0.35)?,
                varying_component([0.2, -0.4, 0.6], transform)?,
            ],
            0,
            domain(3.0)?,
            0.25,
        )?;
        let points = (0_u32..9)
            .map(|index| {
                let t = f64::from(index);
                Point::try_new([
                    -1.4 + 0.31 * t,
                    (0.73 * t).sin(),
                    (0.41 * t).cos() + 0.03 * t,
                ])
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut entries = Vec::with_capacity(points.len() * points.len());
        for row in 0..points.len() {
            for column in 0..points.len() {
                entries.push(
                    mixture
                        .try_evaluate(points[row], points[column], KernelDerivativeOrder::Value)?
                        .value(),
                );
            }
        }
        let gram = DMatrix::from_row_slice(points.len(), points.len(), &entries);
        assert!(gram.clone().cholesky().is_some());
        assert!((&gram - gram.transpose()).amax() <= 8.0 * f64::EPSILON);
    }
    Ok(())
}

fn product_rule_mixture() -> Result<LocalTrendMixture<2>, Box<dyn Error>> {
    Ok(LocalTrendMixture::try_new(
        vec![
            background::<2>(0.4)?,
            varying_component([0.35, -0.15], [[0.8, 0.25], [-0.1, 1.15]])?,
        ],
        0,
        domain(4.0)?,
        0.2,
    )?)
}

fn value_at(
    mixture: &LocalTrendMixture<2>,
    query: [f64; 2],
    center: Point<2>,
) -> Result<f64, Box<dyn Error>> {
    Ok(mixture
        .try_evaluate(Point::try_new(query)?, center, KernelDerivativeOrder::Value)?
        .value())
}

#[test]
fn gradient_and_hessian_match_independent_finite_differences() -> Result<(), Box<dyn Error>> {
    let mixture = product_rule_mixture()?;
    let query = [0.7, -0.45];
    let center = Point::try_new([-0.25, 0.55])?;
    let analytic = mixture.try_evaluate(
        Point::try_new(query)?,
        center,
        KernelDerivativeOrder::Second,
    )?;
    let gradient = analytic
        .gradient()
        .ok_or_else(|| std::io::Error::other("gradient demand was not retained"))?;
    let hessian = analytic
        .hessian()
        .ok_or_else(|| std::io::Error::other("Hessian demand was not retained"))?;
    let gradient_step = 2.0e-6;
    for axis in 0..2 {
        let mut plus = query;
        let mut minus = query;
        plus[axis] += gradient_step;
        minus[axis] -= gradient_step;
        let numerical = (value_at(&mixture, plus, center)? - value_at(&mixture, minus, center)?)
            / (2.0 * gradient_step);
        assert!((gradient[axis] - numerical).abs() <= 2.0e-8);
    }

    let h = 2.0e-4;
    let base = value_at(&mixture, query, center)?;
    for axis in 0..2 {
        let mut plus = query;
        let mut minus = query;
        plus[axis] += h;
        minus[axis] -= h;
        let numerical = (value_at(&mixture, plus, center)? - 2.0 * base
            + value_at(&mixture, minus, center)?)
            / (h * h);
        assert!((hessian[axis][axis] - numerical).abs() <= 2.0e-7);
    }
    let pp = value_at(&mixture, [query[0] + h, query[1] + h], center)?;
    let pm = value_at(&mixture, [query[0] + h, query[1] - h], center)?;
    let mp = value_at(&mixture, [query[0] - h, query[1] + h], center)?;
    let mm = value_at(&mixture, [query[0] - h, query[1] - h], center)?;
    let mixed = (pp - pm - mp + mm) / (4.0 * h * h);
    assert!((hessian[0][1] - mixed).abs() <= 2.0e-7);
    assert!((hessian[0][1] - hessian[1][0]).abs() <= f64::EPSILON);
    Ok(())
}

#[test]
fn strict_background_and_policy_failures_are_structured() -> Result<(), Box<dyn Error>> {
    let varying = varying_component([0.0], [[1.0]])?;
    assert!(matches!(
        LocalTrendMixture::try_new(vec![varying], 0, domain(1.0)?, 0.1),
        Err(LocalTrendConstructionError::NonConstantBackground { component: 0 })
    ));

    let zero = background::<1>(0.0)?;
    assert!(matches!(
        LocalTrendMixture::try_new(vec![zero], 0, domain(1.0)?, 0.1),
        Err(LocalTrendConstructionError::ZeroBackgroundWeight { component: 0 })
    ));

    let small = background::<1>(-0.2)?;
    assert!(matches!(
        LocalTrendMixture::try_new(vec![small], 0, domain(1.0)?, 0.25),
        Err(LocalTrendConstructionError::BackgroundBelowMinimum {
            component: 0,
            magnitude: 0.2,
            minimum: 0.25,
        })
    ));
    assert!(matches!(
        SmoothSpatialWeight::<1>::try_constant(1.0e-200),
        Err(
            LocalTrendConstructionError::NonRepresentableWeightAmplitudeSquare {
                amplitude: 1.0e-200,
            }
        )
    ));
    Ok(())
}

#[test]
fn extreme_gaussian_hessian_retains_representable_derivative() -> Result<(), Box<dyn Error>> {
    let radius = 1.0e-150;
    let local = LocalTrendComponent::new(
        KernelDefinition::from(Gaussian::try_new(1.0)?),
        GlobalAnisotropy::try_isotropic(1.0)?,
        SmoothSpatialWeight::try_gaussian(Point::try_new([0.0])?, 1.0, radius)?,
    );
    let mixture = LocalTrendMixture::try_new(
        vec![background::<1>(1.0e-160)?, local],
        0,
        domain(1.0)?,
        1.0e-160,
    )?;
    let evaluation = mixture.try_evaluate(
        Point::try_new([40.0 * radius])?,
        Point::try_new([0.0])?,
        KernelDerivativeOrder::Second,
    )?;
    let hessian = evaluation
        .hessian()
        .ok_or_else(|| std::io::Error::other("Hessian demand was not retained"))?;
    let expected = 5.864_931_460_100_122e-45;
    assert!(hessian[0][0] != 0.0);
    assert!((hessian[0][0] - expected).abs() <= expected * 2.0e-13);
    Ok(())
}

#[test]
fn extreme_gaussian_value_survives_exponential_underflow() -> Result<(), Box<dyn Error>> {
    let radius = 1.0e-150;
    let local = LocalTrendComponent::new(
        KernelDefinition::from(Gaussian::try_new(1.0)?),
        GlobalAnisotropy::try_isotropic(1.0)?,
        SmoothSpatialWeight::try_gaussian(Point::try_new([0.0])?, 1.0e150, radius)?,
    );
    let mixture = LocalTrendMixture::try_new(
        vec![background::<1>(1.0e-160)?, local],
        0,
        domain(1.0)?,
        1.0e-160,
    )?;

    let value = mixture
        .try_evaluate(
            Point::try_new([40.0 * radius])?,
            Point::try_new([0.0])?,
            KernelDerivativeOrder::Value,
        )?
        .value();
    let expected = 3.667_874_584_177_687e-48;
    assert!(value != 0.0);
    assert!((value - expected).abs() <= expected * 2.0e-13);
    Ok(())
}

#[test]
fn gaussian_radius_must_preserve_inverse_square() -> Result<(), Box<dyn Error>> {
    assert!(matches!(
        SmoothSpatialWeight::<1>::try_gaussian(Point::try_new([0.0])?, 1.0e154, 1.0e200),
        Err(LocalTrendConstructionError::NonRepresentableWeightRadius { radius: 1.0e200 })
    ));
    Ok(())
}

#[test]
fn gaussian_gradient_retains_subnormal_displacement_scale() -> Result<(), Box<dyn Error>> {
    let local = LocalTrendComponent::new(
        KernelDefinition::from(Gaussian::try_new(1.0)?),
        GlobalAnisotropy::try_isotropic(1.0)?,
        SmoothSpatialWeight::try_gaussian(Point::try_new([0.0])?, 1.0e154, 3.0)?,
    );
    let mixture =
        LocalTrendMixture::try_new(vec![background::<1>(0.5)?, local], 0, domain(1.0)?, 0.25)?;
    let point = Point::try_new([f64::from_bits(1)])?;

    let evaluation = mixture.try_evaluate(point, point, KernelDerivativeOrder::First)?;
    let gradient = evaluation
        .gradient()
        .ok_or_else(|| std::io::Error::other("gradient demand was not retained"))?[0];
    let expected = -5.489_618_287_124_962e-17;
    assert!(gradient != 0.0);
    assert!(
        (gradient - expected).abs() <= expected.abs() * 2.0e-15,
        "gradient {gradient:e} differs from truth {expected:e}"
    );
    Ok(())
}

#[test]
fn gaussian_hessian_retains_mixed_scaled_coordinate_product() -> Result<(), Box<dyn Error>> {
    let local = LocalTrendComponent::new(
        KernelDefinition::from(Gaussian::try_new(1.0)?),
        GlobalAnisotropy::try_isotropic(1.0)?,
        SmoothSpatialWeight::try_gaussian(Point::try_new([0.0, 0.0])?, 1.0, 1.0e-154)?,
    );
    let mixture =
        LocalTrendMixture::try_new(vec![background::<2>(0.5)?, local], 0, domain(1.0)?, 0.25)?;
    let delta = f64::from_bits(1);
    let point = Point::try_new([delta, delta])?;

    let evaluation = mixture.try_evaluate(point, point, KernelDerivativeOrder::Second)?;
    let hessian = evaluation
        .hessian()
        .ok_or_else(|| std::io::Error::other("Hessian demand was not retained"))?;
    let expected = 2.441_008_624_005_280_7e-31;
    assert_eq!(hessian[0][1].to_bits(), hessian[1][0].to_bits());
    for entry in [hessian[0][1], hessian[1][0]] {
        assert!(entry != 0.0);
        assert!(
            (entry - expected).abs() <= expected * 2.0e-13,
            "mixed Hessian entry {entry:e} differs from truth {expected:e}"
        );
    }
    Ok(())
}

#[test]
fn gaussian_hessian_retains_diagonal_cancellation_result() -> Result<(), Box<dyn Error>> {
    let kernel = KernelDefinition::from(Gaussian::try_new(1.0e100)?);
    let background = LocalTrendComponent::new(
        kernel,
        GlobalAnisotropy::try_isotropic(1.0)?,
        SmoothSpatialWeight::try_constant(0.5)?,
    );
    let local = LocalTrendComponent::new(
        kernel,
        GlobalAnisotropy::try_isotropic(1.0)?,
        SmoothSpatialWeight::try_gaussian(Point::try_new([0.0])?, 1.0, 3.0)?,
    );
    let mixture = LocalTrendMixture::try_new(vec![background, local], 0, domain(4.0)?, 0.25)?;
    let point = Point::try_new([f64::from_bits(0x4008_0000_0000_0001)])?;

    let evaluation = mixture.try_evaluate(point, point, KernelDerivativeOrder::Second)?;
    let hessian = evaluation
        .hessian()
        .ok_or_else(|| std::io::Error::other("Hessian demand was not retained"))?;
    let expected = 1.210_157_706_295_617_6e-17;
    assert!(hessian[0][0] > 0.0);
    assert!(
        (hessian[0][0] - expected).abs() <= expected * 2.0e-13,
        "diagonal Hessian entry {:e} differs from truth {expected:e}",
        hessian[0][0]
    );
    Ok(())
}

#[test]
fn coverage_and_value_skip_irrelevant_weight_hessian_overflow() -> Result<(), Box<dyn Error>> {
    let local = LocalTrendComponent::new(
        KernelDefinition::from(Gaussian::try_new(1.0)?),
        GlobalAnisotropy::try_isotropic(1.0)?,
        SmoothSpatialWeight::try_gaussian(Point::try_new([0.0])?, 1.0e150, 1.0e-80)?,
    );
    let mixture =
        LocalTrendMixture::try_new(vec![background::<1>(0.5)?, local], 0, domain(1.0)?, 0.25)?;
    let point = Point::try_new([0.0])?;

    let coverage = mixture.try_coverage(point)?;
    assert!(coverage.squared_weight_sum().is_finite());
    let value = mixture.try_evaluate(point, point, KernelDerivativeOrder::Value)?;
    assert!(value.value().is_finite());
    assert!(matches!(
        mixture.try_evaluate(point, point, KernelDerivativeOrder::Second),
        Err(LocalTrendEvaluationError::NonFiniteWeightDerivative {
            component: 1,
            quantity: georbf::LocalTrendQuantity::Hessian { row: 0, column: 0 },
        })
    ));
    Ok(())
}

#[test]
fn cpd_components_are_rejected_without_polynomial_workaround() -> Result<(), Box<dyn Error>> {
    let cpd = LocalTrendComponent::new(
        KernelDefinition::from(Multiquadric::try_new(1.0)?),
        GlobalAnisotropy::try_isotropic(1.0)?,
        SmoothSpatialWeight::try_constant(0.5)?,
    );
    assert!(matches!(
        LocalTrendMixture::try_new(vec![background::<2>(1.0)?, cpd], 0, domain(2.0)?, 0.1),
        Err(
            LocalTrendConstructionError::ConditionallyPositiveDefiniteComponent {
                component: 1,
                order: 1,
            }
        )
    ));
    Ok(())
}

#[test]
fn hessian_capability_is_checked_at_centers() -> Result<(), Box<dyn Error>> {
    let rough = LocalTrendComponent::new(
        KernelDefinition::from(Matern::try_new(MaternSmoothness::OneHalf, 1.0)?),
        GlobalAnisotropy::try_isotropic(1.0)?,
        SmoothSpatialWeight::try_constant(0.5)?,
    );
    let mixture = LocalTrendMixture::try_new(vec![rough], 0, domain(2.0)?, 0.1)?;
    assert_eq!(
        mixture.derivative_capability(KernelDerivativeOrder::Second),
        KernelDerivativeCapability::SupportedAwayFromCenters
    );
    let point = Point::try_new([0.25, -0.5])?;
    assert!(matches!(
        mixture.try_evaluate(point, point, KernelDerivativeOrder::Second),
        Err(LocalTrendEvaluationError::UnavailableDerivative {
            demanded: KernelDerivativeOrder::Second,
            at_center: true,
        })
    ));
    assert!(
        mixture
            .try_evaluate(
                point,
                Point::try_new([-0.5, 0.25])?,
                KernelDerivativeOrder::Second,
            )
            .is_ok()
    );
    Ok(())
}

#[test]
fn coverage_retains_strict_background_when_local_weight_underflows() -> Result<(), Box<dyn Error>> {
    let mixture = LocalTrendMixture::try_new(
        vec![background::<1>(0.5)?, varying_component([0.0], [[1.0]])?],
        0,
        domain(1.0)?,
        0.25,
    )?;
    let inside = mixture.try_coverage(Point::try_new([0.0])?)?;
    assert!(inside.inside_operational_domain());
    assert_eq!(inside.active_components(), 2);
    assert!(inside.squared_weight_sum() > inside.background_squared_weight());
    assert!((inside.background_squared_weight() - 0.25).abs() <= f64::EPSILON);

    let outside = mixture.try_coverage(Point::try_new([1.0e100])?)?;
    assert!(!outside.inside_operational_domain());
    assert_eq!(outside.active_components(), 1);
    assert!((outside.squared_weight_sum() - 0.25).abs() <= f64::EPSILON);
    assert!((mixture.diagnostics().background_policy_ratio() - 2.0).abs() <= f64::EPSILON);
    Ok(())
}

fn dimension_smoke<const D: usize>() -> Result<(), Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    let mixture = LocalTrendMixture::try_new(vec![background::<D>(0.5)?], 0, domain(1.0)?, 0.1)?;
    let evaluation = mixture.try_evaluate(
        Point::try_new([0.25; D])?,
        Point::try_new([-0.25; D])?,
        KernelDerivativeOrder::Second,
    )?;
    assert!(evaluation.value() > 0.0);
    assert!(evaluation.gradient().is_some());
    assert!(evaluation.hessian().is_some());
    Ok(())
}

#[test]
fn dimensions_and_public_types_are_thread_safe() -> Result<(), Box<dyn Error>> {
    dimension_smoke::<1>()?;
    dimension_smoke::<2>()?;
    dimension_smoke::<3>()?;

    assert_send_sync::<OperationalDomain<3>>();
    assert_send_sync::<SmoothSpatialWeight<3>>();
    assert_send_sync::<LocalTrendComponent<3>>();
    assert_send_sync::<LocalTrendMixture<3>>();
    assert_send_sync::<LocalTrendConstructionError<3>>();
    assert_send_sync::<LocalTrendEvaluationError<3>>();
    Ok(())
}

#[test]
fn input_boundaries_are_rejected_without_panics() -> Result<(), Box<dyn Error>> {
    assert!(matches!(
        OperationalDomain::try_new(Point::try_new([1.0])?, Point::try_new([-1.0])?),
        Err(LocalTrendConstructionError::ReversedDomainAxis { axis: 0, .. })
    ));
    assert!(matches!(
        SmoothSpatialWeight::<1>::try_gaussian(Point::try_new([0.0])?, 1.0, 0.0),
        Err(LocalTrendConstructionError::NonPositiveWeightRadius { radius: 0.0 })
    ));
    assert!(matches!(
        LocalTrendMixture::<1>::try_new(Vec::new(), 0, domain::<1>(1.0)?, 0.1),
        Err(LocalTrendConstructionError::EmptyMixture)
    ));
    Ok(())
}
