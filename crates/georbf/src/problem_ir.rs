//! Provenance-preserving semantic and solver-neutral canonical problem IRs.
//!
//! The semantic layer owns user-facing meaning and compiled observation
//! functionals. Canonicalization accepts an explicit caller linearizer so the
//! later basis and assembly layers remain separate from relation mapping.
//!
//! IR dimension bounds are compile-time enforced:
//!
//! ```compile_fail
//! use georbf::SemanticProblemIr;
//!
//! let _ = std::mem::size_of::<SemanticProblemIr<4>>();
//! ```

use std::error::Error;
use std::fmt;
use std::num::NonZeroUsize;

#[cfg(test)]
use std::cell::Cell;

use crate::dimension::{Dim, SupportedDimension};
use crate::functional::ObservationFunctional;

/// Stable observation identifier used by semantic diagnostics.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[must_use]
pub struct ObservationId(u64);

impl ObservationId {
    /// Constructs an identifier from a caller-controlled stable value.
    pub const fn new(identifier: u64) -> Self {
        Self(identifier)
    }

    /// Returns the caller-controlled value.
    #[must_use]
    pub const fn identifier(self) -> u64 {
        self.0
    }
}

/// One-based source location for a semantic observation.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub struct SourceLocation {
    path: String,
    line: NonZeroUsize,
}

impl SourceLocation {
    /// Validates and constructs a source location.
    ///
    /// # Errors
    ///
    /// Returns [`ProblemIrError::EmptyMetadata`] for an empty path.
    pub fn try_new(path: String, line: NonZeroUsize) -> Result<Self, ProblemIrError> {
        validate_text(&path, SemanticMetadataField::SourcePath)?;
        Ok(Self { path, line })
    }

    /// Borrows the source path exactly as supplied.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns the one-based source line.
    #[must_use]
    pub const fn line(&self) -> NonZeroUsize {
        self.line
    }
}

/// Complete source provenance retained for one semantic constraint.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub struct SemanticProvenance {
    observation_id: ObservationId,
    source: SourceLocation,
    original_units: String,
    field_path: String,
    constraint_group: Option<String>,
}

impl SemanticProvenance {
    /// Validates and constructs semantic provenance.
    ///
    /// # Errors
    ///
    /// Returns [`ProblemIrError::EmptyMetadata`] when units, field path, or a
    /// supplied constraint group is empty.
    pub fn try_new(
        observation_id: ObservationId,
        source: SourceLocation,
        original_units: String,
        field_path: String,
        constraint_group: Option<String>,
    ) -> Result<Self, ProblemIrError> {
        validate_text(&original_units, SemanticMetadataField::OriginalUnits)?;
        validate_text(&field_path, SemanticMetadataField::FieldPath)?;
        if let Some(group) = &constraint_group {
            validate_text(group, SemanticMetadataField::ConstraintGroup)?;
        }
        Ok(Self {
            observation_id,
            source,
            original_units,
            field_path,
            constraint_group,
        })
    }

    /// Returns the stable observation identifier.
    pub const fn observation_id(&self) -> ObservationId {
        self.observation_id
    }

    /// Borrows the source location.
    pub const fn source(&self) -> &SourceLocation {
        &self.source
    }

    /// Borrows the original unit label.
    #[must_use]
    pub fn original_units(&self) -> &str {
        &self.original_units
    }

    /// Borrows the semantic field path.
    #[must_use]
    pub fn field_path(&self) -> &str {
        &self.field_path
    }

    /// Borrows the optional constraint group.
    #[must_use]
    pub fn constraint_group(&self) -> Option<&str> {
        self.constraint_group.as_deref()
    }

    fn try_clone_for_canonical(&self) -> Result<Self, ProblemIrError> {
        Ok(Self {
            observation_id: self.observation_id,
            source: SourceLocation {
                path: try_clone_provenance_text(&self.source.path)?,
                line: self.source.line,
            },
            original_units: try_clone_provenance_text(&self.original_units)?,
            field_path: try_clone_provenance_text(&self.field_path)?,
            constraint_group: self
                .constraint_group
                .as_deref()
                .map(try_clone_provenance_text)
                .transpose()?,
        })
    }
}

/// Semantic metadata field rejected during validation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SemanticMetadataField {
    /// Source path.
    SourcePath,
    /// Original unit label.
    OriginalUnits,
    /// Field path.
    FieldPath,
    /// Constraint group.
    ConstraintGroup,
    /// Canonical variable-block name.
    VariableBlockName,
}

/// Explicit soft-loss family retained in semantic IR.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SoftLoss {
    /// Squared L2 loss.
    SquaredL2,
    /// Absolute L1 loss.
    AbsoluteL1,
    /// Huber loss with a positive transition value.
    Huber {
        /// Positive Huber transition value.
        delta: f64,
    },
}

/// Hard feasibility or an explicit soft penalty.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Enforcement {
    /// Preserve the relation as a hard feasibility condition.
    Hard,
    /// Retain an explicit soft scale and loss for a later objective compiler.
    Soft {
        /// Positive residual scale.
        scale: f64,
        /// Explicit loss family.
        loss: SoftLoss,
    },
}

impl Enforcement {
    fn validate(self) -> Result<(), ProblemIrError> {
        let Self::Soft { scale, loss } = self else {
            return Ok(());
        };
        if !scale.is_finite() || scale <= 0.0 {
            return Err(ProblemIrError::InvalidSoftScale { value: scale });
        }
        if let SoftLoss::Huber { delta } = loss
            && (!delta.is_finite() || delta <= 0.0)
        {
            return Err(ProblemIrError::InvalidHuberDelta { value: delta });
        }
        Ok(())
    }
}

