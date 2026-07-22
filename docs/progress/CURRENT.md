# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review complete, fresh Repair required
- Requirement: REQ-TREND-002, Issue #108
- Branch: `codex/req-trend-002-region-controls`
- Draft pull request: #109
- Exact independently reviewed fourth Repair head: `accad99`
- Stable full-gate head: `accad99`
- Dependencies: REQ-TREND-001, REQ-PROJECT-001, and REQ-NORMAL-001 are
  integrated
- Registry state in this change: `implemented`

## Fresh fourth-Repair re-review result

- TREND002-REV-008 is closed for its exact published value counterexample and
  both argument orders. TREND002-REV-007 remains closed for exact compact query
  and center factors through Hessian demand.
- P1 TREND002-REV-009: opposite-sign factors with equal rounded binary64 log
  magnitudes are incorrectly marked as mathematically exact cancellation. A
  valid compiled regional gradient loses a finite approximately `1.87e211`
  contribution.
- P1 TREND002-REV-010: an individually overflowing query-weight Hessian is
  rejected before a small center weight makes the complete mixture Hessian a
  finite approximately `-4.38e285`.
- P1 TREND002-REV-011: a fixed Gaussian kernel value that merely underflows is
  marked mathematically exact zero. Valid large weight factors should recover
  an independently derived local value of approximately `5.23e-23`.
- No additional P0, P2, or P3 finding was identified. PR #109 remains Draft
  and the registry remains `implemented`.

## Validation state

- The isolated reviewer passed the exact TREND002-REV-008 and retained
  TREND002-REV-007 focused regressions plus all five private `local_trend`
  regressions, and independently derived the three new counterexamples.
- The parent Review task independently passed all eleven public
  `trend_controls` tests, all five private `local_trend` regressions, workspace
  format, all 58 requirement checks, and working-diff whitespace validation.
- Exact stable head `accad99` passed workspace format, warning-denying workspace
  all-target/all-feature Clippy, all-feature workspace tests, workspace Rustdoc,
  all 58 requirement checks, and complete diff whitespace validation.
- The evidence tail after `accad99` changes only this review record and the
  bounded Markdown handoff. Draft Ubuntu CI passed on evidence head `d8c5a9e`;
  Ready-only Windows/Ubuntu/macOS and benchmark-smoke CI remain intentionally
  unexecuted while PR #109 is Draft. The full workspace gate was not rerun
  because no production, test, manifest, schema, CI, or build input changed.

## Next task boundary

A fresh Repair task must address only TREND002-REV-009,
TREND002-REV-010, and TREND002-REV-011. It must add the specified public D=1
regional-gradient, rescued-Hessian, and fixed-kernel-underflow regressions
before the smallest production changes; retain exact compact-support
short-circuits and REV-008; run focused checks and one final complete standard
gate on the stable head; update the review record and bounded handoff; push;
and stop for a fresh independent re-review. Do not mark the PR ready, merge,
or begin another requirement.

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
