//! Contract tests for kernel metadata and derivative capability classification.

use std::error::Error;

use georbf::{
    CpdOrder, CpdOrderError, KernelDefiniteness, KernelDerivativeCapabilities,
    KernelDerivativeCapabilitiesError, KernelDerivativeCapability, KernelDerivativeOrder,
    KernelDimensions, KernelDimensionsError, KernelMetadata, KernelMetadataError,
    KernelParameterConstraint, KernelParameterDefinition, KernelParameterDefinitionError,
    KernelParameterUnit, KernelParameterValueError, KernelSupport,
};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

fn length_scale_definition() -> TestResult<KernelParameterDefinition<'static>> {
    Ok(KernelParameterDefinition::try_new(
        "length_scale",
        KernelParameterUnit::CoordinateLength,
        KernelParameterConstraint::Positive,
        "Positive scale measured in the active coordinate length unit.",
    )?)
}

fn support_radius_definition() -> TestResult<KernelParameterDefinition<'static>> {
    Ok(KernelParameterDefinition::try_new(
        "support_radius",
        KernelParameterUnit::CoordinateLength,
        KernelParameterConstraint::Positive,
        "Positive compact-support radius in the active coordinate length unit.",
    )?)
}

fn third_order_everywhere() -> TestResult<KernelDerivativeCapabilities> {
    Ok(KernelDerivativeCapabilities::try_new(
        KernelDerivativeOrder::Third,
        Some(KernelDerivativeOrder::Third),
    )?)
}

#[test]
fn definiteness_keeps_spd_and_positive_cpd_order_distinct() -> TestResult {
    assert!(matches!(CpdOrder::try_new(0), Err(CpdOrderError::Zero)));
    let order_one = CpdOrder::try_new(1)?;
    assert_eq!(order_one.maximum_polynomial_degree(), 0);

    let order = CpdOrder::try_new(3)?;
    assert_eq!(order.get(), 3);
    assert_eq!(order.maximum_polynomial_degree(), 2);

    let largest_order = CpdOrder::try_new(usize::MAX)?;
    assert_eq!(largest_order.get(), usize::MAX);
    assert_eq!(largest_order.maximum_polynomial_degree(), usize::MAX - 1);

    let spd = KernelDefiniteness::StrictlyPositiveDefinite;
    let cpd = KernelDefiniteness::ConditionallyPositiveDefinite { order };
    assert_ne!(spd, cpd);
    assert!(matches!(
        cpd,
        KernelDefiniteness::ConditionallyPositiveDefinite { order: actual }
            if actual == order
    ));
    Ok(())
}

#[test]
fn dimension_sets_are_nonempty_and_query_only_supported_dimensions() -> TestResult {
    assert!(matches!(
        KernelDimensions::try_new(false, false, false),
        Err(KernelDimensionsError::Empty)
    ));

    let all = KernelDimensions::try_new(true, true, true)?;
    assert!(all.supports::<1>());
    assert!(all.supports::<2>());
    assert!(all.supports::<3>());

    let planar = KernelDimensions::try_new(false, true, false)?;
    assert!(!planar.supports::<1>());
    assert!(planar.supports::<2>());
    assert!(!planar.supports::<3>());
    assert_eq!(planar.flags(), [false, true, false]);
    Ok(())
}

