//! Explicit geological level variables and deterministic order-DAG validation.
//!
//! A level problem owns semantic level definitions, field-membership
//! observations, and minimum-gap order edges. Compilation produces only
//! solver-neutral hard constraints and independently scaled soft objectives;
//! optimization remains the responsibility of an approved convex backend.
//! Validation closes equality transitively across shared mathematical Value
//! evaluations. It rejects a positive order path or distinct fixed values
//! between levels in one equality component and retains the selected membership
//! chain as evidence. Distinct fixed or prior anchors prove contrast only when
//! their levels belong to different membership-equality components.

use std::error::Error;
use std::fmt;
use std::num::NonZeroUsize;

use crate::diagnostics::{
    ContrastDiagnostic, DiagnosticPath, DiagnosticPathError, DiagnosticValueError, GaugeDiagnostic,
    LevelId,
};
use crate::dimension::{Dim, SupportedDimension};
use crate::functional::{FunctionalAtom, ObservationFunctional};
use crate::problem_ir::{
    AffineExpression, AffineTerm, CanonicalEquality, CanonicalLinearBound, CanonicalProblem,
    CanonicalSoftObjective, CanonicalSoftRelation, ObservationId, ProblemIrError,
    SemanticProvenance, SoftLoss, VariableBlock,
};

const LEVEL_VARIABLE_BLOCK: &str = "levels";

/// Explicit value policy for one level variable.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LevelValue {
    /// A hard fixed scalar value.
    Fixed(f64),
    /// An unconstrained scalar value whose gauge must be supplied elsewhere.
    Unknown,
    /// An unknown scalar value carrying an explicit soft prior.
    Prior(LevelPrior),
}

impl LevelValue {
    /// Validates and constructs a fixed level value.
    ///
    /// # Errors
    ///
    /// Returns [`LevelProblemError::NonFiniteFixedValue`] for NaN or infinity.
    pub fn try_fixed(value: f64) -> Result<Self, LevelProblemError> {
        if !value.is_finite() {
            return Err(LevelProblemError::NonFiniteFixedValue { value });
        }
        Ok(Self::Fixed(value))
    }

    /// Constructs an unknown level value.
    #[must_use]
    pub const fn unknown() -> Self {
        Self::Unknown
    }

    /// Returns the hard fixed value, when present.
    #[must_use]
    pub const fn fixed(self) -> Option<f64> {
        match self {
            Self::Fixed(value) => Some(value),
            Self::Unknown | Self::Prior(_) => None,
        }
    }

    /// Returns the soft prior, when present.
    #[must_use]
    pub const fn prior(self) -> Option<LevelPrior> {
        match self {
            Self::Prior(prior) => Some(prior),
            Self::Fixed(_) | Self::Unknown => None,
        }
    }

    const fn anchors_gauge(self) -> bool {
        !matches!(self, Self::Unknown)
    }
}

/// Explicit soft prior for one unknown level variable.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct LevelPrior {
    mean: f64,
    scale: f64,
    loss: SoftLoss,
}

impl LevelPrior {
    /// Validates a finite mean, positive finite scale, and valid loss.
    ///
    /// # Errors
    ///
    /// Returns a structured numeric error for invalid prior metadata.
    pub fn try_new(mean: f64, scale: f64, loss: SoftLoss) -> Result<Self, LevelProblemError> {
        if !mean.is_finite() {
            return Err(LevelProblemError::NonFinitePriorMean { value: mean });
        }
        if !scale.is_finite() || scale <= 0.0 {
            return Err(LevelProblemError::InvalidPriorScale { value: scale });
        }
        if let SoftLoss::Huber { delta } = loss
            && (!delta.is_finite() || delta <= 0.0)
        {
            return Err(LevelProblemError::InvalidHuberDelta { value: delta });
        }
        Ok(Self { mean, scale, loss })
    }

    /// Returns the prior mean in scalar-field units.
    #[must_use]
    pub const fn mean(self) -> f64 {
        self.mean
    }

    /// Returns the positive residual scale.
    #[must_use]
    pub const fn scale(self) -> f64 {
        self.scale
    }

    /// Returns the declared loss family.
    #[must_use]
    pub const fn loss(self) -> SoftLoss {
        self.loss
    }
}

/// One stable level definition and its source provenance.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct LevelDefinition {
    level_id: LevelId,
    value: LevelValue,
    provenance: SemanticProvenance,
}

impl LevelDefinition {
    /// Constructs a level definition; [`LevelProblem::try_new`] revalidates
    /// directly constructed fixed values before accepting the definition.
    pub const fn new(level_id: LevelId, value: LevelValue, provenance: SemanticProvenance) -> Self {
        Self {
            level_id,
            value,
            provenance,
        }
    }

    /// Returns the stable level identifier.
    pub const fn level_id(&self) -> LevelId {
        self.level_id
    }

    /// Returns the explicit value policy.
    #[must_use]
    pub const fn value(&self) -> LevelValue {
        self.value
    }

    /// Borrows the definition provenance.
    pub const fn provenance(&self) -> &SemanticProvenance {
        &self.provenance
    }
}

/// One hard field-to-level membership `f(x) - h_k = 0`.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct LevelMembership<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    level_id: LevelId,
    functional: ObservationFunctional<D>,
    provenance: SemanticProvenance,
}

impl<const D: usize> LevelMembership<D>
where
    Dim<D>: SupportedDimension,
{
    /// Constructs a membership from a compiled observation functional.
    ///
    /// [`LevelProblem::try_new`] accepts only one coefficient-1
    /// [`FunctionalAtom::Value`] term so the functional has the value units and
    /// joint-shift gauge required by `f(x_i) - h_k = 0`.
    pub const fn new(
        level_id: LevelId,
        functional: ObservationFunctional<D>,
        provenance: SemanticProvenance,
    ) -> Self {
        Self {
            level_id,
            functional,
            provenance,
        }
    }

    /// Returns the referenced level.
    pub const fn level_id(&self) -> LevelId {
        self.level_id
    }

    /// Borrows the compiled field functional.
    pub const fn functional(&self) -> &ObservationFunctional<D> {
        &self.functional
    }

    /// Borrows complete membership provenance.
    pub const fn provenance(&self) -> &SemanticProvenance {
        &self.provenance
    }
}

/// One directed minimum-gap edge `h_upper - h_lower >= minimum_gap`.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct LevelOrder {
    lower: LevelId,
    upper: LevelId,
    minimum_gap: f64,
    provenance: SemanticProvenance,
}

impl LevelOrder {
    /// Validates and constructs an order edge.
    ///
    /// # Errors
    ///
    /// Rejects a self edge or a negative/non-finite gap.
    pub fn try_new(
        lower: LevelId,
        upper: LevelId,
        minimum_gap: f64,
        provenance: SemanticProvenance,
    ) -> Result<Self, LevelProblemError> {
        if lower == upper {
            return Err(LevelProblemError::SelfOrderEdge { level_id: lower });
        }
        if !minimum_gap.is_finite() || minimum_gap < 0.0 {
            return Err(LevelProblemError::InvalidMinimumGap { minimum_gap });
        }
        Ok(Self {
            lower,
            upper,
            minimum_gap,
            provenance,
        })
    }

    /// Returns the lower level identifier.
    pub const fn lower(&self) -> LevelId {
        self.lower
    }

    /// Returns the upper level identifier.
    pub const fn upper(&self) -> LevelId {
        self.upper
    }

    /// Returns the finite nonnegative minimum gap.
    #[must_use]
    pub const fn minimum_gap(&self) -> f64 {
        self.minimum_gap
    }

    /// Borrows complete order-edge provenance.
    pub const fn provenance(&self) -> &SemanticProvenance {
        &self.provenance
    }
}

/// Deterministic validation evidence retained by a level problem.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct LevelProblemDiagnostics {
    topological_order: Vec<LevelId>,
    gauge_anchor_count: usize,
}

impl LevelProblemDiagnostics {
    /// Borrows the deterministic topological level order.
    pub fn topological_order(&self) -> &[LevelId] {
        &self.topological_order
    }

    /// Returns the number of fixed or prior gauge anchors.
    #[must_use]
    pub const fn gauge_anchor_count(&self) -> usize {
        self.gauge_anchor_count
    }
}

