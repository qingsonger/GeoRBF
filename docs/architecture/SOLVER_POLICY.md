# Solver and Numerical Policy

Production backends are selected only after the recorded spikes:

- SPIKE-001: dense SPD and symmetric-indefinite factorization;
- SPIKE-002: rank-revealing QR and SVD;
- SPIKE-003: compact sparse assembly and solution; and
- SPIKE-004: canonical QP and SOCP mapping.

REQ-CPD-001 adopts nalgebra 0.35.0 behind the private CPD numerical adapter for
column-pivoted QR screening, bounded SVD rank review, and QR-supported
null-space construction. GeoRBF-owned row-major matrices and diagnostics are
the only public types. The adapter does not expose or call pseudoinverse or
minimum-norm solve APIs.

Each spike reviews correctness, scaling, maintenance, license, MSRV, unsafe
use, platforms, binary size, alternatives, and deterministic behavior, then
produces an ADR before dependency lock-in.

Target dispatch is Cholesky for SPD equality systems, pivoted LDLT for CPD KKT
systems, RRQR for rank risk, SVD for rank review or justified minimum-norm
work, projected weighted least squares for L2 losses, a mature convex QP backend
for linear inequalities and epigraph losses, and a mature SOCP backend for
angular and thickness cones.

Every solve performs finite and unit checks, coordinate and direction
normalization, explicit constraint scaling, duplicate/conflict review,
polynomial rank and kernel capability checks, anisotropy validation, memory
estimation, symmetry checks, conditioning estimation, iterative refinement, and
residual review in original units.

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
automatic solver adjustment.

Regularization is None, Explicit(value), or AutomaticWithin(maximum). Any
automatic choice records requested and actual solver, amount added, original
and effective rank, condition estimates, and trigger. No pseudoinverse, jitter,
or fallback may hide failure.
