//! Dimension-safe geometry primitives with validated invariants.
//!
//! Public geometry is available only in one, two, or three dimensions. For
//! example, zero-dimensional and four-dimensional points do not compile:
//!
//! ```compile_fail
//! use georbf::Point;
//!
//! let _ = Point::<0>::try_new([]);
//! ```
//!
//! ```compile_fail
//! use georbf::Point;
//!
//! let _ = Point::<4>::try_new([0.0; 4]);
//! ```

use std::error::Error;
use std::fmt;

use crate::dimension::{Dim, SupportedDimension};

/// Error returned when geometry components violate a constructor invariant.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum GeometryError {
    /// A component is NaN or positive or negative infinity.
    NonFiniteComponent {
        /// Zero-based component index.
        index: usize,
        /// Rejected component value.
        value: f64,
    },
    /// A direction has no nonzero component.
    ZeroDirection,
}

impl fmt::Display for GeometryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteComponent { index, value } => {
                write!(formatter, "component {index} must be finite, got {value}")
            }
            Self::ZeroDirection => formatter.write_str("a direction must be nonzero"),
        }
    }
}

impl Error for GeometryError {}

/// A point with exactly `D` finite coordinates.
///
/// ```
/// use georbf::Point;
///
/// let point = Point::<3>::try_new([1.0, 2.0, 3.0])?;
/// assert_eq!(point.components(), &[1.0, 2.0, 3.0]);
/// # Ok::<(), georbf::GeometryError>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct Point<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    components: [f64; D],
}

impl<const D: usize> Point<D>
where
    Dim<D>: SupportedDimension,
{
    /// Constructs a point after validating that every component is finite.
    ///
    /// # Errors
    ///
    /// Returns [`GeometryError::NonFiniteComponent`] for the first non-finite
    /// component.
    pub fn try_new(components: [f64; D]) -> Result<Self, GeometryError> {
        validate_finite(&components)?;
        Ok(Self { components })
    }

    /// Borrows the point components in axis order.
    #[must_use]
    pub const fn components(&self) -> &[f64; D] {
        &self.components
    }

    /// Returns the owned component array.
    #[must_use]
    pub const fn into_components(self) -> [f64; D] {
        self.components
    }
}

impl<const D: usize> TryFrom<[f64; D]> for Point<D>
where
    Dim<D>: SupportedDimension,
{
    type Error = GeometryError;

    fn try_from(components: [f64; D]) -> Result<Self, Self::Error> {
        Self::try_new(components)
    }
}

/// A vector with exactly `D` finite components.
///
/// Zero vectors are valid. Use [`Direction`] when the nonzero invariant is
/// required.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct Vector<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    components: [f64; D],
}

impl<const D: usize> Vector<D>
where
    Dim<D>: SupportedDimension,
{
    /// Constructs a vector after validating that every component is finite.
    ///
    /// # Errors
    ///
    /// Returns [`GeometryError::NonFiniteComponent`] for the first non-finite
    /// component.
    pub fn try_new(components: [f64; D]) -> Result<Self, GeometryError> {
        validate_finite(&components)?;
        Ok(Self { components })
    }

    /// Borrows the vector components in axis order.
    #[must_use]
    pub const fn components(&self) -> &[f64; D] {
        &self.components
    }

    /// Returns the owned component array.
    #[must_use]
    pub const fn into_components(self) -> [f64; D] {
        self.components
    }
}

impl<const D: usize> TryFrom<[f64; D]> for Vector<D>
where
    Dim<D>: SupportedDimension,
{
    type Error = GeometryError;

    fn try_from(components: [f64; D]) -> Result<Self, Self::Error> {
        Self::try_new(components)
    }
}

/// A nonzero direction with exactly `D` finite components.
///
/// A `Direction` preserves the supplied magnitude. Call [`Direction::unit`]
/// when a normalized representation is required.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct Direction<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    components: [f64; D],
}