/// Validated immutable level-variable problem.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct LevelProblem<const D: usize>
where
    Dim<D>: SupportedDimension,
{
    levels: Vec<LevelDefinition>,
    memberships: Vec<LevelMembership<D>>,
    orders: Vec<LevelOrder>,
    diagnostics: LevelProblemDiagnostics,
}

impl<const D: usize> LevelProblem<D>
where
    Dim<D>: SupportedDimension,
{
    /// Validates definitions, memberships, order topology, gauges, and contrast.
    ///
    /// # Errors
    ///
    /// Returns a structured source-aware error without returning a partial
    /// problem when any semantic invariant fails.
    #[allow(clippy::too_many_lines)]
    pub fn try_new(
        levels: impl IntoIterator<Item = LevelDefinition>,
        memberships: impl IntoIterator<Item = LevelMembership<D>>,
        orders: impl IntoIterator<Item = LevelOrder>,
    ) -> Result<Self, LevelProblemError> {
        let levels = try_collect(levels, LevelStorage::Definitions)?;
        let memberships = try_collect(memberships, LevelStorage::Memberships)?;
        let orders = try_collect(orders, LevelStorage::Orders)?;
        if levels.len() < 2 {
            return Err(LevelProblemError::InsufficientLevels {
                count: levels.len(),
            });
        }
        if memberships.is_empty() {
            return Err(LevelProblemError::EmptyMemberships);
        }
        validate_unique_levels(&levels)?;
        validate_level_values(&levels)?;
        validate_unique_provenance(&levels, &memberships, &orders)?;
        validate_membership_functionals(&memberships)?;

        let membership_indices = membership_level_indices(&levels, &memberships)?;
        let order_indices = order_level_indices(&levels, &orders)?;
        validate_isolation(&levels, &membership_indices, &order_indices)?;
        let membership_equality =
            MembershipEquality::try_new(levels.len(), &memberships, &membership_indices)?;
        validate_fixed_memberships(&levels, &memberships, &membership_equality)?;
        let topological_indices = validate_dag(&levels, &orders, &order_indices)?;
        validate_membership_order_paths(
            &memberships,
            &orders,
            &membership_indices,
            &order_indices,
            &topological_indices,
            &membership_equality,
        )?;
        validate_fixed_order_paths(&levels, &orders, &order_indices, &topological_indices)?;
        validate_gauge(&levels, &membership_indices, &order_indices)?;
        validate_contrast(
            &levels,
            &orders,
            &membership_indices,
            &order_indices,
            &topological_indices,
            &membership_equality,
        )?;

        let mut topological_order = Vec::new();
        topological_order
            .try_reserve_exact(topological_indices.len())
            .map_err(|_| {
                allocation_error(LevelStorage::TopologicalOrder, topological_indices.len())
            })?;
        topological_order.extend(
            topological_indices
                .into_iter()
                .map(|index| levels[index].level_id),
        );
        let gauge_anchor_count = levels
            .iter()
            .filter(|level| level.value.anchors_gauge())
            .count();
        Ok(Self {
            levels,
            memberships,
            orders,
            diagnostics: LevelProblemDiagnostics {
                topological_order,
                gauge_anchor_count,
            },
        })
    }

    /// Borrows level definitions in deterministic insertion order.
    pub fn levels(&self) -> &[LevelDefinition] {
        &self.levels
    }

    /// Borrows membership equalities in deterministic insertion order.
    pub fn memberships(&self) -> &[LevelMembership<D>] {
        &self.memberships
    }

    /// Borrows order edges in deterministic insertion order.
    pub fn orders(&self) -> &[LevelOrder] {
        &self.orders
    }

    /// Borrows validation diagnostics.
    pub const fn diagnostics(&self) -> &LevelProblemDiagnostics {
        &self.diagnostics
    }

    /// Compiles memberships, fixed values, and order gaps into canonical form.
    ///
    /// The caller linearizes only field functionals into the supplied field
    /// variable blocks. This compiler appends the explicit `levels` block and
    /// owns every level coefficient and relation sign.
    ///
    /// # Errors
    ///
    /// Returns indexed linearization, affine-validation, allocation, or IR
    /// errors without exposing a partial canonical problem.
    #[allow(clippy::too_many_lines)]
    pub fn try_compile<E>(
        &self,
        field_variable_blocks: impl IntoIterator<Item = VariableBlock>,
        mut linearize: impl FnMut(
            &ObservationFunctional<D>,
            &SemanticProvenance,
        ) -> Result<AffineExpression, E>,
    ) -> Result<CompiledLevelProblem, LevelCanonicalizationError<E>> {
        let mut blocks =
            collect_blocks(field_variable_blocks).map_err(LevelCanonicalizationError::Ir)?;
        if blocks.is_empty() {
            return Err(LevelCanonicalizationError::EmptyFieldVariableSpace);
        }
        let field_variable_count = blocks
            .iter()
            .try_fold(0_usize, |count, block| {
                count
                    .checked_add(block.len().get())
                    .ok_or(ProblemIrError::VariableCountOverflow)
            })
            .map_err(LevelCanonicalizationError::Ir)?;
        let level_count = NonZeroUsize::new(self.levels.len())
            .ok_or(LevelCanonicalizationError::EmptyLevelVariableSpace)?;
        blocks.try_reserve_exact(1).map_err(|_| {
            LevelCanonicalizationError::Ir(allocation_error_ir(
                LevelStorage::VariableBlocks,
                blocks.len().saturating_add(1),
            ))
        })?;
        blocks.push(
            VariableBlock::try_new(LEVEL_VARIABLE_BLOCK.to_owned(), level_count)
                .map_err(LevelCanonicalizationError::Ir)?,
        );

        let fixed_count = self
            .levels
            .iter()
            .filter(|level| matches!(level.value, LevelValue::Fixed(_)))
            .count();
        let equality_count = self
            .memberships
            .len()
            .checked_add(fixed_count)
            .ok_or_else(|| {
                LevelCanonicalizationError::Ir(ProblemIrError::MemoryEstimateOverflow)
            })?;
        let mut equalities = Vec::new();
        equalities.try_reserve_exact(equality_count).map_err(|_| {
            LevelCanonicalizationError::Ir(allocation_error_ir(
                LevelStorage::CanonicalEqualities,
                equality_count,
            ))
        })?;

        for (membership_index, membership) in self.memberships.iter().enumerate() {
            let affine =
                linearize(&membership.functional, &membership.provenance).map_err(|source| {
                    LevelCanonicalizationError::Linearization {
                        membership_index,
                        observation_id: membership.provenance.observation_id(),
                        source,
                    }
                })?;
            affine
                .validate_variable_count(field_variable_count)
                .map_err(|source| LevelCanonicalizationError::InvalidLinearization {
                    membership_index,
                    observation_id: membership.provenance.observation_id(),
                    source,
                })?;
            let level_index = find_level(&self.levels, membership.level_id)
                .ok_or(LevelCanonicalizationError::InvalidValidatedState)?;
            let level_variable =
                field_variable_count
                    .checked_add(level_index)
                    .ok_or_else(|| {
                        LevelCanonicalizationError::Ir(ProblemIrError::VariableCountOverflow)
                    })?;
            let mut terms = Vec::new();
            let term_count = affine.terms().len().checked_add(1).ok_or_else(|| {
                LevelCanonicalizationError::Ir(ProblemIrError::MemoryEstimateOverflow)
            })?;
            terms.try_reserve_exact(term_count).map_err(|_| {
                LevelCanonicalizationError::Ir(allocation_error_ir(
                    LevelStorage::AffineTerms,
                    term_count,
                ))
            })?;
            terms.extend_from_slice(affine.terms());
            terms.push(
                AffineTerm::try_new(level_variable, -1.0)
                    .map_err(LevelCanonicalizationError::Ir)?,
            );
            let rhs = -affine.constant();
            let row =
                AffineExpression::try_new(terms, 0.0).map_err(LevelCanonicalizationError::Ir)?;
            equalities.push(CanonicalEquality::from_parts(
                row,
                rhs,
                membership
                    .provenance
                    .try_clone_for_canonical()
                    .map_err(LevelCanonicalizationError::Ir)?,
            ));
        }

        for (level_index, level) in self.levels.iter().enumerate() {
            let LevelValue::Fixed(value) = level.value else {
                continue;
            };
            let variable = field_variable_count
                .checked_add(level_index)
                .ok_or_else(|| {
                    LevelCanonicalizationError::Ir(ProblemIrError::VariableCountOverflow)
                })?;
            let row = AffineExpression::try_new(
                [AffineTerm::try_new(variable, 1.0).map_err(LevelCanonicalizationError::Ir)?],
                0.0,
            )
            .map_err(LevelCanonicalizationError::Ir)?;
            equalities.push(CanonicalEquality::from_parts(
                row,
                value,
                level
                    .provenance
                    .try_clone_for_canonical()
                    .map_err(LevelCanonicalizationError::Ir)?,
            ));
        }

        let mut linear_bounds = Vec::new();
        linear_bounds
            .try_reserve_exact(self.orders.len())
            .map_err(|_| {
                LevelCanonicalizationError::Ir(allocation_error_ir(
                    LevelStorage::CanonicalLinearBounds,
                    self.orders.len(),
                ))
            })?;
        for order in &self.orders {
            let lower_index = find_level(&self.levels, order.lower)
                .ok_or(LevelCanonicalizationError::InvalidValidatedState)?;
            let upper_index = find_level(&self.levels, order.upper)
                .ok_or(LevelCanonicalizationError::InvalidValidatedState)?;
            let lower_variable =
                field_variable_count
                    .checked_add(lower_index)
                    .ok_or_else(|| {
                        LevelCanonicalizationError::Ir(ProblemIrError::VariableCountOverflow)
                    })?;
            let upper_variable =
                field_variable_count
                    .checked_add(upper_index)
                    .ok_or_else(|| {
                        LevelCanonicalizationError::Ir(ProblemIrError::VariableCountOverflow)
                    })?;
            let terms = if lower_variable < upper_variable {
                [
                    AffineTerm::try_new(lower_variable, -1.0)
                        .map_err(LevelCanonicalizationError::Ir)?,
                    AffineTerm::try_new(upper_variable, 1.0)
                        .map_err(LevelCanonicalizationError::Ir)?,
                ]
            } else {
                [
                    AffineTerm::try_new(upper_variable, 1.0)
                        .map_err(LevelCanonicalizationError::Ir)?,
                    AffineTerm::try_new(lower_variable, -1.0)
                        .map_err(LevelCanonicalizationError::Ir)?,
                ]
            };
            let row =
                AffineExpression::try_new(terms, 0.0).map_err(LevelCanonicalizationError::Ir)?;
            linear_bounds.push(CanonicalLinearBound::from_parts(
                row,
                Some(order.minimum_gap),
                None,
                order
                    .provenance
                    .try_clone_for_canonical()
                    .map_err(LevelCanonicalizationError::Ir)?,
            ));
        }

        let mut priors = Vec::new();
        let prior_count = self
            .levels
            .iter()
            .filter(|level| matches!(level.value, LevelValue::Prior(_)))
            .count();
        priors.try_reserve_exact(prior_count).map_err(|_| {
            LevelCanonicalizationError::Ir(allocation_error_ir(
                LevelStorage::CanonicalPriors,
                prior_count,
            ))
        })?;
        let mut soft_objectives = Vec::new();
        soft_objectives
            .try_reserve_exact(prior_count)
            .map_err(|_| {
                LevelCanonicalizationError::Ir(allocation_error_ir(
                    LevelStorage::CanonicalPriors,
                    prior_count,
                ))
            })?;
        for (level_index, level) in self.levels.iter().enumerate() {
            let LevelValue::Prior(prior) = level.value else {
                continue;
            };
            let variable = field_variable_count
                .checked_add(level_index)
                .ok_or_else(|| {
                    LevelCanonicalizationError::Ir(ProblemIrError::VariableCountOverflow)
                })?;
            let objective_row = AffineExpression::try_new(
                [AffineTerm::try_new(variable, 1.0).map_err(LevelCanonicalizationError::Ir)?],
                0.0,
            )
            .map_err(LevelCanonicalizationError::Ir)?;
            let objective_relation = CanonicalEquality::from_parts(
                objective_row,
                prior.mean,
                level
                    .provenance
                    .try_clone_for_canonical()
                    .map_err(LevelCanonicalizationError::Ir)?,
            );
            soft_objectives.push(CanonicalSoftObjective::from_parts(
                CanonicalSoftRelation::Equality(objective_relation),
                prior.scale,
                prior.loss,
            ));
            priors.push(CanonicalLevelPrior {
                level_id: level.level_id,
                variable,
                prior,
                provenance: level
                    .provenance
                    .try_clone_for_canonical()
                    .map_err(LevelCanonicalizationError::Ir)?,
            });
        }

        let canonical = CanonicalProblem::try_from_linear_parts_and_objectives(
            blocks,
            equalities,
            linear_bounds,
            soft_objectives,
        )
        .map_err(LevelCanonicalizationError::Ir)?;
        let mut level_ids = Vec::new();
        level_ids
            .try_reserve_exact(self.levels.len())
            .map_err(|_| {
                LevelCanonicalizationError::Ir(allocation_error_ir(
                    LevelStorage::CompiledLevelIds,
                    self.levels.len(),
                ))
            })?;
        level_ids.extend(self.levels.iter().map(|level| level.level_id));
        Ok(CompiledLevelProblem {
            canonical,
            priors,
            level_ids,
            level_variable_offset: field_variable_count,
        })
    }
}

