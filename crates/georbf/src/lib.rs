//! The `GeoRBF` scalar-field core.
//!
//! The initial public API provides validated, dimension-safe geometry and
//! coordinate transforms, radial kernel derivative calculus, kernel metadata,
//! polyharmonic/surface-spline kernels, smooth global-support kernels, and
//! Wendland compact-support kernels for exactly one, two, or three dimensions,
//! validated geological orientation conversions in two and three dimensions,
//! fixed global anisotropy metrics with chain-rule derivatives, and complete
//! dimension-generic polynomial spaces. Functionals, assembly, and solvers are
//! introduced by separately reviewed requirements.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod anisotropy;
pub mod coordinates;
pub mod dimension;
pub mod geometry;
pub mod kernel;
pub mod kernel_calculus;
pub mod orientation;
pub mod polynomial;
pub mod transform;
pub mod units;

pub use anisotropy::{
    AnisotropyConditionPolicy, AnisotropyDiagnostics, AnisotropyError, GlobalAnisotropy,
};
pub use coordinates::{
    AxisOrder, CoordinateMetadata, CoordinateMetadataError, CoordinateMetadataField, CrsMetadata,
    Handedness, VerticalDirection,
};
pub use dimension::{Dim, SupportedDimension};
pub use geometry::{Direction, GeometryError, Point, UnitDirection, Vector};
pub use kernel::{
    CpdOrder, CpdOrderError, Gaussian, InverseMultiquadric, KernelDefiniteness,
    KernelDerivativeCapabilities, KernelDerivativeCapabilitiesError, KernelDerivativeCapability,
    KernelDerivativeOrder, KernelDimensions, KernelDimensionsError, KernelMetadata,
    KernelMetadataError, KernelParameterConstraint, KernelParameterDefinition,
    KernelParameterDefinitionError, KernelParameterUnit, KernelParameterValueError, KernelSupport,
    Matern, MaternSmoothness, Multiquadric, PolyharmonicSpline,
    PolyharmonicSplineConstructionError, PolyharmonicSplineEvaluationError,
    SmoothKernelConstructionError, SmoothKernelEvaluationError, SmoothKernelFamily, SurfaceSpline,
    SurfaceSplineConstructionError, Wendland, WendlandConstructionError, WendlandEvaluationError,
    WendlandSmoothness,
};
pub use kernel_calculus::{
    KernelArgument, KernelCalculusError, RadialDerivativeOrder, RadialExpansionCoefficient,
    RadialExpansionCoefficients, RadialJet, RadialJetLocation, RadialSeparation, SpatialKernelJet,
};
pub use orientation::{
    LinearOrientation, OrientationAngleField, OrientationError, OrientationPolarity,
    PlanarOrientation, SupportedOrientationDimension,
};
pub use polynomial::{MultiIndex, PolynomialOutput, PolynomialSpace, PolynomialSpaceError};
pub use transform::{AffineNormalization, TransformError, TransformOperation};
pub use units::{AngleUnit, LengthUnit, UnitError};
