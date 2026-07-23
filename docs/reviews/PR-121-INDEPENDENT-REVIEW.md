# PR #121 Independent Review

- Requirement: REQ-CENTER-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/120
- Pull request: https://github.com/qingsonger/GeoRBF/pull/121
- Branch: `codex/req-center-001-rank-safe-centers`
- Base head: `aa128ed87236c85aa6d310127ad05c70c0a2092b`
- Reviewed head: `63a9f9035ec280124ea0fc230692b3c271436f59`
- Re-reviewed Repair head:
  `75110a549ca4c8033b3a1d6207765e3a1817f349`
- Stable implementation gate head:
  `bf850a8f9a4b673425724e71abc46d955258cd6e`
- Stable Repair gate head:
  `75110a549ca4c8033b3a1d6207765e3a1817f349`
- Draft CI run: 30004560859
- Repair Draft CI run: 30009925065
- Review date: 2026-07-23
- Result: clean fresh re-review; all four findings closed and no P0--P3
  finding remains

## Scope and independence

A fresh read-only project `math_reviewer` received only the bounded
REQ-CENTER-001 summary and integrated dependency closure, Issue #120
acceptance criteria and exclusions, the M7 plan, architecture and solver
policy, relevant mathematical contracts and ADRs, the complete base-to-head
diff, tests, benchmark evidence, and exact validation results. It inherited no
Implement reasoning and made no repository or remote change.

The reviewer independently checked formulae, signs, dimensions, units, SPD
and CPD classification, center and polynomial rank policy, scale-aware
decisions, deterministic seeds and ties, hard failure behavior, allocation and
memory policy, hidden regularization or fallback, interface dispositions,
benchmark validity, and requirement evidence.

## Findings

### P1 CENTER001-REV-001: greedy rank rejection is not scale-aware

`crates/georbf/src/center_selection.rs:756-761` derives one global threshold
from the largest candidate diagonal, and
`crates/georbf/src/center_selection.rs:800-808` rejects every later pivot at or
below that threshold before the authoritative equilibrated RRQR/SVD review.
This conflicts with the rank-invariance contract in
`docs/architecture/SOLVER_POLICY.md:105-110`: an equivalent nonzero scaling
must not change rank classification.

For the exactly representable SPD matrix

```text
K = diag(1, 2^-100),
```

the congruence `D K D` with `D = diag(1, 2^50)` is the identity, so both
representers are independent. Power-greedy selects the unit diagonal first,
then rejects the independent `2^-100` pivot against the implemented
`2 * epsilon` threshold. Residual-greedy reaches the same false
`InsufficientBasisRank` result for a target that selects the unit diagonal
first. The final scale-aware rank review is never reached.

Required Repair: replace the global-diagonal greedy decision with a
scale-aware rule that preserves equivalent nonzero basis scaling without
weakening the final RRQR/SVD and checked-Cholesky review. Add the diagonal
fixture and its congruently rescaled identity counterpart for both residual-
and power-greedy, requiring the same full-rank classification; selection order
need not match.

### P2 CENTER001-REV-002: CPD applicability is neither implemented nor explicitly classified

`CenterSelectionProblem` at
`crates/georbf/src/center_selection.rs:162-188` accepts only locations, a raw
symmetric candidate matrix, and targets. Greedy selection at
`crates/georbf/src/center_selection.rs:296-316` and final review at
`crates/georbf/src/center_selection.rs:902-941` have no CPD order, complete
polynomial action `Q`, null-space, or projected-energy input, and always review
raw `K` through checked Cholesky.

The generic public contract and requirement evidence do not declare an
SPD-only capability or provide a typed CPD rejection. This is incompatible
with the CPD center truth in `docs/math/MATH_SPEC.md:81-116` and
`docs/math/CPD_AND_POLYNOMIALS.md:40-64` and `:85-119`, where rank requires
complete polynomial actions and positivity of `Z^T K Z`.

For the documented order-one signed polyharmonic kernel `k(r) = -r` at D=1
centers zero and one,

```text
K = [[0, -1], [-1, 0]]
Q = [[1], [1]]
Z = (1, -1) / sqrt(2)
```

