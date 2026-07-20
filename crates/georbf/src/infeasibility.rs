//! Source-aware duplicate and near-duplicate review for hard constraints.
//!
//! The review is diagnostic only: it never removes, merges, rescales, or
//! relaxes a canonical relation. Exact infeasibility remains a construction
//! error, while general convex infeasibility remains subject to the solver's
//! independently reviewed certificate path.

use crate::problem_ir::{
    AffineExpression, CanonicalProblem, ProblemIrError, ProblemIrStorage, SemanticProvenance,
};

/// Dimensionless infinity-distance threshold used for near-duplicate rows.
///
/// Each nonzero row is independently infinity-normalized, both orientations
/// are compared, and a nonzero distance at most this value is diagnostic only.
pub const NEAR_DUPLICATE_THRESHOLD: f64 = 128.0 * f64::EPSILON;

/// Hard canonical affine relation family participating in duplicate review.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub enum HardAffineConstraintKind {
    /// Hard affine equality.
    Equality,
    /// Hard affine lower, upper, or closed-interval bound.
    LinearBound,
}

/// Orientation placing two reviewed functionals in their closest alignment.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub enum FunctionalOrientation {
    /// The normalized coefficient signs align directly.
    Same,
    /// Negating the second normalized row gives the closest alignment.
    Reversed,
}

/// Scale-aware similarity classification for two hard functionals.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub enum FunctionalSimilarity {
    /// The sparse rows are exactly proportional by one finite nonzero scalar.
    Duplicate,
    /// The normalized rows are distinct but within the recorded threshold.
    NearDuplicate,
}

/// Complete source-aware evidence for one duplicate or near-duplicate pair.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct ConstraintPairDiagnostics {
    /// Exact or near-duplicate classification.
    pub similarity: FunctionalSimilarity,
    /// First relation family in canonical equality-then-bound order.
    pub first_kind: HardAffineConstraintKind,
    /// Complete first source provenance.
    pub first_provenance: SemanticProvenance,
    /// Second relation family in canonical equality-then-bound order.
    pub second_kind: HardAffineConstraintKind,
    /// Complete second source provenance.
    pub second_provenance: SemanticProvenance,
    /// Closest normalized sign alignment.
    pub orientation: FunctionalOrientation,
    /// Infinity distance between independently normalized sparse rows.
    pub normalized_row_distance: f64,
    /// Explicit dimensionless threshold used for the classification.
    pub comparison_threshold: f64,
}

/// Deterministic, immutable review of hard canonical affine functionals.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct ConstraintReviewDiagnostics {
    /// Number of reviewed hard equality and linear-bound rows.
    pub reviewed_hard_affine_constraints: usize,
    /// Hard ordered second-order cones excluded from affine pair comparison.
    pub hard_second_order_cones: usize,
    /// Soft objectives excluded from hard-feasibility decisions.
    pub excluded_soft_objectives: usize,
    /// Duplicate and near-duplicate pairs in deterministic second-then-first order.
    pub pairs: Vec<ConstraintPairDiagnostics>,
}

#[derive(Clone, Copy)]
struct ReviewedRow<'a> {
    kind: HardAffineConstraintKind,
    row: &'a AffineExpression,
    provenance: &'a SemanticProvenance,
}

