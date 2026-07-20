# PR #97 Independent Review

- Requirement: REQ-THICK-002
- Issue: https://github.com/qingsonger/GeoRBF/issues/96
- Pull request: https://github.com/qingsonger/GeoRBF/pull/97
- Branch: `codex/req-thick-002-sampled-validation`
- Reviewed head: `5878055d7ccbc7250aad3e6837f00375161e0052`
- Base head: `13b40816e03c10db42dc1e51a9e9be4ed3242870`
- Review date: 2026-07-21
- Result: one P1, one P2, and two P3 findings; Repair required

## Scope and independence

A fresh read-only project `math_reviewer` received only the bounded
REQ-THICK-002 summary and integrated dependency closure, Issue #96 acceptance
criteria and exclusions, the M5 plan, relevant thickness, fitted-model,
architecture, and ADR contracts, the exact PR diff, tests, benchmark, registry
state, handoff, CI state, and validation evidence. It inherited no Implement
reasoning and made no repository or remote change.

The reviewer independently checked normal orientation, bracketing and
bisection behavior, dimensions and original-coordinate units, curved and
tangential cases, type-7 quantiles, finite arithmetic, low-gradient handling,
distance semantics, hard-constraint separation, proposal/refit boundaries,
D=1/D=2/D=3 bounds, immutability, allocations, determinism, provenance,
structured errors, capabilities, interface dispositions, benchmark and CI
wiring, and requirement truthfulness.

## Findings

### P1 THICK002-REV-001: nominal search parameters can misreport Euclidean thickness

`crates/georbf/src/thickness_validation.rs:731`, lines 843-852, and lines
964-966 evaluate and retain two different quantities. `point_along` evaluates
a rounded floating-point `Point`, while a successful search retains the
unrounded nominal parameter `distance`; the report then sums the two nominal
parameters instead of measuring the returned intersections.

For the finite analytic D=1 field `f(x)=x`, select `x=1e16`, lower and upper
values `1e16 - 2` and `1e16 + 2`, maximum search distance two, and four uniform
steps. At nominal `t=1.5`, IEEE arithmetic rounds `x - t` and `x + t` to the
target points two units away. Both targets are accepted, but the report returns
the nominal span `1.5 + 1.5 = 3`; the Euclidean separation of the stored
intersections is four. A requested minimum of `3.5` consequently creates a
false violation and may create a false proposed hard local constraint. This
contradicts the original-coordinate Euclidean-distance contract in
`docs/math/THICKNESS.md:99-105`.

Required regression: fit or independently evaluate `f(x)=x` at `x=1e16` and
require the reported distance to equal the Euclidean separation of the returned
intersections, or require excessive coordinate-rounding distortion to return a
structured error. Also prove that a threshold between the nominal and actual
spans produces no false violation or proposal.

### P2 THICK002-REV-002: the caller-unbounded validation operation cannot be cancelled

The only public entry point at
`crates/georbf/src/thickness_validation.rs:598-601` accepts no
`ExecutionControl`. The location loop at line 673, bracketing loop at line 841,
and refinement loop at line 900 contain no cancellation checkpoint. Location
count is caller-controlled, and both nonzero `u32` iteration limits permit more
than four billion iterations. A valid request can therefore monopolize a
caller without a typed cancellation result, contrary to the long-operation
contract in `docs/architecture/ARCHITECTURE.md:185-212`.

Required regression: add a controlled validation entry point, trigger
cancellation after a deterministic evaluation count, and require a typed
cancellation error, no partial report, and no evaluations after the checkpoint.

### P3 THICK002-REV-003: claimed provenance regression evidence is absent

`changes/REQ-THICK-002.md:37-41` claims independent tests cover violation and
proposal provenance. The integration test at
`crates/georbf/tests/thickness_validation.rs:141-147` checks only violation and
proposal counts and measurement geometry. Repeat report equality proves
determinism, not correct provenance propagation. The implementation appears to
clone provenance correctly, but the recorded evidence is overstated.

Required regression: assert that the stable observation identifier, source
location, original unit, semantic path, and note on a measurement, violation,
and proposed constraint exactly match the corresponding input location.

### P3 THICK002-REV-004: documented tangential-contact behavior lacks a regression

`docs/math/THICKNESS.md:93-98` explicitly promises that a tangential root is
reported as not found unless a uniform sample lands within value tolerance.
The search logic at `crates/georbf/src/thickness_validation.rs:852-855`
implements that rule, but the curved-field test beginning at line 1398 never
exercises a discriminant-zero tangential intersection.

Required regression: use the analytic tangent case for the curved field at
sample `x=0.5`, select a grid that does not land on the tangent root, and require
`ThicknessIntersectionFailure::NotFound`. Optionally pair it with a grid or
tolerance hit that proves deterministic acceptance.

No other P0, P1, P2, or P3 finding was reported.

## Independent mathematical review

For a usable fitted gradient, `n = grad f / ||grad f||` has positive
directional derivative `grad f . n = ||grad f||`. Searching the lower value
along `-n` and the upper value along `+n` therefore has the correct local
orientation. The sign-changing bracket invariant and bisection side updates
are correct. Type-7 quantiles correctly use rank `q(n - 1)` and preserve caller
probability order.

