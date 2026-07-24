# PR #136 Independent Review

- Requirement: REQ-CONTOUR-002
- Issue: https://github.com/qingsonger/GeoRBF/issues/135
- Pull request: https://github.com/qingsonger/GeoRBF/pull/136
- Branch: `codex/req-contour-002-isolines`
- Base head: `6e622c708be8a7d030213f29fc42ef4f9ce256ef`
- Reviewed head: `2b93db4c7efce551601e45836ec43ff4a3c7f622`
- Fresh re-reviewed head:
  `98a5572001457ce8cedaac6180c9be2f43cb5900`
- Stable implementation gate head:
  `4e766af`
- First repair implementation and standard-gate head:
  `9510b6c`
- CONTOUR002-REREV-006 repair implementation and standard-gate head:
  `6dee8e73487b94b4c79a9d8b11206159814e8ba0`
- Draft CI runs: 30095419189, 30097440898
- Review and re-review date: 2026-07-24
- Repair date: 2026-07-24
- Result: original five findings closed; P2 CONTOUR002-REREV-006 repaired on
  `6dee8e7` and awaiting fresh independent re-review

## Scope and independence

A fresh isolated read-only project `math_reviewer` received only the bounded
REQ-CONTOUR-002 summary and integrated dependency closure, Issue #135
acceptance criteria and exclusions, the M8 plan, relevant architecture and
ADRs, the complete base-to-head diff, tests, benchmark evidence, exact
validation results, and Draft CI state. It inherited no Implement reasoning
and made no repository or remote change.

The reviewer independently checked formulae, signs, dimensions and units,
the asymptotic decider and tie rule, bracket invariants and endpoint limits,
scale-aware decisions, degeneracy and no-partial-report behavior,
original-coordinate behavior, CPD/SPD interactions, topology reconstruction,
determinism, overflow and allocation handling, cancellation and progress,
hidden fitting or regularization, interface dispositions, documentation,
registry truth, and benchmark and CI routing.

## Findings

### P1 CONTOUR002-REV-001: value-tolerance acceptance merges distinct bracketed components

`crates/georbf/src/contour/isoline.rs:1404-1419` accepts a nonzero bracket
endpoint when its residual is within `value_tolerance`, but assigns the
grid-vertex key rather than the crossed-edge key. Distinct roots on separate
edges can therefore collapse into one topological vertex, contrary to the
canonical-identity and no-spatial-merge contract in
`docs/user-guide/ISOLINES.md:42-45`.

For the exact quadratic `f(x, y) = x^2 - delta`, use X nodes `-1, 0, 1`, one Y
cell, and `0 < delta < value_tolerance`. Each horizontal edge has a true sign
bracket, and the exact level set has two open components at
`x = +/-sqrt(delta)`. Both brackets select the nonzero sample at `x = 0` and
the same vertex key. The two cell segments then become exact duplicates and
produce one false component at `x = 0`.

The Repair must add an exact-CPD quadratic regression with `cells_x = 2`,
`cells_y = 1`, coordinate tolerance much smaller than `sqrt(delta)`, and value
tolerance greater than `delta`. It must require two distinct open polylines
and distinct crossed-edge identities even when accepted coordinates lie
within the caller's value tolerance.

### P1 CONTOUR002-REV-002: ordinary exact grid-vertex topology is rejected

`crates/georbf/src/contour/isoline.rs:1207-1225` counts edge records before
canonical-key deduplication. Every exact corner is counted once on each
incident edge, so an ordinary two-unique-intersection square pattern either
reaches the four-edge exact-corner rejection or later collapses at
`crates/georbf/src/contour/isoline.rs:1508-1511`. This conflicts with
`docs/user-guide/ISOLINES.md:42-53`, which promises canonical identity
deduplication before topology and rejects only exact-vertex patterns that
cannot form an ordinary case.

For exact affine truth `f(x, y) = x - y` on the single square `[0, 1]^2`, the
only unique boundary intersections are the lower-left and upper-right
vertices, so ordinary marching squares has one open diagonal segment. The
implementation receives `[LL, UR, UR, LL]`, counts four, observes exact corner
samples, and returns `DegenerateCellTopology`.

The Repair must add an exact-CPD affine marching-squares regression on this
one-cell domain. It must require one two-vertex open polyline, with both
rectangle sides recorded at each corner endpoint, while retaining structured
failure for one-unique-intersection and genuinely underdetermined patterns.

### P2 CONTOUR002-REV-003: the final permitted bisection cannot satisfy coordinate tolerance