/// Reviews hard canonical affine rows for duplicate and near-duplicate functionals.
///
/// Equalities precede linear bounds, preserving each family's canonical order.
/// For each later row, earlier rows are compared in order. Soft objectives do
/// not participate because they are penalties rather than feasibility
/// conditions. Ordered second-order cones are counted but are not treated as
/// affine functionals.
///
/// # Errors
///
/// Returns [`ProblemIrError::AllocationFailed`] if source-aware diagnostic
/// storage cannot be reserved or cloned. The canonical problem is unchanged.
pub fn try_review_constraints(
    problem: &CanonicalProblem,
) -> Result<ConstraintReviewDiagnostics, ProblemIrError> {
    let reviewed_hard_affine_constraints = problem
        .equalities()
        .len()
        .checked_add(problem.linear_bounds().len())
        .ok_or(ProblemIrError::MemoryEstimateOverflow)?;
    let mut pairs = Vec::new();

    for second_index in 0..reviewed_hard_affine_constraints {
        let second = reviewed_row(problem, second_index);
        if second.row.terms().is_empty() {
            continue;
        }
        for first_index in 0..second_index {
            let first = reviewed_row(problem, first_index);
            if first.row.terms().is_empty() {
                continue;
            }
            let (distance, orientation) = normalized_row_distance(first.row, second.row);
            let exact_scale = super::problem_ir::exact_row_scale(first.row, second.row);
            let similarity = if exact_scale.is_some() {
                FunctionalSimilarity::Duplicate
            } else if distance <= NEAR_DUPLICATE_THRESHOLD {
                FunctionalSimilarity::NearDuplicate
            } else {
                continue;
            };
            pairs
                .try_reserve(1)
                .map_err(|_| ProblemIrError::AllocationFailed {
                    storage: ProblemIrStorage::ConstraintDiagnostics,
                    requested: pairs.len().saturating_add(1),
                })?;
            pairs.push(ConstraintPairDiagnostics {
                similarity,
                first_kind: first.kind,
                first_provenance: first.provenance.try_clone_for_canonical()?,
                second_kind: second.kind,
                second_provenance: second.provenance.try_clone_for_canonical()?,
                orientation,
                normalized_row_distance: distance,
                comparison_threshold: NEAR_DUPLICATE_THRESHOLD,
            });
        }
    }

    Ok(ConstraintReviewDiagnostics {
        reviewed_hard_affine_constraints,
        hard_second_order_cones: problem.second_order_cones().len(),
        excluded_soft_objectives: problem.soft_objectives().len(),
        pairs,
    })
}

fn reviewed_row(problem: &CanonicalProblem, index: usize) -> ReviewedRow<'_> {
    if let Some(equality) = problem.equalities().get(index) {
        return ReviewedRow {
            kind: HardAffineConstraintKind::Equality,
            row: equality.row(),
            provenance: equality.provenance(),
        };
    }
    let bound = &problem.linear_bounds()[index - problem.equalities().len()];
    ReviewedRow {
        kind: HardAffineConstraintKind::LinearBound,
        row: bound.row(),
        provenance: bound.provenance(),
    }
}

fn normalized_row_distance(
    first: &AffineExpression,
    second: &AffineExpression,
) -> (f64, FunctionalOrientation) {
    let first_scale = row_infinity_norm(first);
    let second_scale = row_infinity_norm(second);
    let same = oriented_distance(first, first_scale, second, second_scale, 1.0);
    let reversed = oriented_distance(first, first_scale, second, second_scale, -1.0);
    if reversed < same {
        (reversed, FunctionalOrientation::Reversed)
    } else {
        (same, FunctionalOrientation::Same)
    }
}

fn row_infinity_norm(row: &AffineExpression) -> f64 {
    row.terms()
        .iter()
        .map(|term| term.coefficient().abs())
        .fold(0.0, f64::max)
}

fn oriented_distance(
    first: &AffineExpression,
    first_scale: f64,
    second: &AffineExpression,
    second_scale: f64,
    second_sign: f64,
) -> f64 {
    let first_terms = first.terms();
    let second_terms = second.terms();
    let mut first_index = 0;
    let mut second_index = 0;
    let mut distance = 0.0_f64;
    while first_index < first_terms.len() || second_index < second_terms.len() {
        let first_variable = first_terms.get(first_index).map(|term| term.variable());
        let second_variable = second_terms.get(second_index).map(|term| term.variable());
        match (first_variable, second_variable) {
            (Some(first_variable), Some(second_variable)) if first_variable == second_variable => {
                let left = first_terms[first_index].coefficient() / first_scale;
                let right = second_sign * second_terms[second_index].coefficient() / second_scale;
                distance = distance.max((left - right).abs());
                first_index += 1;
                second_index += 1;
            }
            (Some(first_variable), Some(second_variable)) if first_variable < second_variable => {
                distance =
                    distance.max((first_terms[first_index].coefficient() / first_scale).abs());
                first_index += 1;
            }
            (Some(_) | None, Some(_)) => {
                distance =
                    distance.max((second_terms[second_index].coefficient() / second_scale).abs());
                second_index += 1;
            }
            (Some(_), None) => {
                distance =
                    distance.max((first_terms[first_index].coefficient() / first_scale).abs());
                first_index += 1;
            }
            (None, None) => break,
        }
    }
    distance
}
