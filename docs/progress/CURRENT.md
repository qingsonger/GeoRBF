# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Implement
- Requirement: REQ-PERF-001
- Issue: #129
- Branch: `codex/req-perf-001-performance-baseline`
- Draft pull request: pending initial push
- Dependencies: REQ-SPARSE-001, REQ-CENTER-001, and REQ-TUNE-001 are integrated
- Registry status in this change: `in_progress`

## Implemented scope

- Dense all-representer assembly uses deterministic fixed 32-by-32
  upper-triangle blocks, evaluates each kernel pair once, reflects each
  off-diagonal entry once, and records block and work-count evidence.
- `FittedFieldEvaluationWorkspace<D>` supports allocation-stable reusable
  serial value/gradient batches for D=1, D=2, and D=3.
- `BatchEvaluationOptions` requires explicit nonzero worker and logical-memory
  limits. Scoped workers own isolated scratch, use deterministic contiguous
  ranges, preserve input order, and never configure a global thread pool.
- Batch diagnostics record exact center visits and checked output, per-worker
  workspace, total workspace, logical peak, and caller-limit bytes.
- Sparse workspaces reserve complete center-index capacity before evaluation;
  exact compact-support filtering therefore performs no per-query allocation.
- `georbf.performance.v1` is a fixed CSV benchmark schema for dense and sparse
  D=3 value/gradient batches at one, two, and four workers.

## Validation evidence

- Six independent performance integration tests pass: D=1/D=2/D=3 parity,
  one/four-worker bit identity, exact cross-block upper-triangle counts and
  symmetry, sparse locality, memory preflight, empty batches, workspace
  compatibility, zero-allocation warmed reuse, and allocation-count
  independence from query count.
- Focused warning-denying all-target/all-feature Clippy passes.
- The new release benchmark smoke passes and emits identical checksums at one,
  two, and four workers for both dense and sparse workloads.
- Four consecutive full local benchmark runs and their environment, ranges,
  memory, center visits, checksums, and directional scaling are recorded in
  `docs/benchmarks/REQ-PERF-001.md`.
- The 58-requirement registry check passes.
- After the last production and test change, the stable implementation tree
  passed the complete standard local gate: format, warning-denying
  workspace/all-target/all-feature Clippy, all-feature workspace tests,
  workspace doctests, and the 58-requirement registry check.

## Next task boundary

Finish only this Implement task: commit, push, open a Draft PR for Issue #129,
link its exact head here and in the registry, validate the evidence-only
follow-up, push, and stop.

The next fresh task must be Review mode for only the resulting REQ-PERF-001
Draft PR. It must use an isolated read-only project `math_reviewer` because the
change affects numerical assembly traversal and performance-sensitive
evaluation. Do not repair production code or begin another requirement in
that Review task.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #129
- Requirement summary: `changes/REQ-PERF-001.md`
- Benchmark and allocation evidence: `docs/benchmarks/REQ-PERF-001.md`
- Production batch implementation: `crates/georbf/src/performance.rs`
- Dense block assembly: `crates/georbf/src/field.rs`
- Independent tests: `crates/georbf/tests/performance.rs`
- Release benchmark: `crates/georbf/benches/performance_baseline.rs`
- Dense backend policy: `docs/adr/ADR-0010-nalgebra-dense-factorization-backend.md`
- Sparse backend policy: `docs/adr/ADR-0012-rstar-faer-compact-sparse-backends.md`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, and API/ABI/schema snapshot checks are tracked by
later requirements and release gates. Local `actionlint` is unavailable. No
unavailable check is claimed as passed.