#[test]
fn derivative_capabilities_distinguish_center_away_and_unsupported() -> TestResult {
    let capabilities = KernelDerivativeCapabilities::try_new(
        KernelDerivativeOrder::Third,
        Some(KernelDerivativeOrder::First),
    )?;

    assert_eq!(
        capabilities.capability(KernelDerivativeOrder::Value),
        KernelDerivativeCapability::SupportedEverywhere
    );
    assert_eq!(
        capabilities.capability(KernelDerivativeOrder::First),
        KernelDerivativeCapability::SupportedEverywhere
    );
    assert_eq!(
        capabilities.capability(KernelDerivativeOrder::Second),
        KernelDerivativeCapability::SupportedAwayFromCenters
    );
    assert_eq!(
        capabilities.capability(KernelDerivativeOrder::Third),
        KernelDerivativeCapability::SupportedAwayFromCenters
    );
    assert_eq!(
        capabilities.maximum_away_order(),
        KernelDerivativeOrder::Third
    );
    assert_eq!(
        capabilities.maximum_center_order(),
        Some(KernelDerivativeOrder::First)
    );

    let away_only = KernelDerivativeCapabilities::try_new(KernelDerivativeOrder::Value, None)?;
    assert_eq!(
        away_only.capability(KernelDerivativeOrder::Value),
        KernelDerivativeCapability::SupportedAwayFromCenters
    );
    assert!(matches!(
        KernelDerivativeCapabilities::try_new(
            KernelDerivativeOrder::First,
            Some(KernelDerivativeOrder::Second),
        ),
        Err(KernelDerivativeCapabilitiesError::CenterExceedsAway {
            away: KernelDerivativeOrder::First,
            center: KernelDerivativeOrder::Second,
        })
    ));
    Ok(())
}

#[test]
fn matrix_and_query_capabilities_include_center_derivative_demand() -> TestResult {
    let capabilities = KernelDerivativeCapabilities::try_new(
        KernelDerivativeOrder::Third,
        Some(KernelDerivativeOrder::Second),
    )?;

    assert_eq!(
        capabilities.matrix_capability(KernelDerivativeOrder::First, KernelDerivativeOrder::First,),
        KernelDerivativeCapability::SupportedEverywhere
    );
    assert_eq!(
        capabilities.query_capability(KernelDerivativeOrder::Second, KernelDerivativeOrder::First,),
        KernelDerivativeCapability::SupportedAwayFromCenters
    );
    assert_eq!(
        capabilities
            .matrix_capability(KernelDerivativeOrder::Second, KernelDerivativeOrder::Second,),
        KernelDerivativeCapability::Unsupported
    );
    assert_eq!(
        capabilities.query_capability(KernelDerivativeOrder::Third, KernelDerivativeOrder::Third,),
        KernelDerivativeCapability::Unsupported
    );
    Ok(())
}

#[test]
fn combined_capability_matches_the_independent_total_order_table() -> TestResult {
    use KernelDerivativeCapability::{
        SupportedAwayFromCenters as Away, SupportedEverywhere as Everywhere, Unsupported,
    };

    let orders = [
        KernelDerivativeOrder::Value,
        KernelDerivativeOrder::First,
        KernelDerivativeOrder::Second,
        KernelDerivativeOrder::Third,
    ];
    // Independent truth for away-through third and center-through second.
    // Each cell is classified by row order + column order, without consulting
    // the implementation's private numeric representation.
    let expected = [
        [Everywhere, Everywhere, Everywhere, Away],
        [Everywhere, Everywhere, Away, Unsupported],
        [Everywhere, Away, Unsupported, Unsupported],
        [Away, Unsupported, Unsupported, Unsupported],
    ];
    let capabilities = KernelDerivativeCapabilities::try_new(
        KernelDerivativeOrder::Third,
        Some(KernelDerivativeOrder::Second),
    )?;

    for (row, first) in orders.iter().copied().enumerate() {
        for (column, second) in orders.iter().copied().enumerate() {
            assert_eq!(
                capabilities.matrix_capability(first, second),
                expected[row][column],
                "matrix capability mismatch at ({first:?}, {second:?})"
            );
            assert_eq!(
                capabilities.query_capability(first, second),
                expected[row][column],
                "query capability mismatch at ({first:?}, {second:?})"
            );
        }
    }
    Ok(())
}

