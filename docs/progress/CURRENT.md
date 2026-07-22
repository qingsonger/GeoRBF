# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair complete, fresh independent re-review required
- Requirement: REQ-TREND-002, Issue #108
- Branch: `codex/req-trend-002-region-controls`
- Draft pull request: #109
- Exact independently reviewed head: `4820021`
- Third Repair code/test/evidence head: `8203876`
- Stable full-gate head: `8203876`
- Dependencies: REQ-TREND-001, REQ-PROJECT-001, and REQ-NORMAL-001 are
  integrated
- Registry state in this change: `implemented`

## Third Repair result

- TREND002-REV-007 is repaired pending fresh independent re-review. The public
  compact-control regression now covers both argument orders for the reviewed
  D=1 extreme input through Hessian demand.
- An exactly zero center weight skips the component before fixed-kernel
  evaluation. Every value and query-derivative term contains that factor, so
  the short-circuit preserves exact algebra and restores argument-symmetric
  compact support.
- Rustdoc, the anisotropy contract, and the requirement change fragment state
  the symmetric query-jet/center-factor behavior.

## Validation state

- The new symmetric regression first reproduced
  `NonFiniteTransformedDisplacementComponent` before the production change.
- Focused validation passed all ten public `trend_controls` tests, all five
  private local-trend regressions, warning-denying georbf all-target/all-feature
  Clippy, the runnable example, and the release benchmark smoke.
- After the last Rust change and a rustfmt-only correction, the complete
  standard gate was rerun from the beginning and passed on exact head
  `8203876`: workspace format, warning-denying workspace all-target/all-feature
  Clippy, all-feature workspace tests, workspace Rustdoc, all 58 requirement
  checks, and complete diff whitespace validation.
- The evidence/handoff tail after `8203876` is Markdown-only. Draft PR #109 and
  `implemented` registry state are intentionally retained.

## Next task boundary

A fresh Review task must use an isolated read-only project `math_reviewer` to
verify TREND002-REV-007 on exact Repair head `8203876` and inspect the complete
PR for new P0-P3 findings. If any finding remains, record it and stop without a
production repair. Only after a clean review and green local final-head checks
may that fresh task mark PR #109 Ready, wait for the complete Windows/Ubuntu/
macOS and benchmark-smoke CI on the exact Ready head, merge once, and record
truthful integration state. Do not begin another requirement.

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
