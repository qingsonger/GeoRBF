# PR #130 Independent Review

- Requirement: REQ-PERF-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/129
- Pull request: https://github.com/qingsonger/GeoRBF/pull/130
- Branch: `codex/req-perf-001-performance-baseline`
- Base head: `01b9fa5eaf10e8cfa040eaf309c382c9d8803b6c`
- Reviewed head: `293bcd15f44128dae6d067e9e1eb119bc5c2e0ae`
- Repair head: `c5b5b8d56c0fae009345c74a0cee425571392ede`
- Stable implementation gate head:
  `c5b5b8d56c0fae009345c74a0cee425571392ede`
- Draft CI run: 30067909616
- Review date: 2026-07-24
- Original result: changes requested; one P1, two P2, and one P3 finding
- Current state: all four findings repaired; fresh independent re-review required

## Scope and independence

A fresh isolated read-only project `math_reviewer` received only the bounded
REQ-PERF-001 summary and integrated dependency closure, Issue #129 acceptance
criteria and exclusions, the M7 plan, ADR-0010 and ADR-0012, the complete
base-to-head diff, tests, benchmark evidence, exact validation results, and
Draft CI state. It inherited no Implement reasoning and made no repository or
remote change.

The reviewer independently checked block traversal and symmetry work counts,
dimensions and units, center and compact-support limits, deterministic ordering
and bit identity, thread isolation and failure containment, allocation
stability, checked logical memory accounting, public API behavior, immutable
model safety, interface dispositions, benchmark and checksum evidence, tests,
CI wiring, hidden regularization, and registry and handoff accuracy.

## Findings

### P1 PERF001-REV-001: sparse scratch capacity counts centers, not indexed terms

`crates/georbf/src/performance.rs:594-598` derives sparse workspace capacity
from `self.centers().len()`, and `crates/georbf/src/model.rs:1036-1043`
reserves the same retained-center count. The compact R-tree instead contains
one item per atomic center term at
`crates/georbf/src/sparse.rs:367-370`. Query evaluation pushes every supported
term's center identifier before sorting and deduplicating at
`crates/georbf/src/sparse.rs:417-451`.

A valid center representer may contain more than one term. For three two-term
centers whose terms are all within support, the workspace reserves three
indices but the query pushes six before deduplication. The capacity check in
the query path then grows the vector. This allocates inside evaluation and
`try_batch_memory_diagnostics` reports only three `usize` entries, allowing a
caller limit below the actual scratch payload.

This violates the explicit pre-allocation memory limit and the no-per-query
allocation guarantee for supported public inputs.

Required Repair: fit a compact sparse model with
`indexed_terms > centers.len()` and supported duplicate center hits. Establish
the reusable workspace and output first, then require zero allocations during
batch-into evaluation. Workspace diagnostics must use the indexed-term
capacity, and a limit strictly between the old and corrected estimates must
fail before evaluation.

### P2 PERF001-REV-002: incompatible workspace failure leaves stale caller output

`crates/georbf/src/performance.rs:347-358` validates workspace compatibility
before clearing caller output, although the public contract says output is
cleared on every failure. The test at
`crates/georbf/tests/performance.rs:306-323` passes a newly constructed empty
output vector and therefore does not exercise the advertised behavior.

After a successful batch has populated `output`, a subsequent request with a
workspace from an incompatible model returns `IncompatibleWorkspace` while the
old evaluations remain present. A caller can therefore observe usable-looking
stale data after the failed request.

Required Repair: populate output with a successful call, invoke the method with
an incompatible workspace, and assert both the structured error and
`output.is_empty()`. Audit the other fallible exits so every documented failure
clears caller output.

### P2 PERF001-REV-003: shared scratch construction makes one-point sparse evaluation global

The full retained-center reservation was added to the shared scratch
constructor at `crates/georbf/src/model.rs:1027-1047`. That constructor is also
used by ordinary one-point value, value/gradient, and Hessian evaluation at
`crates/georbf/src/model.rs:908-924` and
`crates/georbf/src/model.rs:969-974`.

A sparse query whose exact support touches one local center now reserves
index storage proportional to every retained center before doing any local
work. Increasing only far-away centers therefore increases allocation bytes
and can make an otherwise local one-point evaluation fail. REQ-PERF-001 needs
complete capacity for explicit reusable batch workspaces; it does not authorize
turning all one-shot sparse APIs into global-allocation paths.

Required Repair: compare one-point sparse evaluation allocation bytes for
models with identical bounded local support and increasing far-away center
counts. One-shot evaluation scratch must remain locality-scaled, while an
explicit reusable batch workspace retains the complete corrected indexed-term
capacity.

### P3 PERF001-REV-004: the exact-work test does not observe evaluator calls

`crates/georbf/tests/performance.rs:197-251` asserts the diagnostic's
closed-form `n(n+1)/2` count and final matrix symmetry, but it never counts
closure invocations or records the visited `(query, center)` pairs. A traversal
regression that evaluates one entry twice, omits another, and continues to
report the closed-form count could pass. Symmetry verifies reflected values,
not exact evaluation work.

