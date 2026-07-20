//! Scalar-gap labels and sampled local first-order thickness cones.
//!
//! A scalar level gap and a geometric distance are deliberately different
//! quantities. [`LevelOrder::thickness_diagnostics`] labels the former as a
//! scalar-only relation. [`LocalNormalThickness`] represents the sampled local
//! sufficient condition
//!
//! ```text
//! T_min ||grad f(x)||_2 <= h_upper - h_lower,
//! ```
//!
//! where `T_min` and `x` use one caller-consistent coordinate system. It lowers
//! into Cartesian directional derivatives and one hard ordered Lorentz cone.
//! The guarantee is local and first order; this module does not search level
//! sets or claim a global Euclidean separation.
//!
//! Local constraints are available only in one, two, or three dimensions:
//!
//! ```compile_fail
//! use georbf::LocalNormalThickness;
//!
//! let _ = std::mem::size_of::<LocalNormalThickness<4>>();
//! ```

use std::error::Error;
use std::fmt;

use crate::LevelId;
use crate::dimension::{Dim, SupportedDimension};
use crate::functional::{
    FunctionalAtom, FunctionalError, FunctionalExpr, FunctionalProvenance, FunctionalTerm,
    ObservationFunctional,
};
use crate::geometry::{GeometryError, Point, UnitDirection};
use crate::levels::{CompiledLevelProblem, LevelOrder};
use crate::problem_ir::{
    AffineExpression, AffineTerm, CanonicalSecondOrderCone, ObservationId, ProblemIrError,
    SemanticProvenance,
};

/// Stable diagnostic label for the two currently implemented thickness-related relations.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ThicknessDiagnosticKind {
    /// A scalar level-value gap with no geometric-distance guarantee.
    ScalarLevelGap,
    /// A sampled local first-order normal-thickness Lorentz cone.
    SampledLocalNormalCone,
}

/// Explicit extent of the geometric claim made by a diagnostic label.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ThicknessGuarantee {
    /// Only a scalar-field level difference is constrained.
    ScalarOnly,
    /// The condition is sufficient only at the supplied sample point to first order.
    SampledLocalFirstOrder,
}

/// Compact diagnostic classification that prevents scalar and local constraints
/// from being reported as global geometric validation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub struct ThicknessDiagnostics {
    kind: ThicknessDiagnosticKind,
    guarantee: ThicknessGuarantee,
}

impl ThicknessDiagnostics {
    const SCALAR_GAP: Self = Self {
        kind: ThicknessDiagnosticKind::ScalarLevelGap,
        guarantee: ThicknessGuarantee::ScalarOnly,
    };
    const LOCAL_CONE: Self = Self {
        kind: ThicknessDiagnosticKind::SampledLocalNormalCone,
        guarantee: ThicknessGuarantee::SampledLocalFirstOrder,
    };

    /// Returns the stable relation label.
    #[must_use]
    pub const fn kind(self) -> ThicknessDiagnosticKind {
        self.kind
    }

    /// Returns the precise extent of the geometric guarantee.
    #[must_use]
    pub const fn guarantee(self) -> ThicknessGuarantee {
        self.guarantee
    }
}

impl LevelOrder {
    /// Labels this minimum level gap as scalar-only thickness-related evidence.
    pub const fn thickness_diagnostics(&self) -> ThicknessDiagnostics {
        ThicknessDiagnostics::SCALAR_GAP
    }
}

/// One hard sampled local normal-thickness condition between two explicit levels.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct LocalNormalThickness<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    lower: LevelId,
    upper: LevelId,
    point: Point<D>,
    minimum_thickness: f64,
    provenance: SemanticProvenance,
}

