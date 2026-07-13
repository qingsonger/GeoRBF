//! Validated coordinate metadata for one, two, or three dimensions.
//!
//! Unsupported dimensions fail at compile time:
//!
//! ```compile_fail
//! use georbf::AxisOrder;
//!
//! let _ = AxisOrder::<0>::identity();
//! ```
//!
//! ```compile_fail
//! use georbf::AxisOrder;
//!
//! let _ = AxisOrder::<4>::identity();
//! ```

use std::error::Error;
use std::fmt;

use crate::dimension::{Dim, SupportedDimension};
use crate::units::{AngleUnit, LengthUnit};

/// Coordinate metadata field that differs between two data sources.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CoordinateMetadataField {
    /// Coordinate length unit.
    LengthUnit,
    /// CRS identifier or WKT text.
    Crs,
    /// Component-to-canonical axis order.
    AxisOrder,
    /// Direction considered positive along the vertical axis.
    VerticalDirection,
    /// Coordinate-system handedness.
    Handedness,
    /// External angle unit.
    AngleUnit,
}

/// Error returned by coordinate metadata validation or compatibility checks.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CoordinateMetadataError {
    /// EPSG code zero is reserved as invalid metadata.
    InvalidEpsgCode,
    /// WKT metadata contains no non-whitespace text.
    EmptyWkt,
    /// WKT metadata has leading or trailing whitespace.
    SurroundingWhitespaceInWkt,
    /// An axis index is outside `0..D`.
    AxisOutOfRange {
        /// Component position containing the invalid index.
        position: usize,
        /// Rejected canonical axis index.
        axis: usize,
        /// Coordinate dimension.
        dimension: usize,
    },
    /// An axis index occurs more than once.
    DuplicateAxis {
        /// Component position containing the duplicate.
        position: usize,
        /// Duplicated canonical axis index.
        axis: usize,
    },
    /// Two metadata values cannot be mixed without an explicit conversion.
    Mismatch {
        /// First field that differs in deterministic comparison order.
        field: CoordinateMetadataField,
    },
}

impl fmt::Display for CoordinateMetadataError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidEpsgCode => formatter.write_str("EPSG code zero is invalid"),
            Self::EmptyWkt => formatter.write_str("CRS WKT must not be empty"),
            Self::SurroundingWhitespaceInWkt => {
                formatter.write_str("CRS WKT must not have surrounding whitespace")
            }
            Self::AxisOutOfRange {
                position,
                axis,
                dimension,
            } => write!(
                formatter,
                "axis index {axis} at component {position} is outside dimension {dimension}"
            ),
            Self::DuplicateAxis { position, axis } => write!(
                formatter,
                "axis index {axis} at component {position} is duplicated"
            ),
            Self::Mismatch { field } => {
                write!(
                    formatter,
                    "coordinate metadata field {field:?} does not match"
                )
            }
        }
    }
}

impl Error for CoordinateMetadataError {}

/// Opaque coordinate reference system metadata.
///
/// The core preserves these values and compares them exactly. It does not look
/// up CRS definitions or reproject coordinates.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub struct CrsMetadata {
    epsg: Option<u32>,
    wkt: Option<String>,
}

impl CrsMetadata {
    /// Constructs optional EPSG and WKT metadata.
    ///
    /// # Errors
    ///
    /// Returns [`CoordinateMetadataError::InvalidEpsgCode`] for code zero, or
    /// a WKT error for blank or padded text.
    pub fn try_new(
        epsg: Option<u32>,
        wkt: Option<String>,
    ) -> Result<Self, CoordinateMetadataError> {
        if epsg == Some(0) {
            return Err(CoordinateMetadataError::InvalidEpsgCode);
        }
        if let Some(text) = &wkt {
            validate_wkt(text)?;
        }
        Ok(Self { epsg, wkt })
    }

    /// Constructs metadata with no declared CRS.
    pub const fn unspecified() -> Self {
        Self {
            epsg: None,
            wkt: None,
        }
    }

    /// Constructs EPSG-only metadata.
    ///
    /// # Errors
    ///
    /// Returns [`CoordinateMetadataError::InvalidEpsgCode`] for code zero.
    pub fn from_epsg(code: u32) -> Result<Self, CoordinateMetadataError> {
        Self::try_new(Some(code), None)
    }

    /// Constructs WKT-only metadata.
    ///
    /// # Errors
    ///
    /// Returns a WKT validation error for blank or padded text.
    pub fn from_wkt(wkt: impl Into<String>) -> Result<Self, CoordinateMetadataError> {
        Self::try_new(None, Some(wkt.into()))
    }

    /// Returns the optional EPSG code.
    #[must_use]
    pub const fn epsg(&self) -> Option<u32> {
        self.epsg
    }

    /// Borrows the optional WKT text.
    #[must_use]
    pub fn wkt(&self) -> Option<&str> {
        self.wkt.as_deref()
    }
}

/// Permutation mapping component positions to canonical axis indices.
///
/// `[1, 0, 2]` means that component positions contain canonical axes Y, X,
/// and Z respectively.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub struct AxisOrder<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    components: [usize; D],
}

