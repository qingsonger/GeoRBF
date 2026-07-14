//! Metadata and derivative capabilities shared by all kernel families.
//!
//! This module describes kernels without implementing a radial formula. It
//! contains no geological semantics, fitted-model policy, or adapter surface.
//! Dimension queries remain sealed to D=1, D=2, and D=3:
//!
//! ```compile_fail
//! use georbf::KernelDimensions;
//!
//! fn unsupported(dimensions: KernelDimensions) -> bool {
//!     dimensions.supports::<4>()
//! }
//! ```
//!
//! ```compile_fail
//! use georbf::KernelDimensions;
//!
//! fn unsupported(dimensions: KernelDimensions) -> bool {
//!     dimensions.supports::<0>()
//! }
//! ```

use std::error::Error;
use std::fmt;

use crate::dimension::{Dim, SupportedDimension};

mod polyharmonic;
mod smooth_global;
mod wendland;

pub use polyharmonic::{
    PolyharmonicSpline, PolyharmonicSplineConstructionError, PolyharmonicSplineEvaluationError,
    SurfaceSpline, SurfaceSplineConstructionError,
};
pub use smooth_global::{
    Gaussian, InverseMultiquadric, Matern, MaternSmoothness, Multiquadric,
    SmoothKernelConstructionError, SmoothKernelEvaluationError, SmoothKernelFamily,
};
pub use wendland::{
    Wendland, WendlandConstructionError, WendlandEvaluationError, WendlandSmoothness,
};

/// Positive conditional-positive-definiteness order.
///
/// For CPD order `m`, `GeoRBF`'s later polynomial side space is the complete
/// space of total degree at most `m - 1`. Thus order one requires constants,
/// order two requires constants and linear terms, and so on. This type records
/// that contract without constructing the polynomial space.
///
/// Strictly positive definite kernels use a distinct
/// [`KernelDefiniteness`] variant, so a CPD order of zero is not represented.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[must_use]
pub struct CpdOrder(usize);

impl CpdOrder {
    /// Constructs a positive CPD order.
    ///
    /// # Errors
    ///
    /// Returns [`CpdOrderError::Zero`] when `order` is zero.
    pub const fn try_new(order: usize) -> Result<Self, CpdOrderError> {
        if order == 0 {
            Err(CpdOrderError::Zero)
        } else {
            Ok(Self(order))
        }
    }

    /// Returns the positive CPD order.
    #[must_use]
    pub const fn get(self) -> usize {
        self.0
    }

    /// Returns the maximum total degree of the required complete polynomial space.
    ///
    /// This subtraction cannot underflow because construction rejects order
    /// zero and the representation is private.
    #[must_use]
    pub const fn maximum_polynomial_degree(self) -> usize {
        self.0 - 1
    }
}

/// Error returned when constructing a CPD order.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CpdOrderError {
    /// CPD order zero is invalid because SPD has its own classification.
    Zero,
}

impl fmt::Display for CpdOrderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("conditionally positive definite order must be positive")
    }
}

impl Error for CpdOrderError {}

/// Positive-definiteness classification declared by a kernel.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum KernelDefiniteness {
    /// Strict positive definiteness in every declared supported dimension.
    StrictlyPositiveDefinite,
    /// Conditional positive definiteness with the stated polynomial order in
    /// every declared supported dimension.
    ConditionallyPositiveDefinite {
        /// Positive CPD order `m`, requiring complete polynomials through
        /// total degree `m - 1` in the later polynomial-space requirement.
        order: CpdOrder,
    },
}

/// Spatial derivative order understood by the kernel capability model.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum KernelDerivativeOrder {
    /// Kernel value, or derivative order zero.
    Value = 0,
    /// First spatial derivative or gradient.
    First = 1,
    /// Second spatial derivative or Hessian.
    Second = 2,
    /// Third spatial derivative tensor.
    Third = 3,
}

impl KernelDerivativeOrder {
    const fn index(self) -> u8 {
        self as u8
    }

    const fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(Self::Value),
            1 => Some(Self::First),
            2 => Some(Self::Second),
            3 => Some(Self::Third),
            _ => None,
        }
    }
}

