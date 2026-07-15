# REQ-SPIKE-001

Evaluated checked dense Cholesky and symmetric-pivoted Bunch--Kaufman LBLT and
accepted ADR-0010, which selects the nalgebra 0.35 release line for later
private production adoption. The decision records correctness, failure
semantics, bounded iterative refinement, maintenance, licenses, MSRV, unsafe
exposure, advisories, dependency and binary size, deterministic performance,
three-platform strategy, and native-LAPACK and handwritten alternatives.

Added an excluded reproducible comparison crate pinned to faer 0.24.4 and
nalgebra 0.35.0. Eight independent cases cover analytic SPD truth; a two-by-two
zero-diagonal symmetric-indefinite system whose inspected Bunch--Kaufman factors
contain the mandatory 2-by-2 pivot block; checked Cholesky and singular-system
rejection; overflowed nonfinite residual evidence; ill-conditioned diagonal
scaling with at most three explicit refinement corrections; deterministic
repetition; and invalid input. Every accepted solution, residual, norm,
denominator, and backward error is finite. An instrumented regression proves
that one factorization is reused for the initial solution and every refinement
correction. CI covers both single-backend paths, the combined path, the negative
empty-backend configuration, and a benchmark smoke workload.

The production workspace gains no new dependency, dense solver, public matrix
type, user API, hidden regularization, diagonal substitution, pseudoinverse, or
fallback. Rust, CLI, C, C++, and Python interface dispositions remain N/A for
this dependency spike. Production adapter integration and user-visible dense
solve behavior belong to later M3 requirements.