impl<const D: usize> LocalNormalThickness<D>
where
    Dim<D>: SupportedDimension,
{
    /// Constructs a sampled local first-order thickness condition.
    ///
    /// `minimum_thickness` must use the same coordinate units as `point` and
    /// the gradient produced by the later linearizer.
    ///
    /// # Errors
    ///
    /// Rejects equal endpoint levels or a non-positive/non-finite thickness.
    pub fn try_new(
        lower: LevelId,
        upper: LevelId,
        point: Point<D>,
        minimum_thickness: f64,
        provenance: SemanticProvenance,
    ) -> Result<Self, LocalNormalThicknessError> {
        if lower == upper {
            return Err(LocalNormalThicknessError::EqualLevels { level_id: lower });
        }
        if !minimum_thickness.is_finite() || minimum_thickness <= 0.0 {
            return Err(LocalNormalThicknessError::InvalidMinimumThickness { minimum_thickness });
        }
        Ok(Self {
            lower,
            upper,
            point,
            minimum_thickness,
            provenance,
        })
    }

    /// Returns the lower level identifier in `h_upper - h_lower`.
    pub const fn lower(&self) -> LevelId {
        self.lower
    }

    /// Returns the upper level identifier in `h_upper - h_lower`.
    pub const fn upper(&self) -> LevelId {
        self.upper
    }

    /// Returns the explicit sample point.
    pub const fn point(&self) -> Point<D> {
        self.point
    }

    /// Returns the positive minimum thickness in the point coordinate units.
    #[must_use]
    pub const fn minimum_thickness(&self) -> f64 {
        self.minimum_thickness
    }

    /// Borrows complete caller-owned provenance.
    pub const fn provenance(&self) -> &SemanticProvenance {
        &self.provenance
    }

    /// Labels the limited local first-order guarantee without claiming sampled
    /// geometric validation.
    pub const fn diagnostics(&self) -> ThicknessDiagnostics {
        ThicknessDiagnostics::LOCAL_CONE
    }
}

impl CompiledLevelProblem {
    /// Appends hard sampled local normal-thickness cones in input order.
    ///
    /// The linearizer is called once per Cartesian derivative, in axis order,
    /// and must return an affine expression over only the field-variable prefix
    /// of this compiled level problem. This method owns the positive thickness
    /// scale and the `h_upper - h_lower` sign. The resulting solver problem has
    /// no geological or thickness-specific vocabulary.
    ///
    /// # Errors
    ///
    /// Returns indexed errors for unknown levels, derivative construction or
    /// linearization failure, an invalid field-variable expression, an
    /// unrepresentable thickness product, duplicate observation identifiers,
    /// allocation failure, or canonical validation failure. No partial problem
    /// is returned.
    #[allow(clippy::too_many_lines)]
    pub fn try_compose_local_normal_thickness<const D: usize, E>(
        self,
        constraints: impl IntoIterator<Item = LocalNormalThickness<D>>,
        mut linearize: impl FnMut(
            &ObservationFunctional<D>,
            &SemanticProvenance,
        ) -> Result<AffineExpression, E>,
    ) -> Result<Self, ThicknessCanonicalizationError<E>>
    where
        Dim<D>: SupportedDimension,
    {
        let constraints = try_collect_constraints(constraints)?;
        if constraints.is_empty() {
            return Err(ThicknessCanonicalizationError::EmptyConstraints);
        }
        for (index, constraint) in constraints.iter().enumerate() {
            let identifier = constraint.provenance.observation_id();
            if self.canonical_problem().contains_observation_id(identifier)
                || constraints[..index]
                    .iter()
                    .any(|existing| existing.provenance.observation_id() == identifier)
            {
                return Err(ThicknessCanonicalizationError::Ir(
                    ProblemIrError::DuplicateObservationId { identifier },
                ));
            }
        }
        let field_variable_count = self
            .canonical_problem()
            .variable_blocks()
            .last()
            .map(|(_, offset, _)| offset)
            .ok_or(ThicknessCanonicalizationError::InvalidCompiledLevelProblem)?;
        let mut cones = Vec::new();
        cones.try_reserve_exact(constraints.len()).map_err(|_| {
            ThicknessCanonicalizationError::AllocationFailed {
                requested: constraints.len(),
            }
        })?;

        for (constraint_index, constraint) in constraints.iter().enumerate() {
            let lower_variable = self.level_variable(constraint.lower).ok_or(
                ThicknessCanonicalizationError::UnknownLowerLevel {
                    constraint_index,
                    level_id: constraint.lower,
                },
            )?;
            let upper_variable = self.level_variable(constraint.upper).ok_or(
                ThicknessCanonicalizationError::UnknownUpperLevel {
                    constraint_index,
                    level_id: constraint.upper,
                },
            )?;
            let mut lhs = Vec::new();
            lhs.try_reserve_exact(D)
                .map_err(|_| ThicknessCanonicalizationError::AllocationFailed { requested: D })?;
            for axis in 0..D {
                let functional =
                    cartesian_derivative(constraint, axis).map_err(|source| match source {
                        CartesianDerivativeError::Geometry(source) => {
                            ThicknessCanonicalizationError::Geometry {
                                constraint_index,
                                axis,
                                source,
                            }
                        }
                        CartesianDerivativeError::Functional(source) => {
                            ThicknessCanonicalizationError::Functional {
                                constraint_index,
                                axis,
                                source,
                            }
                        }
                    })?;
                let affine = linearize(&functional, &constraint.provenance).map_err(|source| {
                    ThicknessCanonicalizationError::Linearization {
                        constraint_index,
                        axis,
                        observation_id: constraint.provenance.observation_id(),
                        source,
                    }
                })?;
                affine
                    .validate_variable_count(field_variable_count)
                    .map_err(
                        |source| ThicknessCanonicalizationError::InvalidLinearization {
                            constraint_index,
                            axis,
                            observation_id: constraint.provenance.observation_id(),
                            source,
                        },
                    )?;
                lhs.push(scale_affine(&affine, constraint.minimum_thickness).map_err(
                    |failure| match failure {
                        ScaledAffineError::Coefficient {
                            variable,
                            input,
                            scale,
                            result,
                        } => ThicknessCanonicalizationError::ScaledGradientCoefficientNotRepresentable {
                            constraint_index,
                            axis,
                            observation_id: constraint.provenance.observation_id(),
                            variable,
                            input,
                            scale,
                            result,
                        },
                        ScaledAffineError::Constant {
                            input,
                            scale,
                            result,
                        } => ThicknessCanonicalizationError::ScaledGradientConstantNotRepresentable {
                            constraint_index,
                            axis,
                            observation_id: constraint.provenance.observation_id(),
                            input,
                            scale,
                            result,
                        },
                        ScaledAffineError::Ir(source) => {
                            ThicknessCanonicalizationError::UnrepresentableScaledGradient {
                                constraint_index,
                                axis,
                                observation_id: constraint.provenance.observation_id(),
                                source,
                            }
                        }
                    },
                )?);
            }
            let rhs = level_gap_expression(lower_variable, upper_variable)
                .map_err(ThicknessCanonicalizationError::Ir)?;
            cones.push(CanonicalSecondOrderCone::from_parts(
                lhs,
                rhs,
                constraint
                    .provenance
                    .try_clone_for_canonical()
                    .map_err(ThicknessCanonicalizationError::Ir)?,
            ));
        }

        self.try_append_hard_cones(cones)
            .map_err(ThicknessCanonicalizationError::Ir)
    }
}

