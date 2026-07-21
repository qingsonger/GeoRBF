//! Estimate principal axes and select bounded relative axis ratios.

use std::error::Error;

use georbf::{
    OrientationTensorEstimator, OrientationTensorSample, PrincipalAxisRatios, UnitDirection,
};

fn main() -> Result<(), Box<dyn Error>> {
    let samples = [
        OrientationTensorSample::try_new(UnitDirection::try_new([1.0, 0.1, 0.0])?, 3.0)?,
        OrientationTensorSample::try_new(UnitDirection::try_new([-1.0, 0.05, 0.0])?, 2.0)?,
        OrientationTensorSample::try_new(UnitDirection::try_new([0.9, -0.2, 0.1])?, 1.0)?,
    ];
    let estimator = OrientationTensorEstimator::try_cross_validated(
        vec![
            PrincipalAxisRatios::try_new([1.0, 1.0, 1.0])?,
            PrincipalAxisRatios::try_new([2.0, 1.5, 1.0])?,
            PrincipalAxisRatios::try_new([4.0, 2.0, 1.0])?,
        ],
        4.0,
        0.05,
    )?;
    let estimate = estimator.try_estimate(&samples)?;

    println!("tensor: {:?}", estimate.tensor());
    println!(
        "principal axes: {:?}",
        estimate
            .principal_axes()
            .map(georbf::UnitDirection::into_components)
    );
    println!("selected ratios: {:?}", estimate.axis_ratios().values());
    println!("isotropic: {}", estimate.diagnostics().is_isotropic());
    println!(
        "maximum outlier influence: {}",
        estimate.diagnostics().maximum_outlier_influence()
    );
    Ok(())
}