/// Availability of a requested spatial derivative.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum KernelDerivativeCapability {
    /// The derivative is defined at centers and at every positive separation.
    SupportedEverywhere,
    /// The derivative is defined only for every positive query-center separation.
    ///
    /// For a compactly supported kernel, positive separations include its
    /// support boundary and zero exterior branch.
    SupportedAwayFromCenters,
    /// The derivative is not supplied by the kernel.
    Unsupported,
}

/// Maximum derivative orders away from centers and at smooth centers.
///
/// Support is hierarchical: declaring an order also declares every lower
/// order. Center support may be absent or lower than away support, but can
/// never exceed it. Away support covers the whole strictly positive radial
/// domain, including a compact kernel's support boundary; it is not merely an
/// interior-formula capability.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub struct KernelDerivativeCapabilities {
    away_through: KernelDerivativeOrder,
    center_through: Option<KernelDerivativeOrder>,
}

impl KernelDerivativeCapabilities {
    /// Constructs a consistent derivative-capability declaration.
    ///
    /// `center_through = None` means even the kernel value is unavailable at
    /// coincident points.
    ///
    /// # Errors
    ///
    /// Returns [`KernelDerivativeCapabilitiesError::CenterExceedsAway`] when
    /// the smooth-center order is greater than the away order.
    pub const fn try_new(
        away_through: KernelDerivativeOrder,
        center_through: Option<KernelDerivativeOrder>,
    ) -> Result<Self, KernelDerivativeCapabilitiesError> {
        if let Some(center) = center_through
            && center.index() > away_through.index()
        {
            return Err(KernelDerivativeCapabilitiesError::CenterExceedsAway {
                away: away_through,
                center,
            });
        }
        Ok(Self {
            away_through,
            center_through,
        })
    }

    /// Returns the maximum derivative order supported away from centers.
    #[must_use]
    pub const fn maximum_away_order(self) -> KernelDerivativeOrder {
        self.away_through
    }

    /// Returns the maximum smooth-center order, or `None` if center values fail.
    #[must_use]
    pub const fn maximum_center_order(self) -> Option<KernelDerivativeOrder> {
        self.center_through
    }

    /// Classifies one spatial derivative order.
    #[must_use]
    pub const fn capability(self, order: KernelDerivativeOrder) -> KernelDerivativeCapability {
        if let Some(center) = self.center_through
            && order.index() <= center.index()
        {
            return KernelDerivativeCapability::SupportedEverywhere;
        }
        if order.index() <= self.away_through.index() {
            KernelDerivativeCapability::SupportedAwayFromCenters
        } else {
            KernelDerivativeCapability::Unsupported
        }
    }

    /// Classifies matrix demand as observation order plus center-functional order.
    #[must_use]
    pub const fn matrix_capability(
        self,
        observation_order: KernelDerivativeOrder,
        center_order: KernelDerivativeOrder,
    ) -> KernelDerivativeCapability {
        self.combined_capability(observation_order, center_order)
    }

    /// Classifies query demand as output order plus center-functional order.
    #[must_use]
    pub const fn query_capability(
        self,
        output_order: KernelDerivativeOrder,
        center_order: KernelDerivativeOrder,
    ) -> KernelDerivativeCapability {
        self.combined_capability(output_order, center_order)
    }

    const fn combined_capability(
        self,
        first: KernelDerivativeOrder,
        second: KernelDerivativeOrder,
    ) -> KernelDerivativeCapability {
        let combined = first.index() + second.index();
        match KernelDerivativeOrder::from_index(combined) {
            Some(order) => self.capability(order),
            None => KernelDerivativeCapability::Unsupported,
        }
    }
}

/// Error returned by derivative-capability construction.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum KernelDerivativeCapabilitiesError {
    /// Center smoothness was declared higher than away smoothness.
    CenterExceedsAway {
        /// Maximum away-from-center derivative order.
        away: KernelDerivativeOrder,
        /// Invalid maximum center derivative order.
        center: KernelDerivativeOrder,
    },
}

impl fmt::Display for KernelDerivativeCapabilitiesError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CenterExceedsAway { away, center } => write!(
                formatter,
                "center derivative order {center:?} exceeds away order {away:?}"
            ),
        }
    }
}

impl Error for KernelDerivativeCapabilitiesError {}

/// Nonempty subset of the three supported spatial dimensions.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub struct KernelDimensions {
    flags: [bool; 3],
}

