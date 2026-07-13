//! Compile-time dimension support.

/// Type-level representation of a spatial dimension.
///
/// Public geometry types use a `Dim<D>: SupportedDimension` bound, so only the
/// explicitly implemented dimensions can enter the API.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Dim<const D: usize>;

/// Marker implemented by the spatial dimensions supported by `GeoRBF`.
///
/// Implementations exist only for [`Dim<1>`], [`Dim<2>`], and [`Dim<3>`]. A
/// downstream crate cannot add an implementation for another `Dim<D>` because
/// both this trait and [`Dim`] are owned by `GeoRBF`.
pub trait SupportedDimension {}

impl SupportedDimension for Dim<1> {}
impl SupportedDimension for Dim<2> {}
impl SupportedDimension for Dim<3> {}
