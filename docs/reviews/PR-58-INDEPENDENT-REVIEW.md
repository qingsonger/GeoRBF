# PR #58 Independent Review

- Requirement: REQ-SOLVE-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/57
- Pull request: https://github.com/qingsonger/GeoRBF/pull/58
- Branch: `codex/req-solve-001-dense-equality-solvers`
- Reviewed head: `0b3ae418b0668cfcf343a6357af299b0c1f60219`
- Re-reviewed repair head: `9361de7c4215eb8b663e24aec35165e0ee5ffad0`
- Clean re-reviewed repair head: `56397625cca96252d9fce171bceedb5f1e8e9fda`
- Base head: `66d9796963f57769f0d5c05dc535c3ae19e53d65`
- Review date: 2026-07-16
- Result: clean second re-review; P1-1 is closed and no P0-P3 finding remains

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
repair response required the fresh independent re-review recorded below. Do
not begin REQ-MODEL-001.

## Fresh independent re-review

A fresh read-only project reviewer inspected exact repair head `9361de7` using
only the bounded requirement and dependency summaries, Issue #57 acceptance
criteria, scoped normative documents, the complete PR and repair diffs, and
recorded validation evidence. It independently reran all eleven public solver
tests, all three private solver regressions, the runnable example, the
two-iteration 64-by-64 Cholesky/LBLT benchmark smoke, all 58 requirement
checks, and `git diff --check`; all passed and the worktree remained clean.
Draft Ubuntu CI run 29469494039 also passed on that exact head.

The reviewer confirmed that the conservative six-matrix peak covers the
worst-case RRQR live set, the field path separately counts the assembled
matrix and right-hand side, the SVD and factorization paths fit within the
matrix/vector/index budgets, and checked estimate arithmetic and the directly
used solve paths reject before nalgebra dispatch. It also independently
verified the SPD and mandatory-2-by-2-pivot indefinite truth cases and found no
hidden regularization, factorization fallback, pseudoinverse, or changed
factorization during refinement.

### P1-1 remains open: public field conversion bypasses its memory limit

`DenseEqualitySystem::try_from_field` remains public at
`crates/georbf/src/solver.rs:324-332`. It copies the assembled matrix and
right-hand side at lines 330-331 without checking a peak estimate or the
field's `ExecutionOptions`, and the returned solver-owned system does not
retain that field memory limit. A caller can therefore bypass the checked
`try_solve_field` path at lines 977-998, perform the solver-owned copy despite
an insufficient field limit, and later solve using options that have lost the
field limit. The regression at `crates/georbf/tests/solver.rs:161-176` covers
only `try_solve_field` and does not exercise this public bypass.

Required repair: make the unchecked conversion private so the checked field
solve is the only public boundary, or make the conversion retain and enforce
the effective field/solver limit before copying. If the public conversion is
retained, add a direct regression using a one-byte field limit that proves a
structured pre-copy memory failure and proves the limit cannot be lost before
solve. If it becomes private, add appropriate API or compile-fail coverage
showing callers cannot bypass the checked boundary.

No other P0, P1, P2, or P3 finding was found. PR #58 must remain Draft for a
new bounded Repair task addressing only this remaining P1-1 path, followed by
another fresh independent re-review.

## Second bounded repair response

The next Repair task addressed only the remaining public conversion bypass.
`DenseEqualitySystem::try_from_field` is now private, while public
`try_solve_field` remains the sole assembled-field solver boundary and
continues to enforce the smaller retained field or explicit solver memory
limit before the solver-owned copy.

A new compile-fail Rustdoc regression references the former public conversion
directly. It compiled, and therefore failed as expected, before the visibility
repair; after the repair it fails to compile at the external API boundary and
the regression passes. The existing assembled-field truth case and one-byte
field-limit pre-copy rejection also pass unchanged.

After the final production and Rustdoc change, the stable repair worktree
passed formatting, warning-denying workspace Clippy, all-feature workspace
tests, workspace doctests, and all 58 requirement checks. The subsequent
review-record and bounded-handoff validation notes are Markdown-only evidence
updates and do not change production, test, manifest, schema, or build inputs.