impl KernelDimensions {
    /// Constructs dimension support flags for D=1, D=2, and D=3.
    ///
    /// # Errors
    ///
    /// Returns [`KernelDimensionsError::Empty`] when no dimension is selected.
    pub const fn try_new(
        supports_1d: bool,
        supports_2d: bool,
        supports_3d: bool,
    ) -> Result<Self, KernelDimensionsError> {
        if supports_1d || supports_2d || supports_3d {
            Ok(Self {
                flags: [supports_1d, supports_2d, supports_3d],
            })
        } else {
            Err(KernelDimensionsError::Empty)
        }
    }

    /// Returns whether the kernel supports the compile-time dimension `D`.
    #[must_use]
    pub const fn supports<const D: usize>(self) -> bool
    where
        Dim<D>: SupportedDimension,
    {
        match D {
            1 => self.flags[0],
            2 => self.flags[1],
            3 => self.flags[2],
            _ => false,
        }
    }

    /// Returns support flags ordered as D=1, D=2, and D=3.
    #[must_use]
    pub const fn flags(self) -> [bool; 3] {
        self.flags
    }
}

/// Error returned when constructing a kernel dimension set.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum KernelDimensionsError {
    /// A kernel must support at least one of D=1, D=2, or D=3.
    Empty,
}

impl fmt::Display for KernelDimensionsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("kernel dimension support cannot be empty")
    }
}

impl Error for KernelDimensionsError {}

/// Physical dimension used to interpret a kernel parameter.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum KernelParameterUnit {
    /// A pure number with no physical length dimension.
    Dimensionless,
    /// A length expressed in the active coordinate length unit.
    CoordinateLength,
    /// Reciprocal of the active coordinate length unit.
    InverseCoordinateLength,
}

/// Scalar domain enforced for a kernel parameter value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum KernelParameterConstraint {
    /// Any finite value.
    Finite,
    /// A finite value greater than or equal to zero.
    NonNegative,
    /// A finite value strictly greater than zero.
    Positive,
}

/// Named, unit-documented definition of one kernel parameter.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub struct KernelParameterDefinition<'a> {
    name: &'a str,
    unit: KernelParameterUnit,
    constraint: KernelParameterConstraint,
    description: &'a str,
}

impl<'a> KernelParameterDefinition<'a> {
    /// Constructs a validated parameter definition.
    ///
    /// Names use deterministic lower snake case. The generic name
    /// `shape_parameter` is reserved because every kernel parameter must state
    /// its mathematical role explicitly.
    ///
    /// # Errors
    ///
    /// Returns a [`KernelParameterDefinitionError`] for an invalid name,
    /// reserved generic name, or empty description.
    pub fn try_new(
        name: &'a str,
        unit: KernelParameterUnit,
        constraint: KernelParameterConstraint,
        description: &'a str,
    ) -> Result<Self, KernelParameterDefinitionError> {
        if !is_lower_snake_case(name) {
            return Err(KernelParameterDefinitionError::InvalidName);
        }
        if name == "shape_parameter" {
            return Err(KernelParameterDefinitionError::ReservedGenericName);
        }
        if description.trim().is_empty() {
            return Err(KernelParameterDefinitionError::EmptyDescription);
        }
        Ok(Self {
            name,
            unit,
            constraint,
            description,
        })
    }

    /// Returns the explicit lower-snake-case parameter name.
    #[must_use]
    pub const fn name(self) -> &'a str {
        self.name
    }

    /// Returns the parameter's physical unit dimension.
    #[must_use]
    pub const fn unit(self) -> KernelParameterUnit {
        self.unit
    }

    /// Returns the finite scalar constraint for parameter values.
    #[must_use]
    pub const fn constraint(self) -> KernelParameterConstraint {
        self.constraint
    }

    /// Returns the nonempty unit-aware parameter description.
    #[must_use]
    pub const fn description(self) -> &'a str {
        self.description
    }

    /// Validates one configured value against this definition.
    ///
    /// # Errors
    ///
    /// Returns [`KernelParameterValueError::NonFinite`] for NaN or infinity,
    /// or [`KernelParameterValueError::ViolatesConstraint`] when the finite
    /// value is outside the declared scalar domain.
    pub fn validate_value(self, value: f64) -> Result<(), KernelParameterValueError> {
        if !value.is_finite() {
            return Err(KernelParameterValueError::NonFinite { value });
        }
        let valid = match self.constraint {
            KernelParameterConstraint::Finite => true,
            KernelParameterConstraint::NonNegative => value >= 0.0,
            KernelParameterConstraint::Positive => value > 0.0,
        };
        if valid {
            Ok(())
        } else {
            Err(KernelParameterValueError::ViolatesConstraint {
                value,
                constraint: self.constraint,
            })
        }
    }
}

