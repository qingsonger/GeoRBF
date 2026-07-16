# Solver and Numerical Policy

Production backends are selected only after the recorded spikes:

- SPIKE-001: dense SPD and symmetric-indefinite factorization;
- SPIKE-002: rank-revealing QR and SVD;
- SPIKE-003: compact sparse assembly and solution; and
- SPIKE-004: canonical QP and SOCP mapping.

REQ-SPIKE-001 selects the nalgebra 0.35 release line as the preferred later
production backend for checked dense Cholesky and Bunch--Kaufman LBLT with
symmetric 1-by-1 and 2-by-2 pivot blocks. The spike itself remains isolated;
production adoption must stay behind the private numerical adapter, repeat the
dependency and advisory audit, and keep nalgebra matrix types out of public
geometry and model APIs. Checked Cholesky must never use the substitute-
diagonal entry point. A zero pivot, nonfinite result, failed original-unit
residual review, or nondecreasing refinement correction is an explicit failure.

REQ-CPD-001 adopts nalgebra 0.35.0 behind the private CPD numerical adapter for
column-pivoted QR screening, bounded SVD rank review, and QR-supported
null-space construction. GeoRBF-owned row-major matrices and diagnostics are
the only public types. The adapter does not expose or call pseudoinverse or
minimum-norm solve APIs.

Each spike reviews correctness, scaling, maintenance, license, MSRV, unsafe
use, platforms, binary size, alternatives, and deterministic behavior, then
produces an ADR before dependency lock-in.

Target dispatch is checked Cholesky for SPD equality systems, pivoted
Bunch--Kaufman LBLT for symmetric-indefinite KKT systems, RRQR for rank risk,
SVD for rank review or justified minimum-norm work, projected weighted least
squares for L2 losses, a mature convex QP backend for linear inequalities and
epigraph losses, and a mature SOCP backend for angular and thickness cones.

Every solve performs finite and unit checks, coordinate and direction
normalization, explicit constraint scaling, duplicate/conflict review,
polynomial rank and kernel capability checks, anisotropy validation, memory
estimation, symmetry checks, conditioning estimation, iterative refinement, and
residual review in original units.

Iterative refinement is bounded and explicit. It reuses the unchanged
mathematical matrix and requested factorization, records the initial and final
original-unit residuals and accepted correction count, and accepts a correction
only when that residual strictly decreases. It never authorizes a factorization
switch, diagonal substitution, jitter, pseudoinverse, constraint relaxation,
or other implicit problem change.

Rank and condition decisions use an explicitly recorded dimensionless
equilibration. Diagnostics retain row and column scales, matrix norms, rank
thresholds, effective rank, condition estimates, and both scaled and
original-unit residuals. Unit changes and equivalent nonzero row scaling must
not change feasibility or rank classification. A large condition estimate may
trigger a warning or an explicit policy error, but never an unrequested change
to the mathematical problem.

The CPD rank policy uses eight alternating infinity-norm equilibration passes,
the dimension-times-epsilon RRQR and SVD thresholds recorded in
`docs/math/CPD_AND_POLYNOMIALS.md`, and a factor-16 SVD ambiguity guard band.
RRQR/SVD disagreement or threshold adjacency is an explicit error, not an
automatic solver adjustment. Equilibration rejects a cumulative multiplier or
individual nonzero entry that becomes unrepresentable. Bounded SVD
non-convergence retains completed equilibration and RRQR evidence while
marking every SVD-derived field and the final rank decision unavailable.
Null-space QR uses that same safely equilibrated matrix, maps the basis back
through exponent-aware products with the recorded row scaling, and
reorthogonalizes before binding matrix-infinity residual checks. Original-unit
residuals use a fixed stack exact-binary superaccumulator, round only the final
signed sum, and fail explicitly when a nonzero result would round to zero or
overflow instead of fabricating a finite diagnostic.

Regularization is None, Explicit(value), or AutomaticWithin(maximum). Any
automatic choice records requested and actual solver, amount added, original
and effective rank, condition estimates, and trigger. No pseudoinverse, jitter,
or fallback may hide failure.

## Dense equality implementation

`REQ-SOLVE-001` implements the square dense equality path behind GeoRBF-owned
row-major types. The caller must select checked Cholesky or symmetric-pivoted
Bunch--Kaufman LBLT; the recorded requested and actual choices are identical
because factorization fallback is forbidden. Every effective matrix receives
the fixed eight-pass RRQR screen and bounded SVD review above before one
factorization. SVD is diagnostic in this requirement: no pseudoinverse,
minimum-norm solution, or SVD-based solve is exposed.

The factorization uses a symmetric congruence scaling derived from the recorded
row and column equilibration multipliers. The right-hand side and solution are
mapped consistently, and every accepted result is reviewed against the matrix
in original units with the fixed-stack exact-binary accumulator. Refinement
reuses that one factorization, is bounded at eight requested steps, and accepts
a candidate only when its exact original-unit infinity residual strictly
decreases. The final backward-error tolerance is `128*n*epsilon`.

Every dense solve also requires a nonzero explicit memory limit. Before any
nalgebra dispatch, checked `usize` arithmetic computes a conservative peak
payload covering six simultaneous `n*n` scalar matrix buffers, 32 `n`-scalar
vectors, two `n`-entry backend pivot/index-pair buffers, retained diagnostic
objects, and the fixed exact-dot workspace. These terms cover equilibration,
the source and effective matrices, the nalgebra matrix and RRQR clone, the
materialized `R` matrix, bounded-SVD bidiagonal work, Cholesky or LBLT storage,
the LBLT `D` inspection, solves, residuals, and refinement candidates. A field
solve additionally counts the still-live assembled matrix and right-hand side
while the solver-owned copy exists.

Estimate overflow and a peak above the effective limit are structured errors
returned before backend dispatch. `try_solve_field` applies the smaller of the
solver limit and `ExecutionOptions::memory_limit_bytes`, and performs this
check before copying the assembled field matrix. Accepted diagnostics retain
both the estimate and effective limit. The estimate is deliberately
conservative payload accounting rather than a promise about allocator or OS
resident-set overhead.

This requirement exposes `None` and `Explicit(value)` regularization only.
Explicit regularization is validated before use and records both the original
and effective rank decisions, the exact amount applied, and the final residual
against the unmodified matrix. `AutomaticWithin(maximum)` remains a normative
future policy, not a placeholder success path in the current API.
