//! Semantic constructors for scalar-field linear inequalities.
//!
//! This layer gives lower, upper, interval, region-side, scalar-gap, and
//! directional-monotonicity constraints explicit sign and unit conventions.
//! It lowers each validated value to the existing provenance-bearing
//! [`crate::problem_ir::SemanticConstraint`]. Canonicalization remains owned by
//! [`crate::problem_ir::SemanticProblemIr`], and no geological name or relation
//! reaches a solver.
//!
//! Inside/outside observations are necessarily relative to an explicit scalar
//! orientation. Boundaries are closed: a value exactly on the boundary
//! satisfies both sides. Scalar gaps use `upper - lower >= minimum_gap`.
//! Directional monotonicity acts on exactly one coefficient-one directional
//! derivative; increasing means `u^T grad f >= minimum_rate`, while decreasing
//! means `u^T grad f <= -minimum_rate`.
//!
//! Linear semantics are available only in one, two, or three dimensions:
//!
//! ```compile_fail
//! use georbf::LinearConstraint;
//!
//! let _ = std::mem::size_of::<LinearConstraint<4>>();
//! ```

use std::error::Error;
use std::fmt;

use crate::dimension::{Dim, SupportedDimension};
use crate::functional::{
    FunctionalAtom, FunctionalError, FunctionalExpr, FunctionalTerm, ObservationFunctional,
};
use crate::problem_ir::{
    Enforcement, ProblemIrError, SemanticConstraint, SemanticExpression, SemanticProvenance,
    SemanticRelation,
};

/// Which closed side of an explicitly oriented scalar boundary is observed.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RegionSide {
    /// The observation lies on the declared inside side or on the boundary.
    Inside,
    /// The observation lies on the declared outside side or on the boundary.
    Outside,
}

/// Explicit scalar-field orientation used to interpret inside and outside.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum InsideOrientation {
    /// Inside values are less than or equal to the boundary value.
    InsideAtOrBelow,
    /// Inside values are greater than or equal to the boundary value.
    InsideAtOrAbove,
}

/// Directional monotonicity sense along a functional's unit direction.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MonotonicitySense {
    /// Require `u^T grad f` to be at least the nonnegative minimum rate.
    Increasing,
    /// Require `u^T grad f` to be at most the negative minimum rate.
    Decreasing,
}

#[derive(Clone, Debug, PartialEq)]
enum LinearConstraintKind<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    Lower {
        functional: ObservationFunctional<D>,
        minimum: f64,
    },
    Upper {
        functional: ObservationFunctional<D>,
        maximum: f64,
    },
    Interval {
        functional: ObservationFunctional<D>,
        lower: f64,
        upper: f64,
    },
    Region {
        functional: ObservationFunctional<D>,
        boundary: f64,
        side: RegionSide,
        orientation: InsideOrientation,
    },
    ScalarGap {
        lower: ObservationFunctional<D>,
        upper: ObservationFunctional<D>,
        minimum_gap: f64,
    },
    Monotonicity {
        derivative: ObservationFunctional<D>,
        sense: MonotonicitySense,
        minimum_rate: f64,
    },
}

/// One validated provenance-bearing semantic linear constraint.
///
/// Construction validates the semantic sign and scalar rules. Consuming
/// [`Self::try_into_semantic_constraint`] lowers the value to the shared
/// problem IR without choosing a solver, scaling, or regularization policy.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct LinearConstraint<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    provenance: SemanticProvenance,
    enforcement: Enforcement,
    kind: LinearConstraintKind<D>,
}

