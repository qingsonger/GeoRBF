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

## Independent re-review

A fresh read-only project `math_reviewer` independently reviewed exact PR head
`aa6c134d68ddabd6750220dcca1c158ea81e3bc4` against base
`13b40816e03c10db42dc1e51a9e9be4ed3242870`. It received only the bounded
requirement and dependency summaries, Issue #96 criteria and exclusions,
relevant thickness and architecture contracts, the exact PR and repair diffs,
prior findings and repair evidence, focused tests, benchmark, registry,
handoff, CI, and validation evidence. It inherited no Implement or Repair
reasoning and made no repository or remote change.

- THICK002-REV-001 is closed. Complete measurements use the scale-safe
  Euclidean separation of the two returned original-coordinate intersections.
  The analytic `f(x)=x` regression at `x=1e16` requires stored distance four
  and proves threshold 3.5 creates neither a violation nor a proposal.
- THICK002-REV-002 is closed. The public controlled entry point has a checked
  maximum evaluation budget and pre/post-evaluation cancellation checkpoints.
  Its regression cancels after exactly three evaluations and requires a typed
  cancellation error without a report, later evaluation event, or `Completed`
  event.
- THICK002-REV-003 is closed. The public integration regression compares every
  provenance field on each measurement, violation, and proposed constraint
  with the corresponding input location.
- THICK002-REV-004 is closed. The analytic curved-field regression exercises an
  off-grid discriminant-zero lower tangency and requires `NotFound`, while the
  upper intersection succeeds.

### P2 THICK002-REV-005: sampled validation cannot accept explicit execution options

The public controlled entry point at
`crates/georbf/src/thickness_validation.rs:618` accepts only the request and an
`ExecutionControl`. `validation_progress` then passes
`ExecutionOptions::default()` unconditionally at line 730. A caller therefore
cannot explicitly request one thread, cannot request more than one thread and
receive the required `UnsupportedThreadCount`, and cannot select the
determinism value that progress events must report. The algorithm is currently
serial and deterministic, but its public long-operation interface cannot
express the caller choices required by `docs/architecture/ARCHITECTURE.md`,
lines 185-186 and 216-224.

Required regressions: pass an explicit thread count of two through the public
sampled-validation API and require
`SampledThicknessValidationError::Execution(ExecutionError::UnsupportedThreadCount)`
before any fitted-field evaluation; then pass one thread with determinism false
and require successful unchanged validation whose progress events all report
that exact thread count and determinism selection.

No other P0, P1, P2, or P3 finding was reported.

The parent Review task passed all five public thickness-validation integration
tests, all six module truth/numerical tests, all eight execution-control tests,
both sampled-validation Rustdoc tests, all 58 requirement checks, and the
complete PR diff whitespace check. Draft CI run 29787331468 passed its
configured Ubuntu correctness gate on exact reviewed head `aa6c134`; the
Ready-only Windows, Ubuntu, macOS, and benchmark-smoke matrix correctly did not
run. Exact repair code/test head `6bc6c1d` retains its recorded complete
standard local gate. No unexecuted check is claimed as passed.

This evidence-only Review change updates only this review record and
`docs/progress/CURRENT.md`; it changes no production, test, manifest, schema,
CI, build, API, numerical, registry, or dependency input. PR #97 must remain
Draft and REQ-THICK-002 must remain `implemented`. Open a fresh Repair task
scoped only to THICK002-REV-005, add the independent regressions, implement the
smallest repair, rerun focused and final standard checks, push, and stop for a
fresh independent re-review. Do not begin REQ-PROJECT-001.

## THICK002-REV-005 repair evidence pending fresh independent re-review

The Repair task changed the controlled public sampled-validation entry point
to accept explicit `ExecutionOptions` and pass them unchanged into the shared
serial `ProgressTracker`. The convenience entry point continues to use default
execution metadata. Sampled-thickness search, refinement, distance, quantile,
violation, and proposal mathematics are unchanged.

- The two-thread public regression requires the typed
  `ExecutionError::UnsupportedThreadCount` for sampled validation and an empty
  progress event sequence. Because every fitted-field evaluation is guarded by
  a progress boundary, the empty sequence proves rejection occurs before
  evaluation storage or fitted-field evaluation.
- The one-thread, false-determinism public regression requires the same report
  as convenience validation and checks every event, including the terminal
  event, for exactly one effective worker, false determinism, and the sampled-
  validation operation label.