/// Explicit execution metadata retained with a semantic problem.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub struct ExecutionOptions {
    deterministic: bool,
    thread_count: Option<NonZeroUsize>,
    memory_limit_bytes: Option<NonZeroUsize>,
}

impl ExecutionOptions {
    /// Constructs execution metadata without selecting a runtime or solver.
    pub const fn new(
        deterministic: bool,
        thread_count: Option<NonZeroUsize>,
        memory_limit_bytes: Option<NonZeroUsize>,
    ) -> Self {
        Self {
            deterministic,
            thread_count,
            memory_limit_bytes,
        }
    }

    /// Returns whether deterministic execution was requested.
    #[must_use]
    pub const fn deterministic(self) -> bool {
        self.deterministic
    }

    /// Returns the explicit thread count, if any.
    #[must_use]
    pub const fn thread_count(self) -> Option<NonZeroUsize> {
        self.thread_count
    }

    /// Returns the explicit memory limit, if any.
    #[must_use]
    pub const fn memory_limit_bytes(self) -> Option<NonZeroUsize> {
        self.memory_limit_bytes
    }
}

impl Default for ExecutionOptions {
    fn default() -> Self {
        Self::new(true, None, None)
    }
}

/// One compiled functional expression plus a finite scalar constant.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct SemanticExpression<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    functional: ObservationFunctional<D>,
    constant: f64,
}

impl<const D: usize> SemanticExpression<D>
where
    Dim<D>: SupportedDimension,
{
    /// Validates and constructs a semantic affine expression.
    ///
    /// # Errors
    ///
    /// Returns [`ProblemIrError::NonFiniteSemanticScalar`] for NaN or
    /// infinity.
    pub fn try_new(
        functional: ObservationFunctional<D>,
        constant: f64,
    ) -> Result<Self, ProblemIrError> {
        if !constant.is_finite() {
            return Err(ProblemIrError::NonFiniteSemanticScalar { value: constant });
        }
        Ok(Self {
            functional,
            constant,
        })
    }

    /// Borrows the compiled observation functional.
    pub const fn functional(&self) -> &ObservationFunctional<D> {
        &self.functional
    }

    /// Returns the finite constant.
    #[must_use]
    pub const fn constant(&self) -> f64 {
        self.constant
    }
}

/// A semantic constraint relation over compiled functional expressions.
#[derive(Clone, Debug, PartialEq)]
pub enum SemanticRelation<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    /// One affine expression equal to a finite target.
    Equality {
        /// Left-hand expression.
        expression: SemanticExpression<D>,
        /// Right-hand target.
        target: f64,
    },
    /// One affine expression with a lower bound, upper bound, or both.
    LinearBound {
        /// Bounded expression.
        expression: SemanticExpression<D>,
        /// Optional finite lower bound.
        lower: Option<f64>,
        /// Optional finite upper bound.
        upper: Option<f64>,
    },
    /// A second-order cone relation `||lhs||_2 <= rhs`.
    SecondOrderCone {
        /// Nonempty vector-valued left-hand expression.
        lhs: Vec<SemanticExpression<D>>,
        /// Scalar right-hand expression.
        rhs: SemanticExpression<D>,
    },
}

impl<const D: usize> SemanticRelation<D>
where
    Dim<D>: SupportedDimension,
{
    fn validate(&self) -> Result<(), ProblemIrError> {
        match self {
            Self::Equality { target, .. } => validate_relation_scalar(*target),
            Self::LinearBound { lower, upper, .. } => {
                if lower.is_none() && upper.is_none() {
                    return Err(ProblemIrError::MissingLinearBound);
                }
                if let Some(value) = lower {
                    validate_relation_scalar(*value)?;
                }
                if let Some(value) = upper {
                    validate_relation_scalar(*value)?;
                }
                if let (Some(lower), Some(upper)) = (lower, upper)
                    && lower > upper
                {
                    return Err(ProblemIrError::ReversedLinearBounds {
                        lower: *lower,
                        upper: *upper,
                    });
                }
                Ok(())
            }
            Self::SecondOrderCone { lhs, .. } => {
                if lhs.is_empty() {
                    Err(ProblemIrError::EmptyConeLeftHandSide)
                } else {
                    Ok(())
                }
            }
        }
    }
}

/// One provenance-bearing semantic constraint.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct SemanticConstraint<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    provenance: SemanticProvenance,
    relation: SemanticRelation<D>,
    enforcement: Enforcement,
}

impl<const D: usize> SemanticConstraint<D>
where
    Dim<D>: SupportedDimension,
{
    /// Validates and constructs a semantic constraint.
    ///
    /// # Errors
    ///
    /// Returns a structured relation or enforcement error.
    pub fn try_new(
        provenance: SemanticProvenance,
        relation: SemanticRelation<D>,
        enforcement: Enforcement,
    ) -> Result<Self, ProblemIrError> {
        relation.validate()?;
        enforcement.validate()?;
        Ok(Self {
            provenance,
            relation,
            enforcement,
        })
    }

    /// Borrows complete semantic provenance.
    pub const fn provenance(&self) -> &SemanticProvenance {
        &self.provenance
    }

    /// Borrows the semantic relation.
    #[must_use]
    pub const fn relation(&self) -> &SemanticRelation<D> {
        &self.relation
    }

    /// Returns explicit enforcement metadata.
    #[must_use]
    pub const fn enforcement(&self) -> Enforcement {
        self.enforcement
    }
}

/// Immutable provenance-preserving semantic problem IR.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct SemanticProblemIr<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    constraints: Vec<SemanticConstraint<D>>,
    execution: ExecutionOptions,
}