The fitted model supplies original-coordinate gradients through its affine
normalization chain rule. The validator requires no Hessian and makes none
unconditionally available. Diagnostic labels keep scalar gaps, hard sampled
local cones, and sampled geometric evidence distinct. Proposed constraints are
returned values only; the implementation does not mutate a problem, solve,
soften, regularize, or refit. Compile-time bounds restrict public validation
types to D=1, D=2, and D=3, and no new SPD/CPD, polynomial, rank,
infeasibility, persistence, adapter, or global-distance claim is introduced.

## Validation and disposition

- Local and remote branch heads matched exact reviewed head `5878055d`; the
  worktree was clean before this evidence-only Review change.
- Draft CI run 29758953857 passed its configured Ubuntu correctness gate on
  exact reviewed head `5878055d`. The Ready-only Windows, Ubuntu, macOS, and
  benchmark-smoke matrix was skipped as designed and is not claimed as passed.
- The independent reviewer passed four focused integration tests, four module
  truth/numerical tests, and `git diff --check`. It independently reproduced
  the P1 case with nominal span three and returned-point Euclidean span four.
- The parent Review task independently passed the same four integration tests,
  four module tests, both thickness-validation Rustdoc examples, all 58
  requirement checks, and the complete PR diff whitespace check. It separately
  reproduced the P1 IEEE result.
- Exact implementation head `5878055d` retains the complete standard local gate
  recorded by the Implement task: workspace format, warning-denying
  all-target/all-feature Clippy, all-feature workspace tests, workspace
  Rustdoc, all 58 requirement checks, and `git diff --check`.
- `cargo-nextest`, `cargo-deny`, `cargo-audit`, `cargo-semver-checks`, Miri,
  sanitizers, executable fuzzing, mutation testing, general allocation
  instrumentation, API/ABI/schema snapshots, and local `actionlint` remain
  unavailable or deferred exactly as recorded in the bounded handoff. No
  unexecuted check is claimed as passed.
- This Review changes only this evidence record and the bounded handoff. It
  changes no production, test, manifest, schema, CI, build input, API,
  numerical behavior, registry, or dependency input.
- The final staged evidence diff passed `git diff --cached --check` and contains
  only this review record and `docs/progress/CURRENT.md`.

PR #97 must remain Draft and REQ-THICK-002 must remain `implemented`. Open a
fresh Repair task scoped only to THICK002-REV-001 through THICK002-REV-004. Add
the independent regressions, implement the smallest repairs, rerun focused and
final standard checks after the last code change, update this review evidence
and the bounded handoff, commit, push, and stop for a fresh independent
re-review. Do not begin REQ-PROJECT-001 or another requirement.

## Repair evidence pending fresh independent re-review

Repair code, tests, and normative documentation are committed at
`6bc6c1dc53bdd093110858cbf5d0787e97c702e9`. This section records the Repair
task's evidence only; it does not independently close the findings.

- THICK002-REV-001: complete measurements now compute the scale-safe Euclidean
  separation of the returned original-coordinate intersections. An analytic
  `f(x)=x` regression at `x=1e16` reproduces nominal span three versus stored
  span four and proves a threshold of 3.5 creates neither a violation nor a
  proposed constraint.
- THICK002-REV-002: `try_validate_sampled_thickness_with_control` accepts one
  borrowed `ExecutionControl`. A checked maximum evaluation budget drives
  typed deterministic progress, cancellation is checked before and after each
  fitted-field evaluation, and cancellation returns
  `SampledThicknessValidationError::Execution` with no partial report. The
  public regression cancels exactly after evaluation three and observes no
  later evaluation or `Completed` event.
- THICK002-REV-003: the fitted parallel-level integration test now compares
  observation identifier, source path and line, original unit, semantic path,
  and optional note on every measurement, violation, and proposal against its
  input location.
- THICK002-REV-004: the analytic curved field at `x=0.5` now exercises the
  discriminant-zero lower contact on a four-step grid that misses the tangent
  root and requires `ThicknessIntersectionFailure::NotFound` while the upper
  intersection succeeds.

Focused validation passed five public thickness-validation integration tests,
six module truth/numerical tests, eight execution-control integration tests,
both sampled-validation Rustdoc tests, warning-denying all-target/all-feature
Clippy for `georbf`, and the complete diff whitespace check. The repaired
optimized benchmark measured 2299.12 microseconds per 32-location validation
with unchanged checksums `16000` and `1000.0`; smoke measured 1793.80
microseconds with unchanged checksums `32` and `2.0`.

After the final production/test change, exact code/test head `6bc6c1d` passed
the complete standard local gate: workspace format, warning-denying
all-target/all-feature Clippy, all-feature workspace tests, workspace Rustdoc,
all 58 requirement checks, and `git diff --check`. This following evidence
update changes only this review record and the bounded handoff, so it does not
invalidate that immutable-head gate.

PR #97 remains Draft and REQ-THICK-002 remains `implemented`, not `integrated`.
A fresh Review task must independently re-review the repaired head before any
Ready transition, complete three-platform/benchmark-smoke CI, merge, or
integration-state change. Do not begin REQ-PROJECT-001.
