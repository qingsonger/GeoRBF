# PR #49 Independent Review

- Requirement: REQ-SPIKE-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/48
- Pull request: https://github.com/qingsonger/GeoRBF/pull/49
- Branch: `codex/req-spike-001-dense-factorization`
- Original reviewed head: `b194061163e3e15add68c044a9ed040b23f3bdd8`
- Re-reviewed repair head: `7b9226e656eddbafbc6f5f17e7726fc3f8d4c770`
- Base head: `8fee4315f7335c48d919cc5f04a217e6db829a07`
- Review date: 2026-07-15
- Result: original P1-1 through P1-3 closed; new P3-1 finding; PR must
  remain Draft
- Repair status: fresh Repair required for P3-1 only

## Scope and independence

A fresh read-only `math_reviewer` received only the bounded requirement and
dependency summaries, Issue #48 acceptance criteria, the milestone and solver
policies, ADR-0010, the complete PR diff, and the recorded validation and
benchmark evidence. It did not inherit the implementation reasoning and made
no repository or remote changes.

The reviewer independently checked the SPD and indefinite truth cases,
factorization and failure semantics, original-unit backward error, bounded
iterative refinement, hidden regularization and fallback exclusions,
determinism, dependency and interface isolation, benchmark evidence, CI
coverage, and requirement state. Three P1 findings block the ready-head
integration sequence. No additional P0, P2, or P3 finding was identified.

## Findings

### P1-1: nonfinite residual evidence can pass residual review

`spikes/factorization-backends/src/main.rs:155`, `:170`, and `:197` use
`f64::max` to fold the residual norm, do not require the residual entries,
norms, denominator, or backward error to be finite, and reject only when
`backward_error > 1e-8`. A NaN comparison is false, so this path can report
success without a finite original-unit residual review.

For the finite one-by-one input `A = [f64::MAX]`, `b = [0]` and finite
candidate `x = [2]`, floating evaluation gives `A*x = +infinity`, an infinite
residual norm and denominator, and therefore a NaN backward error. This is an
explicit counterexample to the ADR and solver-policy rule that nonfinite solve
or residual evidence must fail.

Required repair: return an error when any residual entry, residual norm,
matrix/vector norm, denominator, or backward error is nonfinite. Add the
one-by-one counterexample as an independent regression and require
`residual_metrics(&case, &[2.0])` to fail.

Repair implementation: residual review now rejects every nonfinite residual
entry, norm, denominator, and backward error. The finite-input one-by-one
overflow counterexample is an independent regression.

### P1-2: refinement reconstructs the factorization for every correction

`spikes/factorization-backends/src/main.rs:251` creates the initial solve, but
line 264 sends each correction through `solve` again. The backend paths at
lines 213 and 232 then reconstruct the matrix factorization. The experiment
keeps the matrix values and factorization kind unchanged, but it does not reuse
the same factors as required by `docs/architecture/SOLVER_POLICY.md:41` and
`docs/adr/ADR-0010-nalgebra-dense-factorization-backend.md:44`.

Standard iterative refinement factorizes once and repeatedly solves
`A * delta_k = b - A * x_k` with those same factors before accepting
`x_(k+1) = x_k + delta_k`. Re-factorizing also makes the benchmark charge each
accepted correction for a new factorization, so the recorded experiment does
not measure the stated policy.

Required repair: separate factor construction from right-hand-side solves and
reuse one factorization for the initial solution and all zero-to-three
corrections. Add instrumented regression evidence that one `refine` call
constructs exactly one factorization.

Repair implementation: backend factor construction is separate from
right-hand-side solves, and `refine` owns one factorization for its initial and
correction solves. An injected factory and solve counter proves one
factorization and multiple solves during an accepted correction.

### P1-3: the claimed mandatory 2-by-2 pivot case permits all 1-by-1 pivots

`spikes/factorization-backends/src/main.rs:452-459` uses

```text
[ 0  2  0 ]
[ 2  0  1 ]
[ 0  1 -3 ]
```

The matrix is symmetric, nonsingular, and indefinite, but it does not require
a 2-by-2 pivot. After the symmetric permutation `(3, 2, 1)`, an LDLT
factorization can use the three nonzero 1-by-1 pivots `-3`, `1/3`, and `-12`.
The test checks only the analytic solution and Cholesky rejection; it never
establishes a 2-by-2 block. The ADR, README, and change fragment therefore
overstate the evidence.

Required repair: use the two-by-two matrix `[[0, 2], [2, 0]]`. It has
eigenvalues `-2` and `2`, determinant `-4`, and zero diagonal under every
symmetric permutation, so a symmetric LDLT first step must use a 2-by-2
block. Verify its analytic solution and checked-Cholesky rejection for both
backends, and inspect the block structure as well if the candidate API exposes
it.