This is repair evidence, not independent finding closure. PR #58 remains
Draft, REQ-SOLVE-001 remains `implemented`, and a fresh read-only re-review
must confirm P1-1 is closed without new P0-P3 findings before any Ready or
integration action.

## Second fresh independent re-review

A fresh read-only project `math_reviewer` independently inspected exact repair
head `56397625cca96252d9fce171bceedb5f1e8e9fda` against base
`66d9796963f57769f0d5c05dc535c3ae19e53d65`. It received only the bounded
requirement and dependency summaries, Issue #57 acceptance criteria, the M3
plan, scoped normative documents, complete PR and repair diffs, and recorded
validation evidence. It made no repository or remote changes.

P1-1 is closed. `DenseEqualitySystem::try_from_field` is private at
`crates/georbf/src/solver.rs:335`, while public `try_solve_field` selects the
smaller retained field or explicit solver memory limit, includes the still-live
field matrix and right-hand side in the peak estimate, and enforces that limit
before invoking the private copy at lines 995-1008. The compile-fail Rustdoc at
lines 13-17 passed and proves an external caller cannot use the former direct
conversion. The focused repair changed no numerical path or memory estimate.

The reviewer also checked the complete PR for formulae, signs, dimensions,
SPD and indefinite selection, congruence scaling, rank and condition policy,
hard constraints, explicit regularization, hidden fallback or pseudoinverse,
unchanged-factorization refinement, exact original-unit residuals,
allocations, diagnostics, interface dispositions, and benchmark routing. It
found no P0, P1, P2, or P3 issue.

Independent bounded validation passed all eleven public solver tests, all
three private solver regressions, the complete workspace Rustdoc set including
the new compile-fail case, the runnable example, the two-iteration 64-by-64
Cholesky/LBLT benchmark smoke, all 58 requirement checks, and
`git diff --check`. Draft Ubuntu CI run 29470504173 also passed on exact
re-reviewed head `5639762`; the Ready-only Windows, Ubuntu, macOS, and
benchmark-smoke matrix has not run.

Residual coverage gaps remain non-findings: no production regression forces
an accepted refinement correction; independent row scaling reaches the
private rank review rather than the public symmetric solve; exact-accumulator
tests do not exhaust cancellation, underflow rejection, or overflow
boundaries; and memory accounting is conservative payload estimation rather
than allocator or operating-system resident-set instrumentation.

PR #58 remains Draft and REQ-SOLVE-001 remains `implemented`. A fresh Review
task may now perform the mandatory integration sequence: mark the PR Ready,
wait for the complete three-platform and benchmark-smoke CI on that exact
ready head, merge exactly once only if it is green, and record truthful
integration state. Do not begin REQ-MODEL-001 in this task.

## Integration result

The fresh integration task confirmed that post-review commit
`1ca3c171307f3abf250b30e06e321bf96d1dc9dd` changes only this review record
and the bounded handoff relative to independently reviewed repair head
`56397625cca96252d9fce171bceedb5f1e8e9fda`. It changes no production code,
test, manifest, schema, CI, benchmark input, or dependency.

Exact Ready head `1ca3c171307f3abf250b30e06e321bf96d1dc9dd` passed the complete Windows,
Ubuntu, and macOS correctness matrix with every benchmark-smoke workload in CI
run 29474665806. PR #58 then squash-merged exactly once as
`3ccf7842b6e3cf4ef3fd783051b237847ea4c501`, and Issue #57 closed as completed.
Post-merge `main` run 29475146095 passed the same complete three-platform
correctness, benchmark-smoke, and requirement-registry gate on that exact
merge commit.

The isolated integration-state change updates only the registry, this review
evidence, the history index, and the bounded handoff. It changes no production
code, tests, manifests, schemas, CI, build inputs, APIs, numerical behavior,
dependencies, tags, or releases. REQ-SOLVE-001 may therefore be recorded as
`integrated`; REQ-MODEL-001 must begin only in a fresh task after the isolated
integration-state pull request is green and merged.
