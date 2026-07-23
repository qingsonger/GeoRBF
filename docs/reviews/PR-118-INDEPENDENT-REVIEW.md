# PR #118 Independent Review

- Requirement: REQ-SPARSE-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/117
- Pull request: https://github.com/qingsonger/GeoRBF/pull/118
- Branch: `codex/req-sparse-001-compact-support`
- Reviewed head: `806bbff67753de37322d1e1d9298e69610438eea`
- Base head: `c6696f2b75a0b492f10bccb90f8ef3059e3f8eb9`
- Stable implementation gate head: `a0fd9fe`
- Repair implementation and gate head: `a24699525aa811f2a55b3eecf880eb64e685ee76`
- Review date: 2026-07-23
- Result: the original review found one P1 and two P2 findings; Repair
  addressed all three, pending fresh independent re-review

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
- Original-unit residuals use the unchanged CSC system and the documented
  `128*n*epsilon` tolerance with the expected `A*x-b` sign convention.
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