Required Repair: have the `FnMut` evaluator record every visited pair. Require
exactly `n(n+1)/2` unique upper-triangle visits in deterministic block order,
with each pair visited once and no lower-triangle evaluation.

No other P0, P1, P2, or P3 finding was identified.

## Repair evidence

Repair head `c5b5b8d56c0fae009345c74a0cee425571392ede` addresses only
PERF001-REV-001 through PERF001-REV-004:

- PERF001-REV-001: reusable sparse batch workspaces and their checked logical
  memory now use `CompactNeighborhoodDiagnostics::indexed_terms`, and the
  explicit workspace constructor reserves that complete pre-deduplication
  capacity. A three-center, six-indexed-term regression verifies the corrected
  48-byte scratch estimate, rejection at a limit strictly between the old and
  corrected peaks, and zero allocations on the first batch-into query after
  caller output capacity and workspace construction are established.
- PERF001-REV-002: `try_evaluate_batch_into` clears caller output before
  compatibility validation and explicitly clears it on the checked
  center-count overflow path. The regression first populates output
  successfully, then proves an incompatible workspace returns its structured
  error with empty output.
- PERF001-REV-003: ordinary value, value/gradient, and Hessian APIs construct
  locality-scaled sparse scratch without a retained-global reservation.
  Explicit reusable batch workspaces alone request the complete indexed-term
  capacity. A one-local-hit regression proves identical one-point allocation
  bytes for otherwise equivalent models with 3 and 128 retained centers.
- PERF001-REV-004: the dense block regression now records every evaluator
  invocation and proves exactly `n(n+1)/2` unique visits, no lower-triangle
  call, and the complete deterministic 32-by-32 block order before separately
  checking reflection symmetry.

The repair task passed the eight-test focused performance suite, the
mixed-value/derivative dense-sparse parity regression, and the release
benchmark smoke. The smoke retained exact dense/sparse center visits of 864
and 136 and unchanged checksums at one, two, and four workers.

After the last production or test change, exact repair head `c5b5b8d` passed:

- `cargo fmt --all -- --check`;
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`;
- `cargo test --workspace --all-features`;
- `cargo test --doc --workspace`; and
- `cargo xtask requirements check` for all 58 v1 requirements.

The repair did not mark the PR Ready, merge it, change REQ-PERF-001 from
`in_progress`, or begin another requirement. A fresh isolated re-review must
independently confirm closure and check for new P0--P3 findings.

## Independent truth and validation

The isolated reviewer independently confirmed:

- for block edge 32 and `B = ceil(n / 32)`, the implemented formulae
  `B(B+1)/2` upper blocks, `n(n+1)/2` evaluated entries, and `n(n-1)/2`
  reflected off-diagonals are correct;
- the fixed dense benchmark performs 32,768 center visits and the compact
  benchmark performs 2,175 visits per batch;
- the documented single-term compact-fixture logical peaks of 26,304, 28,032,
  and 31,488 bytes reproduce the current formula, while that fixture does not
  expose the multi-term capacity defect;
- deterministic contiguous result ordering and one/four-worker bit identity
  hold for the tested inputs;
- no global thread pool, hidden regularization, jitter, pseudoinverse,
  densification, constraint relaxation, or backend fallback was added; and
- Rust exports, later-milestone interface N/A dispositions, Draft/Ready CI
  separation, and the `in_progress` registry state are consistent.

The isolated reviewer passed:

- `cargo test -p georbf --test performance`;
- the focused mixed value/derivative sparse parity test;
- `cargo bench -p georbf --bench performance_baseline -- --smoke`; and
- the complete base-to-head whitespace check.

The parent Review task independently passed:

- all six all-feature performance integration tests;
- all seven runnable and 36 compile-fail georbf Rustdoc tests;
- the release benchmark smoke at one, two, and four workers;
- the 58-requirement registry check; and
- the complete base-to-head whitespace check.

Draft CI run 30067909616 passed its configured Ubuntu correctness job on exact
reviewed head `293bcd1`. The Ready-only Windows, Ubuntu, and macOS benchmark
matrix was skipped as designed and is not claimed. Stable implementation head
`236ec26` had already passed the complete standard local gate after the last
production, test, manifest, CI, and registry change. This Review changes only
Markdown review and bounded-handoff evidence, so that immutable gate remains
applicable.

The full workspace gate and full 20-iteration benchmark were not rerun in this
Review task. `cargo-nextest`, `cargo-deny`, `cargo-audit`,
`cargo-semver-checks`, Miri for pinned Rust 1.96.1, sanitizers, executable
fuzzing, mutation testing, API/ABI/schema snapshot checks, and local
`actionlint` remain unavailable or assigned to later requirements. Worker
spawn and worker panic have structured code paths but no deterministic
regression seam; this remains residual coverage risk rather than a separate
finding.

PR #130 remains Draft and REQ-PERF-001 remains `in_progress`. Repair head
`c5b5b8d` and the evidence above supersede the original Repair instruction.
The next task is a fresh isolated re-review; this Repair does not mark the PR
Ready, merge it, or begin another requirement.
