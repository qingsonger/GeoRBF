//! Coordinate units used by the scalar-field core.

use std::error::Error;
use std::fmt;

/// Error returned when a unit identifier is not canonical metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum UnitError {
    /// The identifier contains no characters.
    EmptyIdentifier,
    /// The identifier has leading or trailing whitespace.
    SurroundingWhitespace,
    /// The identifier contains a control character.
    ControlCharacter {
        /// Byte offset of the first control character.
        byte_index: usize,
    },
}

impl fmt::Display for UnitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyIdentifier => formatter.write_str("a unit identifier must not be empty"),
            Self::SurroundingWhitespace => {
                formatter.write_str("a unit identifier must not have surrounding whitespace")
            }
            Self::ControlCharacter { byte_index } => write!(
                formatter,
                "a unit identifier contains a control character at byte {byte_index}"
            ),
        }
    }
}

impl Error for UnitError {}

/// An exact, validated identifier for a coordinate length unit.
///
/// Identifiers are deliberately not aliased or converted by the core. For
/// example, `m` and `metre` remain different identifiers unless an input
/// adapter explicitly canonicalizes them before construction.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub struct LengthUnit {
    identifier: String,
}

impl LengthUnit {
    /// Constructs an exact length-unit identifier.
    ///
    /// # Errors
    ///
    /// Returns [`UnitError`] when the identifier is empty, has surrounding
    /// whitespace, or contains a control character.
    pub fn try_new(identifier: impl Into<String>) -> Result<Self, UnitError> {
        let identifier = identifier.into();
        if identifier.is_empty() {
            return Err(UnitError::EmptyIdentifier);
        }
        if identifier.trim() != identifier {
            return Err(UnitError::SurroundingWhitespace);
        }
        if let Some((byte_index, _)) = identifier
            .char_indices()
            .find(|(_, character)| character.is_control())
        {
            return Err(UnitError::ControlCharacter { byte_index });
        }
        Ok(Self { identifier })
    }

    /// Borrows the exact unit identifier.
    #[must_use]
    pub fn identifier(&self) -> &str {
        &self.identifier
    }
}

/// Unit used by an external angular convention.
///
/// Geometry and kernel internals use radians. This value records how an input
/// coordinate convention describes angles; geological orientation constructors
/// accept it explicitly and convert validated inputs before evaluation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AngleUnit {
    /// Radians.
    Radians,
    /// Degrees.
    Degrees,
}
