# PR #136 Independent Review

- Requirement: REQ-CONTOUR-002
- Issue: https://github.com/qingsonger/GeoRBF/issues/135
- Pull request: https://github.com/qingsonger/GeoRBF/pull/136
- Branch: `codex/req-contour-002-isolines`
- Base head: `6e622c708be8a7d030213f29fc42ef4f9ce256ef`
- Reviewed head: `2b93db4c7efce551601e45836ec43ff4a3c7f622`
- Stable implementation gate head:
  `4e766af`
- Draft CI run: 30095419189
- Review date: 2026-07-24
- Result: five actionable findings; fresh Repair required

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
