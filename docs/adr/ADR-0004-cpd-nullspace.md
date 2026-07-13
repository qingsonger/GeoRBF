# ADR-0004: CPD Polynomial Null-Space Formulation

- Status: Accepted
- Date: 2026-07-13

## Context

Conditionally positive-definite kernels require polynomial augmentation and the
side condition `Q^T w = 0`. Fixed point selections, cubic-only logic, coordinate
perturbation, or pseudoinverse fallback do not generalize across dimensions and
hide unisolvency failure.

## Decision

Generate the complete polynomial space for the kernel's CPD order. Form center
actions `Q`, diagnose rank with scale-aware RRQR and SVD review, construct a
null-space basis `Z`, and solve with `w = Z y` and projected energy when that is
the selected formulation. A mathematically equivalent pivoted KKT path may be
used and must agree in tests.

## Consequences

CPD support works in D=1, D=2, and D=3 for arbitrary implemented orders.
Rank-deficient inputs fail explicitly; user coordinates are not mutated to make
them appear valid.