/// One explicit canonical soft-prior objective term.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct CanonicalLevelPrior {
    level_id: LevelId,
    variable: usize,
    prior: LevelPrior,
    provenance: SemanticProvenance,
}

impl CanonicalLevelPrior {
    /// Returns the level identifier.
    pub const fn level_id(&self) -> LevelId {
        self.level_id
    }

    /// Returns the zero-based canonical variable index.
    #[must_use]
    pub const fn variable(&self) -> usize {
        self.variable
    }

    /// Returns the explicit prior parameters.
    pub const fn prior(&self) -> LevelPrior {
        self.prior
    }

    /// Borrows complete prior provenance.
    pub const fn provenance(&self) -> &SemanticProvenance {
        &self.provenance
    }
}

/// Canonical constraints and objectives plus a stable level-prior identity view.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct CompiledLevelProblem {
    canonical: CanonicalProblem,
    priors: Vec<CanonicalLevelPrior>,
    level_ids: Vec<LevelId>,
    level_variable_offset: usize,
}

impl CompiledLevelProblem {
    /// Borrows the solver-neutral hard constraints and soft objectives.
    pub const fn canonical_problem(&self) -> &CanonicalProblem {
        &self.canonical
    }

    /// Borrows explicit soft priors in level insertion order.
    pub fn priors(&self) -> &[CanonicalLevelPrior] {
        &self.priors
    }

    /// Returns the canonical variable assigned to a stable level identifier.
    #[must_use]
    pub fn level_variable(&self, level_id: LevelId) -> Option<usize> {
        self.level_ids
            .iter()
            .position(|candidate| *candidate == level_id)
            .and_then(|index| self.level_variable_offset.checked_add(index))
    }

    /// Consumes the wrapper and returns the canonical problem.
    pub fn into_canonical_problem(self) -> CanonicalProblem {
        self.canonical
    }
}

/// Allocation category for level semantic and canonical storage.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LevelStorage {
    /// Level definitions.
    Definitions,
    /// Membership observations.
    Memberships,
    /// Order edges.
    Orders,
    /// Temporary membership-to-level indices.
    MembershipIndices,
    /// Temporary order endpoint indices.
    OrderIndices,
    /// Deterministic topological order.
    TopologicalOrder,
    /// Graph work vectors.
    GraphWork,
    /// Source paths for a structured conflict.
    ConflictSources,
    /// Caller field blocks plus the level block.
    VariableBlocks,
    /// Canonical equality rows.
    CanonicalEqualities,
    /// Canonical linear-bound rows.
    CanonicalLinearBounds,
    /// Sparse affine terms.
    AffineTerms,
    /// Explicit canonical prior metadata.
    CanonicalPriors,
    /// Stable compiled level identifiers.
    CompiledLevelIds,
}

