# PR #118 Independent Review

- Requirement: REQ-SPARSE-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/117
- Pull request: https://github.com/qingsonger/GeoRBF/pull/118
- Branch: `codex/req-sparse-001-compact-support`
- Reviewed head: `806bbff67753de37322d1e1d9298e69610438eea`
- Base head: `c6696f2b75a0b492f10bccb90f8ef3059e3f8eb9`
- Stable implementation gate head: `a0fd9fe`
- Repair implementation and gate head: `a24699525aa811f2a55b3eecf880eb64e685ee76`
- Second Repair implementation and gate head:
  `eca914287138baa42fddd09313596be60aa4a681`
- Second re-reviewed evidence head:
  `f7c832181ff6529ca554fd212afe60580f7d7633`
- Review date: 2026-07-23
- Result: the original review found one P1 and two P2 findings; the first
  Repair closed both P2 findings, while fresh re-review retained one P1 and
  added one evidence-only P3. The second Repair closed both remaining
  findings, but the next fresh independent re-review found one new P1 sparse
  solver peak-memory finding.

## Scope and independence

A fresh read-only project `math_reviewer` received only the bounded
REQ-SPARSE-001 summary and integrated dependency closure, Issue #117 acceptance
criteria and exclusions, the M7 plan, architecture and solver policy,
ADR-0012, the complete fixed PR diff, tests, dependency-audit and benchmark
evidence, and exact validation results. It inherited no Implement reasoning and
made no repository or remote change.

The reviewer independently checked formulae, signs, dimensions, Wendland SPD
classification and center limits, the CPD and polynomial-side-condition
exclusion, exact strict support, global-anisotropy candidate bounds, hard
constraints and explicit failures, original-unit residual review, immutable
local value/gradient/Hessian evaluation, allocation and memory policy, hidden
regularization or fallback, interface dispositions, and requirement evidence.

## Findings

### P1 SPARSE001-REV-001: the explicit memory limit is not a peak limit

`crates/georbf/src/sparse.rs:421-428` shows that the returned sparse system
simultaneously retains the canonical problem, execution and fit options, CSC
matrix, right-hand side, diagnostics, and neighborhood index. Assembly at
`crates/georbf/src/sparse.rs:877-879` instead enforces the final estimate from
`retained_storage_bytes`, whose calculation at
`crates/georbf/src/sparse.rs:1470-1496` counts only an estimated index payload,
CSC arrays, and one right-hand-side vector. It omits the retained
`CanonicalProblem` and the simultaneous assembly peak containing the index,
neighbor pairs, entries, row offsets, canonical equalities, CSC, and right-hand
side.

The per-vector guard at `crates/georbf/src/sparse.rs:1331-1353` checks only one
vector's requested logical length, not the aggregate live payload; a reserve
may also create capacity beyond that logical length. The solve estimate at
`crates/georbf/src/sparse.rs:1545-1576` counts two CSC payloads, dense worst-case
fill, and solver vectors, but omits the still-live borrowed
`SparseFieldSystem` neighborhood index and canonical rows.

The implementation can therefore report success below the explicit limit while
its checked logical live payload already exceeds that limit. Allocation may
then fail or exhaust substantially more memory than the caller authorized, and
the published conservative-peak diagnostic is not truthful. This violates
Issue #117's checked-memory and explicit-memory-limit acceptance criteria.

Required Repair: account with checked arithmetic for every simultaneously live
logical component at each assembly and solve peak, including retained canonical
and index state and temporary capacities, and reject before the corresponding
allocation. Add a regression that first obtains component counts under a wide
limit, then proves a limit equal to the current retained estimate rejects the
larger assembly peak. Add a second limit between the corrected retained
assembly payload and corrected solve peak, proving assembly succeeds and solve
returns `MemoryLimitExceeded`. Assert that diagnostics equal the sum of all
checked live components.

