//! Actual allocator-call regression for orientation-tensor estimation.

use std::error::Error;

use georbf::{
    OrientationTensorEstimator, OrientationTensorSample, PrincipalAxisRatios, UnitDirection,
};

type TestResult = Result<(), Box<dyn Error>>;

fn allocation_count(sample_count: usize, cross_validated: bool) -> Result<u64, Box<dyn Error>> {
    let ratios = PrincipalAxisRatios::try_new([3.0, 2.0, 1.0])?;
    let estimator = if cross_validated {
        OrientationTensorEstimator::try_cross_validated(
            vec![ratios, PrincipalAxisRatios::try_new([4.0, 2.0, 1.0])?],
            4.0,
            0.0,
        )?
    } else {
        OrientationTensorEstimator::try_fixed(ratios, 0.0)?
    };
    let directions = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    let samples = (0..sample_count)
        .map(|index| {
            Ok(OrientationTensorSample::try_new(
                UnitDirection::try_new(directions[index % directions.len()])?,
                1.0,
            )?)
        })
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;

    let warm_up = estimator.try_estimate(&samples)?;
    drop(warm_up);

    let mut estimate = None;
    let allocation_info = allocation_counter::measure(|| {
        estimate = Some(estimator.try_estimate(&samples));
    });
    let estimate = estimate.ok_or("measurement did not execute estimator")??;
    drop(estimate);
    Ok(allocation_info.count_total)
}

#[test]
fn actual_allocations_are_independent_of_sample_count() -> TestResult {
    let fixed_four = allocation_count(4, false)?;
    let fixed_sixteen = allocation_count(16, false)?;
    assert_eq!(fixed_four, 2);
    assert_eq!(fixed_sixteen, fixed_four);

    let cross_validated_four = allocation_count(4, true)?;
    let cross_validated_sixteen = allocation_count(16, true)?;
    assert_eq!(cross_validated_four, 5);
    assert_eq!(cross_validated_sixteen, cross_validated_four);
    Ok(())
}