/// Error returned while validating level semantics.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum LevelProblemError {
    /// Fewer than two distinct levels were supplied.
    InsufficientLevels {
        /// Supplied definition count.
        count: usize,
    },
    /// No field membership was supplied.
    EmptyMemberships,
    /// Two definitions used one stable level identifier.
    DuplicateLevelId {
        /// Duplicated identifier.
        level_id: LevelId,
    },
    /// Two semantic records used one observation identifier.
    DuplicateObservationId {
        /// Duplicated identifier.
        observation_id: ObservationId,
    },
    /// A membership or order referenced an undefined level.
    UndefinedLevel {
        /// Missing identifier.
        level_id: LevelId,
        /// Source record identifier.
        observation_id: ObservationId,
    },
    /// A fixed scalar was not finite.
    NonFiniteFixedValue {
        /// Rejected value.
        value: f64,
    },
    /// A prior mean was not finite.
    NonFinitePriorMean {
        /// Rejected value.
        value: f64,
    },
    /// A prior scale was not finite and positive.
    InvalidPriorScale {
        /// Rejected value.
        value: f64,
    },
    /// A Huber transition was not finite and positive.
    InvalidHuberDelta {
        /// Rejected value.
        value: f64,
    },
    /// An order edge referenced the same endpoint twice.
    SelfOrderEdge {
        /// Repeated level identifier.
        level_id: LevelId,
    },
    /// An order gap was negative or non-finite.
    InvalidMinimumGap {
        /// Rejected gap.
        minimum_gap: f64,
    },
    /// A membership was not exactly one unit-weight scalar-field value.
    InvalidMembershipFunctional {
        /// Source membership identifier.
        observation_id: ObservationId,
    },
    /// An unknown level had neither membership nor order edge.
    IsolatedUnknownLevel {
        /// Isolated identifier.
        level_id: LevelId,
    },
    /// Order edges contained a directed cycle.
    OrderCycle {
        /// Sources participating in the unresolved cyclic subgraph.
        sources: Vec<DiagnosticPath>,
    },
    /// Two membership-equality-coupled levels required different fixed values.
    FixedMembershipConflict {
        /// First fixed level.
        first_level: LevelId,
        /// Second fixed level.
        second_level: LevelId,
        /// Complete conflicting sources.
        sources: Vec<DiagnosticPath>,
    },
    /// Membership-equality-coupled levels contradicted a positive order path.
    MembershipOrderConflict {
        /// Lower level on the conflicting order path.
        lower: LevelId,
        /// Upper level on the conflicting order path.
        upper: LevelId,
        /// The selected equality-chain memberships and every selected path edge.
        sources: Vec<DiagnosticPath>,
    },
    /// A transitive minimum-gap path contradicted fixed endpoints.
    FixedOrderConflict {
        /// Fixed lower level.
        lower: LevelId,
        /// Fixed upper level.
        upper: LevelId,
        /// Longest required path gap.
        required_gap: f64,
        /// Actual fixed endpoint gap.
        fixed_gap: f64,
        /// Complete endpoint and path sources.
        sources: Vec<DiagnosticPath>,
    },
    /// One or more connected components lacked a fixed value or prior.
    MissingGauge {
        /// Structured gauge evidence.
        diagnostic: GaugeDiagnostic,
    },
    /// The level system imposed no nonzero contrast.
    MissingContrast {
        /// Structured contrast evidence.
        diagnostic: ContrastDiagnostic,
    },
    /// Structured diagnostic construction failed.
    DiagnosticValue(DiagnosticValueError),
    /// Copying source provenance failed.
    DiagnosticPath(DiagnosticPathError),
    /// Storage could not be reserved.
    AllocationFailed {
        /// Storage category.
        storage: LevelStorage,
        /// Requested entry count.
        requested: usize,
    },
    /// A checked graph or record count overflowed `usize`.
    CountOverflow,
    /// An internal graph invariant failed after input validation.
    InvalidGraphState,
}

impl fmt::Display for LevelProblemError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid level problem: {self:?}")
    }
}

impl Error for LevelProblemError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::DiagnosticValue(source) => Some(source),
            Self::DiagnosticPath(source) => Some(source),
            _ => None,
        }
    }
}

/// Error returned while compiling a validated level problem.
#[derive(Debug)]
#[non_exhaustive]
pub enum LevelCanonicalizationError<E> {
    /// No field variable block was supplied.
    EmptyFieldVariableSpace,
    /// The validated problem unexpectedly had no levels.
    EmptyLevelVariableSpace,
    /// Semantic or canonical IR validation failed.
    Ir(ProblemIrError),
    /// The caller field linearizer failed.
    Linearization {
        /// Membership insertion index.
        membership_index: usize,
        /// Stable observation identifier.
        observation_id: ObservationId,
        /// Caller diagnostic.
        source: E,
    },
    /// The caller referenced a variable outside the field blocks.
    InvalidLinearization {
        /// Membership insertion index.
        membership_index: usize,
        /// Stable observation identifier.
        observation_id: ObservationId,
        /// Affine validation diagnostic.
        source: ProblemIrError,
    },
    /// An invariant of an already validated problem was violated.
    InvalidValidatedState,
}

impl<E> fmt::Display for LevelCanonicalizationError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFieldVariableSpace => {
                formatter.write_str("no field variable block supplied")
            }
            Self::EmptyLevelVariableSpace => {
                formatter.write_str("validated level problem is empty")
            }
            Self::Ir(source) => source.fmt(formatter),
            Self::Linearization {
                membership_index,
                observation_id,
                source,
            } => write!(
                formatter,
                "could not linearize level membership {membership_index} (observation {}): {source}",
                observation_id.identifier()
            ),
            Self::InvalidLinearization {
                membership_index,
                observation_id,
                source,
            } => write!(
                formatter,
                "invalid level membership linearization {membership_index} (observation {}): {source}",
                observation_id.identifier()
            ),
            Self::InvalidValidatedState => {
                formatter.write_str("validated level state is inconsistent")
            }
        }
    }
}

impl<E> Error for LevelCanonicalizationError<E>
where
    E: Error + 'static,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Ir(source) | Self::InvalidLinearization { source, .. } => Some(source),
            Self::Linearization { source, .. } => Some(source),
            Self::EmptyFieldVariableSpace
            | Self::EmptyLevelVariableSpace
            | Self::InvalidValidatedState => None,
        }
    }
}

fn validate_unique_levels(levels: &[LevelDefinition]) -> Result<(), LevelProblemError> {
    for (index, level) in levels.iter().enumerate() {
        if levels[..index]
            .iter()
            .any(|candidate| candidate.level_id == level.level_id)
        {
            return Err(LevelProblemError::DuplicateLevelId {
                level_id: level.level_id,
            });
        }
    }
    Ok(())
}

fn validate_level_values(levels: &[LevelDefinition]) -> Result<(), LevelProblemError> {
    for level in levels {
        if let LevelValue::Fixed(value) = level.value
            && !value.is_finite()
        {
            return Err(LevelProblemError::NonFiniteFixedValue { value });
        }
    }
    Ok(())
}

fn validate_unique_provenance<const D: usize>(
    levels: &[LevelDefinition],
    memberships: &[LevelMembership<D>],
    orders: &[LevelOrder],
) -> Result<(), LevelProblemError>
where
    Dim<D>: SupportedDimension,
{
    let total = levels
        .len()
        .checked_add(memberships.len())
        .and_then(|count| count.checked_add(orders.len()))
        .ok_or(LevelProblemError::CountOverflow)?;
    let mut identifiers = Vec::new();
    identifiers
        .try_reserve_exact(total)
        .map_err(|_| allocation_error(LevelStorage::GraphWork, total))?;
    for identifier in levels
        .iter()
        .map(|level| level.provenance.observation_id())
        .chain(
            memberships
                .iter()
                .map(|membership| membership.provenance.observation_id()),
        )
        .chain(orders.iter().map(|order| order.provenance.observation_id()))
    {
        if identifiers.contains(&identifier) {
            return Err(LevelProblemError::DuplicateObservationId {
                observation_id: identifier,
            });
        }
        identifiers.push(identifier);
    }
    Ok(())
}

fn validate_membership_functionals<const D: usize>(
    memberships: &[LevelMembership<D>],
) -> Result<(), LevelProblemError>
where
    Dim<D>: SupportedDimension,
{
    for membership in memberships {
        let terms = membership.functional.expression().terms();
        let is_unit_value = matches!(
            terms,
            [term]
                if term.coefficient().to_bits() == 1.0_f64.to_bits()
                    && matches!(term.atom(), FunctionalAtom::Value { .. })
        );
        if !is_unit_value {
            return Err(LevelProblemError::InvalidMembershipFunctional {
                observation_id: membership.provenance.observation_id(),
            });
        }
    }
    Ok(())
}