### P2 SPARSE001-REV-002: support coverage counts nonzero actions, not exact-support neighbors

The public diagnostics at `crates/georbf/src/sparse.rs:189-194` describe
isolated centers and minimum/maximum row counts as supported representers.
Exact-support representer pairs are created by the strict radius predicate at
`crates/georbf/src/sparse.rs:717-727`. However,
`crates/georbf/src/sparse.rs:775-805` increments `row_neighbors` only when the
evaluated kernel action is numerically nonzero, and
`crates/georbf/src/sparse.rs:880-898` publishes those counts beside the
exact-support `pairs.len()`.

For co-located Value and DirectionalDerivative representers, a differentiable
radial Wendland kernel has exact center gradient zero. Their off-diagonal
kernel action is therefore zero even though the representer pair satisfies
`radius < support_radius`. The current diagnostics can report both centers as
isolated and understate minimum and maximum supported-neighbor counts while
`supported_pairs` records the pair under the exact-support definition.

Required Repair: derive support coverage from the accepted exact-support pair
graph, independently of whether a particular functional action evaluates to
zero. Add a co-located Value/DirectionalDerivative regression that requires
three upper-triangle supported pairs, an exact zero off-diagonal action, zero
isolated centers, and two support-neighbor representers in each row.

### P2 SPARSE001-REV-003: the change evidence overstates acceptance coverage

`changes/REQ-SPARSE-001.md:42-48` claims that the six sparse integration tests
cover canonical conflict and the complete listed cancellation and memory-limit
failures. The only combined failure test at
`crates/georbf/tests/sparse.rs:383-421` covers pre-cancelled assembly, a one-byte
assembly limit, and factorization rejection for duplicate rows with identical
right-hand sides. It does not construct conflicting right-hand sides, exercise
solve-stage cancellation or memory rejection, or exercise a nonfinite or
unrepresentable sparse candidate boundary.

Issue #117 also requires deterministic ordering and subquadratic storage
scaling. `crates/georbf/tests/sparse.rs:361-380` checks only one 512-point
fixture, so it establishes a small density at one size but not a growth-rate
comparison. No test repeats assembly and compares the complete CSC and
diagnostics.

Required Repair:

- repeat one fixture and compare every CSC array and deterministic diagnostic;
- use repeated locations with conflicting right-hand sides and require the
  canonical hard-conflict error;
- independently exercise sparse-solve memory and cancellation rejection;
- cover `UnrepresentableCandidateRadius` or an equivalent sparse nonfinite
  boundary using otherwise finite input; and
- compare at least two fixed-density grids, requiring bounded nonzeros per
  point and growth materially below the corresponding dense `n*n` storage.

Update the requirement change fragment to claim only coverage that those
regressions actually establish.

No other P0, P1, P2, or P3 finding was identified.

## Repair evidence

Repair commit `a24699525aa811f2a55b3eecf880eb64e685ee76`
addresses only SPARSE001-REV-001, SPARSE001-REV-002, and
SPARSE001-REV-003:

- SPARSE001-REV-001: assembly now checks aggregate live payloads before index,
  pair, entry, row, canonical, right-hand-side, and CSC allocation stages.
  Public diagnostics separately sum retained index, canonical allocation
  capacities, CSC, right-hand side, pair, entry, row, and bulk-load
  components, and record the maximum checked stage. Sparse solve adds the
  complete retained borrowed system to its backend CSC, conservative dense
  factor fill, vector, and exact-accumulator sum. The
  `peak_memory_diagnostics_and_stage_limits_are_exact` regression proves every
  published sum, rejects an assembly limit equal to retained storage, permits
  assembly at the assembly peak, and rejects solve at that same limit.
- SPARSE001-REV-002: support coverage now increments from every sorted unique
  exact-support pair before kernel action zero filtering. The
  `exact_support_coverage_includes_zero_colocated_actions` regression uses
  co-located Value and DirectionalDerivative representers, requires three
  upper-triangle support pairs and an exact-zero cross action, and requires
  zero isolated centers with two neighbors in both rows.
