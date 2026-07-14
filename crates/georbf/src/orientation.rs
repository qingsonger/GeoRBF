//! Validated geological planar and linear orientation conversions.
//!
//! Geological orientations are meaningful in two- and three-dimensional
//! models. One-dimensional and unsupported-dimensional values do not compile:
//!
//! ```compile_fail
//! use georbf::PlanarOrientation;
//!
//! let _ = PlanarOrientation::<1>::from_normal_components(
//!     [1.0],
//!     georbf::OrientationPolarity::Positive,
//! );
//! ```
//!
//! ```compile_fail
//! use georbf::LinearOrientation;
//!
//! let _ = LinearOrientation::<4>::from_direction_components(
//!     [1.0, 0.0, 0.0, 0.0],
//!     georbf::OrientationPolarity::Positive,
//! );
//! ```

use std::error::Error;
use std::f64::consts::{FRAC_PI_2, PI};
use std::fmt;

use crate::dimension::{Dim, SupportedDimension};
use crate::geometry::{GeometryError, UnitDirection};
use crate::units::AngleUnit;

/// Marker implemented by dimensions with geological orientation semantics.
///
/// Implementations exist only for [`Dim<2>`] and [`Dim<3>`]. Downstream
/// crates cannot add another implementation because both this trait and
/// [`Dim`] are owned by `GeoRBF`.
pub trait SupportedOrientationDimension: SupportedDimension {}

impl SupportedOrientationDimension for Dim<2> {}
impl SupportedOrientationDimension for Dim<3> {}

/// Sign relationship between a reported orientation and its stored direction.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum OrientationPolarity {
    /// Keep the reported or convention-derived reference direction.
    Positive,
    /// Store the antipode of the reported or convention-derived direction.
    Negative,
    /// Keep a deterministic representative but treat its sign as unknown.
    Unknown,
}

impl OrientationPolarity {
    /// Returns whether the orientation is axial and invariant under sign.
    #[must_use]
    pub const fn is_axial(self) -> bool {
        matches!(self, Self::Unknown)
    }
}

/// Angular field being validated by an orientation conversion.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum OrientationAngleField {
    /// Signed dip in a two-dimensional vertical section.
    SignedDip,
    /// Right-hand-rule strike azimuth.
    Strike,
    /// Down-dip azimuth.
    DipDirection,
    /// Unsigned plane dip from horizontal.
    Dip,
    /// Lineation azimuth.
    Azimuth,
    /// Signed lineation plunge, positive downward.
    Plunge,
}

/// Error returned by geological orientation validation.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum OrientationError {
    /// An angular input is NaN or positive or negative infinity.
    NonFiniteAngle {
        /// Angular field containing the invalid value.
        field: OrientationAngleField,
        /// Rejected value in the declared unit.
        value: f64,
        /// Unit declared for the rejected value.
        unit: AngleUnit,
    },
    /// An angular input lies outside its canonical interval.
    AngleOutOfRange {
        /// Angular field containing the invalid value.
        field: OrientationAngleField,
        /// Rejected value in the declared unit.
        value: f64,
        /// Unit declared for the rejected value and limits.
        unit: AngleUnit,
        /// Inclusive lower limit.
        minimum: f64,
        /// Upper limit.
        maximum: f64,
        /// Whether the upper limit is included.
        maximum_inclusive: bool,
    },
    /// Direction components violate a geometry invariant.
    InvalidDirection(GeometryError),
}

impl fmt::Display for OrientationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteAngle { field, value, unit } => {
                write!(
                    formatter,
                    "{field:?} angle must be finite, got {value} {unit:?}"
                )
            }
            Self::AngleOutOfRange {
                field,
                value,
                unit,
                minimum,
                maximum,
                maximum_inclusive,
            } => {
                let closing = if *maximum_inclusive { ']' } else { ')' };
                write!(
                    formatter,
                    "{field:?} angle {value} {unit:?} is outside [{minimum}, {maximum}{closing}"
                )
            }
            Self::InvalidDirection(source) => write!(formatter, "invalid orientation: {source}"),
        }
    }
}

impl Error for OrientationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidDirection(source) => Some(source),
            Self::NonFiniteAngle { .. } | Self::AngleOutOfRange { .. } => None,
        }
    }
}

impl From<GeometryError> for OrientationError {
    fn from(source: GeometryError) -> Self {
        Self::InvalidDirection(source)
    }
}