impl<const D: usize> SemanticProblemIr<D>
where
    Dim<D>: SupportedDimension,
{
    /// Constructs a nonempty problem in deterministic iterator order.
    ///
    /// # Errors
    ///
    /// Rejects empty input, duplicate observation identifiers, or allocation
    /// failure without returning a partial problem.
    pub fn try_new(
        constraints: impl IntoIterator<Item = SemanticConstraint<D>>,
        execution: ExecutionOptions,
    ) -> Result<Self, ProblemIrError> {
        let constraints = try_collect(constraints, ProblemIrStorage::SemanticConstraints)?;
        if constraints.is_empty() {
            return Err(ProblemIrError::EmptySemanticProblem);
        }
        for index in 0..constraints.len() {
            let identifier = constraints[index].provenance.observation_id;
            if constraints[..index]
                .iter()
                .any(|constraint| constraint.provenance.observation_id == identifier)
            {
                return Err(ProblemIrError::DuplicateObservationId { identifier });
            }
        }
        Ok(Self {
            constraints,
            execution,
        })
    }

    /// Borrows constraints in deterministic insertion order.
    pub fn constraints(&self) -> &[SemanticConstraint<D>] {
        &self.constraints
    }

    /// Returns execution metadata.
    pub const fn execution_options(&self) -> ExecutionOptions {
        self.execution
    }

    /// Compiles this semantic IR through an explicit affine linearizer.
    ///
    /// The linearizer is called in semantic insertion order and once per
    /// referenced functional expression. It must not insert relation signs or
    /// constants: those remain owned by this compiler.
    ///
    /// # Errors
    ///
    /// Returns structured source-indexed errors for unsupported soft paths,
    /// linearizer failures, invalid affine output, non-finite shifted bounds,
    /// allocation failure, or memory-estimate overflow.
    #[allow(clippy::too_many_lines)]
    pub fn try_compile<E>(
        &self,
        variable_blocks: impl IntoIterator<Item = VariableBlock>,
        mut linearize: impl FnMut(
            &ObservationFunctional<D>,
            &SemanticProvenance,
        ) -> Result<AffineExpression, E>,
    ) -> Result<CanonicalProblem, CanonicalizationError<E>> {
        let blocks = VariableSpace::try_new(variable_blocks).map_err(CanonicalizationError::Ir)?;
        let mut equalities = Vec::new();
        let mut linear_bounds = Vec::new();
        let mut cones = Vec::new();

        equalities
            .try_reserve_exact(self.constraints.len())
            .map_err(|_| {
                allocation_error(
                    ProblemIrStorage::CanonicalEqualities,
                    self.constraints.len(),
                )
            })?;
        linear_bounds
            .try_reserve_exact(self.constraints.len())
            .map_err(|_| {
                allocation_error(
                    ProblemIrStorage::CanonicalLinearBounds,
                    self.constraints.len(),
                )
            })?;
        cones
            .try_reserve_exact(self.constraints.len())
            .map_err(|_| {
                allocation_error(ProblemIrStorage::CanonicalCones, self.constraints.len())
            })?;

        for (constraint_index, constraint) in self.constraints.iter().enumerate() {
            if matches!(constraint.enforcement, Enforcement::Soft { .. }) {
                return Err(CanonicalizationError::UnsupportedSoftEnforcement {
                    constraint_index,
                    observation_id: constraint.provenance.observation_id,
                });
            }
            match &constraint.relation {
                SemanticRelation::Equality { expression, target } => {
                    let affine = linearize_expression(
                        expression,
                        &constraint.provenance,
                        constraint_index,
                        0,
                        blocks.variable_count,
                        &mut linearize,
                    )?;
                    let rhs = checked_subtract(*target, affine.constant).map_err(|value| {
                        CanonicalizationError::NonFiniteShiftedScalar {
                            constraint_index,
                            observation_id: constraint.provenance.observation_id,
                            value,
                        }
                    })?;
                    equalities.push(CanonicalEquality {
                        row: affine.without_constant(),
                        rhs,
                        provenance: constraint
                            .provenance
                            .try_clone_for_canonical()
                            .map_err(CanonicalizationError::Ir)?,
                    });
                }
                SemanticRelation::LinearBound {
                    expression,
                    lower,
                    upper,
                } => {
                    let affine = linearize_expression(
                        expression,
                        &constraint.provenance,
                        constraint_index,
                        0,
                        blocks.variable_count,
                        &mut linearize,
                    )?;
                    let shifted_lower =
                        shift_optional_bound(*lower, affine.constant).map_err(|value| {
                            CanonicalizationError::NonFiniteShiftedScalar {
                                constraint_index,
                                observation_id: constraint.provenance.observation_id,
                                value,
                            }
                        })?;
                    let shifted_upper =
                        shift_optional_bound(*upper, affine.constant).map_err(|value| {
                            CanonicalizationError::NonFiniteShiftedScalar {
                                constraint_index,
                                observation_id: constraint.provenance.observation_id,
                                value,
                            }
                        })?;
                    linear_bounds.push(CanonicalLinearBound {
                        row: affine.without_constant(),
                        lower: shifted_lower,
                        upper: shifted_upper,
                        provenance: constraint
                            .provenance
                            .try_clone_for_canonical()
                            .map_err(CanonicalizationError::Ir)?,
                    });
                }
                SemanticRelation::SecondOrderCone { lhs, rhs } => {
                    let mut canonical_lhs = Vec::new();
                    canonical_lhs.try_reserve_exact(lhs.len()).map_err(|_| {
                        allocation_error(ProblemIrStorage::CanonicalConeRows, lhs.len())
                    })?;
                    for (expression_index, expression) in lhs.iter().enumerate() {
                        canonical_lhs.push(linearize_expression(
                            expression,
                            &constraint.provenance,
                            constraint_index,
                            expression_index,
                            blocks.variable_count,
                            &mut linearize,
                        )?);
                    }
                    let canonical_rhs = linearize_expression(
                        rhs,
                        &constraint.provenance,
                        constraint_index,
                        lhs.len(),
                        blocks.variable_count,
                        &mut linearize,
                    )?;
                    cones.push(CanonicalSecondOrderCone {
                        lhs: canonical_lhs,
                        rhs: canonical_rhs,
                        provenance: constraint
                            .provenance
                            .try_clone_for_canonical()
                            .map_err(CanonicalizationError::Ir)?,
                    });
                }
            }
        }

        CanonicalProblem::try_new(blocks, equalities, linear_bounds, cones)
            .map_err(CanonicalizationError::Ir)
    }
}