fn membership_level_indices<const D: usize>(
    levels: &[LevelDefinition],
    memberships: &[LevelMembership<D>],
) -> Result<Vec<usize>, LevelProblemError>
where
    Dim<D>: SupportedDimension,
{
    let mut output = Vec::new();
    output
        .try_reserve_exact(memberships.len())
        .map_err(|_| allocation_error(LevelStorage::MembershipIndices, memberships.len()))?;
    for membership in memberships {
        let index =
            find_level(levels, membership.level_id).ok_or(LevelProblemError::UndefinedLevel {
                level_id: membership.level_id,
                observation_id: membership.provenance.observation_id(),
            })?;
        output.push(index);
    }
    Ok(output)
}

fn order_level_indices(
    levels: &[LevelDefinition],
    orders: &[LevelOrder],
) -> Result<Vec<(usize, usize)>, LevelProblemError> {
    let mut output = Vec::new();
    output
        .try_reserve_exact(orders.len())
        .map_err(|_| allocation_error(LevelStorage::OrderIndices, orders.len()))?;
    for order in orders {
        let lower = find_level(levels, order.lower).ok_or(LevelProblemError::UndefinedLevel {
            level_id: order.lower,
            observation_id: order.provenance.observation_id(),
        })?;
        let upper = find_level(levels, order.upper).ok_or(LevelProblemError::UndefinedLevel {
            level_id: order.upper,
            observation_id: order.provenance.observation_id(),
        })?;
        output.push((lower, upper));
    }
    Ok(output)
}

fn validate_isolation(
    levels: &[LevelDefinition],
    membership_indices: &[usize],
    order_indices: &[(usize, usize)],
) -> Result<(), LevelProblemError> {
    for (index, level) in levels.iter().enumerate() {
        if matches!(level.value, LevelValue::Unknown)
            && !membership_indices.contains(&index)
            && !order_indices
                .iter()
                .any(|(lower, upper)| *lower == index || *upper == index)
        {
            return Err(LevelProblemError::IsolatedUnknownLevel {
                level_id: level.level_id,
            });
        }
    }
    Ok(())
}

fn validate_fixed_memberships<const D: usize>(
    levels: &[LevelDefinition],
    memberships: &[LevelMembership<D>],
    membership_equality: &MembershipEquality,
) -> Result<(), LevelProblemError>
where
    Dim<D>: SupportedDimension,
{
    for first in 0..levels.len() {
        let Some(first_value) = levels[first].value.fixed() else {
            continue;
        };
        for second in (first + 1)..levels.len() {
            let Some(second_value) = levels[second].value.fixed() else {
                continue;
            };
            if (first_value - second_value).abs() > 0.0
                && membership_equality.same_component(first, second)
            {
                let path = membership_equality
                    .path_memberships(first, second)?
                    .ok_or(LevelProblemError::InvalidGraphState)?;
                let last = path
                    .len()
                    .checked_sub(1)
                    .ok_or(LevelProblemError::InvalidGraphState)?;
                let requested = path
                    .len()
                    .checked_add(2)
                    .ok_or(LevelProblemError::CountOverflow)?;
                let mut sources = Vec::new();
                sources
                    .try_reserve_exact(requested)
                    .map_err(|_| allocation_error(LevelStorage::ConflictSources, requested))?;
                sources.push(
                    DiagnosticPath::try_observation_at_level(
                        &levels[first].provenance,
                        levels[first].level_id,
                    )
                    .map_err(LevelProblemError::DiagnosticPath)?,
                );
                for membership_index in &path[..last] {
                    let membership = &memberships[*membership_index];
                    sources.push(
                        DiagnosticPath::try_observation_at_level(
                            &membership.provenance,
                            membership.level_id,
                        )
                        .map_err(LevelProblemError::DiagnosticPath)?,
                    );
                }
                sources.push(
                    DiagnosticPath::try_observation_at_level(
                        &levels[second].provenance,
                        levels[second].level_id,
                    )
                    .map_err(LevelProblemError::DiagnosticPath)?,
                );
                let last_membership = &memberships[path[last]];
                sources.push(
                    DiagnosticPath::try_observation_at_level(
                        &last_membership.provenance,
                        last_membership.level_id,
                    )
                    .map_err(LevelProblemError::DiagnosticPath)?,
                );
                return Err(LevelProblemError::FixedMembershipConflict {
                    first_level: levels[first].level_id,
                    second_level: levels[second].level_id,
                    sources,
                });
            }
        }
    }
    Ok(())
}

fn same_membership_functional<const D: usize>(
    first: &ObservationFunctional<D>,
    second: &ObservationFunctional<D>,
) -> bool
where
    Dim<D>: SupportedDimension,
{
    let [first_term] = first.expression().terms() else {
        return false;
    };
    let [second_term] = second.expression().terms() else {
        return false;
    };
    match (first_term.atom(), second_term.atom()) {
        (
            FunctionalAtom::Value {
                point: first_point, ..
            },
            FunctionalAtom::Value {
                point: second_point,
                ..
            },
        ) => first_point == second_point,
        _ => false,
    }
}

#[derive(Clone, Copy, Debug)]
struct MembershipEqualityEdge {
    first_level: usize,
    second_level: usize,
    first_membership: usize,
    second_membership: usize,
}

#[derive(Clone, Debug)]
struct MembershipEquality {
    parent: Vec<usize>,
    edges: Vec<MembershipEqualityEdge>,
}

impl MembershipEquality {
    fn try_new<const D: usize>(
        level_count: usize,
        memberships: &[LevelMembership<D>],
        membership_indices: &[usize],
    ) -> Result<Self, LevelProblemError>
    where
        Dim<D>: SupportedDimension,
    {
        let mut parent = Vec::new();
        parent
            .try_reserve_exact(level_count)
            .map_err(|_| allocation_error(LevelStorage::GraphWork, level_count))?;
        parent.extend(0..level_count);
        let edge_capacity = level_count.saturating_sub(1);
        let mut edges = Vec::new();
        edges
            .try_reserve_exact(edge_capacity)
            .map_err(|_| allocation_error(LevelStorage::GraphWork, edge_capacity))?;

        for first in 0..memberships.len() {
            for second in (first + 1)..memberships.len() {
                let first_level = membership_indices[first];
                let second_level = membership_indices[second];
                if first_level == second_level
                    || !same_membership_functional(
                        &memberships[first].functional,
                        &memberships[second].functional,
                    )
                    || find_root(&parent, first_level) == find_root(&parent, second_level)
                {
                    continue;
                }
                union(&mut parent, first_level, second_level);
                edges.push(MembershipEqualityEdge {
                    first_level,
                    second_level,
                    first_membership: first,
                    second_membership: second,
                });
            }
        }
        Ok(Self { parent, edges })
    }

    fn same_component(&self, first: usize, second: usize) -> bool {
        find_root(&self.parent, first) == find_root(&self.parent, second)
    }

