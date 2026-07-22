# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Sixth Repair complete, fresh independent re-review required
- Requirement: REQ-TREND-002, Issue #108
- Branch: `codex/req-trend-002-region-controls`
- Draft pull request: #109
- Sixth Repair code/test/contract head: `cc5fa6f`
- Stable full-gate head: `cc5fa6f`
- Dependencies: REQ-TREND-001, REQ-PROJECT-001, and REQ-NORMAL-001 are
  integrated
- Registry state in this change: `implemented`

## Sixth Repair result

- TREND002-REV-012 is addressed by retaining fixed-Gaussian value, gradient,
  and Hessian as analytic signed stable factors until each complete mixture
  term is formed with both weights.
- The existing fixed-Gaussian-underflow regression now demands Hessian order
  and independently checks the approximately `5.23e-23` value,
  `2.04e-21` gradient, and `7.95e-20` Hessian. Before the repair, the new
  gradient assertion reproduced actual zero.
- The Repair preserves the exact compact-support short-circuits and all prior
  TREND002-REV-007 through TREND002-REV-011 regressions. Its evidence does not
  close TREND002-REV-012 without a fresh independent re-review.
- PR #109 remains Draft and the requirement remains `implemented`.

## Validation state

- Focused validation passed all fourteen public `trend_controls` tests, all
  fifteen `local_trend` integration tests, all five private local-trend
  regressions, warning-denying georbf all-target/all-feature Clippy, complete
  diff whitespace validation, and the release-mode benchmark smoke at about
  8.7/36.0 us for four/sixteen controls.
- Exact stable head `cc5fa6f` passed workspace format, warning-denying workspace
  all-target/all-feature Clippy, all-feature workspace tests, workspace Rustdoc,
  all 58 requirement checks, and complete diff whitespace validation.
- The evidence tail after `cc5fa6f` changes only this review record and bounded
  Markdown handoff. Ready-only Windows/Ubuntu/macOS and benchmark-smoke CI
  remain intentionally unexecuted while PR #109 is Draft.

## Next task boundary

A fresh isolated read-only `math_reviewer` must re-review exact sixth Repair
code/test/contract head `cc5fa6f`, verify TREND002-REV-012 against its published
value/gradient/Hessian input, retain the prior closures, inspect for new P0-P3
findings, record the result, and stop. Do not repair production code in that
Review task. Do not mark the PR ready, merge, or begin another requirement.

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
