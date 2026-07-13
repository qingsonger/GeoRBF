//! The `GeoRBF` scalar-field core.
//!
//! The initial public API provides validated, dimension-safe geometry and
//! coordinate transforms for exactly one, two, or three dimensions. Kernels,
//! functionals, assembly, and solvers are introduced by separately reviewed
//! requirements.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod coordinates;
pub mod dimension;
pub mod geometry;
pub mod transform;
pub mod units;

pub use coordinates::{
    AxisOrder, CoordinateMetadata, CoordinateMetadataError, CoordinateMetadataField, CrsMetadata,
    Handedness, VerticalDirection,
};
pub use dimension::{Dim, SupportedDimension};
pub use geometry::{Direction, GeometryError, Point, UnitDirection, Vector};
pub use transform::{AffineNormalization, TransformError, TransformOperation};
pub use units::{AngleUnit, LengthUnit, UnitError};
