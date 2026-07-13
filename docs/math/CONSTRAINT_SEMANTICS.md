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

The compiler checks cycles, fixed-value contradictions, isolated unknown
levels, missing additive gauge, and missing nonzero field contrast. Reference
point differencing is an optional elimination of level variables, never a
separate model type.

## Canonical mappings

Linear equalities become `A_eq z = b_eq`. Bounds and intervals become lower and
upper limits on rows of `A_lin z`. L2 penalties become quadratic objective
terms. L1 and Huber losses use explicit epigraph forms in an approved convex
backend. No solver receives geological semantics.

Duplicate and near-duplicate functionals are diagnosed with scale-aware
criteria. Conflicting hard constraints return source-aware infeasibility; they
are not silently reconciled.
