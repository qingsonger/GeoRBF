# ADR-0003: Explicit Level Variables

- Status: Accepted
- Date: 2026-07-13

## Context

Points belonging to the same geological level share an unknown scalar value.
Pairwise differencing can eliminate that value but obscures level order, priors,
provenance, gauge freedom, and conflict diagnostics.

## Decision

Represent every level with an explicit `h_k`. Membership is
`f(x_i) - h_k = 0`; a level is fixed, unknown, or carries a soft prior. Order
edges form a DAG with `h_b - h_a >= delta_ab`. Compilation checks cycles,
fixed conflicts, isolation, additive gauge, and nonzero contrast. Differencing
is an optional internal elimination only.

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
canonical problem retains the original individual hard constraints.
