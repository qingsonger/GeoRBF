# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Seventh Repair complete, fresh independent re-review required
- Requirement: REQ-TREND-002, Issue #108
- Branch: `codex/req-trend-002-region-controls`
- Draft pull request: #109
- Seventh Repair code/test/contract head: `42c5686`
- Stable full-gate head: `42c5686`
- Dependencies: REQ-TREND-001, REQ-PROJECT-001, and REQ-NORMAL-001 are
  integrated
- Registry state in this change: `implemented`

## Seventh Repair result

- TREND002-REV-013 is repaired at exact code/test/contract head `42c5686`,
  pending fresh independent re-review. Gaussian evaluation now enters the
  demand-bounded stable jet before any generic represented derivative is
  required; non-Gaussian evaluation and error mapping are unchanged.
- A public D=1 regression uses fixed Gaussian length `1e-100`, condition-one
  metric lengths `1e-154`, strength `1e-154`, control/query zero, and center
  `5e-255`. `Value` demand succeeds without evaluating the unused overflowing
  Hessian, and `Second` demand retains the independently log-evaluated finite
  Hessian of approximately `-6.62e199` after both weights.
- PR #109 remains Draft and the requirement remains `implemented`.

## Validation state

- Focused checks passed all fifteen public `trend_controls` tests, all fifteen
  `local_trend` integration tests, all five private local-trend regressions,
  and complete diff whitespace validation.
- Exact stable head `42c5686` passed workspace format, warning-denying workspace
  all-target/all-feature Clippy, all-feature workspace tests, workspace Rustdoc,
  all 58 requirement checks, and complete diff whitespace validation.
- The evidence tail after `42c5686` changes only this review record and bounded
  Markdown handoff. Ready-only Windows/Ubuntu/macOS and benchmark-smoke CI
  remain intentionally unexecuted while PR #109 is Draft.

## Next task boundary

A fresh Review task must independently re-review exact Repair head `42c5686`,
confirm TREND002-REV-013 is closed for its published regression, and search for
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