impl<const D: usize> LinearConstraint<D>
where
    Dim<D>: SupportedDimension,
{
    /// Constructs `functional >= minimum`.
    ///
    /// # Errors
    ///
    /// Rejects a non-finite minimum or invalid enforcement metadata.
    pub fn try_lower(
        provenance: SemanticProvenance,
        functional: ObservationFunctional<D>,
        minimum: f64,
        enforcement: Enforcement,
    ) -> Result<Self, LinearConstraintError> {
        validate_threshold(minimum)?;
        validate_enforcement(enforcement)?;
        Ok(Self {
            provenance,
            enforcement,
            kind: LinearConstraintKind::Lower {
                functional,
                minimum,
            },
        })
    }

    /// Constructs `functional <= maximum`.
    ///
    /// # Errors
    ///
    /// Rejects a non-finite maximum or invalid enforcement metadata.
    pub fn try_upper(
        provenance: SemanticProvenance,
        functional: ObservationFunctional<D>,
        maximum: f64,
        enforcement: Enforcement,
    ) -> Result<Self, LinearConstraintError> {
        validate_threshold(maximum)?;
        validate_enforcement(enforcement)?;
        Ok(Self {
            provenance,
            enforcement,
            kind: LinearConstraintKind::Upper {
                functional,
                maximum,
            },
        })
    }

    /// Constructs `lower <= functional <= upper`.
    ///
    /// # Errors
    ///
    /// Rejects non-finite endpoints, a reversed interval, or invalid
    /// enforcement metadata.
    pub fn try_interval(
        provenance: SemanticProvenance,
        functional: ObservationFunctional<D>,
        lower: f64,
        upper: f64,
        enforcement: Enforcement,
    ) -> Result<Self, LinearConstraintError> {
        validate_threshold(lower)?;
        validate_threshold(upper)?;
        if lower > upper {
            return Err(LinearConstraintError::ReversedInterval { lower, upper });
        }
        validate_enforcement(enforcement)?;
        Ok(Self {
            provenance,
            enforcement,
            kind: LinearConstraintKind::Interval {
                functional,
                lower,
                upper,
            },
        })
    }

    /// Constructs a closed inside/outside constraint relative to an explicit
    /// scalar orientation and boundary value.
    ///
    /// # Errors
    ///
    /// Rejects a non-finite boundary or invalid enforcement metadata.
    pub fn try_region(
        provenance: SemanticProvenance,
        functional: ObservationFunctional<D>,
        boundary: f64,
        side: RegionSide,
        orientation: InsideOrientation,
        enforcement: Enforcement,
    ) -> Result<Self, LinearConstraintError> {
        validate_threshold(boundary)?;
        validate_enforcement(enforcement)?;
        Ok(Self {
            provenance,
            enforcement,
            kind: LinearConstraintKind::Region {
                functional,
                boundary,
                side,
                orientation,
            },
        })
    }

    /// Constructs `upper - lower >= minimum_gap` for two scalar-valued
    /// functional expressions.
    ///
    /// # Errors
    ///
    /// Rejects a negative or non-finite gap or invalid enforcement metadata.
    pub fn try_scalar_gap(
        provenance: SemanticProvenance,
        lower: ObservationFunctional<D>,
        upper: ObservationFunctional<D>,
        minimum_gap: f64,
        enforcement: Enforcement,
    ) -> Result<Self, LinearConstraintError> {
        if !minimum_gap.is_finite() || minimum_gap < 0.0 {
            return Err(LinearConstraintError::InvalidMinimumGap { minimum_gap });
        }
        validate_enforcement(enforcement)?;
        Ok(Self {
            provenance,
            enforcement,
            kind: LinearConstraintKind::ScalarGap {
                lower,
                upper,
                minimum_gap,
            },
        })
    }

    /// Constructs directional monotonicity from exactly one coefficient-one
    /// directional-derivative atom.
    ///
    /// # Errors
    ///
    /// Rejects any other functional shape, a negative or non-finite minimum
    /// rate, or invalid enforcement metadata.
    pub fn try_monotonicity(
        provenance: SemanticProvenance,
        derivative: ObservationFunctional<D>,
        sense: MonotonicitySense,
        minimum_rate: f64,
        enforcement: Enforcement,
    ) -> Result<Self, LinearConstraintError> {
        if !minimum_rate.is_finite() || minimum_rate < 0.0 {
            return Err(LinearConstraintError::InvalidMinimumRate { minimum_rate });
        }
        validate_monotonicity_functional(&derivative)?;
        validate_enforcement(enforcement)?;
        Ok(Self {
            provenance,
            enforcement,
            kind: LinearConstraintKind::Monotonicity {
                derivative,
                sense,
                minimum_rate,
            },
        })
    }

    /// Borrows complete semantic provenance.
    pub const fn provenance(&self) -> &SemanticProvenance {
        &self.provenance
    }

    /// Returns explicit hard or soft enforcement metadata.
    #[must_use]
    pub const fn enforcement(&self) -> Enforcement {
        self.enforcement
    }

    /// Lowers this value to the shared semantic problem IR relation.
    ///
    /// # Errors
    ///
    /// Returns a structured functional allocation/coefficient error while
    /// combining a scalar gap, or a shared IR validation error.
    pub fn try_into_semantic_constraint(
        self,
    ) -> Result<SemanticConstraint<D>, LinearConstraintError> {
        let relation = match self.kind {
            LinearConstraintKind::Lower {
                functional,
                minimum,
            } => linear_bound(functional, Some(minimum), None)?,
            LinearConstraintKind::Upper {
                functional,
                maximum,
            } => linear_bound(functional, None, Some(maximum))?,
            LinearConstraintKind::Interval {
                functional,
                lower,
                upper,
            } => linear_bound(functional, Some(lower), Some(upper))?,
            LinearConstraintKind::Region {
                functional,
                boundary,
                side,
                orientation,
            } => {
                let inside_is_lower = orientation == InsideOrientation::InsideAtOrBelow;
                let use_upper = (side == RegionSide::Inside) == inside_is_lower;
                if use_upper {
                    linear_bound(functional, None, Some(boundary))?
                } else {
                    linear_bound(functional, Some(boundary), None)?
                }
            }
            LinearConstraintKind::ScalarGap {
                lower,
                upper,
                minimum_gap,
            } => {
                let functional = try_gap_functional(lower, upper)?;
                linear_bound(functional, Some(minimum_gap), None)?
            }
            LinearConstraintKind::Monotonicity {
                derivative,
                sense,
                minimum_rate,
            } => match sense {
                MonotonicitySense::Increasing => {
                    linear_bound(derivative, Some(minimum_rate), None)?
                }
                MonotonicitySense::Decreasing => {
                    linear_bound(derivative, None, Some(-minimum_rate))?
                }
            },
        };
        SemanticConstraint::try_new(self.provenance, relation, self.enforcement)
            .map_err(LinearConstraintError::Ir)
    }
}

