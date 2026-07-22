//! Independent truth and property tests for orientation-tensor estimation.

#![allow(clippy::float_cmp)]

use std::error::Error;

use georbf::{
    AxisRatioSelectionKind, OrientationTensorError, OrientationTensorEstimator,
    OrientationTensorSample, OrientationTensorSpectralBackend, PrincipalAxisRatios, UnitDirection,
};

type TestResult = Result<(), Box<dyn Error>>;

fn sample<const D: usize>(
    direction: [f64; D],
    weight: f64,
) -> Result<OrientationTensorSample<D>, Box<dyn Error>>
where
    georbf::Dim<D>: georbf::SupportedDimension,
{
    Ok(OrientationTensorSample::try_new(
        UnitDirection::try_new(direction)?,
        weight,
    )?)
}

fn assert_close(left: f64, right: f64, tolerance: f64) {
    assert!(
        (left - right).abs() <= tolerance,
        "left={left:.17e}, right={right:.17e}, tolerance={tolerance:.3e}"
    );
}

fn assert_matrix_close<const D: usize>(
    left: &[[f64; D]; D],
    right: &[[f64; D]; D],
    tolerance: f64,
) {
    for row in 0..D {
        for column in 0..D {
            assert_close(left[row][column], right[row][column], tolerance);
        }
    }
}

#[test]
fn d1_has_the_only_axis_and_explicit_sole_sample_influence() -> TestResult {
    let estimator =
        OrientationTensorEstimator::try_fixed(PrincipalAxisRatios::try_new([1.0])?, 0.0)?;
    let estimate = estimator.try_estimate(&[sample([-7.0], f64::MAX)?])?;

    assert_eq!(estimate.tensor(), &[[1.0]]);
    assert_eq!(estimate.normalized_eigenvalues(), &[1.0]);
    assert_eq!(estimate.principal_axes()[0].components(), &[1.0]);
    assert_eq!(estimate.diagnostics().axis_confidence(), &[1.0]);
    assert!(estimate.diagnostics().is_isotropic());
    assert_eq!(estimate.influences()[0].normalized_tensor_change(), 1.0);
    Ok(())
}

#[test]
fn outer_products_are_sign_invariant_with_independent_d2_truth() -> TestResult {
    let ratios = PrincipalAxisRatios::try_new([2.0, 1.0])?;
    let estimator = OrientationTensorEstimator::try_fixed(ratios, 0.1)?;
    let positive = [sample([1.0, 0.0], 4.0)?, sample([0.0, 1.0], 2.0)?];
    let reversed = [sample([-1.0, 0.0], 4.0)?, sample([0.0, -1.0], 2.0)?];
    let first = estimator.try_estimate(&positive)?;
    let second = estimator.try_estimate(&reversed)?;

    assert_matrix_close(
        first.tensor(),
        &[[2.0 / 3.0, 0.0], [0.0, 1.0 / 3.0]],
        2.0e-16,
    );
    assert_eq!(first.tensor(), second.tensor());
    assert_eq!(first.eigenvalues(), second.eigenvalues());
    assert_eq!(first.principal_axes()[0].components(), &[1.0, 0.0]);
    assert_eq!(first.principal_axes()[1].components(), &[0.0, 1.0]);
    assert_eq!(first.diagnostics().normalized_eigenvalue_gaps().len(), 1);
    assert_close(
        first.diagnostics().normalized_eigenvalue_gaps()[0],
        1.0 / 3.0,
        2.0e-16,
    );
    assert!(!first.diagnostics().is_isotropic());
    Ok(())
}

#[test]
fn generic_single_d2_direction_preserves_psd_and_trace_normalization() -> TestResult {
    let direction = UnitDirection::try_new([1.0, 30.0])?;
    let estimator =
        OrientationTensorEstimator::try_fixed(PrincipalAxisRatios::try_new([1.0, 1.0])?, 0.0)?;
    let estimate = estimator.try_estimate(&[OrientationTensorSample::try_new(direction, 1.0)?])?;

    let tensor = estimate.tensor();
    assert_eq!(tensor[0][0] + tensor[1][1], 1.0);
    assert!(
        tensor[0][0].mul_add(tensor[1][1], -tensor[0][1] * tensor[1][0]) >= 0.0,
        "represented tensor must remain positive semidefinite: {tensor:?}"
    );
    assert!(estimate.eigenvalues().iter().all(|value| *value >= 0.0));
    assert_eq!(
        estimate.diagnostics().spectral_backend(),
        OrientationTensorSpectralBackend::PositiveSemidefiniteSvd
    );
    assert!((0.0..1.0).contains(&estimate.diagnostics().tensor_correlation_scale()));
    assert_eq!(estimate.influences()[0].normalized_tensor_change(), 1.0);
    Ok(())
}