    fn path_memberships(
        &self,
        source: usize,
        target: usize,
    ) -> Result<Option<Vec<usize>>, LevelProblemError> {
        if source == target {
            return Ok(Some(Vec::new()));
        }
        if !self.same_component(source, target) {
            return Ok(None);
        }

        let mut visited = try_false(self.parent.len(), LevelStorage::GraphWork)?;
        let mut parent_edge = try_none_usize(self.parent.len(), LevelStorage::GraphWork)?;
        let mut queue = Vec::new();
        queue
            .try_reserve_exact(self.parent.len())
            .map_err(|_| allocation_error(LevelStorage::GraphWork, self.parent.len()))?;
        visited[source] = true;
        queue.push(source);
        let mut next = 0;
        while next < queue.len() && !visited[target] {
            let node = queue[next];
            next += 1;
            for (edge_index, edge) in self.edges.iter().enumerate() {
                let neighbor = if edge.first_level == node {
                    edge.second_level
                } else if edge.second_level == node {
                    edge.first_level
                } else {
                    continue;
                };
                if !visited[neighbor] {
                    visited[neighbor] = true;
                    parent_edge[neighbor] = Some(edge_index);
                    queue.push(neighbor);
                }
            }
        }
        if !visited[target] {
            return Err(LevelProblemError::InvalidGraphState);
        }

        let mut path_edges = Vec::new();
        path_edges
            .try_reserve_exact(self.edges.len())
            .map_err(|_| allocation_error(LevelStorage::GraphWork, self.edges.len()))?;
        let mut cursor = target;
        while cursor != source {
            let edge_index = parent_edge[cursor].ok_or(LevelProblemError::InvalidGraphState)?;
            path_edges.push(edge_index);
            let edge = self.edges[edge_index];
            cursor = if edge.first_level == cursor {
                edge.second_level
            } else if edge.second_level == cursor {
                edge.first_level
            } else {
                return Err(LevelProblemError::InvalidGraphState);
            };
        }
        path_edges.reverse();

        let requested = path_edges
            .len()
            .checked_mul(2)
            .ok_or(LevelProblemError::CountOverflow)?;
        let mut path = Vec::new();
        path.try_reserve_exact(requested)
            .map_err(|_| allocation_error(LevelStorage::ConflictSources, requested))?;
        cursor = source;
        for edge_index in path_edges {
            let edge = self.edges[edge_index];
            let (first_membership, second_membership, next_level) = if edge.first_level == cursor {
                (
                    edge.first_membership,
                    edge.second_membership,
                    edge.second_level,
                )
            } else if edge.second_level == cursor {
                (
                    edge.second_membership,
                    edge.first_membership,
                    edge.first_level,
                )
            } else {
                return Err(LevelProblemError::InvalidGraphState);
            };
            if path.last().copied() != Some(first_membership) {
                path.push(first_membership);
            }
            if path.last().copied() != Some(second_membership) {
                path.push(second_membership);
            }
            cursor = next_level;
        }
        Ok(Some(path))
    }
}

fn validate_membership_order_paths<const D: usize>(
    memberships: &[LevelMembership<D>],
    orders: &[LevelOrder],
    membership_indices: &[usize],
    order_indices: &[(usize, usize)],
    topological_indices: &[usize],
    membership_equality: &MembershipEquality,
) -> Result<(), LevelProblemError>
where
    Dim<D>: SupportedDimension,
{
    let level_count = topological_indices.len();
    let mut reachable = try_false(level_count, LevelStorage::GraphWork)?;
    let mut positive = try_false(level_count, LevelStorage::GraphWork)?;
    let mut parent_edge = try_none_usize(level_count, LevelStorage::GraphWork)?;
    let mut has_membership = try_false(level_count, LevelStorage::GraphWork)?;
    for level_index in membership_indices {
        has_membership[*level_index] = true;
    }
    for source_level_index in 0..level_count {
        if !has_membership[source_level_index] {
            continue;
        }
        reachable.fill(false);
        positive.fill(false);
        parent_edge.fill(None);
        reachable[source_level_index] = true;

        for node in topological_indices {
            if !reachable[*node] {
                continue;
            }
            for (edge_index, ((lower, upper), order)) in
                order_indices.iter().zip(orders).enumerate()
            {
                if *lower != *node {
                    continue;
                }
                let candidate_positive = positive[*node] || order.minimum_gap > 0.0;
                if !reachable[*upper] || (candidate_positive && !positive[*upper]) {
                    reachable[*upper] = true;
                    positive[*upper] = candidate_positive;
                    parent_edge[*upper] = Some(edge_index);
                }
            }
        }

        for target_level_index in 0..level_count {
            if target_level_index == source_level_index
                || !has_membership[target_level_index]
                || !positive[target_level_index]
                || !membership_equality.same_component(source_level_index, target_level_index)
            {
                continue;
            }

            let membership_path = membership_equality
                .path_memberships(source_level_index, target_level_index)?
                .ok_or(LevelProblemError::InvalidGraphState)?;
            let last_membership = membership_path
                .len()
                .checked_sub(1)
                .ok_or(LevelProblemError::InvalidGraphState)?;

            let mut path = Vec::new();
            path.try_reserve_exact(orders.len()).map_err(|_| {
                allocation_error(
                    LevelStorage::ConflictSources,
                    orders.len().saturating_add(2),
                )
            })?;
            let mut cursor = target_level_index;
            while cursor != source_level_index {
                let edge_index = parent_edge[cursor].ok_or(LevelProblemError::InvalidGraphState)?;
                path.push(edge_index);
                cursor = order_indices[edge_index].0;
            }
            path.reverse();

            let requested = path
                .len()
                .checked_add(membership_path.len())
                .ok_or(LevelProblemError::CountOverflow)?;
            let mut sources = Vec::new();
            sources
                .try_reserve_exact(requested)
                .map_err(|_| allocation_error(LevelStorage::ConflictSources, requested))?;
            for membership_index in &membership_path[..last_membership] {
                sources.push(
                    DiagnosticPath::try_observation(&memberships[*membership_index].provenance)
                        .map_err(LevelProblemError::DiagnosticPath)?,
                );
            }
            for edge_index in path {
                sources.push(
                    DiagnosticPath::try_observation(&orders[edge_index].provenance)
                        .map_err(LevelProblemError::DiagnosticPath)?,
                );
            }
            sources.push(
                DiagnosticPath::try_observation(
                    &memberships[membership_path[last_membership]].provenance,
                )
                .map_err(LevelProblemError::DiagnosticPath)?,
            );
            return Err(LevelProblemError::MembershipOrderConflict {
                lower: memberships[membership_path[0]].level_id,
                upper: memberships[membership_path[last_membership]].level_id,
                sources,
            });
        }
    }
    Ok(())
}

fn validate_dag(
    levels: &[LevelDefinition],
    orders: &[LevelOrder],
    order_indices: &[(usize, usize)],
) -> Result<Vec<usize>, LevelProblemError> {
    let mut indegree = try_zeroed(levels.len(), LevelStorage::GraphWork)?;
    for (_, upper) in order_indices {
        indegree[*upper] = indegree[*upper]
            .checked_add(1)
            .ok_or(LevelProblemError::CountOverflow)?;
    }
    let mut output = Vec::new();
    output
        .try_reserve_exact(levels.len())
        .map_err(|_| allocation_error(LevelStorage::TopologicalOrder, levels.len()))?;
    let mut emitted = try_false(levels.len(), LevelStorage::GraphWork)?;
    while output.len() < levels.len() {
        let next = indegree
            .iter()
            .enumerate()
            .find(|(index, degree)| !emitted[*index] && **degree == 0)
            .map(|(index, _)| index);
        let Some(next) = next else {
            let mut sources = Vec::new();
            sources
                .try_reserve_exact(orders.len())
                .map_err(|_| allocation_error(LevelStorage::ConflictSources, orders.len()))?;
            let mut visited = try_false(levels.len(), LevelStorage::GraphWork)?;
            let mut stack = Vec::new();
            stack
                .try_reserve_exact(levels.len())
                .map_err(|_| allocation_error(LevelStorage::GraphWork, levels.len()))?;
            for (edge, (lower, upper)) in orders.iter().zip(order_indices) {
                if !emitted[*lower]
                    && !emitted[*upper]
                    && path_exists(*upper, *lower, order_indices, &mut visited, &mut stack)
                {
                    sources.push(
                        DiagnosticPath::try_observation(&edge.provenance)
                            .map_err(LevelProblemError::DiagnosticPath)?,
                    );
                }
            }
            return Err(LevelProblemError::OrderCycle { sources });
        };
        emitted[next] = true;
        output.push(next);
        for (_, upper) in order_indices.iter().filter(|(lower, _)| *lower == next) {
            indegree[*upper] = indegree[*upper]
                .checked_sub(1)
                .ok_or(LevelProblemError::InvalidGraphState)?;
        }
    }
    Ok(output)
}

fn path_exists(
    start: usize,
    target: usize,
    order_indices: &[(usize, usize)],
    visited: &mut [bool],
    stack: &mut Vec<usize>,
) -> bool {
    visited.fill(false);
    stack.clear();
    visited[start] = true;
    stack.push(start);
    while let Some(node) = stack.pop() {
        if node == target {
            return true;
        }
        for (_, upper) in order_indices.iter().filter(|(lower, _)| *lower == node) {
            if !visited[*upper] {
                visited[*upper] = true;
                stack.push(*upper);
            }
        }
    }
    false
}

