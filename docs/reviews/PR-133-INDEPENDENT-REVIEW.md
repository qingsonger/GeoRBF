# PR #133 Independent Review

- Requirement: REQ-CONTOUR-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/132
- Pull request: https://github.com/qingsonger/GeoRBF/pull/133
- Branch: `codex/req-contour-001-level-points`
- Base head: `a3e89eec038abe5fbb9420e544b84100eaef4f6c`
- Reviewed head: `323fcd9821b775cc4a459f8745bd9b1be6267d2e`
- Stable implementation gate head:
  `b41e48229a3d2f0f8e02d26709dfdf6da9ac86e9`
- Repair implementation head:
  `1280cd2c772d2e049eb1e28203077f711fb16036`
- Draft CI run: 30077398167
- Review date: 2026-07-24
- Repair date: 2026-07-24
- Result: repairs complete; fresh isolated re-review required

## Scope and independence

A fresh isolated read-only project `math_reviewer` received only the bounded
REQ-CONTOUR-001 summary and integrated dependency closure, Issue #132
acceptance criteria and exclusions, the M8 plan, relevant architecture,
model, solver, coordinate-derivative, and level-variable contracts, the
complete base-to-head diff, tests, benchmark evidence, exact validation
results, and Draft CI state. It inherited no Implement reasoning and made no
repository or remote change.

The reviewer independently checked formulas, signs, dimensions and units,
original-coordinate derivative use, root and stationary bracketing,
bracket-preserving bisection, center limits, boundary, tangency, and
degenerate behavior, hard failure semantics, capability checks,
cancellation, progress and work arithmetic, determinism, allocations, hidden
regularization or refitting, interface dispositions, documentation and
registry truth, and benchmark and CI obligations.

## Findings

### P1 CONTOUR001-REV-001: away-from-center gradients can fabricate stationary points

`crates/georbf/src/contour.rs:779` requests gradient scratch but does not reject
`SupportedAwayFromCenters`. Derivative sign changes are treated as stationary
brackets at `crates/georbf/src/contour.rs:920`, and coordinate-width
termination at `crates/georbf/src/contour.rs:1032` can return a sample whose
derivative is not close to zero. The sample is retained unconditionally at
`crates/georbf/src/contour.rs:942` and promoted to an at-level stationary root
when its value residual passes.

For the existing one-center Matérn-1/2 model, the exact field is

```text
f(x) = exp(-abs(x)).
```

Its derivative is positive for `x < 0`, negative for `x > 0`, and undefined at
the center `x = 0`. On domain `[-1, 2]` with one requested scan interval, the
nodes are `[-1, 0.5, 2]`, so no node equals the center. The derivative changes
sign across `[-1, 0.5]`; because zero is at the non-dyadic fraction `2/3` of
that bracket, midpoint bisection never evaluates it exactly. With value
tolerance `1e-6`, coordinate tolerance `1e-8`, derivative tolerance `1e-12`,
and 64 refinement iterations, refinement terminates by coordinate width at
`x != 0`, where `abs(f'(x))` is approximately one. Its value nevertheless lies
within `1e-6` of target level one, so the implementation reports a stationary
root even though the only at-level location is nondifferentiable.

The kernel capability contract distinguishes derivatives available everywhere
from derivatives available only away from centers. Derivative-sign bisection
is justified by the intermediate-value property only on an interval where the
derivative exists throughout. This request must therefore fail explicitly
rather than return false analytic evidence.

Required Repair: reuse the existing center-limited model, request level one on
`[-1, 2]` with the settings above, and require a structured capability or
evaluation failure with no report. The current exact-center failure regression
does not cover an unsampled center inside a derivative bracket.

### P2 CONTOUR001-REV-002: stationary diagnostics include intervals without a sign change

`LevelPointInterval` at `crates/georbf/src/contour.rs:444` and
`stationary_brackets()` at `crates/georbf/src/contour.rs:495` promise
derivative sign-change intervals. However,
`crates/georbf/src/contour.rs:887-892` inserts a
`stationary_node_interval` whenever one node derivative is within tolerance
and one neighbor is outside tolerance. The isolation check at
`crates/georbf/src/contour.rs:1108` never checks the neighboring signs.
`docs/user-guide/LEVEL_POINTS.md:21` repeats the sign-bracket claim.

For exact smooth truth

```text
f(x) = x^3 + epsilon*x
f'(x) = 3*x^2 + epsilon > 0,
```

choose scan nodes `[-1, 0, 1]` and
`epsilon < derivative_tolerance < 3 + epsilon`. The implementation records
`[-1, 1]` as a stationary sign bracket even though both endpoint derivatives
are positive and no stationary point exists. A tolerance-small sampled
derivative can be retained as numerical candidate evidence, but it is not a
derivative sign bracket.

Required Repair: reproduce the exact cubic with the complete CPD cubic
polynomial space and require every reported `stationary_brackets()` interval
to have an endpoint zero or an actual endpoint sign change. If near-zero-node
candidate evidence remains public, distinguish it from sign-bracket evidence
in the API and documentation.

### P3 CONTOUR001-REV-003: transformed truth test does not verify derivative units

The nontrivially normalized regression at
`crates/georbf/tests/contour.rs:184` checks transformed root and stationary
coordinates and the stationary value, but does not check the derivatives
returned by `LevelPoint` or `StationaryLevelPoint`. It would therefore pass if
contour diagnostics accidentally exposed normalized-coordinate derivatives.