/// A validated planar orientation represented by its unit normal.
///
/// The type stores only the normal direction and polarity. It does not create
/// a normal observation, impose a gradient magnitude, or compile a constraint.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct PlanarOrientation<const D: usize>
where
    Dim<D>: SupportedOrientationDimension,
{
    normal: UnitDirection<D>,
    polarity: OrientationPolarity,
}

impl<const D: usize> PlanarOrientation<D>
where
    Dim<D>: SupportedOrientationDimension,
{
    /// Constructs a planar orientation from finite, nonzero normal components.
    ///
    /// [`OrientationPolarity::Negative`] reverses the supplied direction;
    /// [`OrientationPolarity::Unknown`] preserves it as an axial representative.
    ///
    /// # Errors
    ///
    /// Returns [`OrientationError::InvalidDirection`] when a component is
    /// non-finite or every component is zero.
    pub fn from_normal_components(
        components: [f64; D],
        polarity: OrientationPolarity,
    ) -> Result<Self, OrientationError> {
        Ok(Self {
            normal: signed_unit_direction(components, polarity)?,
            polarity,
        })
    }

    /// Borrows the signed unit normal.
    pub const fn normal(&self) -> &UnitDirection<D> {
        &self.normal
    }

    /// Returns the signed unit normal.
    pub const fn into_normal(self) -> UnitDirection<D> {
        self.normal
    }

    /// Returns the preserved polarity metadata.
    #[must_use]
    pub const fn polarity(&self) -> OrientationPolarity {
        self.polarity
    }

    /// Returns whether the normal sign is unknown.
    #[must_use]
    pub const fn is_axial(&self) -> bool {
        self.polarity.is_axial()
    }
}

impl PlanarOrientation<2> {
    /// Converts a signed section dip to an upward reference normal.
    ///
    /// The canonical D=2 frame is X horizontal and Y up. Positive dip means
    /// that the represented line descends toward +X. For signed dip `d`, the
    /// positive reference normal is `[sin(d), cos(d)]`.
    ///
    /// # Errors
    ///
    /// Returns [`OrientationError`] unless the dip is finite and lies in
    /// `[-pi/2, pi/2]` radians or `[-90, 90]` degrees.
    pub fn from_signed_dip(
        signed_dip: f64,
        unit: AngleUnit,
        polarity: OrientationPolarity,
    ) -> Result<Self, OrientationError> {
        let dip = checked_angle(
            signed_dip,
            unit,
            OrientationAngleField::SignedDip,
            AngleRange::SignedQuarter,
        )?;
        let (sin_dip, cos_dip) = exact_sin_cos(dip);
        Self::from_normal_components(canonical_zeros([sin_dip, cos_dip]), polarity)
    }
}

impl PlanarOrientation<3> {
    /// Converts right-hand-rule strike and dip to an upward reference normal.
    ///
    /// The canonical local frame is X easting, Y northing, and Z up. Strike is
    /// clockwise from +Y and follows the right-hand rule: the down-dip azimuth
    /// is `strike + pi/2` modulo `2pi`.
    ///
    /// # Errors
    ///
    /// Returns [`OrientationError`] unless strike is finite and in `[0, 2pi)`
    /// and dip is finite and in `[0, pi/2]`, with corresponding degree ranges.
    pub fn from_strike_dip(
        strike: f64,
        dip: f64,
        unit: AngleUnit,
        polarity: OrientationPolarity,
    ) -> Result<Self, OrientationError> {
        let strike = checked_angle(
            strike,
            unit,
            OrientationAngleField::Strike,
            AngleRange::Full,
        )?;
        let dip = checked_angle(
            dip,
            unit,
            OrientationAngleField::Dip,
            AngleRange::PositiveQuarter,
        )?;
        Self::from_dip_direction_radians(add_quarter_turn(strike), dip, polarity)
    }