`Q` has full polynomial rank, `Z^T K Z = 1`, and the augmented CPD system is
nonsingular. The current greedy paths reject the zero first diagonal, while
all-representer, user-provided, and farthest-point paths reject raw `K` through
Cholesky.

Required Repair: add this two-center CPD independent-truth regression and
implement the complete CPD rank/projected-positive selection path. If the
atomic capability is intentionally SPD-only, the minimum acceptable repair is
an explicit public definiteness contract and typed CPD rejection before any
generic pivot, rank, or factorization failure, with all requirement and
architecture claims narrowed consistently.

### P3 CENTER001-REV-003: farthest-point seeded tie behavior lacks required evidence

The farthest-point test at
`crates/georbf/tests/center_selection.rs:85-97` contains no exact distance tie
and runs only once. The seeded repeatability test at
`crates/georbf/tests/center_selection.rs:139-160` covers only residual- and
power-greedy. Issue #120's explicit farthest-point seed and tie criterion is
therefore unproved.

Required Repair: use symmetric D=1 locations `[-1, 0, 1]` with a seed that
selects the middle point first. Assert the documented seeded choice between
the exactly tied endpoints and identical repeated results.

### P3 CENTER001-REV-004: claimed malformed-shape and finiteness tests are absent

The malformed-request test at
`crates/georbf/tests/center_selection.rs:201-231` covers empty input,
asymmetry, duplicate or out-of-range user indices, and excessive count. It
does not exercise the Gram-length, target-length, nonfinite-Gram, or
nonfinite-target validations at
`crates/georbf/src/center_selection.rs:193-217`. The malformed-shape evidence
claim in `changes/REQ-CENTER-001.md:43-47` is therefore overstated.

Required Repair: add table-driven independent assertions for
`GramLengthMismatch`, `TargetLengthMismatch`, `NonFiniteGram`, and
`NonFiniteTarget`, then make the change-fragment claim match the regressions.

No other P0, P1, P2, or P3 finding was identified.

## Read-only validation

The independent reviewer inspected all 12 changed paths, passed the existing
nine-test center-selection executable, ran the release benchmark smoke across
all five strategies, and confirmed a clean complete PR whitespace check.

The parent Review task independently passed:

- `cargo test -p georbf --all-features --test center_selection`;
- the center-selection rustdoc example;
- `cargo bench -p georbf --bench center_selection -- --smoke`;
- `cargo xtask requirements check`; and
- `git diff --check origin/main...HEAD`.

Draft CI run 30004560859 passed its configured Ubuntu correctness job on exact
reviewed head `63a9f90`; the Ready-only Windows, Ubuntu, macOS, and benchmark
matrix was skipped as designed and is not claimed. The worktree remained clean
throughout the independent review.

PR #121 must remain Draft and REQ-CENTER-001 remains `planned`. A fresh Repair
task must address only CENTER001-REV-001 through CENTER001-REV-004, add the
specified regressions, run focused checks and one complete stable-head
standard gate after the last production or test change, update this record and
the bounded handoff, push, and stop for another fresh independent re-review.
This Review does not repair production code, mark the PR ready, merge it, or
begin REQ-TUNE-001.

## Repair evidence

The fresh Repair task addressed only the four recorded findings:

- CENTER001-REV-001: greedy pivot acceptance now uses the selected
  candidate-local threshold `n * epsilon * abs(K_ii)`. The independent
  `diag(1, 2^-100)` fixture and its congruently scaled identity counterpart
  receive the same full-rank classification under both residual- and
  power-greedy selection.
- CENTER001-REV-002: `CenterSelectionProblem::try_from_row_major` now requires
  `KernelDefiniteness` and supports only the explicitly declared SPD path. The
  order-one two-center `k(r) = -r` regression independently verifies
  `Q^T Z = 0` and `Z^T K Z = 1`, then requires the dedicated typed CPD
  capability error before generic pivot, rank, or factorization work.