#[test]
fn minimum_subnormal_d2_direction_preserves_exact_dyadic_psd() -> TestResult {
    let minimum_subnormal = f64::from_bits(1);
    let estimator =
        OrientationTensorEstimator::try_fixed(PrincipalAxisRatios::try_new([1.0, 1.0])?, 0.0)?;
    let estimate = estimator.try_estimate(&[sample([1.0, minimum_subnormal], 1.0)?])?;

    let tensor = estimate.tensor();
    assert_eq!(tensor[0][0] + tensor[1][1], 1.0);
    assert!(
        tensor[1][1] != 0.0 || tensor[0][1] == 0.0,
        "represented tensor has exact dyadic determinant -m^2: {tensor:?}"
    );
    assert!(estimate.eigenvalues().iter().all(|value| *value >= 0.0));
    Ok(())
}

#[test]
fn extreme_d2_direction_reaches_the_greatest_represented_psd_scale() -> TestResult {
    let estimator =
        OrientationTensorEstimator::try_fixed(PrincipalAxisRatios::try_new([1.0, 1.0])?, 0.0)?;
    let estimate = estimator.try_estimate(&[sample([1.0, 2.0_f64.powi(-538)], 1.0)?])?;

    let tensor = estimate.tensor();
    assert_eq!(tensor[0][0] + tensor[1][1], 1.0);
    assert_eq!(tensor[1][1], 0.0);
    assert_eq!(tensor[0][1], 0.0);
    assert_eq!(tensor[1][0], 0.0);
    assert!(estimate.eigenvalues().iter().all(|value| *value >= 0.0));
    assert_eq!(
        estimate.diagnostics().tensor_correlation_scale(),
        2.0_f64.powi(-537)
    );
    Ok(())
}

#[test]
fn d3_axes_and_tensor_are_rotation_covariant_away_from_degeneracy() -> TestResult {
    let angle = 0.47_f64;
    let (sine, cosine) = angle.sin_cos();
    let rotation = [[cosine, -sine, 0.0], [sine, cosine, 0.0], [0.0, 0.0, 1.0]];
    let base = [
        sample([1.0, 0.0, 0.0], 6.0)?,
        sample([0.0, 1.0, 0.0], 3.0)?,
        sample([0.0, 0.0, 1.0], 1.0)?,
    ];
    let rotated = [
        sample([cosine, sine, 0.0], 6.0)?,
        sample([-sine, cosine, 0.0], 3.0)?,
        sample([0.0, 0.0, 1.0], 1.0)?,
    ];
    let estimator = OrientationTensorEstimator::try_fixed(
        PrincipalAxisRatios::try_new([3.0, 2.0, 1.0])?,
        0.01,
    )?;
    let base_estimate = estimator.try_estimate(&base)?;
    let rotated_estimate = estimator.try_estimate(&rotated)?;

    let expected = std::array::from_fn(|row| {
        std::array::from_fn(|column| {
            (0..3)
                .flat_map(|left| (0..3).map(move |right| (left, right)))
                .map(|(left, right)| {
                    rotation[row][left]
                        * base_estimate.tensor()[left][right]
                        * rotation[column][right]
                })
                .sum::<f64>()
        })
    });
    assert_matrix_close(rotated_estimate.tensor(), &expected, 3.0e-16);
    for axis in 0..3 {
        let rotated_base: [f64; 3] = std::array::from_fn(|row| {
            (0..3)
                .map(|column| {
                    rotation[row][column]
                        * base_estimate.principal_axes()[axis].components()[column]
                })
                .sum()
        });
        let absolute_dot = rotated_base
            .iter()
            .zip(rotated_estimate.principal_axes()[axis].components())
            .map(|(left, right)| left * right)
            .sum::<f64>()
            .abs();
        assert_close(absolute_dot, 1.0, 4.0e-15);
    }
    Ok(())
}

