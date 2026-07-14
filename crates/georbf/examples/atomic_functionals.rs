//! Construct and apply atomic scalar-field functionals.

use std::error::Error;
use std::io;

use georbf::{
    CenterRepresenter, FunctionalAtom, FunctionalExpr, FunctionalProvenance, FunctionalTerm,
    Gaussian, ObservationFunctional, Point, PolynomialSpace, RadialSeparation, ScalarFieldSample,
    SpatialKernelJet, UnitDirection,
};

fn main() -> Result<(), Box<dyn Error>> {
    let query = Point::try_new([1.0, 2.0])?;
    let center = Point::try_new([-0.5, 0.25])?;
    let direction = UnitDirection::try_new([1.0, -1.0])?;

    let observation_expression = FunctionalExpr::try_new([
        FunctionalTerm::try_new(
            2.0,
            FunctionalAtom::value(query, FunctionalProvenance::new(100)),
        )?,
        FunctionalTerm::try_new(
            -0.5,
            FunctionalAtom::directional_derivative(
                query,
                direction,
                FunctionalProvenance::new(101),
            ),
        )?,
    ])?;

    let sample = ScalarFieldSample::try_new(5.0, [3.0, -1.0])?;
    let sample_action = observation_expression.try_apply_samples(&[sample, sample])?;
    let polynomial_action =
        observation_expression.try_apply_polynomial(&PolynomialSpace::<2>::try_new(3)?)?;

    let observation = ObservationFunctional::new(observation_expression);
    let representer = CenterRepresenter::new(FunctionalExpr::try_new([FunctionalTerm::try_new(
        1.0,
        FunctionalAtom::value(center, FunctionalProvenance::new(200)),
    )?])?);
    let kernel = Gaussian::try_new(2.0)?;
    let kernel_action = observation.try_apply_kernel(
        &representer,
        |x, y| -> Result<SpatialKernelJet<2>, io::Error> {
            let separation = RadialSeparation::try_new(x, y)
                .map_err(|error| io::Error::other(error.to_string()))?;
            let radial = kernel
                .radial_jet(separation)
                .map_err(|error| io::Error::other(error.to_string()))?;
            SpatialKernelJet::try_new(separation, radial)
                .map_err(|error| io::Error::other(error.to_string()))
        },
    )?;

    println!("sample action: {sample_action:.6}");
    println!("polynomial actions: {polynomial_action:?}");
    println!("Gaussian observation/center action: {kernel_action:.6}");
    Ok(())
}