    /// Converts down-dip azimuth and dip to an upward reference normal.
    ///
    /// Azimuth is clockwise from +Y toward +X in the canonical local
    /// X-easting/Y-northing/Z-up frame. For dip direction `a` and dip `d`, the
    /// positive reference normal is
    /// `[sin(d) sin(a), sin(d) cos(a), cos(d)]`.
    ///
    /// # Errors
    ///
    /// Returns [`OrientationError`] unless dip direction is finite and in
    /// `[0, 2pi)` and dip is finite and in `[0, pi/2]`, with corresponding
    /// degree ranges.
    pub fn from_dip_direction_dip(
        dip_direction: f64,
        dip: f64,
        unit: AngleUnit,
        polarity: OrientationPolarity,
    ) -> Result<Self, OrientationError> {
        let dip_direction = checked_angle(
            dip_direction,
            unit,
            OrientationAngleField::DipDirection,
            AngleRange::Full,
        )?;
        let dip = checked_angle(
            dip,
            unit,
            OrientationAngleField::Dip,
            AngleRange::PositiveQuarter,
        )?;
        Self::from_dip_direction_radians(dip_direction, dip, polarity)
    }

    fn from_dip_direction_radians(
        dip_direction: f64,
        dip: f64,
        polarity: OrientationPolarity,
    ) -> Result<Self, OrientationError> {
        let (sin_azimuth, cos_azimuth) = exact_sin_cos(dip_direction);
        let (sin_dip, cos_dip) = exact_sin_cos(dip);
        Self::from_normal_components(
            canonical_zeros([sin_dip * sin_azimuth, sin_dip * cos_azimuth, cos_dip]),
            polarity,
        )
    }
}

/// A validated linear orientation represented by a unit direction.
///
/// The type stores only the direction and polarity. It does not create a
/// tangent observation, impose a derivative, or compile a constraint.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct LinearOrientation<const D: usize>
where
    Dim<D>: SupportedOrientationDimension,
{
    direction: UnitDirection<D>,
    polarity: OrientationPolarity,
}

impl<const D: usize> LinearOrientation<D>
where
    Dim<D>: SupportedOrientationDimension,
{
    /// Constructs a linear orientation from finite, nonzero components.
    ///
    /// [`OrientationPolarity::Negative`] reverses the supplied direction;
    /// [`OrientationPolarity::Unknown`] preserves it as an axial representative.
    ///
    /// # Errors
    ///
    /// Returns [`OrientationError::InvalidDirection`] when a component is
    /// non-finite or every component is zero.
    pub fn from_direction_components(
        components: [f64; D],
        polarity: OrientationPolarity,
    ) -> Result<Self, OrientationError> {
        Ok(Self {
            direction: signed_unit_direction(components, polarity)?,
            polarity,
        })
    }

    /// Borrows the signed unit direction.
    pub const fn direction(&self) -> &UnitDirection<D> {
        &self.direction
    }

    /// Returns the signed unit direction.
    pub const fn into_direction(self) -> UnitDirection<D> {
        self.direction
    }

    /// Returns the preserved polarity metadata.
    #[must_use]
    pub const fn polarity(&self) -> OrientationPolarity {
        self.polarity
    }

    /// Returns whether the direction sign is unknown.
    #[must_use]
    pub const fn is_axial(&self) -> bool {
        self.polarity.is_axial()
    }
}

impl LinearOrientation<2> {
    /// Converts signed plunge in a vertical section to a lineation direction.
    ///
    /// The canonical D=2 frame is X horizontal and Y up. Plunge is measured
    /// from horizontal and is positive downward. For plunge `p`, the positive
    /// reference direction is `[cos(p), -sin(p)]`.
    ///
    /// # Errors
    ///
    /// Returns [`OrientationError`] unless plunge is finite and lies in
    /// `[-pi/2, pi/2]` radians or `[-90, 90]` degrees.
    pub fn from_plunge(
        plunge: f64,
        unit: AngleUnit,
        polarity: OrientationPolarity,
    ) -> Result<Self, OrientationError> {
        let plunge = checked_angle(
            plunge,
            unit,
            OrientationAngleField::Plunge,
            AngleRange::SignedQuarter,
        )?;
        let (sin_plunge, cos_plunge) = exact_sin_cos(plunge);
        Self::from_direction_components(canonical_zeros([cos_plunge, -sin_plunge]), polarity)
    }
}