- CENTER001-REV-003: the symmetric D=1 `[-1, 0, 1]` fixture starts from the
  middle point, fixes the seeded choice between exactly tied endpoints, and
  requires identical repeated selection and diagnostics.
- CENTER001-REV-004: a table-driven regression now requires
  `GramLengthMismatch`, `TargetLengthMismatch`, `NonFiniteGram`, and
  `NonFiniteTarget` for their independent malformed inputs.

After the last production and test change, the Repair task passed the 13-test
center-selection executable, its rustdoc example, the five-strategy release
benchmark smoke, the complete standard workspace gate, the 58-requirement
registry check, and `git diff --check`. The standard gate comprised format,
warning-denying all-target/all-feature Clippy, all-feature workspace tests,
and workspace doctests.

This Repair records closure evidence but is not an independent re-review.
PR #121 remains Draft and REQ-CENTER-001 remains `planned`. A fresh task must
use an isolated `math_reviewer` to confirm the four findings are closed and
check the complete repaired diff for new P0--P3 findings before any Ready,
full-platform CI, merge, or integration-state action.

## Fresh independent re-review

A new isolated read-only project `math_reviewer` independently reviewed exact
Repair head `75110a549ca4c8033b3a1d6207765e3a1817f349` against base
`aa128ed87236c85aa6d310127ad05c70c0a2092b`. It received only the bounded
requirement summary and integrated dependency closure, Issue #120 acceptance
criteria and exclusions, the M7 plan, applicable architecture, solver, and CPD
contracts, the prior findings and Repair evidence, benchmark evidence, and the
complete 13-file base-to-head diff. It inherited no implementation reasoning
and made no repository or remote change.

The re-review closed every prior finding:

- CENTER001-REV-001 is closed. The selected candidate now uses the local
  threshold `n * epsilon * abs(K_ii)`. Under `K -> D K D`, both its Schur pivot
  and threshold scale by `D_ii^2`. The residual- and power-greedy regressions
  require full rank for both `diag(1, 2^-100)` and its congruently scaled
  identity.
- CENTER001-REV-002 is closed. Construction requires an explicit
  `KernelDefiniteness`, and a CPD declaration returns the dedicated typed
  error with its order before candidate, pivot, generic rank, or Cholesky
  work. The order-one `-r` fixture independently verifies `Q^T Z = 0` and
  `Z^T K Z = 1` before requiring that capability rejection.
- CENTER001-REV-003 is closed. The `[-1, 0, 1]` fixture starts at the middle
  candidate, resolves the exactly tied endpoints through the documented seed
  key, and requires identical repeated indices and diagnostics.
- CENTER001-REV-004 is closed. Table-driven independent regressions require
  all four Gram/target length and nonfinite structured errors, and the
  requirement evidence now names only the exercised cases.

The reviewer also checked formulae, signs, dimensions, SPD and CPD
classification, scale-aware decisions, deterministic ties, hard failure
behavior, allocations and explicit memory policy, hidden regularization or
fallback, interface dispositions, benchmark routing, and the complete repaired
diff. It found no new P0, P1, P2, or P3 issue. Every strategy still receives
the authoritative eight-pass equilibration, RRQR, bounded-SVD, and checked
Cholesky review with `Regularization::None`; no jitter, pseudoinverse,
candidate skipping, or factorization fallback exists.

The isolated reviewer and parent Review task independently passed:

- all 13 center-selection integration tests;
- the center-selection rustdoc example;
- the five-strategy release benchmark smoke;
- the 58-requirement registry check; and
- the complete base-to-head whitespace check.

Repair Draft CI run 30009925065 passed the configured Ubuntu correctness job
on exact Repair head `75110a5`; its Ready-only matrix was skipped as designed.
The stable Repair head already passed the complete standard local gate after
the last production and test change. This re-review evidence commit changes
only this review record and the bounded handoff, so that immutable complete
gate remains valid.

The fresh re-review is clean. PR #121 may now enter the mandatory Ready ->
exact-head Windows/Ubuntu/macOS plus benchmark-smoke CI -> single merge ->
isolated truthful integration-state sequence. REQ-CENTER-001 remains `planned`
until that sequence completes.