#[test]
fn common_weight_scaling_and_extreme_finite_weights_do_not_change_estimate() -> TestResult {
    let estimator =
        OrientationTensorEstimator::try_fixed(PrincipalAxisRatios::try_new([2.0, 1.0])?, 0.0)?;
    let ordinary = estimator.try_estimate(&[
        sample([1.0, 0.0], 2.0)?,
        sample([0.0, 1.0], 1.0)?,
        sample([1.0, 1.0], 0.0)?,
    ])?;
    let extreme = estimator.try_estimate(&[
        sample([1.0, 0.0], f64::MAX)?,
        sample([0.0, 1.0], f64::MAX / 2.0)?,
        sample([-1.0, -1.0], 0.0)?,
    ])?;

    assert_matrix_close(ordinary.tensor(), extreme.tensor(), 2.0e-16);
    assert_close(
        extreme.diagnostics().maximum_weight_fraction(),
        2.0 / 3.0,
        2.0e-16,
    );
    assert_eq!(extreme.diagnostics().positive_sample_count(), 2);
    assert_eq!(extreme.influences()[2].normalized_tensor_change(), 0.0);
    Ok(())
}

#[test]
fn explicit_isotropy_threshold_classifies_exact_and_near_isotropy() -> TestResult {
    let ratios = PrincipalAxisRatios::try_new([1.0, 1.0, 1.0])?;
    let exact = OrientationTensorEstimator::try_fixed(ratios, 0.0)?.try_estimate(&[
        sample([1.0, 0.0, 0.0], 1.0)?,
        sample([0.0, 1.0, 0.0], 1.0)?,
        sample([0.0, 0.0, 1.0], 1.0)?,
    ])?;
    assert!(exact.diagnostics().is_isotropic());
    assert_eq!(exact.diagnostics().axis_confidence(), &[0.0, 0.0, 0.0]);

    let near_samples = [
        sample([1.0, 0.0, 0.0], 1.02)?,
        sample([0.0, 1.0, 0.0], 1.0)?,
        sample([0.0, 0.0, 1.0], 0.98)?,
    ];
    let strict =
        OrientationTensorEstimator::try_fixed(ratios, 0.01)?.try_estimate(&near_samples)?;
    let permissive =
        OrientationTensorEstimator::try_fixed(ratios, 0.02)?.try_estimate(&near_samples)?;
    assert!(!strict.diagnostics().is_isotropic());
    assert!(permissive.diagnostics().is_isotropic());
    Ok(())
}

#[test]
fn outlier_influence_identifies_the_unique_transverse_sample() -> TestResult {
    let estimator =
        OrientationTensorEstimator::try_fixed(PrincipalAxisRatios::try_new([3.0, 1.0])?, 0.05)?;
    let estimate = estimator.try_estimate(&[
        sample([1.0, 0.0], 1.0)?,
        sample([-1.0, 0.0], 1.0)?,
        sample([1.0, 0.0], 1.0)?,
        sample([0.0, 1.0], 1.0)?,
    ])?;

    assert_eq!(estimate.diagnostics().most_influential_sample(), Some(3));
    assert_close(
        estimate.influences()[3].normalized_tensor_change(),
        0.25,
        2.0e-16,
    );
    for influence in &estimate.influences()[..3] {
        assert!(
            influence.normalized_tensor_change()
                < estimate.influences()[3].normalized_tensor_change()
        );
    }
    Ok(())
}

#[test]
fn bounded_cross_validation_prefers_concentrated_candidate_deterministically() -> TestResult {
    let candidates = vec![
        PrincipalAxisRatios::try_new([1.0, 1.0])?,
        PrincipalAxisRatios::try_new([2.0, 1.0])?,
        PrincipalAxisRatios::try_new([4.0, 1.0])?,
    ];
    let samples = [
        sample([1.0, 0.05], 1.0)?,
        sample([-1.0, 0.02], 1.0)?,
        sample([1.0, -0.04], 1.0)?,
        sample([-1.0, -0.01], 1.0)?,
    ];
    let forward = OrientationTensorEstimator::try_cross_validated(candidates.clone(), 4.0, 0.05)?
        .try_estimate(&samples)?;
    let reverse = OrientationTensorEstimator::try_cross_validated(
        candidates.into_iter().rev().collect(),
        4.0,
        0.05,
    )?
    .try_estimate(&samples)?;

    assert_eq!(forward.axis_ratios().values(), &[4.0, 1.0]);
    assert_eq!(forward.axis_ratios(), reverse.axis_ratios());
    assert_eq!(forward.candidate_scores().len(), 3);
    assert_eq!(
        forward.diagnostics().selection_kind(),
        AxisRatioSelectionKind::LeaveOneOut
    );
    assert_eq!(forward.diagnostics().selected_maximum_ratio(), 4.0);
    Ok(())
}