impl<const D: usize> AxisOrder<D>
where
    Dim<D>: SupportedDimension,
{
    /// Validates a permutation of the canonical axis indices `0..D`.
    ///
    /// # Errors
    ///
    /// Returns [`CoordinateMetadataError::AxisOutOfRange`] or
    /// [`CoordinateMetadataError::DuplicateAxis`] for an invalid permutation.
    pub fn try_new(components: [usize; D]) -> Result<Self, CoordinateMetadataError> {
        let mut seen = [false; D];
        for (position, axis) in components.iter().copied().enumerate() {
            if axis >= D {
                return Err(CoordinateMetadataError::AxisOutOfRange {
                    position,
                    axis,
                    dimension: D,
                });
            }
            if seen[axis] {
                return Err(CoordinateMetadataError::DuplicateAxis { position, axis });
            }
            seen[axis] = true;
        }
        Ok(Self { components })
    }

    /// Returns canonical component order `[0, ..., D - 1]`.
    pub fn identity() -> Self {
        Self {
            components: std::array::from_fn(|index| index),
        }
    }

    /// Borrows the component-to-canonical-axis permutation.
    #[must_use]
    pub const fn components(&self) -> &[usize; D] {
        &self.components
    }
}

/// Direction considered positive along the vertical coordinate axis.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum VerticalDirection {
    /// Coordinate values increase upward.
    Up,
    /// Coordinate values increase downward.
    Down,
}

/// Handedness of the external coordinate convention.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Handedness {
    /// Right-handed coordinates.
    Right,
    /// Left-handed coordinates.
    Left,
}

/// Complete coordinate convention metadata for dimension `D`.
///
/// Metadata is compared exactly. Callers must perform any unit conversion,
/// axis remapping, or CRS reprojection before asking the core to combine data.
#[derive(Clone, Debug, Eq, PartialEq)]
#[must_use]
pub struct CoordinateMetadata<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    length_unit: LengthUnit,
    crs: CrsMetadata,
    axis_order: AxisOrder<D>,
    vertical_direction: VerticalDirection,
    handedness: Handedness,
    angle_unit: AngleUnit,
}

impl<const D: usize> CoordinateMetadata<D>
where
    Dim<D>: SupportedDimension,
{
    /// Constructs metadata from already validated components.
    pub const fn new(
        length_unit: LengthUnit,
        crs: CrsMetadata,
        axis_order: AxisOrder<D>,
        vertical_direction: VerticalDirection,
        handedness: Handedness,
        angle_unit: AngleUnit,
    ) -> Self {
        Self {
            length_unit,
            crs,
            axis_order,
            vertical_direction,
            handedness,
            angle_unit,
        }
    }

    /// Borrows the exact length-unit identifier.
    pub const fn length_unit(&self) -> &LengthUnit {
        &self.length_unit
    }

    /// Borrows the opaque CRS metadata.
    pub const fn crs(&self) -> &CrsMetadata {
        &self.crs
    }

    /// Borrows the component-to-canonical-axis permutation.
    pub const fn axis_order(&self) -> &AxisOrder<D> {
        &self.axis_order
    }

    /// Returns the positive vertical direction.
    #[must_use]
    pub const fn vertical_direction(&self) -> VerticalDirection {
        self.vertical_direction
    }

    /// Returns the coordinate-system handedness.
    #[must_use]
    pub const fn handedness(&self) -> Handedness {
        self.handedness
    }

    /// Returns the external angle unit.
    #[must_use]
    pub const fn angle_unit(&self) -> AngleUnit {
        self.angle_unit
    }

    /// Rejects silently mixing different coordinate conventions.
    ///
    /// # Errors
    ///
    /// Returns [`CoordinateMetadataError::Mismatch`] for the first differing
    /// field in length-unit, CRS, axis-order, vertical-direction, handedness,
    /// and angle-unit order.
    pub fn ensure_compatible(&self, other: &Self) -> Result<(), CoordinateMetadataError> {
        let field = if self.length_unit != other.length_unit {
            Some(CoordinateMetadataField::LengthUnit)
        } else if self.crs != other.crs {
            Some(CoordinateMetadataField::Crs)
        } else if self.axis_order != other.axis_order {
            Some(CoordinateMetadataField::AxisOrder)
        } else if self.vertical_direction != other.vertical_direction {
            Some(CoordinateMetadataField::VerticalDirection)
        } else if self.handedness != other.handedness {
            Some(CoordinateMetadataField::Handedness)
        } else if self.angle_unit != other.angle_unit {
            Some(CoordinateMetadataField::AngleUnit)
        } else {
            None
        };

        match field {
            Some(field) => Err(CoordinateMetadataError::Mismatch { field }),
            None => Ok(()),
        }
    }
}

fn validate_wkt(wkt: &str) -> Result<(), CoordinateMetadataError> {
    if wkt.trim().is_empty() {
        return Err(CoordinateMetadataError::EmptyWkt);
    }
    if wkt.trim() != wkt {
        return Err(CoordinateMetadataError::SurroundingWhitespaceInWkt);
    }
    Ok(())
}