#[test]
fn parameter_definitions_require_explicit_names_units_and_documentation() -> TestResult {
    for invalid_name in [
        "",
        "1length_scale",
        "_length_scale",
        "LengthScale",
        "length-scale",
        "length scale",
        "length__scale",
        "length_",
        "l\u{e9}ngth_scale",
    ] {
        assert!(matches!(
            KernelParameterDefinition::try_new(
                invalid_name,
                KernelParameterUnit::CoordinateLength,
                KernelParameterConstraint::Positive,
                "documented",
            ),
            Err(KernelParameterDefinitionError::InvalidName)
        ));
    }
    assert!(matches!(
        KernelParameterDefinition::try_new(
            "shape_parameter",
            KernelParameterUnit::Dimensionless,
            KernelParameterConstraint::Positive,
            "generic parameter",
        ),
        Err(KernelParameterDefinitionError::ReservedGenericName)
    ));
    assert!(matches!(
        KernelParameterDefinition::try_new(
            "length_scale",
            KernelParameterUnit::CoordinateLength,
            KernelParameterConstraint::Positive,
            "  ",
        ),
        Err(KernelParameterDefinitionError::EmptyDescription)
    ));

    let definition = length_scale_definition()?;
    assert_eq!(definition.name(), "length_scale");
    assert_eq!(definition.unit(), KernelParameterUnit::CoordinateLength);
    assert_eq!(definition.constraint(), KernelParameterConstraint::Positive);
    assert!(!definition.description().is_empty());

    let indexed = KernelParameterDefinition::try_new(
        "length_scale2",
        KernelParameterUnit::CoordinateLength,
        KernelParameterConstraint::Positive,
        "A second explicit positive length scale.",
    )?;
    assert_eq!(indexed.name(), "length_scale2");
    Ok(())
}

#[test]
fn parameter_value_constraints_reject_nonfinite_and_out_of_domain_values() -> TestResult {
    let finite = KernelParameterDefinition::try_new(
        "power",
        KernelParameterUnit::Dimensionless,
        KernelParameterConstraint::Finite,
        "Finite dimensionless exponent.",
    )?;
    let nonnegative = KernelParameterDefinition::try_new(
        "smoothness",
        KernelParameterUnit::Dimensionless,
        KernelParameterConstraint::NonNegative,
        "Finite nonnegative smoothness value.",
    )?;
    let positive = length_scale_definition()?;

    finite.validate_value(f64::MIN)?;
    finite.validate_value(f64::MAX)?;
    nonnegative.validate_value(-0.0)?;
    positive.validate_value(f64::from_bits(1))?;
    assert!(matches!(
        finite.validate_value(f64::NAN),
        Err(KernelParameterValueError::NonFinite { .. })
    ));
    assert!(matches!(
        nonnegative.validate_value(-1.0),
        Err(KernelParameterValueError::ViolatesConstraint {
            constraint: KernelParameterConstraint::NonNegative,
            ..
        })
    ));
    assert!(matches!(
        positive.validate_value(0.0),
        Err(KernelParameterValueError::ViolatesConstraint {
            constraint: KernelParameterConstraint::Positive,
            ..
        })
    ));
    assert!(matches!(
        positive.validate_value(-0.0),
        Err(KernelParameterValueError::ViolatesConstraint {
            constraint: KernelParameterConstraint::Positive,
            ..
        })
    ));
    assert!(matches!(
        positive.validate_value(f64::INFINITY),
        Err(KernelParameterValueError::NonFinite { .. })
    ));
    assert!(matches!(
        finite.validate_value(f64::NEG_INFINITY),
        Err(KernelParameterValueError::NonFinite { .. })
    ));
    Ok(())
}

#[test]
fn kernel_names_and_parameter_sets_are_consistency_checked() -> TestResult {
    let dimensions = KernelDimensions::try_new(true, true, true)?;
    let derivatives = third_order_everywhere()?;
    let length_scale = length_scale_definition()?;

    assert!(matches!(
        KernelMetadata::try_new(
            "Invalid-Kernel",
            KernelDefiniteness::StrictlyPositiveDefinite,
            dimensions,
            derivatives,
            KernelSupport::Global,
            &[],
        ),
        Err(KernelMetadataError::InvalidKernelName)
    ));

    let duplicates = [length_scale, length_scale];
    assert!(matches!(
        KernelMetadata::try_new(
            "example_kernel",
            KernelDefiniteness::StrictlyPositiveDefinite,
            dimensions,
            derivatives,
            KernelSupport::Global,
            &duplicates,
        ),
        Err(KernelMetadataError::DuplicateParameter {
            first: 0,
            second: 1
        })
    ));

    let parameters = [length_scale];
    let metadata = KernelMetadata::try_new(
        "example_kernel",
        KernelDefiniteness::StrictlyPositiveDefinite,
        dimensions,
        derivatives,
        KernelSupport::Global,
        &parameters,
    )?;
    assert_eq!(metadata.name(), "example_kernel");
    assert_eq!(metadata.parameters(), &parameters);
    Ok(())
}