#[test]
fn cross_validation_scores_repeated_fold_eigenspaces_rotation_invariantly() -> TestResult {
    let candidates = vec![
        PrincipalAxisRatios::try_new([1.0, 1.0, 1.0])?,
        PrincipalAxisRatios::try_new([2.0, 1.5, 1.0])?,
        PrincipalAxisRatios::try_new([4.0, 2.0, 1.0])?,
    ];
    let base = [
        sample([1.0, 0.0, 0.0], 3.0)?,
        sample([1.0, 0.0, 0.0], 2.0)?,
        sample([0.0, 1.0, 0.0], 1.0)?,
    ];
    let rotated = [
        sample([1.0, 0.0, 0.0], 3.0)?,
        sample([1.0, 0.0, 0.0], 2.0)?,
        sample([0.0, 0.0, 1.0], 1.0)?,
    ];
    let estimator = OrientationTensorEstimator::try_cross_validated(candidates, 4.0, 0.0)?;
    let base_estimate = estimator.try_estimate(&base)?;
    let rotated_estimate = estimator.try_estimate(&rotated)?;

    assert_eq!(base_estimate.axis_ratios(), rotated_estimate.axis_ratios());
    assert_eq!(base_estimate.candidate_scores().len(), 3);
    for (base_score, rotated_score) in base_estimate
        .candidate_scores()
        .iter()
        .zip(rotated_estimate.candidate_scores())
    {
        assert_eq!(base_score.ratios(), rotated_score.ratios());
        assert_close(base_score.score(), rotated_score.score(), 2.0e-15);
    }
    Ok(())
}

#[test]
fn cross_validation_preserves_total_mass_for_fully_unresolved_folds() -> TestResult {
    let smaller = PrincipalAxisRatios::try_new([3.0, 2.0, 1.0])?;
    let larger = PrincipalAxisRatios::try_new([4.0, 2.0, 1.0])?;
    let estimate =
        OrientationTensorEstimator::try_cross_validated(vec![smaller, larger], 4.0, 0.0)?
            .try_estimate(&[
                sample([1.0, 2.0, 2.0], f64::MAX)?,
                sample([1.0, 0.0, 0.0], 1.0)?,
                sample([0.0, 1.0, 0.0], 1.0)?,
                sample([0.0, 0.0, 1.0], 1.0)?,
            ])?;

    let scores = estimate.candidate_scores();
    assert_eq!(scores.len(), 2);
    assert_eq!(scores[0].ratios(), smaller);
    assert_eq!(scores[1].ratios(), larger);
    assert!(
        scores[0].score() < scores[1].score(),
        "expected [3,2,1] score {:.17e} below [4,2,1] score {:.17e}",
        scores[0].score(),
        scores[1].score()
    );
    assert_eq!(estimate.axis_ratios(), smaller);
    assert_close(scores[0].score() * f64::MAX, 1913.0 / 2646.0, 2.0e-14);
    assert_close(scores[1].score() * f64::MAX, 1654.0 / 1323.0, 2.0e-14);
    Ok(())
}

#[test]
fn extreme_finite_weights_keep_every_influence_inside_the_public_range() -> TestResult {
    let estimator =
        OrientationTensorEstimator::try_fixed(PrincipalAxisRatios::try_new([1.0, 1.0, 1.0])?, 0.0)?;
    let estimate = estimator.try_estimate(&[
        sample([1.0, 0.0, 0.0], f64::MAX)?,
        sample([0.0, -200.0, -166.0], 1.0)?,
    ])?;

    for influence in estimate.influences() {
        assert!(
            (0.0..=1.0).contains(&influence.normalized_tensor_change()),
            "sample {} influence is {:.17e}",
            influence.sample_index(),
            influence.normalized_tensor_change()
        );
    }
    assert!((0.0..=1.0).contains(&estimate.diagnostics().maximum_outlier_influence()));
    assert_eq!(estimate.influences()[0].normalized_tensor_change(), 1.0);
    Ok(())
}

