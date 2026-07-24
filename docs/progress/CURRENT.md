# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair complete; fresh isolated re-review required
- Requirement: REQ-PERF-001, dense and sparse performance baseline
- Issue: #129
- Branch: `codex/req-perf-001-performance-baseline`
- Draft pull request: #130
- Repair head: `c5b5b8d`
- Reviewed head: `293bcd1`
- Stable implementation gate head: `c5b5b8d`
- Dependencies: REQ-SPARSE-001, REQ-CENTER-001, and REQ-TUNE-001 are integrated
- Registry status: `in_progress`

## Repair result

Exact repair head `c5b5b8d` addresses only the four findings recorded by the
fresh isolated review of `293bcd1`:

- PERF001-REV-001: batch workspace capacity and logical memory now use the
  complete atomic indexed-term count. A multi-term regression checks the
  corrected estimate, pre-allocation limit rejection, and zero first-query
  allocation with caller capacity already established.
- PERF001-REV-002: every fallible batch-into exit clears caller output,
  including incompatible workspaces and checked center-count overflow.
- PERF001-REV-003: one-point sparse APIs use locality-scaled scratch, while
  explicit reusable batch workspaces reserve the complete index capacity.
- PERF001-REV-004: the dense block test records every real evaluator visit and
  proves unique upper-triangle work in deterministic block order.

Complete original findings, repair mapping, independent truth, and validation
evidence are in
`docs/reviews/PR-130-INDEPENDENT-REVIEW.md`.

## Validation state

- Focused repair validation passed all eight performance tests, the
  mixed-value/derivative sparse parity test, and release benchmark smoke with
  unchanged deterministic center visits and checksums.
- Exact stable implementation head `c5b5b8d` passed formatting, all-target and
  all-feature Clippy with warnings denied, the complete all-feature workspace
  test suite, all workspace Rustdoc tests, and the 58-requirement registry
  check after the final production and test change.
- Draft CI run 30069460773 is remote evidence for the pre-repair review-record
  head only and is not claimed for `c5b5b8d`. The repair push is expected to
  trigger a new Draft Ubuntu correctness run; its result is not yet claimed.
- The Ready-only Windows, Ubuntu, and macOS benchmark matrix has not run on the
  repair head and is not claimed.

## Next task boundary

Start a fresh isolated Review task for only PR #130 and REQ-PERF-001. Supply
the reviewer the bounded requirement/dependency summary, normative documents,
base-to-repair diff, original findings, and exact validation evidence without
the Repair reasoning transcript. Independently confirm PERF001-REV-001 through
PERF001-REV-004 are closed and check for new P0--P3 findings.

If any finding remains, record it and stop without repairing production code.
Only after a clean isolated re-review and a green complete local gate may that
fresh Review task mark PR #130 Ready, wait for exact-head Windows, Ubuntu, and
macOS correctness plus benchmark-smoke CI, merge exactly once, and record
truthful integration state. Do not begin another requirement in that task.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #129
- Draft implementation: GitHub PR #130
- Independent review: `docs/reviews/PR-130-INDEPENDENT-REVIEW.md`
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