For its normalization

```text
x_tilde = (x - 5) / 2
p(x_tilde) = x_tilde^2 + x_tilde - 2,
```

the original-coordinate derivative is

```text
df/dx = (2*x_tilde + 1) / 2.
```

The exact original-coordinate slopes are `-1.5` at `x = 1`, `+1.5` at
`x = 7`, and zero at `x = 4`. The implementation currently delegates through
the correct fitted-model `S^-T` path, but the claimed independent contour
evidence does not guard that contract.

Required Repair: assert those three derivatives directly in the existing
transformed test. A negative normalization scale should additionally cover
reflection and derivative-sign behavior.

No other P0, P1, P2, or P3 finding was identified.

## Repair evidence

Repair head `1280cd2c772d2e049eb1e28203077f711fb16036`
addresses only CONTOUR001-REV-001 through CONTOUR001-REV-003:

- CONTOUR001-REV-001: level-point extraction now requires the fitted gradient
  capability to be `SupportedEverywhere` and returns
  `UnsupportedGradientCapability` before the first fitted-field evaluation for
  `SupportedAwayFromCenters`. The regression uses the retained Matérn-1/2
  center-limited model, level one, domain `[-1, 2]`, one requested scan
  interval, 64 refinements, and the review's exact `1e-6`, `1e-8`, and `1e-12`
  tolerances. The center is not a scan node and no report is returned.
- CONTOUR001-REV-002: a tolerance-small scan-node derivative remains public
  stationary-candidate evidence, but it contributes a diagnostic bracket only
  when its two neighbors actually have opposite derivative signs. An exactly
  zero derivative can instead contribute a zero-width bracket. The new exact
  CPD cubic regression reproduces `x^3 + epsilon*x` with complete degree-three
  polynomial space, `epsilon = 1e-12`, and derivative tolerance `1e-10`; it
  retains the candidate at zero and reports no stationary bracket.
- CONTOUR001-REV-003: the transformed quadratic truth test now asserts
  original-coordinate derivatives `-1.5`, `+1.5`, and zero. A second fit with
  normalization scale `-2` verifies reflected coordinates, root ordering, and
  derivative signs.

Before the production repair, the focused contour test suite reproduced
CONTOUR001-REV-001 and CONTOUR001-REV-002 as two failures. After the repair,
all eight contour integration tests passed. The focused contour Rustdoc
example passed, release benchmark smoke retained checksum
`2.50500000000000000e2`, the 58-requirement registry check passed, and the
complete standard gate passed on the exact repair implementation tree:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features`
- `cargo test --doc --workspace`
- `cargo xtask requirements check`

The later handoff commit changes only this review record and
`docs/progress/CURRENT.md`, so the immutable repair-head gate remains
applicable. PR #133 remains Draft and REQ-CONTOUR-001 remains `in_progress`.
Fresh isolated mathematical/numerical re-review must independently confirm
that all three findings are closed and check for new P0--P3 defects.

## Independent truth and validation

The isolated reviewer inspected all 13 changed paths and independently
reproduced the Matérn-1/2 nondifferentiable-center counterexample, the
positive-derivative cubic diagnostic counterexample, and the transformed
original-coordinate derivative values. It also confirmed that:

- the finite scan remains evidence rather than a global root-count proof;
- isolated-root sorting and tolerance deduplication, exact boundary behavior,
  and constant-field degenerate-interval behavior are consistent for the
  exercised inputs;
- cancellation, unsupported thread count, checked work arithmetic, and
  no-partial-report paths remain explicit;
- no finite difference, hidden regularization, refit, coefficient mutation,
  solver fallback, or adapter-side mathematics was added; and
- Rust exports, later-milestone interface N/A dispositions, Draft/Ready CI
  separation, and the `in_progress` registry state are consistent.

The isolated reviewer and parent Review task independently passed:

- all seven all-feature contour integration tests;
- the focused contour Rustdoc example;
- the release benchmark smoke with checksum
  `2.50500000000000000e2`;
- the 58-requirement registry check; and
- the complete base-to-head whitespace check.

Draft CI run 30077398167 passed its configured Ubuntu correctness job on exact
reviewed head `323fcd9821b775cc4a459f8745bd9b1be6267d2e`. The Ready-only
Windows, Ubuntu, macOS, and benchmark matrix was skipped as designed and is not
claimed. Stable implementation head `b41e482` had already passed the complete
standard local gate after the final production, test, manifest, benchmark, CI,
and registry change. This Review changes only Markdown review and bounded
handoff evidence, so that immutable implementation-head gate remains
applicable.

The full workspace gate and normal 2,000-iteration benchmark were not rerun in
this Review task. `cargo-nextest`, `cargo-deny`, `cargo-audit`,
`cargo-semver-checks`, Miri for pinned Rust 1.96.1, sanitizers, executable
fuzzing, mutation testing, API/ABI/schema snapshot checks, and local
`actionlint` remain unavailable or assigned to later requirements.

PR #133 must remain Draft and REQ-CONTOUR-001 remains `in_progress`. A fresh
Repair task must address only CONTOUR001-REV-001 through
CONTOUR001-REV-003, add the specified regressions, run focused checks and one
complete stable-head standard gate after the last production or test change,
update this record and the bounded handoff, push, and stop for another fresh
independent re-review. This Review does not repair production code, mark the
PR Ready, merge it, or begin REQ-CONTOUR-002.
