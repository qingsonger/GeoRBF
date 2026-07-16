# REQ-SOLVE-001

Added a public GeoRBF-owned dense equality boundary and explicit numerical
policy. Callers select checked Cholesky for SPD systems or symmetric-pivoted
Bunch--Kaufman LBLT for symmetric-indefinite systems. Nalgebra 0.35.0 remains
private, exactly pinned, and unchanged from the accepted CPD dependency graph;
no third-party matrix or factorization type crosses the public API.

Every input is checked for nonempty square shape, finiteness, and exact
symmetry, so the private symmetric factorization never changes an accepted
matrix triangle. Each original and effective matrix receives deterministic eight-pass
row/column equilibration, column-pivoted QR screening, bounded SVD review,
factor-16 ambiguity detection, matrix norms, effective-rank classification,
and a dimensionless condition estimate. Deficiency, disagreement, threshold
adjacency, SVD non-convergence, a configured condition limit, unrepresentable
scaling, checked-Cholesky rejection, and LBLT zero pivots are structured
failures. Completed RRQR evidence survives forced SVD non-convergence.

`None` preserves the matrix exactly. `Explicit(value)` validates and records a
positive finite diagonal amount, then retains both original and effective rank
evidence and the final residual against the unmodified matrix. Automatic
regularization, jitter, substitute diagonals, factorization switching,
pseudoinverse, minimum-norm solving, and constraint relaxation are absent.

Factorization uses a symmetric congruence scaling derived from the rank-review
multipliers. Bounded iterative refinement reuses the unchanged factorization
and accepts only corrections that strictly decrease a fixed-stack exact-binary
original-unit residual. Diagnostics record requested and actual factorization,
regularization, condition warning, 2-by-2 pivot use, initial/final scaled and
original-unit residuals, accepted corrections, and the `128*n*epsilon` residual
tolerance.

Independent tests cover analytic SPD truth, a mandatory 2-by-2 indefinite
pivot, wrong-Cholesky rejection, exact rank failure, uniform unit scaling,
independent row-scale rank invariance, condition warning and rejection,
exact-binary residual roundoff, explicit regularization, malformed input,
forced SVD non-convergence, and the assembled `DenseFieldSystem<D>` boundary.
Rustdoc, a runnable example, a deterministic 64-by-64 Cholesky/LBLT benchmark,
three-platform benchmark smoke routing, production dependency re-audit,
registry, and bounded handoff are synchronized.

CLI, C, C++, and Python are N/A because `REQ-MODEL-001` has not introduced an
immutable fitted model and later schema/API-freeze requirements have not
defined stable user-facing fitting inputs or outputs. All numerical work
remains in the Rust core.