fn validate_threshold(value: f64) -> Result<(), LinearConstraintError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(LinearConstraintError::NonFiniteThreshold { value })
    }
}

fn validate_enforcement(enforcement: Enforcement) -> Result<(), LinearConstraintError> {
    enforcement.validate().map_err(LinearConstraintError::Ir)
}

fn validate_monotonicity_functional<const D: usize>(
    functional: &ObservationFunctional<D>,
) -> Result<(), LinearConstraintError>
where
    Dim<D>: SupportedDimension,
{
    let terms = functional.expression().terms();
    if terms.len() != 1 {
        return Err(LinearConstraintError::InvalidMonotonicityTermCount { count: terms.len() });
    }
    let term = terms[0];
    if term.coefficient().to_bits() != 1.0_f64.to_bits() {
        return Err(LinearConstraintError::InvalidMonotonicityCoefficient {
            coefficient: term.coefficient(),
        });
    }
    if !matches!(term.atom(), FunctionalAtom::DirectionalDerivative { .. }) {
        return Err(LinearConstraintError::MonotonicityRequiresDirectionalDerivative);
    }
    Ok(())
}

fn linear_bound<const D: usize>(
    functional: ObservationFunctional<D>,
    lower: Option<f64>,
    upper: Option<f64>,
) -> Result<SemanticRelation<D>, LinearConstraintError>
where
    Dim<D>: SupportedDimension,
{
    Ok(SemanticRelation::LinearBound {
        expression: SemanticExpression::try_new(functional, 0.0)
            .map_err(LinearConstraintError::Ir)?,
        lower,
        upper,
    })
}

fn try_gap_functional<const D: usize>(
    lower: ObservationFunctional<D>,
    upper: ObservationFunctional<D>,
) -> Result<ObservationFunctional<D>, LinearConstraintError>
where
    Dim<D>: SupportedDimension,
{
    let lower_expression = lower.into_expression();
    let upper_expression = upper.into_expression();
    let count = lower_expression
        .term_count()
        .checked_add(upper_expression.term_count())
        .ok_or(LinearConstraintError::TermCountOverflow)?;
    let mut terms = Vec::new();
    terms
        .try_reserve_exact(count)
        .map_err(|_| LinearConstraintError::AllocationFailed { requested: count })?;
    for term in upper_expression.terms() {
        terms.push(*term);
    }
    for term in lower_expression.terms() {
        terms.push(
            FunctionalTerm::try_new(-term.coefficient(), term.atom())
                .map_err(LinearConstraintError::Functional)?,
        );
    }
    FunctionalExpr::try_new(terms)
        .map(ObservationFunctional::new)
        .map_err(LinearConstraintError::Functional)
}

/// Error returned while validating or lowering linear semantics.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum LinearConstraintError {
    /// A threshold or interval endpoint was NaN or infinite.
    NonFiniteThreshold {
        /// Rejected scalar.
        value: f64,
    },
    /// A closed interval had `lower > upper`.
    ReversedInterval {
        /// Lower endpoint.
        lower: f64,
        /// Upper endpoint.
        upper: f64,
    },
    /// A scalar minimum gap was negative or non-finite.
    InvalidMinimumGap {
        /// Rejected gap.
        minimum_gap: f64,
    },
    /// A directional minimum rate was negative or non-finite.
    InvalidMinimumRate {
        /// Rejected rate.
        minimum_rate: f64,
    },
    /// Monotonicity used something other than one functional term.
    InvalidMonotonicityTermCount {
        /// Supplied term count.
        count: usize,
    },
    /// Monotonicity's sole functional coefficient was not exactly one.
    InvalidMonotonicityCoefficient {
        /// Rejected coefficient.
        coefficient: f64,
    },
    /// Monotonicity used a value atom instead of a directional derivative.
    MonotonicityRequiresDirectionalDerivative,
    /// Combining two functional term counts overflowed `usize`.
    TermCountOverflow,
    /// Scalar-gap term storage could not be reserved.
    AllocationFailed {
        /// Exact requested term count.
        requested: usize,
    },
    /// Functional expression construction failed.
    Functional(FunctionalError),
    /// Shared semantic IR validation failed.
    Ir(ProblemIrError),
}

impl fmt::Display for LinearConstraintError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid linear constraint: {self:?}")
    }
}

impl Error for LinearConstraintError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Functional(source) => Some(source),
            Self::Ir(source) => Some(source),
            _ => None,
        }
    }
}