fn try_collect_constraints<const D: usize, E>(
    constraints: impl IntoIterator<Item = LocalNormalThickness<D>>,
) -> Result<Vec<LocalNormalThickness<D>>, ThicknessCanonicalizationError<E>>
where
    Dim<D>: SupportedDimension,
{
    let iterator = constraints.into_iter();
    let minimum = iterator.size_hint().0;
    let mut values = Vec::new();
    if minimum > 0 {
        values
            .try_reserve_exact(minimum)
            .map_err(|_| ThicknessCanonicalizationError::AllocationFailed { requested: minimum })?;
    }
    for value in iterator {
        if values.len() == values.capacity() {
            let requested = values.len().saturating_add(1);
            try_reserve_constraint_growth(&mut values, requested)?;
        }
        values.push(value);
    }
    Ok(values)
}

#[cfg(test)]
std::thread_local! {
    static FORCE_CONSTRAINT_GROWTH_ALLOCATION_FAILURE: std::cell::Cell<bool> =
        const { std::cell::Cell::new(false) };
}

fn try_reserve_constraint_growth<T, E>(
    values: &mut Vec<T>,
    requested: usize,
) -> Result<(), ThicknessCanonicalizationError<E>> {
    #[cfg(test)]
    if FORCE_CONSTRAINT_GROWTH_ALLOCATION_FAILURE.with(|force| force.replace(false)) {
        return Err(ThicknessCanonicalizationError::AllocationFailed { requested });
    }

    values
        .try_reserve(1)
        .map_err(|_| ThicknessCanonicalizationError::AllocationFailed { requested })
}