Repair implementation: the truth case is now exactly `[[0, 2], [2, 0]]`, with
analytic solution and checked-Cholesky rejection verified for both backends.
The test inspects faer's block subdiagonal and nalgebra's block-diagonal `D` and
requires each to expose the nonzero 2-by-2 pivot block.

## Independently verified evidence

- The reviewed branch and remote PR head both equal `b194061163e3e15add68c044a9ed040b23f3bdd8`.
- The complete 13-file diff from `origin/main` was reviewed, including the
  510-line harness, exact lockfile, CI, manifests, ADR, benchmark record,
  registry, and bounded handoff.
- The analytic SPD matrix is positive definite. The indefinite truth matrix is
  symmetric, nonsingular, and indefinite, and the scaled ill-conditioned SPD
  construction preserves positive definiteness by positive diagonal
  congruence.
- The normwise backward-error formula is the standard original-unit form when
  every intermediate value is finite.
- No jitter, pseudoinverse, factorization fallback, implicit regularization,
  production dependency, or user-interface exposure was found.
- Draft CI run 29400346664 passed on the exact reviewed head. The full
  Windows, Ubuntu, and macOS ready-head matrix was correctly skipped while the
  PR remained Draft.

## Disposition

The bounded Repair task implemented only P1-1, P1-2, and P1-3 and added the
specified regressions. Keep PR #49 Draft and stop for a fresh independent
re-review; this repair record is implementation evidence, not an independent
finding closure. Do not begin REQ-IR-001.

The complete three-platform and benchmark-smoke ready-head gate remains an
integration requirement, not a finding. External maintenance, license, unsafe,
and advisory evidence was reviewed only through the bounded repository record
and exact lockfile; unavailable audit tools were not claimed as executed.

## Repair validation

- Combined, faer-only, and nalgebra-only configurations each passed 8/8 tests;
  the no-backend configuration failed with the required compile error, and
  warning-denying all-target all-feature spike Clippy passed.
- The optimized smoke workload and three consecutive complete 32/64/128
  workloads passed. Repaired timing ranges and single-backend binary sizes are
  recorded in `docs/benchmarks/REQ-SPIKE-001.md`.
- The stable repair code/test head passed the complete standard workspace gate:
  formatting, warning-denying workspace Clippy, all-feature workspace tests,
  workspace doc tests, and all 58 requirement checks. `git diff --check` also
  passed. The subsequent review-record and handoff edits are evidence-only and
  change no code, tests, manifests, schemas, or build inputs.

## Fresh independent re-review of the repair

A new read-only `math_reviewer` received the bounded requirement summary and
dependency closure, normative documents, complete 14-file diff, original P1
findings, repair validation, benchmark evidence, and exact repair head. It did
not inherit the Repair reasoning and made no repository or remote changes.

The reviewer confirmed that P1-1 through P1-3 are closed on
`7b9226e656eddbafbc6f5f17e7726fc3f8d4c770`:

- residual entries, norms, denominator, and backward error are all required to
  be finite, and the finite-input overflow counterexample is a regression;
- one factor object supplies the initial solve and every bounded refinement
  correction, while instrumentation observes one factorization and multiple
  solves during an accepted correction; and
- both backends solve the analytic `[[0, 2], [2, 0]]` truth case, reject its
  checked-Cholesky path, and expose the required nonzero 2-by-2 pivot block.

### P3-1: ADR test count is stale

`docs/adr/ADR-0010-nalgebra-dense-factorization-backend.md` says that all six
independent harness tests passed. The repaired harness contains eight tests,
and the repair evidence correctly records 8/8. The accepted ADR therefore has
a stale, directly verifiable evidence count.

Required repair: change only the ADR evidence count from six to eight, update
the bounded review and handoff evidence, and stop for another fresh independent
re-review. Do not change production or test code and do not begin REQ-IR-001.

No new P0, P1, or P2 finding was identified. The reviewer also found no other
P3 issue in the formulae, factorization semantics, truth cases, original-unit
residual review, refinement policy, hidden-adjustment exclusions, determinism,
benchmark record, dependency isolation, interfaces, CI, or requirement state.

### Re-review validation and disposition

- Combined, faer-only, and nalgebra-only configurations each passed all eight
  tests; warning-denying all-feature spike Clippy and spike formatting passed.
- The no-backend configuration failed with the required compile error, and the
  optimized smoke workload passed with finite results.
- All 58 requirement checks and `git diff --check` passed. The spike remained
  workspace-excluded with exact candidate versions, and the root lockfile was
  unchanged.
- Draft CI run 29402438886 passed on the exact repair head. The ready-only
  Windows, Ubuntu, macOS, and benchmark-smoke matrix correctly remained
  unexecuted while the PR was Draft.

The re-review result is one P3 finding. Keep PR #49 Draft and stop for the
bounded documentation-only Repair. No complete workspace gate was rerun by the
reviewer; the unchanged repair head retains the complete local gate recorded
above, and the exact head passed Draft CI.