fn validate_fixed_order_paths(
    levels: &[LevelDefinition],
    orders: &[LevelOrder],
    order_indices: &[(usize, usize)],
    topological_indices: &[usize],
) -> Result<(), LevelProblemError> {
    for (source_index, source_level) in levels.iter().enumerate() {
        let Some(source_value) = source_level.value.fixed() else {
            continue;
        };
        let mut distance = try_none_scaled(levels.len(), LevelStorage::GraphWork)?;
        let mut parent_edge = try_none_usize(levels.len(), LevelStorage::GraphWork)?;
        distance[source_index] = Some(ScaledMagnitude::ZERO);
        for node in topological_indices {
            let Some(base) = distance[*node] else {
                continue;
            };
            for (edge_index, ((lower, upper), order)) in
                order_indices.iter().zip(orders).enumerate()
            {
                if *lower != *node {
                    continue;
                }
                let candidate = base.add(ScaledMagnitude::from_f64(order.minimum_gap));
                if distance[*upper].is_none_or(|current| candidate.is_greater_than(current)) {
                    distance[*upper] = Some(candidate);
                    parent_edge[*upper] = Some(edge_index);
                }
            }
        }
        for (target_index, target_level) in levels.iter().enumerate() {
            let (Some(required_gap), Some(target_value)) =
                (distance[target_index], target_level.value.fixed())
            else {
                continue;
            };
            if target_index == source_index {
                continue;
            }
            let path_factor = u32::try_from(levels.len()).map_or(f64::from(u32::MAX), f64::from);
            let relative_tolerance = f64::EPSILON * 64.0 * path_factor;
            let available_gap = nonnegative_difference(target_value, source_value);
            if available_gap.is_some_and(|available| {
                !required_gap.exceeds_with_tolerance(available, relative_tolerance)
            }) {
                continue;
            }
            let mut path = Vec::new();
            path.try_reserve_exact(orders.len()).map_err(|_| {
                allocation_error(
                    LevelStorage::ConflictSources,
                    orders.len().saturating_add(2),
                )
            })?;
            let mut cursor = target_index;
            while cursor != source_index {
                let Some(edge_index) = parent_edge[cursor] else {
                    break;
                };
                path.push(edge_index);
                cursor = order_indices[edge_index].0;
            }
            path.reverse();
            let requested = path.len().saturating_add(2);
            let mut sources = Vec::new();
            sources
                .try_reserve_exact(requested)
                .map_err(|_| allocation_error(LevelStorage::ConflictSources, requested))?;
            sources.push(
                DiagnosticPath::try_observation_at_level(
                    &source_level.provenance,
                    source_level.level_id,
                )
                .map_err(LevelProblemError::DiagnosticPath)?,
            );
            for edge_index in path {
                sources.push(
                    DiagnosticPath::try_observation(&orders[edge_index].provenance)
                        .map_err(LevelProblemError::DiagnosticPath)?,
                );
            }
            sources.push(
                DiagnosticPath::try_observation_at_level(
                    &target_level.provenance,
                    target_level.level_id,
                )
                .map_err(LevelProblemError::DiagnosticPath)?,
            );
            return Err(LevelProblemError::FixedOrderConflict {
                lower: source_level.level_id,
                upper: target_level.level_id,
                required_gap: required_gap.to_f64(),
                fixed_gap: target_value - source_value,
                sources,
            });
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug)]
struct ScaledMagnitude {
    mantissa: f64,
    exponent: i32,
}

impl ScaledMagnitude {
    const ZERO: Self = Self {
        mantissa: 0.0,
        exponent: 0,
    };

    fn from_f64(value: f64) -> Self {
        debug_assert!(value.is_finite() && value >= 0.0);
        if value == 0.0 {
            return Self::ZERO;
        }
        let bits = value.to_bits();
        let stored_exponent = i32::try_from((bits >> 52) & 0x7ff).unwrap_or_default();
        let fraction_mask = (1_u64 << 52) - 1;
        let fraction = bits & fraction_mask;
        if stored_exponent == 0 {
            let leading = 63_i32 - i32::try_from(fraction.leading_zeros()).unwrap_or(i32::MAX);
            let shift = u32::try_from(52_i32 - leading).unwrap_or_default();
            let normalized = fraction << shift;
            Self {
                mantissa: f64::from_bits((1023_u64 << 52) | (normalized & fraction_mask)),
                exponent: leading - 1074,
            }
        } else {
            Self {
                mantissa: f64::from_bits((1023_u64 << 52) | fraction),
                exponent: stored_exponent - 1023,
            }
        }
    }

    fn add(self, other: Self) -> Self {
        if self.mantissa == 0.0 {
            return other;
        }
        if other.mantissa == 0.0 {
            return self;
        }
        let (larger, smaller) = if self.exponent >= other.exponent {
            (self, other)
        } else {
            (other, self)
        };
        let exponent_difference = larger.exponent - smaller.exponent;
        let scaled_smaller = if exponent_difference > 1074 {
            0.0
        } else {
            smaller.mantissa * 2.0_f64.powi(-exponent_difference)
        };
        let mantissa = larger.mantissa + scaled_smaller;
        if mantissa >= 2.0 {
            Self {
                mantissa: mantissa * 0.5,
                exponent: larger.exponent + 1,
            }
        } else {
            Self {
                mantissa,
                exponent: larger.exponent,
            }
        }
    }

    fn is_greater_than(self, other: Self) -> bool {
        if other.mantissa == 0.0 {
            return self.mantissa > 0.0;
        }
        if self.mantissa == 0.0 {
            return false;
        }
        self.exponent > other.exponent
            || (self.exponent == other.exponent && self.mantissa > other.mantissa)
    }

    fn exceeds_with_tolerance(self, available: Self, relative_tolerance: f64) -> bool {
        if !self.is_greater_than(available) {
            return false;
        }
        if available.mantissa == 0.0 {
            return true;
        }
        let scale = self;
        let required_at_scale = self.mantissa_at_exponent(scale.exponent);
        let available_at_scale = available.mantissa_at_exponent(scale.exponent);
        required_at_scale - available_at_scale > relative_tolerance * scale.mantissa
    }

    fn mantissa_at_exponent(self, exponent: i32) -> f64 {
        if self.mantissa == 0.0 {
            return 0.0;
        }
        let exponent_difference = exponent - self.exponent;
        if exponent_difference > 1074 {
            0.0
        } else {
            self.mantissa * 2.0_f64.powi(-exponent_difference)
        }
    }

    fn to_f64(self) -> f64 {
        if self.mantissa == 0.0 {
            0.0
        } else if self.exponent > 1023 {
            f64::INFINITY
        } else {
            self.mantissa * 2.0_f64.powi(self.exponent)
        }
    }
}

fn nonnegative_difference(upper: f64, lower: f64) -> Option<ScaledMagnitude> {
    if upper < lower {
        return None;
    }
    if lower < 0.0 && upper >= 0.0 {
        Some(ScaledMagnitude::from_f64(-lower).add(ScaledMagnitude::from_f64(upper)))
    } else {
        Some(ScaledMagnitude::from_f64(upper - lower))
    }
}

fn validate_gauge(
    levels: &[LevelDefinition],
    membership_indices: &[usize],
    order_indices: &[(usize, usize)],
) -> Result<(), LevelProblemError> {
    let field_node = levels.len();
    let node_count = levels
        .len()
        .checked_add(1)
        .ok_or(LevelProblemError::CountOverflow)?;
    let mut parent = Vec::new();
    parent
        .try_reserve_exact(node_count)
        .map_err(|_| allocation_error(LevelStorage::GraphWork, node_count))?;
    parent.extend(0..node_count);
    for level_index in membership_indices {
        union(&mut parent, *level_index, field_node);
    }
    for (lower, upper) in order_indices {
        union(&mut parent, *lower, *upper);
    }
    let mut roots = Vec::new();
    roots
        .try_reserve_exact(levels.len())
        .map_err(|_| allocation_error(LevelStorage::GraphWork, levels.len()))?;
    for index in 0..levels.len() {
        let root = find_root(&parent, index);
        if !roots.contains(&root) {
            roots.push(root);
        }
    }
    let ungauged = roots
        .iter()
        .filter(|root| {
            !levels.iter().enumerate().any(|(index, level)| {
                find_root(&parent, index) == **root && level.value.anchors_gauge()
            })
        })
        .count();
    if ungauged > 0 {
        return Err(LevelProblemError::MissingGauge {
            diagnostic: GaugeDiagnostic::try_new(ungauged)
                .map_err(LevelProblemError::DiagnosticValue)?,
        });
    }
    Ok(())
}

fn validate_contrast(
    levels: &[LevelDefinition],
    orders: &[LevelOrder],
    membership_indices: &[usize],
    order_indices: &[(usize, usize)],
    topological_indices: &[usize],
    membership_equality: &MembershipEquality,
) -> Result<(), LevelProblemError> {
    let field_node = levels.len();
    let node_count = levels
        .len()
        .checked_add(1)
        .ok_or(LevelProblemError::CountOverflow)?;
    let mut parent = Vec::new();
    parent
        .try_reserve_exact(node_count)
        .map_err(|_| allocation_error(LevelStorage::GraphWork, node_count))?;
    parent.extend(0..node_count);
    for level_index in membership_indices {
        union(&mut parent, *level_index, field_node);
    }
    for (lower, upper) in order_indices {
        union(&mut parent, *lower, *upper);
    }
    let field_root = find_root(&parent, field_node);
    let mut has_membership = try_false(levels.len(), LevelStorage::GraphWork)?;
    for level_index in membership_indices {
        has_membership[*level_index] = true;
    }

    let mut has_gap = false;
    let mut reachable = try_false(levels.len(), LevelStorage::GraphWork)?;
    let mut positive = try_false(levels.len(), LevelStorage::GraphWork)?;
    for source in 0..levels.len() {
        if !has_membership[source] {
            continue;
        }
        reachable.fill(false);
        positive.fill(false);
        reachable[source] = true;
        for node in topological_indices {
            if !reachable[*node] {
                continue;
            }
            for ((lower, upper), order) in order_indices.iter().zip(orders) {
                if *lower != *node {
                    continue;
                }
                let candidate_positive = positive[*node] || order.minimum_gap > 0.0;
                reachable[*upper] = true;
                positive[*upper] |= candidate_positive;
            }
        }
        if (0..levels.len()).any(|target| {
            target != source && has_membership[target] && reachable[target] && positive[target]
        }) {
            has_gap = true;
            break;
        }
    }

    let has_distinct_anchors =
        has_distinct_anchor_memberships(levels, &has_membership, membership_equality);
    if !has_gap && !has_distinct_anchors {
        let first_index = membership_indices[0];
        let second_index = membership_indices
            .iter()
            .copied()
            .find(|index| *index != first_index)
            .or_else(|| {
                (0..levels.len())
                    .find(|index| *index != first_index && find_root(&parent, *index) == field_root)
            });
        let diagnostic = if let Some(second_index) = second_index {
            ContrastDiagnostic::try_new(levels[first_index].level_id, levels[second_index].level_id)
                .map_err(LevelProblemError::DiagnosticValue)?
        } else {
            ContrastDiagnostic::single(levels[first_index].level_id)
        };
        return Err(LevelProblemError::MissingContrast { diagnostic });
    }
    Ok(())
}

fn has_distinct_anchor_memberships(
    levels: &[LevelDefinition],
    has_membership: &[bool],
    membership_equality: &MembershipEquality,
) -> bool {
    for first_index in 0..levels.len() {
        if !has_membership[first_index] {
            continue;
        }
        let Some(first_value) = anchor_value(levels[first_index].value) else {
            continue;
        };
        for (second_index, second) in levels.iter().enumerate().skip(first_index + 1) {
            if has_membership[second_index]
                && anchor_value(second.value)
                    .is_some_and(|second_value| (second_value - first_value).abs() > 0.0)
                && !membership_equality.same_component(first_index, second_index)
            {
                return true;
            }
        }
    }
    false
}

const fn anchor_value(value: LevelValue) -> Option<f64> {
    match value {
        LevelValue::Fixed(value) => Some(value),
        LevelValue::Prior(prior) => Some(prior.mean),
        LevelValue::Unknown => None,
    }
}

fn find_level(levels: &[LevelDefinition], level_id: LevelId) -> Option<usize> {
    levels
        .iter()
        .position(|candidate| candidate.level_id == level_id)
}

fn find_root(parent: &[usize], mut node: usize) -> usize {
    while parent[node] != node {
        node = parent[node];
    }
    node
}

fn union(parent: &mut [usize], left: usize, right: usize) {
    let left_root = find_root(parent, left);
    let right_root = find_root(parent, right);
    if left_root != right_root {
        let (smaller, larger) = if left_root < right_root {
            (left_root, right_root)
        } else {
            (right_root, left_root)
        };
        parent[larger] = smaller;
    }
}

fn try_collect<T>(
    values: impl IntoIterator<Item = T>,
    storage: LevelStorage,
) -> Result<Vec<T>, LevelProblemError> {
    let iterator = values.into_iter();
    let minimum = iterator.size_hint().0;
    let mut output = Vec::new();
    output
        .try_reserve_exact(minimum)
        .map_err(|_| allocation_error(storage, minimum))?;
    for value in iterator {
        if output.len() == output.capacity() {
            output
                .try_reserve(1)
                .map_err(|_| allocation_error(storage, output.len().saturating_add(1)))?;
        }
        output.push(value);
    }
    Ok(output)
}

fn collect_blocks(
    values: impl IntoIterator<Item = VariableBlock>,
) -> Result<Vec<VariableBlock>, ProblemIrError> {
    let iterator = values.into_iter();
    let minimum = iterator.size_hint().0;
    let mut output = Vec::new();
    output
        .try_reserve_exact(minimum)
        .map_err(|_| allocation_error_ir(LevelStorage::VariableBlocks, minimum))?;
    for value in iterator {
        if output.len() == output.capacity() {
            output.try_reserve(1).map_err(|_| {
                allocation_error_ir(LevelStorage::VariableBlocks, output.len().saturating_add(1))
            })?;
        }
        output.push(value);
    }
    Ok(output)
}

fn try_zeroed(count: usize, storage: LevelStorage) -> Result<Vec<usize>, LevelProblemError> {
    let mut output = Vec::new();
    output
        .try_reserve_exact(count)
        .map_err(|_| allocation_error(storage, count))?;
    output.resize(count, 0);
    Ok(output)
}

fn try_false(count: usize, storage: LevelStorage) -> Result<Vec<bool>, LevelProblemError> {
    let mut output = Vec::new();
    output
        .try_reserve_exact(count)
        .map_err(|_| allocation_error(storage, count))?;
    output.resize(count, false);
    Ok(output)
}

fn try_none_scaled(
    count: usize,
    storage: LevelStorage,
) -> Result<Vec<Option<ScaledMagnitude>>, LevelProblemError> {
    let mut output = Vec::new();
    output
        .try_reserve_exact(count)
        .map_err(|_| allocation_error(storage, count))?;
    output.resize(count, None);
    Ok(output)
}

fn try_none_usize(
    count: usize,
    storage: LevelStorage,
) -> Result<Vec<Option<usize>>, LevelProblemError> {
    let mut output = Vec::new();
    output
        .try_reserve_exact(count)
        .map_err(|_| allocation_error(storage, count))?;
    output.resize(count, None);
    Ok(output)
}

const fn allocation_error(storage: LevelStorage, requested: usize) -> LevelProblemError {
    LevelProblemError::AllocationFailed { storage, requested }
}

const fn allocation_error_ir(storage: LevelStorage, requested: usize) -> ProblemIrError {
    let ir_storage = match storage {
        LevelStorage::VariableBlocks => crate::problem_ir::ProblemIrStorage::VariableBlocks,
        LevelStorage::CanonicalEqualities => {
            crate::problem_ir::ProblemIrStorage::CanonicalEqualities
        }
        LevelStorage::CanonicalLinearBounds => {
            crate::problem_ir::ProblemIrStorage::CanonicalLinearBounds
        }
        LevelStorage::AffineTerms => crate::problem_ir::ProblemIrStorage::AffineTerms,
        LevelStorage::CanonicalPriors | LevelStorage::CompiledLevelIds => {
            crate::problem_ir::ProblemIrStorage::CanonicalProvenance
        }
        LevelStorage::Definitions
        | LevelStorage::Memberships
        | LevelStorage::Orders
        | LevelStorage::MembershipIndices
        | LevelStorage::OrderIndices
        | LevelStorage::TopologicalOrder
        | LevelStorage::GraphWork
        | LevelStorage::ConflictSources => crate::problem_ir::ProblemIrStorage::SemanticConstraints,
    };
    ProblemIrError::AllocationFailed {
        storage: ir_storage,
        requested,
    }
}