- SPARSE001-REV-003: the sparse suite now repeats and compares complete CSC,
  right-hand side, and diagnostics; requires the source-aware hard canonical
  conflict for repeated locations with distinct targets; exercises
  solve-stage cancellation and memory rejection; rejects an otherwise finite
  fixture whose candidate radius is unrepresentable; and compares fixed 64-
  and 512-point grids with bounded nonzeros per point and growth materially
  below dense-square storage. The change fragment now names only directly
  exercised coverage.

The repaired suite has nine all-feature sparse integration tests. It passed
alongside warning-denying all-target Clippy, and the 64-point release benchmark
smoke retained 352 nonzeros with finite stable phase checksums. After the last
production, test, architecture, and change-fragment edit, exact head
`a24699525aa811f2a55b3eecf880eb64e685ee76` passed the complete standard
workspace gate: format, warning-denying workspace/all-target/all-feature
Clippy, all-feature workspace tests, workspace doctests, and the 58-requirement
registry check.

## Independent truth review

- The Wendland kernels used by the sparse path are strictly positive definite
  in the supported dimensions. CPD kernels and polynomial side rows are
  rejected rather than silently bypassing their complete side condition.
- Candidate index hits are only a prefilter. The original-dimensional
  isotropic or globally anisotropic radius is recomputed and the exact
  `radius < support_radius` predicate is applied before each accepted pair.
- The global-anisotropy inverse-transform bound is conservative, so it cannot
  exclude an exact transformed-space support neighbor.
- Faer's lower LLT path uses the requested AMD ordering and exposes no jitter,
  regularization, pseudoinverse, densification, hard-to-soft conversion,
  constraint relaxation, backend switch, or factorization fallback.
- Original-unit residuals use the unchanged CSC system, accumulate `b-A*x`,
  and apply the documented sign-invariant infinity norm and
  `128*n*epsilon` backward-error tolerance.
- The retained index preserves immutable local-center evaluation. Existing
  kernel capability and center-limit checks govern value, gradient, and
  Hessian calls in D=1, D=2, and D=3.
- Rust implementation and CLI/C/C++/Python deferrals are truthful for the
  current milestone. Ready-head platform and benchmark evidence remains
  outstanding and is not claimed.

## Validation and disposition

- Local and remote branch heads matched exact reviewed head `806bbff`; the base
  was `c6696f2`, and the worktree was clean before this evidence-only change.
- Draft CI run 29990525588 completed its configured Ubuntu correctness job
  successfully on exact reviewed head `806bbff`. The Ready-only Windows,
  Ubuntu, macOS, and benchmark-smoke matrix was skipped as designed and is not
  claimed as passed.
- The independent reviewer and parent Review task each passed all six
  all-feature sparse integration tests. The parent also passed all 58
  requirement checks and the complete PR whitespace check.
- Stable implementation head `a0fd9fe` retains the complete standard local
  gate and release benchmark smoke recorded by Implement. The tail through
  reviewed head `806bbff` changed only validation, registry, handoff, and PR
  evidence.
- This Review change adds only this review record, its registry document link,
  and the bounded handoff. It changes no production code, test, manifest,
  schema, CI, build input, API, numerical behavior, dependency, or benchmark
  result and therefore does not invalidate the stable implementation gate.