Focused validation passed all seven public sampled-thickness integration
tests, all eight execution-control integration tests, both sampled-validation
Rustdoc tests, warning-denying all-target/all-feature `georbf` Clippy, and the
complete diff whitespace check. The one-iteration optimized smoke benchmark
retained checksums `32` and `2.0` and measured 2471.60 microseconds.

After the last production/test change, the exact repair code/test head
`438937bd2b2ed715de23e1444a2cf41d71bf44c1` passed the complete standard local
gate: workspace format, warning-denying all-target/all-feature Clippy,
all-feature workspace tests, workspace Rustdoc, all 58 requirement checks, and
the complete diff whitespace check. This following evidence update changes
only this review record and the bounded handoff, so it does not invalidate that
immutable production/test/build-input gate.

PR #97 remains Draft and REQ-THICK-002 remains `implemented`, not `integrated`.
A fresh independent re-review must close THICK002-REV-005 before any Ready
transition, complete Ready-only CI, merge, or integration-state change. Do not
begin REQ-PROJECT-001.

## Final independent re-review

A fresh read-only project `math_reviewer` independently reviewed exact PR head
`10744e8bd8131cb9619b830812043bb98efc75f9` against base
`13b40816e03c10db42dc1e51a9e9be4ed3242870`. It received only the bounded
requirement and dependency summaries, Issue #96 criteria and exclusions, M5
scope, relevant thickness, fitted-model, architecture, and directly applicable
ADR contracts, the complete exact PR and focused repair diffs, prior findings
and repair evidence, tests, benchmark, registry, handoff, CI workflow, and
validation evidence. It inherited no Implement or Repair reasoning and made no
repository or remote change.

- THICK002-REV-005 is closed. The public controlled entry point accepts
  explicit `ExecutionOptions` and passes them unchanged through
  `validation_progress` into the shared `ProgressTracker`.
- `ProgressTracker::try_new` rejects an explicit two-thread request with typed
  `UnsupportedThreadCount` before any progress event. Propagation with `?`
  makes this precede evaluation scratch, report storage, and every fitted-field
  evaluation. The public regression requires that error and an empty event
  sequence.
- An explicit one-thread request with false determinism succeeds with the same
  report as convenience validation. Its public regression requires every
  event, including `Completed`, to retain false determinism, one effective
  worker, and the sampled-validation operation label.
- The fitted-gradient normal orientation, sign-changing bracketing and
  bisection, original-coordinate gradient transformation and units, returned-
  point Euclidean distance, caller-ordered type-7 quantiles, capability and
  center-limit checks, reusable CPD polynomial scratch, provenance, failures,
  violations, and explicit proposal-only boundary remain correct.
- No Hessian capability is claimed or requested. No problem mutation, solver,
  refit, softening, hidden regularization, kernel-classification change, or
  later-requirement behavior is introduced. Interface dispositions, registry
  state, benchmark evidence, and Ready-only three-platform smoke wiring remain
  truthful.

No P0, P1, P2, or P3 finding remains. On exact reviewed head `10744e8`, the
parent Review task passed all seven public sampled-validation integration
tests, all eight execution-control tests, both sampled-validation Rustdoc
tests, all 58 requirement checks, and the complete PR diff whitespace check.
The independent reviewer also passed six mathematical/module tests and the
complete diff whitespace check. Draft CI run 29792829608 passed its configured
Ubuntu correctness gate on the same exact head; the Ready-only matrix correctly
did not run.

This evidence-only change updates only this review record and
`docs/progress/CURRENT.md`; it changes no production, test, manifest, schema,
CI, build, API, numerical, registry, or dependency input. PR #97 may proceed to
Ready CI. REQ-THICK-002 remains `implemented`, not `integrated`, until the
exact Ready evidence head passes the complete Windows, Ubuntu, and macOS
correctness and benchmark-smoke matrix, PR #97 merges exactly once, and the
isolated integration-state change completes.

## Integration evidence

The implementation integration sequence is complete. Exact Ready evidence head
`8ecf212e1a7210567b33bcaedc792c54c3939b64` passed the complete Windows,
Ubuntu, and macOS correctness matrix, every configured backend path, all
benchmark-smoke workloads including `sampled_thickness_validation`, and the
requirement-registry gate in CI run 29793530667. PR #97 then squash-merged
exactly once as `0de6140ea6208e262e7b7506968cf4188833d2e2`, and Issue #96
closed as completed. Post-merge `main` run 29794149484 passed the same complete
three-platform gate on that exact merge commit.

The isolated integration-state change records only the registry, review
evidence, history index, and bounded handoff in pull request #98. After its own
complete local and exact Ready-head CI gates are green and that pull request is
merged, stop. A fresh task must select the next requirement; this task must not
begin it.