fn cartesian_derivative<const D: usize>(
    constraint: &LocalNormalThickness<D>,
    axis: usize,
) -> Result<ObservationFunctional<D>, CartesianDerivativeError>
where
    Dim<D>: SupportedDimension,
{
    let mut components = [0.0; D];
    components[axis] = 1.0;
    let direction =
        UnitDirection::try_new(components).map_err(CartesianDerivativeError::Geometry)?;
    let atom = FunctionalAtom::directional_derivative(
        constraint.point,
        direction,
        FunctionalProvenance::new(constraint.provenance.observation_id().identifier()),
    );
    Ok(ObservationFunctional::new(
        FunctionalExpr::try_new([
            FunctionalTerm::try_new(1.0, atom).map_err(CartesianDerivativeError::Functional)?
        ])
        .map_err(CartesianDerivativeError::Functional)?,
    ))
}

enum CartesianDerivativeError {
    Geometry(GeometryError),
    Functional(FunctionalError),
}

fn scale_affine(
    affine: &AffineExpression,
    scale: f64,
) -> Result<AffineExpression, ScaledAffineError> {
    let mut terms = Vec::new();
    terms.try_reserve_exact(affine.terms().len()).map_err(|_| {
        ScaledAffineError::Ir(ProblemIrError::AllocationFailed {
            storage: crate::problem_ir::ProblemIrStorage::AffineTerms,
            requested: affine.terms().len(),
        })
    })?;
    for term in affine.terms() {
        let coefficient = term.coefficient() * scale;
        if !coefficient.is_finite() || coefficient == 0.0 {
            return Err(ScaledAffineError::Coefficient {
                variable: term.variable(),
                input: term.coefficient(),
                scale,
                result: coefficient,
            });
        }
        terms.push(
            AffineTerm::try_new(term.variable(), coefficient).map_err(ScaledAffineError::Ir)?,
        );
    }
    let constant = affine.constant() * scale;
    if !constant.is_finite() || (affine.constant() != 0.0 && constant == 0.0) {
        return Err(ScaledAffineError::Constant {
            input: affine.constant(),
            scale,
            result: constant,
        });
    }
    AffineExpression::try_new(terms, constant).map_err(ScaledAffineError::Ir)
}

enum ScaledAffineError {
    Coefficient {
        variable: usize,
        input: f64,
        scale: f64,
        result: f64,
    },
    Constant {
        input: f64,
        scale: f64,
        result: f64,
    },
    Ir(ProblemIrError),
}

fn level_gap_expression(
    lower_variable: usize,
    upper_variable: usize,
) -> Result<AffineExpression, ProblemIrError> {
    let terms = if lower_variable < upper_variable {
        [
            AffineTerm::try_new(lower_variable, -1.0)?,
            AffineTerm::try_new(upper_variable, 1.0)?,
        ]
    } else {
        [
            AffineTerm::try_new(upper_variable, 1.0)?,
            AffineTerm::try_new(lower_variable, -1.0)?,
        ]
    };
    AffineExpression::try_new(terms, 0.0)
}

/// Error returned while constructing one local thickness condition.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum LocalNormalThicknessError {
    /// The lower and upper level identifiers were equal.
    EqualLevels {
        /// Rejected identifier.
        level_id: LevelId,
    },
    /// The minimum thickness was not finite and positive.
    InvalidMinimumThickness {
        /// Rejected value.
        minimum_thickness: f64,
    },
}

impl fmt::Display for LocalNormalThicknessError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid local normal-thickness constraint: {self:?}"
        )
    }
}

impl Error for LocalNormalThicknessError {}