/// Error returned by kernel parameter-definition validation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum KernelParameterDefinitionError {
    /// The name is not nonempty deterministic lower snake case.
    InvalidName,
    /// The prohibited ambiguous name `shape_parameter` was used.
    ReservedGenericName,
    /// The parameter description contains only whitespace or is empty.
    EmptyDescription,
}

impl fmt::Display for KernelParameterDefinitionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidName => formatter
                .write_str("kernel parameter name must be nonempty deterministic lower snake case"),
            Self::ReservedGenericName => {
                formatter.write_str("generic kernel parameter name `shape_parameter` is forbidden")
            }
            Self::EmptyDescription => {
                formatter.write_str("kernel parameter description cannot be empty")
            }
        }
    }
}

impl Error for KernelParameterDefinitionError {}

/// Error returned by configured kernel parameter-value validation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum KernelParameterValueError {
    /// The value is NaN or infinite.
    NonFinite {
        /// Rejected numeric value.
        value: f64,
    },
    /// A finite value violates its declared scalar constraint.
    ViolatesConstraint {
        /// Rejected finite value.
        value: f64,
        /// Constraint the value failed.
        constraint: KernelParameterConstraint,
    },
}

impl fmt::Display for KernelParameterValueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFinite { value } => {
                write!(
                    formatter,
                    "kernel parameter value must be finite, got {value}"
                )
            }
            Self::ViolatesConstraint { value, constraint } => write!(
                formatter,
                "kernel parameter value {value} violates {constraint:?} constraint"
            ),
        }
    }
}

impl Error for KernelParameterValueError {}

/// Whether a kernel has global or parameterized compact support.
///
/// For [`KernelSupport::Compact`] with configured radius `rho`, the radial
/// kernel and its supported derivatives use the exact zero extension for
/// `r >= rho`. The interior formula's one-sided derivatives at `rho` must
/// match that zero extension through the declared away derivative order.
/// Metadata records this promise but does not prove a concrete formula.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum KernelSupport<'a> {
    /// The radial formula is not compactly supported.
    Global,
    /// Compact support controlled by a named radius parameter.
    Compact {
        /// Name of a positive [`KernelParameterUnit::CoordinateLength`]
        /// parameter giving `rho` in the radial coordinate's active length unit.
        radius_parameter: &'a str,
    },
}

/// Complete static description shared by a concrete kernel family.
///
/// The metadata borrows parameter definitions and performs no heap allocation.
/// It does not contain configured parameter values or a radial implementation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub struct KernelMetadata<'a> {
    name: &'a str,
    definiteness: KernelDefiniteness,
    dimensions: KernelDimensions,
    derivatives: KernelDerivativeCapabilities,
    support: KernelSupport<'a>,
    parameters: &'a [KernelParameterDefinition<'a>],
}

