//! The `GeoRBF` scalar-field core.
//!
//! The initial public API provides validated, dimension-safe geometry for
//! exactly one, two, or three dimensions. Kernels, functionals, assembly, and
//! solvers are introduced by separately reviewed requirements.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod dimension;
pub mod geometry;

pub use dimension::{Dim, SupportedDimension};
pub use geometry::{Direction, GeometryError, Point, UnitDirection, Vector};
