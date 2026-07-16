# PR #58 Independent Review

- Requirement: REQ-SOLVE-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/57
- Pull request: https://github.com/qingsonger/GeoRBF/pull/58
- Branch: `codex/req-solve-001-dense-equality-solvers`
- Reviewed head: `0b3ae418b0668cfcf343a6357af299b0c1f60219`
- Base head: `66d9796963f57769f0d5c05dc535c3ae19e53d65`
- Review date: 2026-07-16
- Result: one P1 finding; PR remains Draft

## Scope and independence

A fresh read-only project `math_reviewer` received only the bounded
requirement and integrated dependency summaries, Issue #57 acceptance
criteria, the M3 plan, relevant mathematical, solver-policy, backend-ADR, and
benchmark contracts, the complete PR diff, and recorded validation evidence.
It did not inherit implementation reasoning and made no repository or remote
changes.

The reviewer independently checked formulae, signs, dimensions, SPD and
indefinite dispatch, congruence scaling, RRQR and SVD rank decisions,
conditioning, explicit regularization, hard constraints, hidden fallback,
iterative refinement, exact original-unit residuals, allocations, independent
truth, interface dispositions, diagnostics, benchmark routing, and requirement
evidence.

## Finding

### P1-1: backend allocations bypass memory estimation and structured failure

`crates/georbf/src/solver.rs:1071-1091` constructs a nalgebra matrix, clones it
for RRQR, and allocates RRQR and SVD decomposition workspaces.
`crates/georbf/src/solver.rs:1364-1388` allocates another backend matrix for
factorization and a backend vector for each solve. None of these allocations
is preceded by checked peak-working-set arithmetic or an explicit memory-limit
check, and none can return `DenseSolveError::AllocationFailed`.

A caller-supplied finite matrix can therefore fit through the public boundary
and the fallible GeoRBF-owned `Vec` copies, then abort or panic while nalgebra
allocates a matrix, clone, or decomposition workspace. Independently, the live
working set contains multiple `8*n*n`-byte matrices before RRQR, SVD, and
factorization workspace, so the peak is strictly larger than the accepted
input allocation. This violates the `docs/architecture/SOLVER_POLICY.md`
contract that every solve performs memory estimation and the repository rule
that core code does not panic on user input.

Required repair: add checked peak-working-set arithmetic covering
equilibration, backend matrices and clones, decompositions, factorization, and
vectors; enforce an explicit solver memory limit, including propagation of
field execution limits where applicable. Add a regression whose limit lies
between the input size and estimated peak and prove it returns a structured
memory-limit error before backend dispatch, plus an estimate-overflow
regression.

## Independently verified evidence

- The local and remote branch heads matched exact reviewed head
  `0b3ae418b0668cfcf343a6357af299b0c1f60219`; the worktree was clean before
  review evidence was recorded.
- Draft Ubuntu CI run 29467512174 passed formatting, warning-denying workspace
  Clippy, workspace tests, doctests, spike checks, and all 58 requirement
  checks on the exact reviewed head. The Ready-only Windows, Ubuntu, macOS,
  and benchmark-smoke matrix correctly did not run.
- Nine public solver tests and the forced-SVD non-convergence regression
  passed. The runnable example, two-iteration Cholesky/LBLT benchmark smoke,
  all 58 requirement checks, and `git diff --check` also passed locally.
- The reviewer found the symmetric congruence mapping, explicit factorization
  selection, dimension-times-epsilon thresholds, factor-16 ambiguity band,
  regularization recording, exact original-unit residual, and strictly
  decreasing refinement acceptance internally consistent.
- Exact implementation head `0b3ae41` already passed the complete stable-head
  standard gate recorded in the requirement evidence. This Review task changes
  only review and bounded-handoff documentation.

No other P0, P1, P2, or P3 finding was found.

Residual coverage gaps are not additional findings: no production test forces
an accepted refinement step; solve-level scaling covers one global
power-of-two case while independent row scaling reaches only private rank
review; and the exact accumulator regression does not cover cancellation,
underflow rejection, or overflow boundaries.

## Repair response

The subsequent bounded Repair task addressed only P1-1. `DenseSolveOptions`
now requires a nonzero explicit memory limit, and checked arithmetic estimates
the conservative peak before nalgebra dispatch across GeoRBF matrices and
vectors, RRQR and materialized `R`, bounded-SVD work, Cholesky/LBLT and pivot
storage, solves, residuals, and refinement candidates. Estimate overflow and
an insufficient limit are structured `DenseSolveError` variants. Accepted
diagnostics retain the estimate and effective limit.

`DenseFieldSystem` now retains its semantic `ExecutionOptions`.
`try_solve_field` applies the smaller field or solver memory limit, counts the
still-live field matrix and right-hand side, and enforces the limit before the
solver-owned input copy. The adapter's RRQR helper also bounds the lifetime of
the QR and materialized `R` allocations before SVD begins.

New regressions prove that a limit between input storage and estimated peak is
rejected before backend dispatch, an applicable field execution limit is
enforced before the input copy, and estimate overflow is structured. Eleven
public solver tests, all three private solver regressions, the runnable
example, the two-iteration 64-by-64 Cholesky/LBLT benchmark smoke, all 58
requirement checks, and `git diff --check` passed. The complete stable-head
standard gate also passed after the final production, test, registry, and
build-input change.

This is implementation evidence, not an independent finding closure. A fresh
read-only re-review must verify the peak model and confirm that P1-1 is closed
without new P0-P3 findings.

## Disposition

PR #58 remains Draft and REQ-SOLVE-001 remains `implemented`. The bounded
repair response is ready for a fresh independent re-review. Do not begin
REQ-MODEL-001.
