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

A membership is exactly one coefficient-1 Value atom. Directional derivatives,
scaled values, and multi-atom expressions are rejected because they do not
share the value units or joint-shift gauge of `f(x_i) - h_k = 0`.

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
topological pass rejects every directed cycle. Hard-conflict review forms the
transitive level-equality closure induced by mathematically identical Value
evaluations, independent of functional provenance. A shared evaluation equates
its levels, and chains through a level's other memberships propagate that
equality. Different fixed values or a positive minimum-gap path within one such
component are infeasible; diagnostics retain both fixed definitions when
applicable, a deterministic membership chain proving equality, and every edge
of the selected order path. The review also checks transitive minimum-gap paths
whose fixed endpoint values cannot satisfy the accumulated gap. Path sums and
fixed-endpoint differences use an overflow-safe scaled comparison. Near a
floating-point equality boundary, the precheck is conservative and derives its
relative scale only from the required and available gaps. Exact zero is handled
directly, so a positive scalar-unit rescaling cannot change feasibility. The
original individual hard rows remain unchanged in canonical form.

Gauge review treats memberships as edges to the shared scalar field and order
relations as edges between levels. Every resulting connected component needs a
fixed value or explicit prior. Nonzero contrast must be proved between two
membership-coupled level functionals through a positive minimum-gap path or
distinct fixed/prior anchors on those coupled levels. Distinct anchor values
count only when the two anchored levels belong to different
membership-equality components; in particular, a soft prior mean cannot
override a direct or transitive hard field equality. A positive gap or distinct
anchor on a membershipless level cannot manufacture field contrast.
Missing-contrast evidence names only the failing field component and represents
a component with one membership-bearing level without citing an unrelated
isolated anchor. Reference point differencing is an optional internal
elimination of the same explicit variables, never a separate model type.

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