`crates/georbf/src/contour/isoline.rs:1433-1460` tests bracket width before
updating the bracket with the newly evaluated midpoint. If iteration `N`
reduces the retained bracket to `coordinate_tolerance`, the loop exits with
`RefinementLimitReached` without reviewing the updated bracket. The error can
therefore report endpoints whose width already satisfies the request,
contrary to `docs/user-guide/ISOLINES.md:37-40`.

For exact affine truth `f(x, y) = x - 0.25`, an edge from `x = -1` to `x = 1`,
one permitted iteration, coordinate tolerance `1`, and a strict value
tolerance, the midpoint does not satisfy value tolerance but reduces the
retained bracket from width `2` to `1`. The coordinate criterion is satisfied,
but the implementation reports exhaustion.

The Repair must add an exact-CPD affine regression that succeeds when the
final allowed bisection first satisfies coordinate tolerance. It must also
assert that no `RefinementLimitReached` evidence has bracket width at or below
the requested coordinate tolerance.

### P2 CONTOUR002-REV-004: stable topology sorts bypass structured allocation failure

`crates/georbf/src/contour/isoline.rs:1478` uses stable `sort_by_key`, and
`crates/georbf/src/contour/isoline.rs:1590` uses stable `sort_by`. For
nontrivial slices, the pinned standard-library stable sort may allocate
auxiliary storage through the infallible allocator. That allocation is not
pre-reserved with `try_reserve` or translated to
`IsolineError::AllocationFailed`, as promised by
`crates/georbf/src/contour/isoline.rs:669-675` and
`docs/user-guide/ISOLINES.md:80-82`.

The Repair must use an allocation-safe deterministic ordering path. Its
regression must cover a long endpoint list under allocation instrumentation
and either prove that sorting allocates nothing after fallible preparation or
inject failure into explicitly fallible sort scratch and require the matching
`AllocationFailed` storage diagnostic.

### P2 CONTOUR002-REV-005: cancellation is not checked immediately before each value query

The fitted-field query executes at
`crates/georbf/src/contour/isoline.rs:929-951`; only afterward is its completed
result passed to `ProgressTracker::finish_work` at lines 952-953.
`ProgressTracker::finish_work` and `observe_result` check cancellation only
after their argument has already been evaluated at
`crates/georbf/src/execution.rs:347-370`. This contradicts the public method
guarantee at `crates/georbf/src/contour/isoline.rs:900-903` and the handoff
claim that cancellation is checked around every analytic value query. A
cancellation arriving after the preceding checkpoint can permit one more
expensive evaluation. The existing test at
`crates/georbf/tests/isoline.rs:409-421` covers only cancellation before the
operation starts.

The Repair must add an instrumented evaluation path synchronized so
cancellation occurs after the preceding checkpoint but before the next
invocation. It must require cancellation before the evaluator counter
increments and retain a post-query cancellation-priority assertion.

## Repair evidence

Commit `9510b6c` addresses only CONTOUR002-REV-001 through
CONTOUR002-REV-005:

- Sign-bracket intersections accepted by value tolerance retain their
  canonical crossed-edge key, so distinct bracketed components cannot collapse
  through a shared nonzero grid sample.
- Square and simplex intersection records are deduplicated by canonical key
  before topology classification. The exact affine `x - y` one-cell square
  now produces one ordinary corner-to-corner segment while one-unique and
  underdetermined patterns remain structured failures.
- Refinement updates the retained bracket before checking coordinate
  tolerance, so the final permitted bisection can succeed and every exhausted
  bracket remains wider than the requested coordinate tolerance.
- Endpoint-record and polyline ordering use deterministic in-place unstable
  sorts with total tie breakers. A 4096-element allocator-instrumented
  regression observes zero allocation calls during either ordering operation.
- The fitted-value wrapper performs an explicit cancellation checkpoint
  immediately before invoking the evaluator and retains the existing
  post-query cancellation priority. A synchronized counter regression proves
  both boundaries.

Independent exact-CPD polynomial regressions cover the two nearby quadratic
components, the exact affine corner topology, and final-iteration coordinate
tolerance. The isoline integration suite passes 10 tests, and the internal
allocation, cancellation, and canonical-key suite passes 3 tests. Focused
all-target, all-feature Clippy with warnings denied and the isoline Rustdoc
example pass. Release benchmark smoke passes with unchanged checksum
`1.83299999999997817e4`.