/// A named nonempty block of canonical solver variables.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub struct VariableBlock {
    name: String,
    len: NonZeroUsize,
}

impl VariableBlock {
    /// Validates and constructs a variable block.
    ///
    /// # Errors
    ///
    /// Returns [`ProblemIrError::EmptyMetadata`] for an empty name.
    pub fn try_new(name: String, len: NonZeroUsize) -> Result<Self, ProblemIrError> {
        validate_text(&name, SemanticMetadataField::VariableBlockName)?;
        Ok(Self { name, len })
    }

    /// Borrows the stable block name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the nonzero block length.
    #[must_use]
    pub const fn len(&self) -> NonZeroUsize {
        self.len
    }

    /// Returns `false`; canonical blocks cannot be empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        false
    }
}

/// A finite nonzero sparse coefficient for one canonical variable.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct AffineTerm {
    variable: usize,
    coefficient: f64,
}

impl AffineTerm {
    /// Validates and constructs a sparse affine term.
    ///
    /// # Errors
    ///
    /// Rejects a zero or non-finite coefficient.
    pub fn try_new(variable: usize, coefficient: f64) -> Result<Self, ProblemIrError> {
        if !coefficient.is_finite() {
            return Err(ProblemIrError::NonFiniteAffineCoefficient {
                variable,
                value: coefficient,
            });
        }
        if coefficient == 0.0 {
            return Err(ProblemIrError::ZeroAffineCoefficient { variable });
        }
        Ok(Self {
            variable,
            coefficient,
        })
    }

    /// Returns the zero-based canonical variable index.
    #[must_use]
    pub const fn variable(self) -> usize {
        self.variable
    }

    /// Returns the finite nonzero coefficient.
    #[must_use]
    pub const fn coefficient(self) -> f64 {
        self.coefficient
    }
}

/// A deterministic sparse affine expression `a^T z + constant`.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct AffineExpression {
    terms: Vec<AffineTerm>,
    constant: f64,
}

impl AffineExpression {
    /// Constructs an affine expression with strictly increasing variable indices.
    ///
    /// # Errors
    ///
    /// Rejects non-finite constants, unsorted or duplicate variable indices,
    /// or allocation failure.
    pub fn try_new(
        terms: impl IntoIterator<Item = AffineTerm>,
        constant: f64,
    ) -> Result<Self, ProblemIrError> {
        if !constant.is_finite() {
            return Err(ProblemIrError::NonFiniteAffineConstant { value: constant });
        }
        let terms = try_collect(terms, ProblemIrStorage::AffineTerms)?;
        for pair in terms.windows(2) {
            if pair[0].variable >= pair[1].variable {
                return Err(ProblemIrError::NonIncreasingAffineIndices {
                    previous: pair[0].variable,
                    current: pair[1].variable,
                });
            }
        }
        Ok(Self { terms, constant })
    }

    /// Borrows sparse terms in strictly increasing variable order.
    pub fn terms(&self) -> &[AffineTerm] {
        &self.terms
    }

    /// Returns the finite affine constant.
    #[must_use]
    pub const fn constant(&self) -> f64 {
        self.constant
    }

    fn validate_variable_count(&self, count: usize) -> Result<(), ProblemIrError> {
        if let Some(term) = self.terms.iter().find(|term| term.variable >= count) {
            return Err(ProblemIrError::AffineVariableOutOfRange {
                variable: term.variable,
                variable_count: count,
            });
        }
        Ok(())
    }

    fn without_constant(self) -> Self {
        Self {
            terms: self.terms,
            constant: 0.0,
        }
    }
}

/// One canonical equality row `row = rhs`.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct CanonicalEquality {
    row: AffineExpression,
    rhs: f64,
    provenance: SemanticProvenance,
}

impl CanonicalEquality {
    /// Borrows the zero-constant sparse row.
    pub const fn row(&self) -> &AffineExpression {
        &self.row
    }

    /// Returns the finite right-hand side.
    #[must_use]
    pub const fn rhs(&self) -> f64 {
        self.rhs
    }

    /// Borrows complete originating provenance.
    pub const fn provenance(&self) -> &SemanticProvenance {
        &self.provenance
    }
}

/// One canonical two-sided linear row.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct CanonicalLinearBound {
    row: AffineExpression,
    lower: Option<f64>,
    upper: Option<f64>,
    provenance: SemanticProvenance,
}

impl CanonicalLinearBound {
    /// Borrows the zero-constant sparse row.
    pub const fn row(&self) -> &AffineExpression {
        &self.row
    }

    /// Returns the optional finite lower bound.
    #[must_use]
    pub const fn lower(&self) -> Option<f64> {
        self.lower
    }

    /// Returns the optional finite upper bound.
    #[must_use]
    pub const fn upper(&self) -> Option<f64> {
        self.upper
    }

    /// Borrows complete originating provenance.
    pub const fn provenance(&self) -> &SemanticProvenance {
        &self.provenance
    }
}

