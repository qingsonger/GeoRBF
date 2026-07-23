# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review / REQ-CENTER-001 clean fresh re-review; Ready CI pending
- Requirement: REQ-CENTER-001, open Issue #120
- Branch: `codex/req-center-001-rank-safe-centers`
- Pull request: #121 (Draft; clean re-review authorizes Ready transition)
- Dependencies: REQ-SOLVE-001 and REQ-MODEL-001 are integrated
- Registry status: `planned`

## Fresh re-review outcome

- An isolated read-only project `math_reviewer` reviewed base `aa128ed8`
  through exact Repair head `75110a5`.
- CENTER001-REV-001 through CENTER001-REV-004 are independently closed.
- The complete repaired 13-file PR diff has no remaining or new P0--P3
  finding.

## Review validation

- The isolated reviewer and parent Review task independently passed all 13
  center-selection tests, the center-selection rustdoc example, the
  five-strategy release benchmark smoke, the 58-requirement registry check,
  and the complete PR whitespace check.
- The stable Repair tree passed the complete standard gate: format,
  warning-denying workspace/all-target/all-feature Clippy, all-feature
  workspace tests, workspace doctests, and the 58-requirement registry check.
- Draft CI run 30009925065 passed Ubuntu on exact Repair head `75110a5`.
- The re-review evidence tail changes only this review record and bounded
  handoff, so the immutable stable-head complete gate remains valid.

## Next task boundary

Push this documentation-only clean re-review evidence, mark PR #121 Ready, and
wait for the Windows/Ubuntu/macOS and complete benchmark-smoke CI on that exact
Ready head. Merge exactly once only if the whole matrix is green, then record
the truthful integration state through an isolated documentation-only change.
Do not begin REQ-TUNE-001.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #120
- Draft implementation: GitHub PR #121
- Independent review: `docs/reviews/PR-121-INDEPENDENT-REVIEW.md`
- Requirement summary: `changes/REQ-CENTER-001.md`
- Architecture: `docs/architecture/ARCHITECTURE.md`
- Numerical policy: `docs/architecture/SOLVER_POLICY.md`
- Benchmark: `docs/benchmarks/REQ-CENTER-001.md`
- Production implementation: `crates/georbf/src/center_selection.rs`
- Independent tests: `crates/georbf/tests/center_selection.rs`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, and API/ABI/schema snapshot checks are tracked by
later requirements and release gates. Local `actionlint` is unavailable. No
unavailable check is claimed as passed.