/// Error returned while appending local cones to an explicit level problem.
#[derive(Debug)]
#[non_exhaustive]
pub enum ThicknessCanonicalizationError<E> {
    /// No local constraint was supplied.
    EmptyConstraints,
    /// The compiled problem did not expose its field/level variable boundary.
    InvalidCompiledLevelProblem,
    /// A lower level identifier was absent from the compiled level problem.
    UnknownLowerLevel {
        /// Zero-based local constraint index.
        constraint_index: usize,
        /// Missing identifier.
        level_id: LevelId,
    },
    /// An upper level identifier was absent from the compiled level problem.
    UnknownUpperLevel {
        /// Zero-based local constraint index.
        constraint_index: usize,
        /// Missing identifier.
        level_id: LevelId,
    },
    /// Constructing one Cartesian unit direction failed.
    Geometry {
        /// Zero-based local constraint index.
        constraint_index: usize,
        /// Zero-based Cartesian axis.
        axis: usize,
        /// Geometry diagnostic.
        source: GeometryError,
    },
    /// Constructing one Cartesian derivative functional failed.
    Functional {
        /// Zero-based local constraint index.
        constraint_index: usize,
        /// Zero-based Cartesian axis.
        axis: usize,
        /// Functional diagnostic.
        source: FunctionalError,
    },
    /// The caller's derivative linearizer failed.
    Linearization {
        /// Zero-based local constraint index.
        constraint_index: usize,
        /// Zero-based Cartesian axis.
        axis: usize,
        /// Stable observation identifier.
        observation_id: ObservationId,
        /// Caller diagnostic.
        source: E,
    },
    /// A derivative linearization referenced variables outside the field prefix.
    InvalidLinearization {
        /// Zero-based local constraint index.
        constraint_index: usize,
        /// Zero-based Cartesian axis.
        axis: usize,
        /// Stable observation identifier.
        observation_id: ObservationId,
        /// Canonical diagnostic.
        source: ProblemIrError,
    },
    /// One scaled gradient coefficient overflowed or underflowed to zero.
    ScaledGradientCoefficientNotRepresentable {
        /// Zero-based local constraint index.
        constraint_index: usize,
        /// Zero-based Cartesian axis.
        axis: usize,
        /// Stable observation identifier.
        observation_id: ObservationId,
        /// Canonical field-variable index.
        variable: usize,
        /// Unscaled finite nonzero coefficient.
        input: f64,
        /// Positive minimum-thickness scale.
        scale: f64,
        /// Non-finite or zero product.
        result: f64,
    },
    /// One scaled gradient constant overflowed or underflowed to zero.
    ScaledGradientConstantNotRepresentable {
        /// Zero-based local constraint index.
        constraint_index: usize,
        /// Zero-based Cartesian axis.
        axis: usize,
        /// Stable observation identifier.
        observation_id: ObservationId,
        /// Unscaled finite nonzero constant.
        input: f64,
        /// Positive minimum-thickness scale.
        scale: f64,
        /// Non-finite or zero product.
        result: f64,
    },
    /// Multiplying a derivative expression by the thickness was not representable.
    UnrepresentableScaledGradient {
        /// Zero-based local constraint index.
        constraint_index: usize,
        /// Zero-based Cartesian axis.
        axis: usize,
        /// Stable observation identifier.
        observation_id: ObservationId,
        /// Canonical diagnostic.
        source: ProblemIrError,
    },
    /// Constraint collection or cone storage could not be reserved.
    AllocationFailed {
        /// Requested entry count.
        requested: usize,
    },
    /// Shared canonical validation failed.
    Ir(ProblemIrError),
}

impl<E> fmt::Display for ThicknessCanonicalizationError<E>
where
    E: fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "local thickness canonicalization failed: {self:?}"
        )
    }
}