/// One canonical second-order cone `||lhs||_2 <= rhs`.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct CanonicalSecondOrderCone {
    lhs: Vec<AffineExpression>,
    rhs: AffineExpression,
    provenance: SemanticProvenance,
}

impl CanonicalSecondOrderCone {
    /// Borrows vector-valued affine left-hand expressions.
    pub fn lhs(&self) -> &[AffineExpression] {
        &self.lhs
    }

    /// Borrows the affine right-hand expression.
    pub const fn rhs(&self) -> &AffineExpression {
        &self.rhs
    }

    /// Borrows complete originating provenance.
    pub const fn provenance(&self) -> &SemanticProvenance {
        &self.provenance
    }
}

/// Canonical constraint families present in a problem.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub struct CanonicalCapabilities {
    /// Whether at least one equality row exists.
    pub has_equalities: bool,
    /// Whether at least one linear-bound row exists.
    pub has_linear_bounds: bool,
    /// Whether at least one second-order cone exists.
    pub has_second_order_cones: bool,
}

/// Checked numeric-storage estimate for a canonical problem.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub struct CanonicalMemoryEstimate {
    /// Total stored sparse coefficients.
    pub coefficient_count: usize,
    /// Bytes occupied by coefficient/index pairs and finite scalar values.
    pub numeric_bytes: usize,
}

/// Explicit identity scaling metadata; no automatic scaling is applied.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct CanonicalScaling {
    variable: Vec<f64>,
    equality: Vec<f64>,
    linear_bound: Vec<f64>,
    cone: Vec<f64>,
}

impl CanonicalScaling {
    /// Borrows unit variable scales.
    #[must_use]
    pub fn variable(&self) -> &[f64] {
        &self.variable
    }

    /// Borrows unit equality-row scales.
    #[must_use]
    pub fn equality(&self) -> &[f64] {
        &self.equality
    }

    /// Borrows unit linear-bound-row scales.
    #[must_use]
    pub fn linear_bound(&self) -> &[f64] {
        &self.linear_bound
    }