impl<'a> KernelMetadata<'a> {
    /// Constructs internally consistent kernel metadata.
    ///
    /// # Errors
    ///
    /// Returns [`KernelMetadataError`] for an invalid kernel name, duplicate
    /// parameter name, or inconsistent compact-support radius declaration.
    ///
    /// # Complexity
    ///
    /// For `P` borrowed parameter definitions, construction uses `O(P^2)`
    /// name comparisons to reject duplicates and performs no heap allocation.
    pub fn try_new(
        name: &'a str,
        definiteness: KernelDefiniteness,
        dimensions: KernelDimensions,
        derivatives: KernelDerivativeCapabilities,
        support: KernelSupport<'a>,
        parameters: &'a [KernelParameterDefinition<'a>],
    ) -> Result<Self, KernelMetadataError> {
        if !is_lower_snake_case(name) {
            return Err(KernelMetadataError::InvalidKernelName);
        }
        validate_unique_parameters(parameters)?;
        validate_support(support, parameters)?;
        Ok(Self {
            name,
            definiteness,
            dimensions,
            derivatives,
            support,
            parameters,
        })
    }

    /// Returns the deterministic lower-snake-case kernel family name.
    #[must_use]
    pub const fn name(self) -> &'a str {
        self.name
    }

    /// Returns the kernel's declared positive-definiteness classification.
    #[must_use]
    pub const fn definiteness(self) -> KernelDefiniteness {
        self.definiteness
    }

    /// Returns the nonempty supported-dimension set.
    pub const fn dimensions(self) -> KernelDimensions {
        self.dimensions
    }

    /// Returns derivative and center capability metadata.
    pub const fn derivatives(self) -> KernelDerivativeCapabilities {
        self.derivatives
    }

    /// Returns global or parameterized compact support metadata.
    #[must_use]
    pub const fn support(self) -> KernelSupport<'a> {
        self.support
    }

    /// Returns ordered, uniquely named parameter definitions.
    pub const fn parameters(self) -> &'a [KernelParameterDefinition<'a>] {
        self.parameters
    }

    /// Finds a parameter definition by its exact name.
    ///
    /// This performs a linear scan of the borrowed parameter slice and does
    /// not allocate.
    #[must_use]
    pub fn parameter(self, name: &str) -> Option<&'a KernelParameterDefinition<'a>> {
        self.parameters
            .iter()
            .find(|parameter| parameter.name == name)
    }
}

/// Error returned by kernel metadata consistency validation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum KernelMetadataError {
    /// Kernel family name is not nonempty deterministic lower snake case.
    InvalidKernelName,
    /// Two parameter definitions use the same exact name.
    DuplicateParameter {
        /// Index of the first definition.
        first: usize,
        /// Index of the duplicate definition.
        second: usize,
    },
    /// Compact support references no declared parameter.
    MissingCompactRadiusParameter,
    /// Compact support radius is not measured in coordinate length.
    InvalidCompactRadiusUnit,
    /// Compact support radius is not constrained to be strictly positive.
    InvalidCompactRadiusConstraint,
}

impl fmt::Display for KernelMetadataError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidKernelName => formatter
                .write_str("kernel family name must be nonempty deterministic lower snake case"),
            Self::DuplicateParameter { first, second } => write!(
                formatter,
                "kernel parameter at index {second} duplicates index {first}"
            ),
            Self::MissingCompactRadiusParameter => formatter
                .write_str("compact support radius parameter is not declared by the kernel"),
            Self::InvalidCompactRadiusUnit => formatter
                .write_str("compact support radius parameter must use the coordinate length unit"),
            Self::InvalidCompactRadiusConstraint => formatter.write_str(
                "compact support radius parameter must be constrained strictly positive",
            ),
        }
    }
}

impl Error for KernelMetadataError {}

fn is_lower_snake_case(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.is_empty() || !bytes[0].is_ascii_lowercase() {
        return false;
    }
    let mut previous_underscore = false;
    for byte in bytes.iter().copied() {
        if byte == b'_' {
            if previous_underscore {
                return false;
            }
            previous_underscore = true;
        } else if byte.is_ascii_lowercase() || byte.is_ascii_digit() {
            previous_underscore = false;
        } else {
            return false;
        }
    }
    !previous_underscore
}

fn validate_unique_parameters(
    parameters: &[KernelParameterDefinition<'_>],
) -> Result<(), KernelMetadataError> {
    for (first, parameter) in parameters.iter().enumerate() {
        for (second, candidate) in parameters.iter().enumerate().skip(first + 1) {
            if parameter.name == candidate.name {
                return Err(KernelMetadataError::DuplicateParameter { first, second });
            }
        }
    }
    Ok(())
}

fn validate_support(
    support: KernelSupport<'_>,
    parameters: &[KernelParameterDefinition<'_>],
) -> Result<(), KernelMetadataError> {
    let KernelSupport::Compact { radius_parameter } = support else {
        return Ok(());
    };
    let Some(definition) = parameters
        .iter()
        .find(|parameter| parameter.name == radius_parameter)
    else {
        return Err(KernelMetadataError::MissingCompactRadiusParameter);
    };
    if definition.unit != KernelParameterUnit::CoordinateLength {
        return Err(KernelMetadataError::InvalidCompactRadiusUnit);
    }
    if definition.constraint != KernelParameterConstraint::Positive {
        return Err(KernelMetadataError::InvalidCompactRadiusConstraint);
    }
    Ok(())
}