impl<E> Error for ThicknessCanonicalizationError<E>
where
    E: Error + 'static,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Geometry { source, .. } => Some(source),
            Self::Functional { source, .. } => Some(source),
            Self::Linearization { source, .. } => Some(source),
            Self::InvalidLinearization { source, .. }
            | Self::UnrepresentableScaledGradient { source, .. }
            | Self::Ir(source) => Some(source),
            Self::EmptyConstraints
            | Self::InvalidCompiledLevelProblem
            | Self::UnknownLowerLevel { .. }
            | Self::UnknownUpperLevel { .. }
            | Self::ScaledGradientCoefficientNotRepresentable { .. }
            | Self::ScaledGradientConstantNotRepresentable { .. }
            | Self::AllocationFailed { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;

    use super::*;
    use crate::levels::{LevelDefinition, LevelMembership, LevelProblem, LevelValue};
    use crate::problem_ir::{SourceLocation, VariableBlock};

    type TestResult<T = ()> = Result<T, Box<dyn Error>>;

    struct ForcedConstraintGrowthAllocationFailure;

    impl ForcedConstraintGrowthAllocationFailure {
        fn new() -> Self {
            FORCE_CONSTRAINT_GROWTH_ALLOCATION_FAILURE.with(|force| force.set(true));
            Self
        }
    }

    impl Drop for ForcedConstraintGrowthAllocationFailure {
        fn drop(&mut self) {
            FORCE_CONSTRAINT_GROWTH_ALLOCATION_FAILURE.with(|force| force.set(false));
        }
    }

    struct UnknownLength<T, const N: usize> {
        inner: std::array::IntoIter<T, N>,
    }

    impl<T, const N: usize> Iterator for UnknownLength<T, N> {
        type Item = T;

        fn next(&mut self) -> Option<Self::Item> {
            self.inner.next()
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (0, None)
        }
    }

    fn provenance(identifier: u64, field_path: &str) -> TestResult<SemanticProvenance> {
        Ok(SemanticProvenance::try_new(
            ObservationId::new(identifier),
            SourceLocation::try_new(
                "tests/thickness.rs".to_owned(),
                NonZeroUsize::new(usize::try_from(identifier)? + 1).ok_or("line")?,
            )?,
            "m".to_owned(),
            field_path.to_owned(),
            Some("thickness".to_owned()),
        )?)
    }

    fn value_functional(identifier: u64, x: f64) -> TestResult<ObservationFunctional<1>> {
        Ok(ObservationFunctional::new(FunctionalExpr::try_new([
            FunctionalTerm::try_new(
                1.0,
                FunctionalAtom::value(Point::try_new([x])?, FunctionalProvenance::new(identifier)),
            )?,
        ])?))
    }

    fn compiled_level_problem() -> TestResult<CompiledLevelProblem> {
        let lower = LevelId::new(10);
        let upper = LevelId::new(20);
        let levels = LevelProblem::try_new(
            [
                LevelDefinition::new(
                    lower,
                    LevelValue::try_fixed(0.0)?,
                    provenance(100, "levels.lower")?,
                ),
                LevelDefinition::new(
                    upper,
                    LevelValue::try_fixed(10.0)?,
                    provenance(101, "levels.upper")?,
                ),
            ],
            [
                LevelMembership::new(
                    lower,
                    value_functional(200, 0.0)?,
                    provenance(200, "memberships.lower")?,
                ),
                LevelMembership::new(
                    upper,
                    value_functional(201, 10.0)?,
                    provenance(201, "memberships.upper")?,
                ),
            ],
            [LevelOrder::try_new(
                lower,
                upper,
                2.0,
                provenance(300, "orders.scalar_gap")?,
            )?],
        )?;
        let mut membership_variable = 0_usize;
        Ok(levels.try_compile(
            [VariableBlock::try_new(
                "field".to_owned(),
                NonZeroUsize::new(3).ok_or("field block")?,
            )?],
            |_, _| {
                let variable = membership_variable;
                membership_variable += 1;
                AffineExpression::try_new([AffineTerm::try_new(variable, 1.0)?], 0.0)
            },
        )?)
    }

    fn local_constraint(identifier: u64) -> TestResult<LocalNormalThickness<1>> {
        Ok(LocalNormalThickness::try_new(
            LevelId::new(10),
            LevelId::new(20),
            Point::try_new([0.5])?,
            1.0,
            provenance(identifier, &format!("local_thickness[{identifier}]"))?,
        )?)
    }

    #[test]
    fn unknown_length_growth_allocation_failure_is_structured_without_partial_problem()
    -> TestResult<()> {
        let compiled = compiled_level_problem()?;
        let constraints = UnknownLength {
            inner: [local_constraint(400)?, local_constraint(401)?].into_iter(),
        };
        let _failure = ForcedConstraintGrowthAllocationFailure::new();
        let mut linearizer_called = false;
        let result = compiled.try_compose_local_normal_thickness(constraints, |_, _| {
            linearizer_called = true;
            AffineExpression::try_new([AffineTerm::try_new(2, 1.0)?], 0.0)
        });

        assert!(matches!(
            result,
            Err(ThicknessCanonicalizationError::AllocationFailed { requested: 1 })
        ));
        assert!(!linearizer_called);
        Ok(())
    }
}