After the final production and test change, the exact tree committed as
`9510b6c` passed:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features`
- `cargo test --doc --workspace`
- `cargo xtask requirements check`

This repair evidence is not an independent re-review and does not close the
review contract by itself. PR #136 remains Draft until a fresh isolated
reviewer confirms the repairs and checks for new P0--P3 findings.

## Fresh independent re-review

A second fresh isolated read-only project `math_reviewer` received only the
bounded requirement and integrated dependency summary, Issue #135 acceptance
criteria and exclusions, the M8 plan, relevant scope and architecture
contracts, the complete base-to-head and focused repair diffs, original
findings, tests, benchmark evidence, and validation state. It inherited no
Implement or Repair reasoning and made no repository or remote change.

The reviewer independently closed all five original findings:

- CONTOUR002-REV-001 is closed because value-tolerance acceptance for a true
  sign bracket now retains the crossed-edge identity, and the exact quadratic
  regression proves two distinct components, four canonical vertices, and two
  unique segments.
- CONTOUR002-REV-002 is closed because square and simplex endpoints are
  deduplicated by canonical key before topology classification, and the exact
  affine corner regression proves one ordinary segment with both requested
  boundary sides at each endpoint.
- CONTOUR002-REV-003 is closed because the retained sign bracket is updated
  before the coordinate-width test, and the regression proves both
  final-iteration success and strictly wider evidence on exhaustion.
- CONTOUR002-REV-004 is closed as originally scoped because both production
  topology sorts are deterministic in-place unstable sorts with total tie
  breakers, and the 4096-entry regression invokes those exact functions with
  zero observed allocations.
- CONTOUR002-REV-005 is closed because every fitted-value query passes through
  a wrapper that checks cancellation immediately before invocation and
  preserves post-query cancellation priority; the synchronized counter
  regression proves both boundaries.

### P2 CONTOUR002-REREV-006: fitted-field error propagation allocates infallibly

`IsolineError::Preparation` and `IsolineError::Evaluation` store boxed sources
at `crates/georbf/src/contour/isoline.rs:676-687`. Production constructs those
boxes with `Box::new` at lines 922-926 and 931-935. Unlike the extractor's
`try_vec` and `try_push` paths, `Box::new` does not return an allocation error.
An allocation failure while forming a fitted scratch or evaluation diagnostic
can therefore leave the structured `IsolineError::AllocationFailed`,
source-preservation, and no-partial-report contract.

Repair must remove the infallible diagnostic allocation while preserving the
exact `FittedFieldEvaluationError<2>` through `Error::source`. An internal
allocation-instrumented regression must exercise the same preparation and
evaluation source conversions used by both production call sites, observe zero
allocation calls, and verify the retained source.

Repair head `6dee8e7` stores both fitted-field sources inline and routes the two
production `map_err` call sites through the same private conversions exercised
by the new allocation-instrumented regression. The regression observes zero
allocation calls for both conversions and downcasts each `Error::source` back
to the exact `FittedFieldEvaluationError<2>` with its evidence intact.

On that exact implementation head, the 10-test isoline integration suite, all
4 internal isoline repair tests, focused isoline Rustdoc, warning-denying
focused Clippy, and release benchmark smoke passed with the unchanged checksum
`1.83299999999997817e4`. The final standard gate passed formatting,
warning-denying all-target and all-feature workspace Clippy, the complete
all-feature workspace test suite, all workspace Rustdoc tests, and the
58-requirement registry check. This is Repair evidence, not an independent
closure of CONTOUR002-REREV-006.

No P0, P1, additional P2, or P3 finding was identified. The reviewer
independently rechecked the bilinear coefficients, interior saddle and
`a - b*c/d` value, positive-connectivity pairings and exact tie, dimensions and
scale normalization, bracket signs and endpoints, canonical identities, graph
topology, original-coordinate evaluation, CPD/SPD neutrality, absence of
hidden fitting or regularization, overflow, cancellation and progress,
interface dispositions, registry truth, benchmark construction, and CI
routing.

The reviewer passed the 10-test isoline integration suite, all three private
repair tests, the isoline Rustdoc example, release benchmark smoke with
checksum `1.83299999999997817e4`, formatting, the 58-requirement registry
check, and both complete and repair whitespace checks. Draft CI run
30097440898 then passed the configured Ubuntu correctness gate on exact
re-reviewed head `98a5572`. Ready-only Windows, Ubuntu, macOS, and complete
benchmark-smoke CI remains unexecuted.

## Independent re-review of CONTOUR002-REREV-006

A third fresh isolated read-only project `math_reviewer` received only the
bounded requirement and integrated dependency summary, Issue #135 acceptance
criteria and exclusions, relevant scope, M8, architecture, user-guide, change,
benchmark, and review records, the exact base-to-repair and focused repair
diffs, finding CONTOUR002-REREV-006, and validation evidence. It inherited no
Implement or Repair reasoning and made no repository or remote change.

No P0--P3 finding was identified. CONTOUR002-REREV-006 is closed on exact
implementation head `6dee8e73487b94b4c79a9d8b11206159814e8ba0`:

- `IsolineError::Preparation` and `IsolineError::Evaluation` retain the
  already-formed `FittedFieldEvaluationError<2>` inline. Moving that value
  into the outer enum does not allocate.
- `Error::source` borrows the same concrete inline value, preserving
  downcasting and the fitted-field error's own nested source chain.
- Both production `map_err` paths use the exact private preparation and
  evaluation conversions exercised by the regression. The evaluation closure
  captures only the copied query point and does not allocate.
- The regression measures both exact conversions, observes zero allocation
  calls, downcasts both sources to `FittedFieldEvaluationError<2>`, verifies
  their retained variant evidence, and verifies the evaluation point.
- Replacing `Box<T>` with inline `T` preserves the applicable `Send` and
  `Sync` bounds and valid `Debug`, `Display`, and `Error` behavior. The repair
  changes no released API and no formula, sign, dimension, topology,
  cancellation, CPD/SPD, rank, regularization, Hessian, constraint, benchmark,
  registry, adapter, or CI behavior.

The isolated reviewer passed the exact allocation regression, all four
internal isoline tests, the 10-test isoline integration suite, focused
warning-denying Clippy, the focused isoline Rustdoc example, formatting, the
58-requirement registry check, complete and repair whitespace checks, and
release benchmark smoke with checksum `1.83299999999997817e4`.

The parent Review task independently passed the 10-test isoline integration
suite, all four internal isoline tests, all 45 `georbf` Rustdoc tests,
warning-denying all-target and all-feature focused Clippy, formatting, the
58-requirement registry check, whitespace checks, and release benchmark smoke
with the same checksum. It verified that `6dee8e7..8ad5ac8` changes only this
review record and `docs/progress/CURRENT.md`. No production, test, manifest,
schema, benchmark, CI, registry, or build input changed after the exact
implementation and standard-gate head, so the complete standard gate recorded
for `6dee8e7` remains applicable and was not repeated.

## Ready CI, implementation merge, and post-merge CI

The clean re-review evidence was pushed as exact Ready head
`07061aa6313bad091fc2c13fd7d94d682c4269bf`. Ready CI run `30101363134`
passed the complete Windows, Ubuntu, and macOS workspace jobs on that exact
head. Every configured correctness check, backend feature combination,
requirement validation, and benchmark smoke passed, including the
two-dimensional isoline smoke on all three platforms.

PR #136 was squash-merged exactly once as
`9bb075bcb1347913eeaf991cd4a8ec6f41df65c5`; Issue #135 closed as completed.
Post-merge `main` CI run `30103256581` passed the same complete Windows,
Ubuntu, and macOS gate on exact merge commit `9bb075b`.

The isolated integration-state change updates only this review evidence,
`requirements/v1.yaml`, `docs/progress/HISTORY.md`, and the bounded handoff.
It changes no production code, test, manifest, schema, CI, build input, API,
numerical behavior, dependency, tag, or release. REQ-CONTOUR-002 becomes
truthfully `integrated` only when that isolated change itself merges after its
complete exact-head gate. The isolated change is Draft PR #137. No next
requirement may begin here.

## Verified behavior and residual risk

No P0 finding was identified. The reviewer independently derived the bilinear
formula `F = a + b*x + c*y + d*x*y`, saddle
`(-c/d, -b/d)`, and saddle value `a - b*c/d`. Both alternating-pattern
pairings correctly implement positive connectivity, including the exact-zero
tie.

The review also found no P0--P3 issue in dimensions or units,
original-coordinate evaluation, CPD/SPD neutrality, Hessian capability
claims, hidden fitting or regularization, degree bounds, deterministic graph
tracing, work arithmetic, interface N/A decisions, benchmark construction, or
Ready/main three-platform benchmark routing. These conclusions do not close
the five findings above.

The parent task independently reran the seven focused isoline integration
tests and the focused Rustdoc example on reviewed head `2b93db4`; all passed.
`git diff --check` also passed. Draft CI run 30095419189 passed its configured
Ubuntu correctness gate on exact reviewed head `2b93db4`. It is not the
complete Windows, Ubuntu, macOS, and benchmark-smoke Ready gate.

The complete standard workspace gate recorded on immutable implementation
head `4e766af` remains valid for that implementation. This Review changes only
this evidence record and the bounded handoff. It performs no production,
test, manifest, schema, benchmark, registry, CI, or build-input repair, and
does not claim unavailable checks.