#[test]
fn compact_support_is_consistency_checked() -> TestResult {
    let dimensions = KernelDimensions::try_new(true, true, true)?;
    let derivatives = third_order_everywhere()?;
    let length_scale = length_scale_definition()?;
    let support_radius = support_radius_definition()?;

    let only_length_scale = [length_scale];
    assert!(matches!(
        KernelMetadata::try_new(
            "example_kernel",
            KernelDefiniteness::StrictlyPositiveDefinite,
            dimensions,
            derivatives,
            KernelSupport::Compact {
                radius_parameter: "support_radius",
            },
            &only_length_scale,
        ),
        Err(KernelMetadataError::MissingCompactRadiusParameter)
    ));

    let wrong_unit = [KernelParameterDefinition::try_new(
        "support_radius",
        KernelParameterUnit::Dimensionless,
        KernelParameterConstraint::Positive,
        "Incorrectly dimensionless support radius.",
    )?];
    assert!(matches!(
        KernelMetadata::try_new(
            "example_kernel",
            KernelDefiniteness::StrictlyPositiveDefinite,
            dimensions,
            derivatives,
            KernelSupport::Compact {
                radius_parameter: "support_radius",
            },
            &wrong_unit,
        ),
        Err(KernelMetadataError::InvalidCompactRadiusUnit)
    ));

    let wrong_constraint = [KernelParameterDefinition::try_new(
        "support_radius",
        KernelParameterUnit::CoordinateLength,
        KernelParameterConstraint::NonNegative,
        "Incorrectly permits a zero support radius.",
    )?];
    assert!(matches!(
        KernelMetadata::try_new(
            "example_kernel",
            KernelDefiniteness::StrictlyPositiveDefinite,
            dimensions,
            derivatives,
            KernelSupport::Compact {
                radius_parameter: "support_radius",
            },
            &wrong_constraint,
        ),
        Err(KernelMetadataError::InvalidCompactRadiusConstraint)
    ));

    let parameters = [length_scale, support_radius];
    let metadata = KernelMetadata::try_new(
        "example_kernel",
        KernelDefiniteness::StrictlyPositiveDefinite,
        dimensions,
        derivatives,
        KernelSupport::Compact {
            radius_parameter: "support_radius",
        },
        &parameters,
    )?;
    assert_eq!(metadata.name(), "example_kernel");
    assert_eq!(metadata.parameters(), &parameters);
    assert_eq!(metadata.parameter("support_radius"), Some(&support_radius));
    assert!(metadata.parameter("core_radius").is_none());
    assert_eq!(
        metadata.support(),
        KernelSupport::Compact {
            radius_parameter: "support_radius"
        }
    );
    assert!(metadata.dimensions().supports::<3>());
    assert_eq!(metadata.derivatives(), derivatives);
    assert_eq!(
        metadata.definiteness(),
        KernelDefiniteness::StrictlyPositiveDefinite
    );
    Ok(())
}

#[test]
fn metadata_values_are_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<CpdOrder>();
    assert_send_sync::<KernelDefiniteness>();
    assert_send_sync::<KernelDimensions>();
    assert_send_sync::<KernelDerivativeCapabilities>();
    assert_send_sync::<KernelParameterDefinition<'static>>();
    assert_send_sync::<KernelSupport<'static>>();
    assert_send_sync::<KernelMetadata<'static>>();
}