impl LinearOrientation<3> {
    /// Converts azimuth and signed plunge to a lineation direction.
    ///
    /// The canonical local frame is X easting, Y northing, and Z up. Azimuth
    /// is clockwise from +Y toward +X. Plunge is measured from horizontal and
    /// is positive downward. For azimuth `a` and plunge `p`, the positive
    /// reference direction is
    /// `[cos(p) sin(a), cos(p) cos(a), -sin(p)]`.
    ///
    /// # Errors
    ///
    /// Returns [`OrientationError`] unless azimuth is finite and in `[0, 2pi)`
    /// and plunge is finite and in `[-pi/2, pi/2]`, with corresponding degree
    /// ranges.
    pub fn from_azimuth_plunge(
        azimuth: f64,
        plunge: f64,
        unit: AngleUnit,
        polarity: OrientationPolarity,
    ) -> Result<Self, OrientationError> {
        let azimuth = checked_angle(
            azimuth,
            unit,
            OrientationAngleField::Azimuth,
            AngleRange::Full,
        )?;
        let plunge = checked_angle(
            plunge,
            unit,
            OrientationAngleField::Plunge,
            AngleRange::SignedQuarter,
        )?;
        let (sin_azimuth, cos_azimuth) = exact_sin_cos(azimuth);
        let (sin_plunge, cos_plunge) = exact_sin_cos(plunge);
        Self::from_direction_components(
            canonical_zeros([
                cos_plunge * sin_azimuth,
                cos_plunge * cos_azimuth,
                -sin_plunge,
            ]),
            polarity,
        )
    }
}

#[derive(Clone, Copy)]
enum AngleRange {
    Full,
    PositiveQuarter,
    SignedQuarter,
}

impl AngleRange {
    fn limits(self, unit: AngleUnit) -> (f64, f64, bool) {
        match (self, unit) {
            (Self::Full, AngleUnit::Radians) => (0.0, 2.0 * PI, false),
            (Self::Full, AngleUnit::Degrees) => (0.0, 360.0, false),
            (Self::PositiveQuarter, AngleUnit::Radians) => (0.0, FRAC_PI_2, true),
            (Self::PositiveQuarter, AngleUnit::Degrees) => (0.0, 90.0, true),
            (Self::SignedQuarter, AngleUnit::Radians) => (-FRAC_PI_2, FRAC_PI_2, true),
            (Self::SignedQuarter, AngleUnit::Degrees) => (-90.0, 90.0, true),
        }
    }
}

fn checked_angle(
    value: f64,
    unit: AngleUnit,
    field: OrientationAngleField,
    range: AngleRange,
) -> Result<f64, OrientationError> {
    if !value.is_finite() {
        return Err(OrientationError::NonFiniteAngle { field, value, unit });
    }

    let (minimum, maximum, maximum_inclusive) = range.limits(unit);
    let above_maximum = if maximum_inclusive {
        value > maximum
    } else {
        value >= maximum
    };
    if value < minimum || above_maximum {
        return Err(OrientationError::AngleOutOfRange {
            field,
            value,
            unit,
            minimum,
            maximum,
            maximum_inclusive,
        });
    }

    let radians = match unit {
        AngleUnit::Radians => value,
        AngleUnit::Degrees => (value / 180.0) * PI,
    };
    Ok(if radians == 0.0 { 0.0 } else { radians })
}

fn signed_unit_direction<const D: usize>(
    components: [f64; D],
    polarity: OrientationPolarity,
) -> Result<UnitDirection<D>, OrientationError>
where
    Dim<D>: SupportedOrientationDimension,
{
    let components = match polarity {
        OrientationPolarity::Negative => components.map(|component| -component),
        OrientationPolarity::Positive | OrientationPolarity::Unknown => components,
    };
    UnitDirection::try_new(components).map_err(OrientationError::from)
}

fn exact_sin_cos(angle: f64) -> (f64, f64) {
    let bits = angle.to_bits();
    if bits == 0.0_f64.to_bits() {
        (0.0, 1.0)
    } else if bits == FRAC_PI_2.to_bits() {
        (1.0, 0.0)
    } else if bits == (-FRAC_PI_2).to_bits() {
        (-1.0, 0.0)
    } else if bits == PI.to_bits() {
        (0.0, -1.0)
    } else if bits == (3.0 * FRAC_PI_2).to_bits() {
        (-1.0, 0.0)
    } else {
        angle.sin_cos()
    }
}

fn add_quarter_turn(angle: f64) -> f64 {
    let three_quarter_turns = 3.0 * FRAC_PI_2;
    if angle < three_quarter_turns {
        angle + FRAC_PI_2
    } else {
        angle - three_quarter_turns
    }
}

fn canonical_zeros<const D: usize>(components: [f64; D]) -> [f64; D] {
    components.map(|component| if component == 0.0 { 0.0 } else { component })
}
