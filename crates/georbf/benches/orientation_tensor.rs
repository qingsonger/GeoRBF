//! Dependency-free deterministic benchmark for orientation-tensor estimation.

use std::error::Error;
use std::hint::black_box;
use std::time::Instant;

use georbf::{
    OrientationTensorEstimator, OrientationTensorSample, PrincipalAxisRatios, UnitDirection,
};

fn main() -> Result<(), Box<dyn Error>> {
    let smoke = std::env::args().any(|argument| argument == "--smoke");
    let iterations = if smoke { 2_000 } else { 100_000 };
    let samples = [
        OrientationTensorSample::try_new(UnitDirection::try_new([1.0, 0.1, 0.0])?, 3.0)?,
        OrientationTensorSample::try_new(UnitDirection::try_new([1.0, -0.1, 0.05])?, 2.0)?,
        OrientationTensorSample::try_new(UnitDirection::try_new([0.9, 0.2, -0.1])?, 1.0)?,
        OrientationTensorSample::try_new(UnitDirection::try_new([0.0, 1.0, 0.0])?, 0.25)?,
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

    let started = Instant::now();
    let mut checksum = 0.0;
    for _ in 0..iterations {
        let estimate = black_box(estimator.try_estimate(black_box(&samples))?);
        checksum += estimate.normalized_eigenvalues()[0]
            + estimate.axis_ratios().maximum()
            + estimate.diagnostics().maximum_outlier_influence();
    }
    let elapsed = started.elapsed();
    println!(
        "orientation-tensor D=3 iterations={iterations} elapsed={:.6}s ns/estimate={:.2} checksum={:.17e}",
        elapsed.as_secs_f64(),
        elapsed.as_secs_f64() * 1.0e9 / f64::from(iterations),
        black_box(checksum),
    );
    Ok(())
}
