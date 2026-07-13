//! Constructs illustrative kernel metadata without selecting a radial formula.

use std::error::Error;

use georbf::{
    KernelDefiniteness, KernelDerivativeCapabilities, KernelDerivativeOrder, KernelDimensions,
    KernelMetadata, KernelParameterConstraint, KernelParameterDefinition, KernelParameterUnit,
    KernelSupport,
};

fn main() -> Result<(), Box<dyn Error>> {
    let parameters = [KernelParameterDefinition::try_new(
        "support_radius",
        KernelParameterUnit::CoordinateLength,
        KernelParameterConstraint::Positive,
        "Compact-support radius in the active coordinate length unit.",
    )?];
    parameters[0].validate_value(2.5)?;

    let metadata = KernelMetadata::try_new(
        "illustrative_family",
        KernelDefiniteness::StrictlyPositiveDefinite,
        KernelDimensions::try_new(false, true, true)?,
        KernelDerivativeCapabilities::try_new(
            KernelDerivativeOrder::Third,
            Some(KernelDerivativeOrder::Second),
        )?,
        KernelSupport::Compact {
            radius_parameter: "support_radius",
        },
        &parameters,
    )?;

    println!("kernel family: {}", metadata.name());
    println!("supports D=2: {}", metadata.dimensions().supports::<2>());
    println!(
        "Hessian with first-order center functional: {:?}",
        metadata
            .derivatives()
            .query_capability(KernelDerivativeOrder::Second, KernelDerivativeOrder::First,)
    );
    Ok(())
}