    /// Borrows unit cone scales.
    #[must_use]
    pub fn cone(&self) -> &[f64] {
        &self.cone
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct LocatedVariableBlock {
    block: VariableBlock,
    offset: usize,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct VariableSpace {
    blocks: Vec<LocatedVariableBlock>,
    variable_count: usize,
}

impl VariableSpace {
    fn try_new(blocks: impl IntoIterator<Item = VariableBlock>) -> Result<Self, ProblemIrError> {
        let blocks = try_collect(blocks, ProblemIrStorage::VariableBlocks)?;
        if blocks.is_empty() {
            return Err(ProblemIrError::EmptyVariableSpace);
        }
        let mut located = Vec::new();
        located
            .try_reserve_exact(blocks.len())
            .map_err(|_| ProblemIrError::AllocationFailed {
                storage: ProblemIrStorage::VariableBlocks,
                requested: blocks.len(),
            })?;
        let mut offset = 0_usize;
        for (index, block) in blocks.into_iter().enumerate() {
            if located[..index]
                .iter()
                .any(|existing: &LocatedVariableBlock| existing.block.name == block.name)
            {
                return Err(ProblemIrError::DuplicateVariableBlockName { name: block.name });
            }
            let next = offset
                .checked_add(block.len.get())
                .ok_or(ProblemIrError::VariableCountOverflow)?;
            located.push(LocatedVariableBlock { block, offset });
            offset = next;
        }
        Ok(Self {
            blocks: located,
            variable_count: offset,
        })
    }
}

/// Solver-neutral canonical problem containing no geological semantics.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct CanonicalProblem {
    variable_space: VariableSpace,
    equalities: Vec<CanonicalEquality>,
    linear_bounds: Vec<CanonicalLinearBound>,
    cones: Vec<CanonicalSecondOrderCone>,
    scaling: CanonicalScaling,
    capabilities: CanonicalCapabilities,
    memory_estimate: CanonicalMemoryEstimate,
}

impl CanonicalProblem {
    fn try_new(
        variable_space: VariableSpace,
        equalities: Vec<CanonicalEquality>,
        linear_bounds: Vec<CanonicalLinearBound>,
        cones: Vec<CanonicalSecondOrderCone>,
    ) -> Result<Self, ProblemIrError> {
        let scaling = CanonicalScaling {
            variable: try_unit_vec(variable_space.variable_count, ProblemIrStorage::Scaling)?,
            equality: try_unit_vec(equalities.len(), ProblemIrStorage::Scaling)?,
            linear_bound: try_unit_vec(linear_bounds.len(), ProblemIrStorage::Scaling)?,
            cone: try_unit_vec(cones.len(), ProblemIrStorage::Scaling)?,
        };
        let capabilities = CanonicalCapabilities {
            has_equalities: !equalities.is_empty(),
            has_linear_bounds: !linear_bounds.is_empty(),
            has_second_order_cones: !cones.is_empty(),
        };
        let coefficient_count = count_coefficients(&equalities, &linear_bounds, &cones)?;
        let cone_expression_count = cones.iter().try_fold(0_usize, |count, cone| {
            count
                .checked_add(cone.lhs.len())
                .and_then(|count| count.checked_add(1))
                .ok_or(ProblemIrError::MemoryEstimateOverflow)
        })?;
        let expression_constant_count = equalities
            .len()
            .checked_add(linear_bounds.len())
            .and_then(|count| count.checked_add(cone_expression_count))
            .ok_or(ProblemIrError::MemoryEstimateOverflow)?;
        let relation_scalar_count = equalities
            .len()
            .checked_add(
                linear_bounds
                    .len()
                    .checked_mul(2)
                    .ok_or(ProblemIrError::MemoryEstimateOverflow)?,
            )
            .ok_or(ProblemIrError::MemoryEstimateOverflow)?;
        let scaling_scalar_count = variable_space
            .variable_count
            .checked_add(equalities.len())
            .and_then(|count| count.checked_add(linear_bounds.len()))
            .and_then(|count| count.checked_add(cones.len()))
            .ok_or(ProblemIrError::MemoryEstimateOverflow)?;
        let scalar_count = expression_constant_count
            .checked_add(relation_scalar_count)
            .and_then(|count| count.checked_add(scaling_scalar_count))
            .ok_or(ProblemIrError::MemoryEstimateOverflow)?;
        let coefficient_bytes = coefficient_count
            .checked_mul(std::mem::size_of::<AffineTerm>())
            .ok_or(ProblemIrError::MemoryEstimateOverflow)?;
        let numeric_bytes = scalar_count
            .checked_mul(std::mem::size_of::<f64>())
            .and_then(|bytes| bytes.checked_add(coefficient_bytes))
            .ok_or(ProblemIrError::MemoryEstimateOverflow)?;
        Ok(Self {
            variable_space,
            equalities,
            linear_bounds,
            cones,
            scaling,
            capabilities,
            memory_estimate: CanonicalMemoryEstimate {
                coefficient_count,
                numeric_bytes,
            },
        })
    }

    /// Returns the total canonical variable count.
    #[must_use]
    pub const fn variable_count(&self) -> usize {
        self.variable_space.variable_count
    }

    /// Iterates variable blocks as `(name, offset, length)` in insertion order.
    #[must_use]
    pub fn variable_blocks(&self) -> impl ExactSizeIterator<Item = (&str, usize, usize)> {
        self.variable_space.blocks.iter().map(|located| {
            (
                located.block.name(),
                located.offset,
                located.block.len.get(),
            )
        })
    }

    /// Borrows canonical equality rows.
    pub fn equalities(&self) -> &[CanonicalEquality] {
        &self.equalities
    }

    /// Borrows canonical linear-bound rows.
    pub fn linear_bounds(&self) -> &[CanonicalLinearBound] {
        &self.linear_bounds
    }

    /// Borrows canonical second-order cones.
    pub fn second_order_cones(&self) -> &[CanonicalSecondOrderCone] {
        &self.cones
    }

    /// Borrows explicit identity scaling metadata.
    pub const fn scaling(&self) -> &CanonicalScaling {
        &self.scaling
    }

    /// Returns required solver capabilities.
    pub const fn capabilities(&self) -> CanonicalCapabilities {
        self.capabilities
    }

    /// Returns the checked numeric-storage estimate.
    pub const fn memory_estimate(&self) -> CanonicalMemoryEstimate {
        self.memory_estimate
    }
}

/// Storage category used by allocation diagnostics.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ProblemIrStorage {
    /// Semantic constraints.
    SemanticConstraints,
    /// Canonical variable blocks.
    VariableBlocks,
    /// Sparse affine terms.
    AffineTerms,
    /// Canonical equality rows.
    CanonicalEqualities,
    /// Canonical linear-bound rows.
    CanonicalLinearBounds,
    /// Canonical cones.
    CanonicalCones,
    /// Rows within one canonical cone.
    CanonicalConeRows,
    /// Owned semantic provenance copied into canonical rows and cones.
    CanonicalProvenance,
    /// Explicit identity scaling vectors.
    Scaling,
}

/// Error returned while constructing semantic or canonical IR values.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum ProblemIrError {
    /// A required metadata string was empty or whitespace-only.
    EmptyMetadata {
        /// Rejected field.
        field: SemanticMetadataField,
    },
    /// A semantic problem contained no constraints.
    EmptySemanticProblem,
    /// Two constraints used one stable observation identifier.
    DuplicateObservationId {
        /// Duplicated identifier.
        identifier: ObservationId,
    },
    /// A semantic constant or target was non-finite.
    NonFiniteSemanticScalar {
        /// Rejected value.
        value: f64,
    },
    /// A linear-bound relation supplied neither bound.
    MissingLinearBound,
    /// A closed interval was reversed.
    ReversedLinearBounds {
        /// Lower bound.
        lower: f64,
        /// Upper bound.
        upper: f64,
    },
    /// A second-order cone had no left-hand component.
    EmptyConeLeftHandSide,
    /// A soft residual scale was not finite and positive.
    InvalidSoftScale {
        /// Rejected scale.
        value: f64,
    },
    /// A Huber transition value was not finite and positive.
    InvalidHuberDelta {
        /// Rejected delta.
        value: f64,
    },
    /// No canonical variable block was supplied.
    EmptyVariableSpace,
    /// Two variable blocks used one name.
    DuplicateVariableBlockName {
        /// Duplicated name.
        name: String,
    },
    /// Summing variable-block lengths overflowed `usize`.
    VariableCountOverflow,
    /// A sparse affine coefficient was non-finite.
    NonFiniteAffineCoefficient {
        /// Variable index.
        variable: usize,
        /// Rejected value.
        value: f64,
    },
    /// A sparse affine coefficient was zero.
    ZeroAffineCoefficient {
        /// Variable index.
        variable: usize,
    },
    /// A sparse affine constant was non-finite.
    NonFiniteAffineConstant {
        /// Rejected value.
        value: f64,
    },
    /// Sparse variable indices were not strictly increasing.
    NonIncreasingAffineIndices {
        /// Previous index.
        previous: usize,
        /// Current index.
        current: usize,
    },
    /// A sparse variable index exceeded the declared variable space.
    AffineVariableOutOfRange {
        /// Rejected index.
        variable: usize,
        /// Declared variable count.
        variable_count: usize,
    },
    /// Storage could not be reserved.
    AllocationFailed {
        /// Storage category.
        storage: ProblemIrStorage,
        /// Exact or minimum entry count requested.
        requested: usize,
    },
    /// A checked canonical memory estimate overflowed.
    MemoryEstimateOverflow,
}