- `cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are
  not installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers,
  executable fuzzing, mutation testing, API/ABI/schema snapshots, and local
  `actionlint` remain unavailable or deferred. No unexecuted check is claimed
  as passed.

PR #118 remains Draft and REQ-SPARSE-001 remains `planned`, not `integrated`.
A fresh independent re-review task must verify all three repairs and check for
new P0-P3 findings on the repaired head. This Repair does not mark the PR
ready, merge it, or begin REQ-CENTER-001.

## Fresh independent re-review after the first Repair

- Re-reviewed base: `c6696f2b75a0b492f10bccb90f8ef3059e3f8eb9`
- Re-reviewed evidence head:
  `244a04a5e2d10315d76a668b2122cc895ff0e43c`
- Repair implementation and stable gate head:
  `a24699525aa811f2a55b3eecf880eb64e685ee76`
- Draft CI run: 29992714121
- Result: one P1 remains open and one new P3 evidence finding was identified

A new isolated read-only project `math_reviewer` independently inspected the
complete five-commit base-to-head diff. It received only the bounded
requirement summary and integrated dependency closure, Issue #117 acceptance
criteria and exclusions, the M7 plan, architecture and solver policy,
ADR-0012, the original findings, the complete repaired diff, and exact test,
benchmark, and stable-gate evidence. It inherited no Implement or Repair
reasoning and made no repository or remote change.

SPARSE001-REV-002 is closed. Exact-support row coverage is updated before
numeric-zero filtering, and the co-located Value and
DirectionalDerivative regression proves that an exact-zero action remains in
the support graph.

SPARSE001-REV-003 is closed for deterministic complete CSC and diagnostic
comparison, hard conflict, assembly and solve cancellation, solve memory
rejection, an unrepresentable candidate radius, and two-size bounded-neighbor
growth. Its memory-coverage claim remains dependent on the still-open
SPARSE001-REV-001 below.

### P1 SPARSE001-REV-001 remains open: canonical relation buffers are omitted

`SemanticProblemIr::try_compile` reserves `constraints.len()` capacity for
equalities, linear bounds, cones, and soft objectives at
`crates/georbf/src/problem_ir.rs:516-549`. A field problem populates only
equalities, but the other three vectors remain logically empty with allocated
capacity. `CanonicalProblem::equality_payload_capacity_bytes` verifies that
those vectors are empty but omits their capacities at
`crates/georbf/src/problem_ir.rs:1548-1598`. The sparse pre-allocation bound at
`crates/georbf/src/sparse.rs:1600-1642` omits the same buffers.

Assembly therefore enforces and publishes an incomplete canonical payload at
`crates/georbf/src/sparse.rs:947-961` and
`crates/georbf/src/sparse.rs:1002-1054`, while solve inherits the undercount
through `retained_system_bytes` at `crates/georbf/src/sparse.rs:1866-1877`.
The missing simultaneous payload is at least the capacities of
`CanonicalLinearBound`, `CanonicalSecondOrderCone`, and
`CanonicalSoftObjective`. A caller limit between the reported and actual live
payload can still succeed, so the complete borrowed-system and exact-memory
claims in `changes/REQ-SPARSE-001.md` are not yet true.

Required Repair: include every canonical vector's allocated capacity in the
checked equality-only payload and pre-allocation bound. Add an internal
independent sum over all canonical vector, string, and scaling capacities,
including the three empty relation buffers, and require
`equality_payload_capacity_bytes` to equal it. Add an assembly limit between
the old and corrected canonicalization peaks and require
`MemoryLimitExceeded`; use the corrected assembly result to require the
analogous solve-stage rejection.

### P3 SPARSE001-REV-004: residual-sign evidence is inaccurate

`crates/georbf/src/sparse.rs:1914-1925` accumulates `b - A*x`, while before
this Repair the record's independent-truth section stated `A*x-b`. The exposed
infinity norm and backward error are sign invariant, so numerical acceptance
is unchanged, but that evidence was factually inaccurate.

Required Repair: state `b-A*x`, or explicitly state that the norm is sign
invariant. No behavioral regression is required because the public diagnostic
does not expose the residual sign.

No other P0, P1, P2, or P3 finding was identified. Independent truth checks
confirmed the Wendland SPD and strict-support contracts, the CPD exclusion,
the conservative anisotropy candidate bound, exact symmetric action
reflection, explicit LLT failure, absence of hidden regularization or fallback,
the original-unit backward-error normalization and tolerance, D=1/D=2/D=3
embedding, normalization chain rules, center capabilities, interface
dispositions, benchmark scope, and truthful non-integrated registry state.

Draft CI run 29992714121 passed its configured Ubuntu correctness job on exact
re-reviewed head `244a04a`; the Ready-only Windows, Ubuntu, macOS, and
benchmark-smoke matrix was skipped as designed and is not claimed as passed.
The worktree remained clean throughout the independent review.

PR #118 must remain Draft and REQ-SPARSE-001 remains `planned`. A fresh Repair
task must address only SPARSE001-REV-001 and SPARSE001-REV-004, add the
specified memory regressions, run focused checks and one complete stable-head
standard gate after the last production or test change, update this evidence
and the bounded handoff, push, and stop for another fresh independent
re-review. This Review does not repair production code, mark the PR ready,
merge it, or begin REQ-CENTER-001.

## Second Repair evidence

- Exact implementation and stable-gate head:
  `eca914287138baa42fddd09313596be60aa4a681`.
- SPARSE001-REV-001: the equality-only retained-payload calculation now adds
  the allocated capacities of all four canonical relation vectors. The
  pre-allocation bound reserves the same conservative capacity for
  `CanonicalEquality`, `CanonicalLinearBound`,
  `CanonicalSecondOrderCone`, and `CanonicalSoftObjective`, so both assembly
  and the solve-retained-system base include the three logically empty
  relation buffers.
- The internal
  `equality_payload_counts_every_reserved_canonical_capacity` regression
  independently sums variable-block vectors and names, all four relation
  vectors, equality term vectors, every provenance string, and all five
  scaling vectors. It confirms that the three non-equality vectors are empty
  but allocated, then requires the public-to-sparse retained calculation to
  equal that independent sum.
- The sparse memory regression sets an assembly limit strictly between the old
  and corrected canonicalization peaks and requires the corrected preflight to
  reject it. A separate 64-row fixture sets the inherited limit strictly
  between the old and corrected solve peaks, confirms corrected assembly fits,
  and requires solve to reject the complete corrected retained-system sum.
- SPARSE001-REV-004: the independent-truth evidence now states that the
  implementation accumulates `b-A*x` and that the published infinity norm and
  backward error are sign invariant. Production residual behavior is
  unchanged.

Focused validation on the Repair worktree passed all 44 all-feature core unit
tests, all nine all-feature sparse integration tests, format, and
warning-denying all-target/all-feature Clippy. After the last production or
test change, exact head `eca914287138baa42fddd09313596be60aa4a681` passed the
complete standard workspace gate: format, warning-denying
workspace/all-target/all-feature Clippy, all-feature workspace tests,
workspace doctests, and the 58-requirement registry check.

The second Repair changes no sparse support predicate, kernel action, CSC
value, factorization, solution, residual accumulation, tolerance, fallback, or
regularization policy. PR #118 remains Draft and REQ-SPARSE-001 remains
`planned`. A fresh independent Review/re-review must confirm both repairs and
check for new P0-P3 findings before any Ready transition, complete
three-platform and benchmark-smoke CI, merge, or integration-state update.

## Fresh independent re-review after the second Repair

- Re-reviewed base: `c6696f2b75a0b492f10bccb90f8ef3059e3f8eb9`
- Re-reviewed evidence head:
  `f7c832181ff6529ca554fd212afe60580f7d7633`
- Second Repair implementation and stable-gate head:
  `eca914287138baa42fddd09313596be60aa4a681`
- Draft CI run: 29994904719
- Result: SPARSE001-REV-001 and SPARSE001-REV-004 are closed; one new P1
  finding remains open

A new isolated read-only project `math_reviewer` independently inspected the
complete base-to-head diff. It received only the bounded requirement summary
and integrated dependency closure, Issue #117 acceptance criteria and
exclusions, the M7 plan, architecture and solver policy, ADR-0012, all prior
findings, the complete repaired diff, and exact test, benchmark, and stable-gate
evidence. It inherited no Implement or Repair reasoning and made no repository
or remote change.

SPARSE001-REV-001 is closed. The retained canonical equality-only payload and
canonicalization preflight now count all four canonical relation-vector
capacities, including the three logically empty reserved buffers. The
independent canonical-capacity unit regression and the assembly/solve
between-limit regressions pass.

SPARSE001-REV-002 and SPARSE001-REV-003 remain closed. Exact-support coverage
continues to be recorded before numeric-zero filtering, and the nine sparse
integration tests retain deterministic complete CSC and diagnostics, hard
conflict, assembly and solve cancellation, nonrepresentable candidate radius,
solve memory rejection, and two-size bounded-neighbor growth coverage.

SPARSE001-REV-004 is closed. This review record correctly states that the
implementation accumulates `b-A*x`; the published infinity norm and backward
error are sign invariant.

### P1 SPARSE001-REV-005: faer symbolic-factorization scratch is omitted from the explicit peak-memory limit

`crates/georbf/src/sparse.rs:1318-1325` dispatches faer `sp_cholesky` after
`estimate_sparse_peak_bytes` at `crates/georbf/src/sparse.rs:1848-1876`
counts the retained system, backend CSC copy, dense lower-triangle factor
payload, and residual and solution vectors. The enforced estimate omits faer
AMD and symbolic-analysis scratch, retained symbolic structures, and numeric
scratch.

This is logical backend work memory, not allocator or operating-system
overhead. Pinned faer 0.24.4 constructs the symbolic factor before numeric LLT
and requests AMD scratch approximately equal to
`(3.4*m + 13*n + 1) * size_of::<usize>()` for an unsorted CSC with `m` stored
entries. A permitted 64-by-64 all-supported system has `m = 4096`, so AMD
scratch alone is 118,072 bytes on the reviewed 64-bit targets. GeoRBF's
current dense-lower fill and working-vector allowance for that system totals
only 70,656 bytes. Even treating residual workspace, which is not live during
symbolic analysis, as spare factorization allowance leaves at least 47,416
bytes unaccounted before retained permutations and symbolic factor storage.

A caller limit between the reported and actual logical solve peak can
therefore pass preflight and enter a backend allocation above the explicit
limit. This contradicts Issue #117 and the architecture, solver-policy, and
change-fragment claims that every simultaneously live solve component is
covered.

Required Repair: add checked conservative bounds for the pinned faer AMD and
symbolic-analysis scratch, retained symbolic structures, numeric-factor
storage, and numeric scratch before dispatch. If the high-level API cannot
expose exact requests, derive a conservative bound from `n`, stored `m`, and
the dense-factor fill, or use a lower-level checked adapter. Add an internal
64-by-64 all-supported Wendland regression that independently obtains or
reproduces faer's AMD scratch request, sets a limit strictly between the old
and corrected solve peaks, permits assembly, and requires
`SparseSolveError::MemoryLimitExceeded` before any factorization progress
event.

No other P0, P1, P2, or P3 finding was identified. Independent truth checks
confirmed the conservative anisotropy candidate bound, strict support,
D=1/D=2/D=3 index embedding, sorted and deduplicated pairs, exact reflection,
sorted-unique CSC, Wendland SPD restriction and CPD exclusion, explicit LLT
failure, absence of hidden regularization or fallback, original-unit residual
normalization and tolerance, local normalization and derivative chain rules,
capability-gated Hessians, interface dispositions, benchmark scope, and
truthful non-integrated registry state.

Focused read-only validation passed all nine all-feature sparse integration
tests, the canonical reserved-capacity unit regression, and the complete PR
whitespace check. Exact faer 0.24.4 and rstar 0.13.0 feature resolution was
confirmed, and the worktree remained clean. Draft CI run 29994904719 passed
its Ubuntu correctness job on exact reviewed head `f7c8321`; the Ready-only
Windows, Ubuntu, macOS, and benchmark-smoke matrix was skipped as designed and
is not claimed.

PR #118 must remain Draft and REQ-SPARSE-001 remains `planned`. A fresh Repair
task must address only SPARSE001-REV-005, add the specified regression, run
focused checks and one complete stable-head standard gate after the last
production or test change, update this evidence and the bounded handoff, push,
and stop for another fresh independent re-review. This Review does not repair
production code, mark the PR ready, merge it, or begin REQ-CENTER-001.

## Third Repair evidence

- Exact Repair implementation and stable-gate head:
  `85d7e99356fe3790f2a0d63430b78a4f68ad8a0f`.
- Scope: only P1 SPARSE001-REV-005.

The sparse preflight now reproduces pinned faer 0.24.4's public AMD and
complete symbolic-analysis `StackReq` from the matrix dimension and stored
nonzeros. Checked conservative arithmetic separately bounds retained
simplicial or supernodal symbolic structures, numeric-factor storage, and
numeric-factorization scratch. Diagnostics expose each component and the
symbolic-factorization, numeric-factorization, and solve-and-review peaks; the
maximum is enforced before backend CSC construction.

The new 64-by-64 all-supported Wendland regression independently calls faer's
AMD scratch API for all 4,096 stored entries and requires the diagnostic to
match its request. It constructs a limit strictly above both assembly and the
old solve estimate but below the corrected solve peak, confirms assembly
succeeds, and requires `SparseSolveError::MemoryLimitExceeded`. Its progress
sink observes only `ExecutionStage::Started`, proving neither memory-review nor
factorization success is published before rejection.

Focused validation passed all 44 all-feature core unit tests, all ten
all-feature sparse integration tests, format, and warning-denying
all-target/all-feature Clippy. After the last production or test change, exact
head `85d7e99356fe3790f2a0d63430b78a4f68ad8a0f` passed the complete standard
workspace gate: format, warning-denying workspace/all-target/all-feature
Clippy, all-feature workspace tests, workspace doctests, and the
58-requirement registry check.

This Repair changes no support predicate, kernel action, CSC value,
factorization choice, solution, residual formula or tolerance, regularization,
fallback, interface disposition, or registry integration state. PR #118
remains Draft and REQ-SPARSE-001 remains `planned`. SPARSE001-REV-005 is
addressed but is not recorded as independently closed; a fresh re-review task
must verify this exact Repair and check for new P0-P3 findings. This task does
not mark the PR ready, merge it, or begin REQ-CENTER-001.

## Fresh independent re-review after the third Repair

- Re-reviewed base: `c6696f2b75a0b492f10bccb90f8ef3059e3f8eb9`
- Re-reviewed evidence head:
  `917e6b3b5a12f48588cb5a34676cb2093988a8db`
- Third Repair implementation and stable-gate head:
  `85d7e99356fe3790f2a0d63430b78a4f68ad8a0f`
- Draft CI run: 29997122789
- Result: SPARSE001-REV-005 is closed; no P0-P3 finding remains

A new isolated read-only project `math_reviewer` independently inspected the
complete base-to-head diff. It received only the bounded requirement summary
and integrated dependency closure, Issue #117 acceptance criteria and
exclusions, the M7 plan, architecture and solver policy, ADR-0012, all prior
findings, the complete repaired diff, and exact test, benchmark, and
stable-gate evidence. It inherited no Implement or Repair reasoning and made
no repository or remote change.

SPARSE001-REV-005 is closed. The preflight reconstructs pinned faer 0.24.4's
AMD request and complete symbolic-analysis `StackReq`, matching faer's maximum
of AMD workspace and CSC permutation plus post-ordering symbolic workspace.
The conservative retained bounds cover both symbolic forms, two
permutations, dense worst-case index patterns, numeric supernodal panels, and
numeric scratch. Symbolic-factorization, numeric-factorization, and
solve-and-review live peaks are summed separately, their maximum is enforced,
and rejection occurs before the backend CSC copy begins.

The 64-by-64 all-supported regression independently obtains faer's AMD request
for all 4,096 stored entries. Its memory limit is above both the assembly peak
and the former solve estimate but below the corrected peak. Assembly succeeds,
solve returns `SparseSolveError::MemoryLimitExceeded`, and the progress sink
observes only `ExecutionStage::Started`.

SPARSE001-REV-001 through SPARSE001-REV-004 remain closed. No new P0, P1, P2,
or P3 finding was identified. Independent truth checks confirmed the Wendland
SPD and strict-support contracts, CPD and polynomial-side-condition exclusion,
conservative anisotropy candidates, exact symmetric CSC action, original-unit
residual acceptance, hard-failure behavior, absence of hidden regularization
or fallback, D=1/D=2/D=3 and derivative capabilities, immutable local
evaluation, truthful interface dispositions, benchmark scope, and the
non-integrated registry state.

The independent reviewer passed all ten all-feature sparse integration tests,
the canonical reserved-capacity unit regression, the 64-point release
benchmark smoke with 352 stored nonzeros, and the complete PR whitespace
check. The parent Review task independently passed the same ten sparse tests,
the canonical reserved-capacity regression, the 58-requirement registry check,
and the complete PR whitespace check. Exact faer 0.24.4 and rstar 0.13.0
feature resolution was confirmed without faer's Rayon feature.

Draft CI run 29997122789 passed its configured Ubuntu correctness job on exact
re-reviewed head `917e6b3`; the Ready-only Windows, Ubuntu, macOS, and
benchmark-smoke matrix was skipped as designed and is not claimed. The branch
and remote heads matched, and the worktree remained clean throughout review.

This review evidence changes only this record and the bounded handoff. It
changes no production code, test, manifest, schema, CI, build input, API,
numerical behavior, dependency, or benchmark result, so the complete standard
gate on exact stable implementation head `85d7e99` remains valid.

PR #118 is clean for the mandatory integration sequence. REQ-SPARSE-001
remains `planned` until the exact Ready head passes the complete
Windows/Ubuntu/macOS and benchmark-smoke CI, the PR is merged exactly once,
and isolated integration-state evidence is recorded. This Review does not
begin REQ-CENTER-001.

## Integration evidence

- Exact Ready evidence head:
  `4c436b616f7be780dbe0350fc1649db0b436f821`
- Ready CI run: 29998141180
- Squash merge: `ce93a9862690399c36de7c34dcfd4cd4dbf2ee8c`
- Post-merge `main` CI run: 29999737528
- Integration-state branch: `codex/req-sparse-001-integration-state`
- Integration-state pull request: pending creation

Ready CI run 29998141180 passed the complete Windows, Ubuntu, and macOS
workspace gate on exact Ready head `4c436b6`, including every configured
backend combination, every benchmark-smoke workload, and requirement
validation. PR #118 then squash-merged exactly once as `ce93a98`; Issue #117
closed as completed. Post-merge `main` CI run 29999737528 passed the same
complete three-platform gate on exact merge commit `ce93a98`.

The isolated integration-state change updates only the requirement registry,
this review evidence, the completed-history index, and the bounded handoff. It
changes no production code, test, manifest, schema, CI, build input, public
API, numerical behavior, dependency, tag, or release. The requirement may be
recorded as `integrated` in this change because implementation, tests,
documentation, applicable interfaces, diagnostics, independent review, exact
Ready-head CI, the implementation merge, and post-merge `main` CI are
complete. The integration-state pull request itself must still pass its
complete local standard gate and exact Ready-head CI before it merges.
