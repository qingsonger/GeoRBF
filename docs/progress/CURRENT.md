# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Integration state / REQ-PERF-001 implementation merged
- Requirement: REQ-PERF-001, dense and sparse performance baseline
- Issue: #129, closed
- Implementation branch: `codex/req-perf-001-performance-baseline`
- Implementation pull request: #130, squash-merged as `bee47fbf`
- Integration-state branch: `codex/req-perf-001-integration-state`
- Integration-state pull request: #131 (Draft until final exact-head gates pass)
- Exact implementation Ready head: `7c36721`
- Repair head: `c5b5b8d`
- Re-reviewed evidence head: `b295e9d`
- Stable implementation gate head: `c5b5b8d`
- Dependencies: REQ-SPARSE-001, REQ-CENTER-001, and REQ-TUNE-001 are integrated
- Registry status in this change: `integrated`

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
- Exact Ready CI run 30071031648 passed the complete Windows, Ubuntu, and macOS
  workspace matrix on exact head `7c36721`, including all configured
  correctness checks, backend combinations, requirement validation, and every
  benchmark smoke.
- PR #130 was squash-merged exactly once as `bee47fbf`; Issue #129 closed as
  completed.
- Post-merge `main` CI run 30072182895 passed the same complete three-platform
  gate on exact merge commit `bee47fbf`.
- This isolated integration-state change modifies only the registry, completed
  history index, independent review evidence, and bounded handoff.

## Next task boundary

Commit and push the linked integration evidence after the complete standard
local gate passes on the final integration-state tree. Mark PR #131 Ready,
wait for exact-head Windows, Ubuntu, and macOS correctness plus complete
benchmark-smoke CI, and merge only if green. Then stop. Do not begin another
requirement.

## Durable evidence

- Acceptance criteria and exclusions: closed GitHub Issue #129
- Merged implementation: GitHub PR #130
- Integration-state pull request: GitHub PR #131
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