impl fmt::Display for ProblemIrError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid problem IR: {self:?}")
    }
}

impl Error for ProblemIrError {}

/// Error returned while compiling semantic IR to canonical form.
#[derive(Debug)]
pub enum CanonicalizationError<E> {
    /// Semantic or canonical IR validation failed.
    Ir(ProblemIrError),
    /// Soft objective/epigraph compilation belongs to a later requirement.
    UnsupportedSoftEnforcement {
        /// Semantic constraint index.
        constraint_index: usize,
        /// Stable observation identifier.
        observation_id: ObservationId,
    },
    /// The caller-supplied functional linearizer failed.
    Linearization {
        /// Semantic constraint index.
        constraint_index: usize,
        /// Expression index within the relation.
        expression_index: usize,
        /// Stable observation identifier.
        observation_id: ObservationId,
        /// Caller diagnostic.
        source: E,
    },
    /// The linearizer returned an affine expression outside the declared variable space.
    InvalidLinearization {
        /// Semantic constraint index.
        constraint_index: usize,
        /// Expression index within the relation.
        expression_index: usize,
        /// Stable observation identifier.
        observation_id: ObservationId,
        /// Affine validation diagnostic.
        source: ProblemIrError,
    },
    /// Shifting an affine constant produced a non-finite canonical scalar.
    NonFiniteShiftedScalar {
        /// Semantic constraint index.
        constraint_index: usize,
        /// Stable observation identifier.
        observation_id: ObservationId,
        /// Non-finite shifted value.
        value: f64,
    },
}

impl<E> fmt::Display for CanonicalizationError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ir(source) => source.fmt(formatter),
            Self::UnsupportedSoftEnforcement {
                constraint_index,
                observation_id,
            } => write!(
                formatter,
                "constraint {constraint_index} (observation {}) uses soft enforcement, whose objective compilation is not yet available",
                observation_id.identifier()
            ),
            Self::Linearization {
                constraint_index,
                expression_index,
                observation_id,
                source,
            } => write!(
                formatter,
                "could not linearize expression {expression_index} of constraint {constraint_index} (observation {}): {source}",
                observation_id.identifier()
            ),
            Self::InvalidLinearization {
                constraint_index,
                expression_index,
                observation_id,
                source,
            } => write!(
                formatter,
                "linearizer returned an invalid expression {expression_index} for constraint {constraint_index} (observation {}): {source}",
                observation_id.identifier()
            ),
            Self::NonFiniteShiftedScalar {
                constraint_index,
                observation_id,
                value,
            } => write!(
                formatter,
                "constraint {constraint_index} (observation {}) produced non-finite shifted scalar {value}",
                observation_id.identifier()
            ),
        }
    }
}

impl<E> Error for CanonicalizationError<E>
where
    E: Error + 'static,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Ir(source) | Self::InvalidLinearization { source, .. } => Some(source),
            Self::Linearization { source, .. } => Some(source),
            Self::UnsupportedSoftEnforcement { .. } | Self::NonFiniteShiftedScalar { .. } => None,
        }
    }
}

fn validate_text(value: &str, field: SemanticMetadataField) -> Result<(), ProblemIrError> {
    if value.trim().is_empty() {
        Err(ProblemIrError::EmptyMetadata { field })
    } else {
        Ok(())
    }
}

#[cfg(test)]
std::thread_local! {
    static FORCE_PROVENANCE_COPY_ALLOCATION_FAILURE: Cell<bool> = const { Cell::new(false) };
}

fn try_clone_provenance_text(value: &str) -> Result<String, ProblemIrError> {
    #[cfg(test)]
    if FORCE_PROVENANCE_COPY_ALLOCATION_FAILURE.with(|force| force.replace(false)) {
        return Err(ProblemIrError::AllocationFailed {
            storage: ProblemIrStorage::CanonicalProvenance,
            requested: value.len(),
        });
    }

    let mut cloned = String::new();
    cloned
        .try_reserve_exact(value.len())
        .map_err(|_| ProblemIrError::AllocationFailed {
            storage: ProblemIrStorage::CanonicalProvenance,
            requested: value.len(),
        })?;
    cloned.push_str(value);
    Ok(cloned)
}

fn validate_relation_scalar(value: f64) -> Result<(), ProblemIrError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(ProblemIrError::NonFiniteSemanticScalar { value })
    }
}

fn try_collect<T>(
    values: impl IntoIterator<Item = T>,
    storage: ProblemIrStorage,
) -> Result<Vec<T>, ProblemIrError> {
    let iterator = values.into_iter();
    let minimum = iterator.size_hint().0;
    let mut output = Vec::new();
    output
        .try_reserve_exact(minimum)
        .map_err(|_| ProblemIrError::AllocationFailed {
            storage,
            requested: minimum,
        })?;
    for value in iterator {
        if output.len() == output.capacity() {
            output
                .try_reserve(1)
                .map_err(|_| ProblemIrError::AllocationFailed {
                    storage,
                    requested: output.len().saturating_add(1),
                })?;
        }
        output.push(value);
    }
    Ok(output)
}

