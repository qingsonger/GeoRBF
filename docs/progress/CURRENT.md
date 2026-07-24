# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review complete; Repair required
- Requirement: REQ-PERF-001, dense and sparse performance baseline
- Issue: #129
- Branch: `codex/req-perf-001-performance-baseline`
- Draft pull request: #130
- Reviewed head: `293bcd1`
- Stable implementation gate head: `236ec26`
- Dependencies: REQ-SPARSE-001, REQ-CENTER-001, and REQ-TUNE-001 are integrated
- Registry status: `in_progress`

## Independent review result

The fresh isolated read-only `math_reviewer` found four defects on exact head
`293bcd1`:

- P1 PERF001-REV-001: sparse workspace capacity uses retained centers, but the
  R-tree yields atomic terms before center-ID deduplication. Multi-term centers
  can therefore allocate during a query and exceed reported scratch bytes.
- P2 PERF001-REV-002: `try_evaluate_batch_into` returns an incompatible-
  workspace error before clearing previously populated caller output.
- P2 PERF001-REV-003: the shared scratch constructor reserves all sparse
  centers for ordinary one-point APIs, regressing local evaluation to
  global-size allocation.
- P3 PERF001-REV-004: the dense exact-work regression checks a closed-form
  diagnostic and symmetry but never records actual evaluator visits.

No other P0--P3 finding was identified. Complete scenarios, exact lines,
independent truth, and required regressions are in
`docs/reviews/PR-130-INDEPENDENT-REVIEW.md`.

## Validation state

- The isolated reviewer and parent Review task independently passed the focused
  performance suite and release benchmark smoke. The parent also passed all
  georbf Rustdoc tests, the 58-requirement registry check, and the complete PR
  whitespace check.
- Draft CI run 30067909616 passed Ubuntu on exact reviewed head `293bcd1`.
  The Ready-only Windows, Ubuntu, and macOS benchmark matrix was skipped as
  designed and is not claimed.
- Stable implementation head `236ec26` passed the complete standard local gate
  after the final production, test, manifest, CI, and registry change.
- This Review changes only Markdown review and bounded-handoff evidence, so the
  immutable implementation-head gate remains applicable.

## Next task boundary

Start a fresh Repair task for only PR #130 findings PERF001-REV-001 through
PERF001-REV-004. Add the specified independent regressions before the smallest
production repair. Run focused checks during iteration, then one complete
standard workspace gate after the last production or test change. Update the
review record and this bounded handoff, commit, push, and stop for a fresh
isolated re-review.

Do not mark PR #130 Ready, merge it, change REQ-PERF-001 to `integrated`, or
begin another requirement in the Repair task.

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
