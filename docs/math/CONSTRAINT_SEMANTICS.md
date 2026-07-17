# Constraint Semantics

## Relations and enforcement

A compiled functional expression may use equality, lower bound, upper bound,
closed interval, or second-order-cone relations. Each observation independently
chooses hard enforcement or soft enforcement with a positive scale and a loss:
SquaredL2, AbsoluteL1, or Huber with a positive delta.

Hard constraints are feasibility conditions. Compilation and solution must not
delete them, perturb their targets, add unrequested jitter, or turn them into
soft penalties. Soft losses are explicit objective contributions. Scaling
changes numerical representation, not user semantics, and is reported.

## Level variables

Each geological level has one explicit scalar `h_k`. A point assigned to that
level imposes

```text
f(x_i) - h_k = 0.
```

Level values are fixed, unknown, or soft priors carrying mean, scale, and loss.
An order edge from `a` to `b` imposes

```text
h_b - h_a >= delta_ab.
```

The Rust semantic layer preserves every level as a stable identifier and every
definition, membership, and order edge as a separate provenance-bearing
record. Fixed values compile to hard equality rows. Priors stay as explicit
mean, positive scale, and SquaredL2, AbsoluteL1, or Huber metadata; they are not
silently compiled as hard rows or claimed as solved before the approved soft
objective backend exists.

The compiler requires at least two levels and one field membership. It rejects
duplicate identifiers, missing references, self edges, negative or non-finite
gaps, and unknown levels with neither a membership nor an order edge. A stable
topological pass rejects every directed cycle. Fixed-value review checks both
identical field functionals assigned to different fixed levels and transitive
minimum-gap paths whose fixed endpoint values cannot satisfy the accumulated
gap. Near a floating-point equality boundary, the precheck is conservative;
the original individual hard rows remain unchanged in canonical form.

Gauge review treats memberships as edges to the shared scalar field and order
relations as edges between levels. Every resulting connected component needs a
fixed value or explicit prior. Nonzero contrast must affect the field-connected
component through a positive minimum gap or distinct fixed/prior anchors;
unrelated isolated anchors cannot manufacture field contrast. Reference point
differencing is an optional internal elimination of the same explicit
variables, never a separate model type.

Canonical variable order is the caller's field blocks followed by one
deterministic `levels` block in definition insertion order. Memberships compile
first, fixed rows second, and order bounds third, each retaining its original
source provenance. Solvers receive only sparse equalities and linear bounds,
not level identifiers or geological relations.

## Canonical mappings

Linear equalities become `A_eq z = b_eq`. Bounds and intervals become lower and
upper limits on rows of `A_lin z`. L2 penalties become quadratic objective
terms. L1 and Huber losses use explicit epigraph forms in an approved convex
backend. No solver receives geological semantics.

Duplicate and near-duplicate functionals are diagnosed with scale-aware
criteria. Conflicting hard constraints return source-aware infeasibility; they
are not silently reconciled.