#[test]
fn exact_cross_validation_ties_choose_lexicographically_smaller_ratios() -> TestResult {
    let smaller = PrincipalAxisRatios::try_new([1.0e100, 1.0])?;
    let larger = PrincipalAxisRatios::try_new([2.0e100, 1.0])?;
    let samples = [sample([1.0, 0.0], 1.0)?, sample([-1.0, 0.0], 1.0)?];

    let first =
        OrientationTensorEstimator::try_cross_validated(vec![larger, smaller], 2.0e100, 0.0)?
            .try_estimate(&samples)?;
    let second =
        OrientationTensorEstimator::try_cross_validated(vec![smaller, larger], 2.0e100, 0.0)?
            .try_estimate(&samples)?;

    assert_eq!(
        first.candidate_scores()[0].score().to_bits(),
        0.0_f64.to_bits()
    );
    assert_eq!(
        first.candidate_scores()[1].score().to_bits(),
        0.0_f64.to_bits()
    );
    assert_eq!(first.axis_ratios(), smaller);
    assert_eq!(second.axis_ratios(), smaller);
    Ok(())
}

#[test]
fn candidate_and_weight_failures_are_structured() -> TestResult {
    let direction = UnitDirection::try_new([1.0, 0.0])?;
    assert!(matches!(
        OrientationTensorSample::try_new(direction, f64::NAN),
        Err(OrientationTensorError::NonFiniteWeight { .. })
    ));
    assert!(matches!(
        OrientationTensorSample::try_new(direction, -1.0),
        Err(OrientationTensorError::NegativeWeight { .. })
    ));
    assert!(matches!(
        PrincipalAxisRatios::<2>::try_new([0.5, 1.0]),
        Err(OrientationTensorError::AxisRatioBelowOne { .. })
    ));
    assert!(matches!(
        PrincipalAxisRatios::<2>::try_new([1.0, 2.0]),
        Err(OrientationTensorError::UnorderedAxisRatios { .. })
    ));
    assert!(matches!(
        PrincipalAxisRatios::<2>::try_new([2.0, 1.5]),
        Err(OrientationTensorError::UnnormalizedAxisRatios { .. })
    ));
    let ratio_whose_normalized_share_underflows = 2.0_f64.powi(537);
    assert!(matches!(
        PrincipalAxisRatios::<3>::try_new([
            ratio_whose_normalized_share_underflows,
            ratio_whose_normalized_share_underflows,
            1.0,
        ]),
        Err(OrientationTensorError::NonRepresentableRatioSquare { axis: 2, .. })
    ));

    let unit = PrincipalAxisRatios::try_new([1.0, 1.0])?;
    assert!(matches!(
        OrientationTensorEstimator::<2>::try_cross_validated(Vec::new(), 2.0, 0.1),
        Err(OrientationTensorError::EmptyCandidateSet)
    ));
    assert!(matches!(
        OrientationTensorEstimator::try_cross_validated(vec![unit, unit], 2.0, 0.1),
        Err(OrientationTensorError::DuplicateCandidate { .. })
    ));
    let too_large = PrincipalAxisRatios::try_new([3.0, 1.0])?;
    assert!(matches!(
        OrientationTensorEstimator::try_cross_validated(vec![too_large], 2.0, 0.1),
        Err(OrientationTensorError::CandidateExceedsMaximum { .. })
    ));
    assert!(matches!(
        OrientationTensorEstimator::try_fixed(unit, f64::INFINITY),
        Err(OrientationTensorError::InvalidIsotropyThreshold { .. })
    ));
    Ok(())
}

#[test]
fn empty_zero_and_insufficient_cross_validation_samples_are_rejected() -> TestResult {
    let ratios = PrincipalAxisRatios::try_new([1.0, 1.0])?;
    let fixed = OrientationTensorEstimator::try_fixed(ratios, 0.0)?;
    assert!(matches!(
        fixed.try_estimate(&[]),
        Err(OrientationTensorError::EmptySamples)
    ));
    assert!(matches!(
        fixed.try_estimate(&[sample([1.0, 0.0], 0.0)?]),
        Err(OrientationTensorError::NoPositiveWeight)
    ));

    let cross_validated = OrientationTensorEstimator::try_cross_validated(vec![ratios], 1.0, 0.0)?;
    assert!(matches!(
        cross_validated.try_estimate(&[sample([1.0, 0.0], 1.0)?, sample([0.0, 1.0], 0.0)?,]),
        Err(OrientationTensorError::InsufficientPositiveSamples { positive: 1 })
    ));
    Ok(())
}

#[test]
fn estimator_and_result_are_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<OrientationTensorEstimator<1>>();
    assert_send_sync::<OrientationTensorEstimator<2>>();
    assert_send_sync::<OrientationTensorEstimator<3>>();
    assert_send_sync::<georbf::OrientationTensorEstimate<3>>();
}
