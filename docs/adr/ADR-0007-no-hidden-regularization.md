# ADR-0007: No Hidden Regularization or Constraint Relaxation

- Status: Accepted
- Date: 2026-07-13

## Context

RBF systems can be ill-conditioned, rank deficient, or infeasible. Silent
jitter, pseudoinverse fallback, constraint deletion, or hard-to-soft conversion
can return plausible-looking fields that violate the stated problem.

## Decision

Regularization is None, Explicit(value), or AutomaticWithin(maximum). Any
automatic adjustment is opt-in and records requested and actual solver,
regularization, original and effective rank, condition estimates, and cause.
Hard constraints are never silently changed. Rank deficiency, incompatibility,
ill-conditioning, and infeasibility return structured diagnostics.

## Consequences

Some inputs that other systems silently approximate will fail. The failure is
reproducible and actionable, while accepted results preserve the user's stated
semantics.