fn linearize_expression<const D: usize, E>(
    expression: &SemanticExpression<D>,
    provenance: &SemanticProvenance,
    constraint_index: usize,
    expression_index: usize,
    variable_count: usize,
    linearize: &mut impl FnMut(
        &ObservationFunctional<D>,
        &SemanticProvenance,
    ) -> Result<AffineExpression, E>,
) -> Result<AffineExpression, CanonicalizationError<E>>
where
    Dim<D>: SupportedDimension,
{
    let mut affine = linearize(expression.functional(), provenance).map_err(|source| {
        CanonicalizationError::Linearization {
            constraint_index,
            expression_index,
            observation_id: provenance.observation_id,
            source,
        }
    })?;
    affine
        .validate_variable_count(variable_count)
        .map_err(|source| CanonicalizationError::InvalidLinearization {
            constraint_index,
            expression_index,
            observation_id: provenance.observation_id,
            source,
        })?;
    affine.constant += expression.constant;
    if !affine.constant.is_finite() {
        return Err(CanonicalizationError::NonFiniteShiftedScalar {
            constraint_index,
            observation_id: provenance.observation_id,
            value: affine.constant,
        });
    }
    Ok(affine)
}

fn checked_subtract(left: f64, right: f64) -> Result<f64, f64> {
    let value = left - right;
    if value.is_finite() {
        Ok(value)
    } else {
        Err(value)
    }
}

fn shift_optional_bound(value: Option<f64>, constant: f64) -> Result<Option<f64>, f64> {
    value
        .map(|value| checked_subtract(value, constant))
        .transpose()
}

fn allocation_error<E>(storage: ProblemIrStorage, requested: usize) -> CanonicalizationError<E> {
    CanonicalizationError::Ir(ProblemIrError::AllocationFailed { storage, requested })
}

fn try_unit_vec(count: usize, storage: ProblemIrStorage) -> Result<Vec<f64>, ProblemIrError> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| ProblemIrError::AllocationFailed {
            storage,
            requested: count,
        })?;
    values.resize(count, 1.0);
    Ok(values)
}

fn count_coefficients(
    equalities: &[CanonicalEquality],
    linear_bounds: &[CanonicalLinearBound],
    cones: &[CanonicalSecondOrderCone],
) -> Result<usize, ProblemIrError> {
    let mut count = 0_usize;
    for row in equalities
        .iter()
        .map(|row| &row.row)
        .chain(linear_bounds.iter().map(|row| &row.row))
    {
        count = count
            .checked_add(row.terms.len())
            .ok_or(ProblemIrError::MemoryEstimateOverflow)?;
    }
    for cone in cones {
        for expression in cone.lhs.iter().chain(std::iter::once(&cone.rhs)) {
            count = count
                .checked_add(expression.terms.len())
                .ok_or(ProblemIrError::MemoryEstimateOverflow)?;
        }
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::functional::{FunctionalAtom, FunctionalExpr, FunctionalProvenance, FunctionalTerm};
    use crate::geometry::Point;

    use super::*;

    type TestResult = Result<(), Box<dyn Error>>;

    struct ForcedProvenanceCopyAllocationFailure;

    impl ForcedProvenanceCopyAllocationFailure {
        fn new() -> Self {
            FORCE_PROVENANCE_COPY_ALLOCATION_FAILURE.with(|force| force.set(true));
            Self
        }
    }

    impl Drop for ForcedProvenanceCopyAllocationFailure {
        fn drop(&mut self) {
            FORCE_PROVENANCE_COPY_ALLOCATION_FAILURE.with(|force| force.set(false));
        }
    }

    fn test_expression() -> Result<SemanticExpression<1>, Box<dyn Error>> {
        let atom = FunctionalAtom::value(Point::try_new([0.0])?, FunctionalProvenance::new(1));
        let term = FunctionalTerm::try_new(1.0, atom)?;
        let functional = ObservationFunctional::new(FunctionalExpr::try_new([term])?);
        Ok(SemanticExpression::try_new(functional, 0.0)?)
    }

    fn assert_provenance_copy_allocation_failure(relation: SemanticRelation<1>) -> TestResult {
        let source_path = "input.yaml";
        let provenance = SemanticProvenance::try_new(
            ObservationId::new(1),
            SourceLocation::try_new(source_path.to_owned(), NonZeroUsize::MIN)?,
            "m".to_owned(),
            "fields.scalar".to_owned(),
            Some("group".to_owned()),
        )?;
        let constraint = SemanticConstraint::try_new(provenance, relation, Enforcement::Hard)?;
        let problem = SemanticProblemIr::try_new([constraint], ExecutionOptions::default())?;
        let block = VariableBlock::try_new("z".to_owned(), NonZeroUsize::MIN)?;

        let _failure = ForcedProvenanceCopyAllocationFailure::new();
        let result = problem.try_compile::<ProblemIrError>([block], |_, _| {
            AffineExpression::try_new([AffineTerm::try_new(0, 1.0)?], 0.0)
        });

        assert!(matches!(
            result,
            Err(CanonicalizationError::Ir(
                ProblemIrError::AllocationFailed {
                    storage: ProblemIrStorage::CanonicalProvenance,
                    requested,
                }
            )) if requested == source_path.len()
        ));
        Ok(())
    }

    #[test]
    fn equality_provenance_copy_allocation_failure_is_structured() -> TestResult {
        assert_provenance_copy_allocation_failure(SemanticRelation::Equality {
            expression: test_expression()?,
            target: 0.0,
        })
    }

    #[test]
    fn linear_bound_provenance_copy_allocation_failure_is_structured() -> TestResult {
        assert_provenance_copy_allocation_failure(SemanticRelation::LinearBound {
            expression: test_expression()?,
            lower: Some(0.0),
            upper: None,
        })
    }

    #[test]
    fn cone_provenance_copy_allocation_failure_is_structured() -> TestResult {
        assert_provenance_copy_allocation_failure(SemanticRelation::SecondOrderCone {
            lhs: vec![test_expression()?],
            rhs: test_expression()?,
        })
    }
}
