# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Sixth Repair re-review complete, Repair required
- Requirement: REQ-TREND-002, Issue #108
- Branch: `codex/req-trend-002-region-controls`
- Draft pull request: #109
- Sixth Repair code/test/contract head: `cc5fa6f`
- Stable full-gate head: `cc5fa6f`
- Dependencies: REQ-TREND-001, REQ-PROJECT-001, and REQ-NORMAL-001 are
  integrated
- Registry state in this change: `implemented`

## Sixth Repair re-review result

- Fresh independent re-review closes TREND002-REV-012 for its exact published
  value, gradient, and Hessian input. TREND002-REV-007 through
  TREND002-REV-011 remain closed.
- The re-review found one new P1, TREND002-REV-013: the fixed-Gaussian path
  invokes the generic represented kernel jet before its stable jet, so an
  individually overflowing anisotropic Hessian can be rejected even when two
  small weights make the complete contribution finite.
- The exact accepted D=1 reproducer uses Gaussian length `1e-100`, metric
  lengths `1e-154`, strength `1e-154`, control/query zero, and center
  `5e-255`. All demand orders currently return `NonFiniteSecondDerivative`,
  while the complete Hessian is approximately `-6.62e199`.
- PR #109 remains Draft and the requirement remains `implemented`.

## Validation state

- The fresh reviewer passed all fourteen public `trend_controls` tests, all
  fifteen `local_trend` integration tests, all five private local-trend
  regressions, complete PR and sixth-Repair diff whitespace validation, and
  compact requirement `show` and dependency-closure checks.
- Exact stable head `cc5fa6f` passed workspace format, warning-denying workspace
  all-target/all-feature Clippy, all-feature workspace tests, workspace Rustdoc,
  all 58 requirement checks, and complete diff whitespace validation.
- The evidence tail after `cc5fa6f` changes only this review record and bounded
  Markdown handoff. Ready-only Windows/Ubuntu/macOS and benchmark-smoke CI
  remain intentionally unexecuted while PR #109 is Draft.

## Next task boundary

A fresh Repair task must address only TREND002-REV-013. Add the exact public
D=1 regression before the smallest production repair: `Value` demand must not
evaluate the unused overflowing Hessian, and `Second` demand must retain the
independently finite complete Hessian after both weights. Route Gaussian
evaluation through the demand-bounded stable jet before any individually
represented derivative is required, run focused checks and one final stable-
head standard gate, update evidence, push, and stop for another fresh
independent re-review. Do not mark the PR ready, merge, or begin another
requirement.

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
