# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review / clean fresh re-review; Ready transition pending
- Requirement: REQ-PERF-001, dense and sparse performance baseline
- Issue: #129
- Branch: `codex/req-perf-001-performance-baseline`
- Pull request: #130 (Draft; clean re-review authorizes Ready transition)
- Repair head: `c5b5b8d`
- Re-reviewed evidence head: `b295e9d`
- Stable implementation gate head: `c5b5b8d`
- Dependencies: REQ-SPARSE-001, REQ-CENTER-001, and REQ-TUNE-001 are integrated
- Registry status: `in_progress`

## Fresh re-review result

An isolated read-only project `math_reviewer` reviewed the complete base
`01b9fa5` through evidence head `b295e9d` and the focused repair
`293bcd1..c5b5b8d`. It found no remaining or new P0--P3 issue and independently
closed all four original findings:

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
- The fresh isolated re-review independently passed the same eight performance
  tests, focused mixed value/derivative sparse parity test, release benchmark
  smoke, and complete base-to-head whitespace check.
- Exact stable implementation head `c5b5b8d` passed formatting, all-target and
  all-feature Clippy with warnings denied, the complete all-feature workspace
  test suite, all workspace Rustdoc tests, and the 58-requirement registry
  check after the final production and test change.
- The parent re-review task repeated that complete standard gate successfully
  on exact evidence head `b295e9d`.
- Draft CI run 30070320531 passed its Ubuntu correctness job on exact evidence
  head `b295e9d`.
- The Ready-only Windows, Ubuntu, and macOS benchmark matrix has not run on the
  final re-review evidence head and is not claimed.

## Next task boundary

Push this documentation-only clean re-review evidence, mark PR #130 Ready, and
wait for the Windows, Ubuntu, and macOS correctness plus complete
benchmark-smoke CI on that exact Ready head. Merge exactly once only if the
whole matrix is green, then record truthful integration state through an
isolated documentation-only change. Do not begin another requirement.

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
