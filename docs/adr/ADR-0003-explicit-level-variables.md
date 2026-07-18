# ADR-0003: Explicit Level Variables

- Status: Accepted
- Date: 2026-07-13

## Context

Points belonging to the same geological level share an unknown scalar value.
Pairwise differencing can eliminate that value but obscures level order, priors,
provenance, gauge freedom, and conflict diagnostics.

## Decision

Represent every level with an explicit `h_k`. Membership is exactly one
unit-weight Value evaluation `f(x_i) - h_k = 0`; derivative, scaled, and
multi-atom functionals are not memberships. A level is fixed, unknown, or
carries a soft prior. Order edges form a DAG with
`h_b - h_a >= delta_ab`. Compilation checks cycles, fixed conflicts, isolation,
additive gauge, and nonzero contrast between membership-coupled levels.
Differencing is an optional internal elimination only.

## Consequences

Level semantics remain inspectable through canonicalization and serialization.
Systems are somewhat larger, but rank and infeasibility diagnostics are honest
and priors and inequalities compose directly.

The initial implementation retains soft priors beside the hard canonical
problem as explicit objective metadata. It does not pretend that the current
dense equality solver supports those objectives or order bounds. Gauge is
checked per connected component formed by level-order edges and memberships to
the shared scalar field; contrast must occur in the field-connected component.
Fixed-order conflict review includes transitive path gaps, while the emitted
canonical problem retains the original individual hard constraints. Its
overflow-safe endpoint comparison treats exact zero directly and scales
roundoff allowance only from the compared gaps, preserving feasibility under a
positive change of scalar unit. A positive order path between levels in the
transitive equality closure induced by shared
mathematical Value evaluations is rejected with the deterministic membership
chain and selected path sources because the membership equalities force the
corresponding level gap to zero. Distinct fixed values in one closure are
rejected with the same chain evidence. Likewise, distinct fixed or prior anchor
values establish contrast only when their levels belong to different
membership-equality components; a soft prior is never promoted to a hard
equality to manufacture contrast. A one-level failing field component is
reported without borrowing an unrelated isolated anchor as evidence.