impl<const D: usize> Direction<D>
where
    Dim<D>: SupportedDimension,
{
    /// Constructs a finite, nonzero direction.
    ///
    /// # Errors
    ///
    /// Returns [`GeometryError::NonFiniteComponent`] for the first non-finite
    /// component, or [`GeometryError::ZeroDirection`] when every component is
    /// positive or negative zero.
    pub fn try_new(components: [f64; D]) -> Result<Self, GeometryError> {
        validate_finite(&components)?;
        if components.iter().all(|component| *component == 0.0) {
            return Err(GeometryError::ZeroDirection);
        }
        Ok(Self { components })
    }

    /// Borrows the direction components in axis order.
    #[must_use]
    pub const fn components(&self) -> &[f64; D] {
        &self.components
    }

    /// Returns the owned component array.
    #[must_use]
    pub const fn into_components(self) -> [f64; D] {
        self.components
    }

    /// Normalizes this direction without forming an overflowing or
    /// underflowing Euclidean norm.
    pub fn unit(self) -> UnitDirection<D> {
        UnitDirection::from_direction(self)
    }
}

impl<const D: usize> TryFrom<[f64; D]> for Direction<D>
where
    Dim<D>: SupportedDimension,
{
    type Error = GeometryError;

    fn try_from(components: [f64; D]) -> Result<Self, Self::Error> {
        Self::try_new(components)
    }
}

impl<const D: usize> From<Direction<D>> for Vector<D>
where
    Dim<D>: SupportedDimension,
{
    fn from(direction: Direction<D>) -> Self {
        Self {
            components: direction.components,
        }
    }
}

/// A finite direction normalized to unit Euclidean length, up to roundoff.
///
/// Construction accepts any finite, nonzero magnitude. Normalization first
/// scales by the largest absolute component, avoiding overflow for values near
/// [`f64::MAX`] and underflow for very small nonzero values.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct UnitDirection<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    components: [f64; D],
}

impl<const D: usize> UnitDirection<D>
where
    Dim<D>: SupportedDimension,
{
    /// Constructs a unit direction from finite, nonzero components.
    ///
    /// # Errors
    ///
    /// Returns [`GeometryError::NonFiniteComponent`] for the first non-finite
    /// component, or [`GeometryError::ZeroDirection`] when every component is
    /// positive or negative zero.
    pub fn try_new(components: [f64; D]) -> Result<Self, GeometryError> {
        Direction::try_new(components).map(Self::from_direction)
    }

    /// Normalizes an already validated direction.
    pub fn from_direction(direction: Direction<D>) -> Self {
        Self {
            components: normalized(direction.components),
        }
    }

    /// Borrows the normalized components in axis order.
    #[must_use]
    pub const fn components(&self) -> &[f64; D] {
        &self.components
    }

    /// Returns the owned normalized component array.
    #[must_use]
    pub const fn into_components(self) -> [f64; D] {
        self.components
    }
}

impl<const D: usize> TryFrom<[f64; D]> for UnitDirection<D>
where
    Dim<D>: SupportedDimension,
{
    type Error = GeometryError;

    fn try_from(components: [f64; D]) -> Result<Self, Self::Error> {
        Self::try_new(components)
    }
}

impl<const D: usize> From<UnitDirection<D>> for Direction<D>
where
    Dim<D>: SupportedDimension,
{
    fn from(direction: UnitDirection<D>) -> Self {
        Self {
            components: direction.components,
        }
    }
}

fn validate_finite<const D: usize>(components: &[f64; D]) -> Result<(), GeometryError> {
    for (index, value) in components.iter().copied().enumerate() {
        if !value.is_finite() {
            return Err(GeometryError::NonFiniteComponent { index, value });
        }
    }
    Ok(())
}

fn normalized<const D: usize>(components: [f64; D]) -> [f64; D] {
    let scale = components
        .iter()
        .map(|component| component.abs())
        .fold(0.0_f64, f64::max);
    let scaled_squared_norm = components
        .iter()
        .map(|component| {
            let scaled = component / scale;
            scaled * scaled
        })
        .sum::<f64>();
    let scaled_norm = scaled_squared_norm.sqrt();
    components.map(|component| (component / scale) / scaled_norm)
}
