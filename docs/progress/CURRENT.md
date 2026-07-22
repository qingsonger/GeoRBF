# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Ninth Repair complete, fresh independent re-review required
- Requirement: REQ-TREND-002, Issue #108
- Branch: `codex/req-trend-002-region-controls`
- Draft pull request: #109
- Ninth Repair code/test/contract head: `144a018`
- Stable full-gate head: `144a018`
- Dependencies: REQ-TREND-001, REQ-PROJECT-001, and REQ-NORMAL-001 are
  integrated
- Registry state in this change: `implemented`

## Ninth Repair result

- TREND002-REV-015 is repaired at exact code/test/contract head `144a018`,
  pending fresh independent re-review. The residual-aware factors `d-r` and
  `d+r` are each combined with one inverse-radius-square factor before their
  product, so neither the intermediate curvature nor the complete Hessian is
  erased by underflow.
- The exact public D=1 regional-plateau regression uses radius `2^-500`,
  control `-2^-1074`, query and center `2^-500`, and the reviewed unit kernel,
  metric, strength, region, and constant-`0.5` background. It reproduces the
  old approximately `-0.618` result before the repair and now retains the
  independent positive approximately `1.2750102220326992e128` Hessian.
- PR #109 remains Draft and the requirement remains `implemented`.

## Validation state

- Focused checks passed all seventeen public `trend_controls` tests, all
  fifteen `local_trend` integration tests, all five private local-trend
  regressions, the exact REV-015 regression, and complete diff whitespace
  validation.
- Exact stable head `144a018` passed workspace format, warning-denying workspace
  all-target/all-feature Clippy, all-feature workspace tests, workspace Rustdoc,
  all 58 requirement checks, and complete diff whitespace validation.
- The evidence tail after `144a018` changes only the requirement change
  fragment, independent-review record, and bounded Markdown handoff. Ready-only
  Windows/Ubuntu/macOS and benchmark-smoke CI remain intentionally unexecuted
  while PR #109 is Draft.

## Next task boundary

A fresh Review task must independently re-review exact Repair head `144a018`,
confirm TREND002-REV-015 is closed for its published regression, and search for
new P0-P3 findings without inheriting this Repair reasoning. If any finding
remains, record it and stop without production repair. If the review is clean
and the stable local gate remains valid, follow the mandatory sequence: update
review evidence, mark PR #109 ready, wait for complete Windows/Ubuntu/macOS and
benchmark-smoke CI on that exact ready head, merge only when all are green,
then record truthful integration state. Do not begin another requirement.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #108
- Draft implementation: GitHub PR #109
- Independent review, findings, and Repair evidence:
  `docs/reviews/PR-109-INDEPENDENT-REVIEW.md`
- Requirement summary and benchmark baseline: `changes/REQ-TREND-002.md`
- Public implementation and Rustdoc: `crates/georbf/src/trend_controls.rs`,
  `crates/georbf/src/local_trend.rs`
- Independent property/error tests: `crates/georbf/tests/trend_controls.rs`
- Runnable example: `crates/georbf/examples/trend_controls.rs`
- Focused benchmark: `crates/georbf/benches/trend_control_compilation.rs`
- Mathematical contract: `docs/architecture/ANISOTROPY.md`, ADR-0005, ADR-0008

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, and API/ABI/schema snapshot checks are tracked by
later requirements and release gates. Local `actionlint` is unavailable. No
unavailable check is claimed as passed.
