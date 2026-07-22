# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Independent re-review complete, fresh Repair required
- Requirement: REQ-TREND-002, Issue #108
- Branch: `codex/req-trend-002-region-controls`
- Draft pull request: #109
- Latest reviewed code/test/contract head: `144a018`
- Reviewed evidence head: `d516be7`
- Stable full-gate head: `144a018`
- Dependencies: REQ-TREND-001, REQ-PROJECT-001, and REQ-NORMAL-001 are
  integrated
- Registry state in this change: `implemented`

## Independent re-review result

- A fresh isolated read-only `math_reviewer` closed TREND002-REV-015 for its
  exact regional D=1 regression and found no P0, P2, or P3 issue.
- TREND002-REV-016 is a new P1: the non-regional Gaussian-weight path still
  discards the exact subtraction residual and returns approximately `-0.618`
  instead of the independent positive approximately
  `1.2750102220326992e128` Hessian for the same accepted inputs with no region.
- TREND002-REV-017 is a new P1: the stable fixed-Gaussian path squares a
  represented reciprocal length too early. With accepted length `1e200` and
  two `1e154` spatial weights it returns only the `-2^-1074` background term
  instead of the independently finite approximately `-3.67879441171431e-93`
  complete Hessian.
- PR #109 remains Draft and REQ-TREND-002 remains `implemented`.

## Validation state

- Focused checks passed all seventeen public `trend_controls` tests, all
  fifteen `local_trend` integration tests, all five private local-trend
  regressions, the exact REV-015 regression, and complete diff whitespace
  validation.
- The reviewer and parent Review task independently reproduced both new
  findings. The parent's temporary public-API regressions were removed, and
  the worktree was restored before review evidence was recorded.
- Exact stable head `144a018` passed workspace format, warning-denying workspace
  all-target/all-feature Clippy, all-feature workspace tests, workspace Rustdoc,
  all 58 requirement checks, and complete diff whitespace validation.
- The reviewed evidence tail from `144a018` through `d516be7` changes only the
  requirement change fragment, independent-review record, and bounded Markdown
  handoff. Draft Ubuntu CI run 29931521124 passed on `d516be7`. Ready-only
  Windows/Ubuntu/macOS and benchmark-smoke CI remain intentionally unexecuted.

## Next task boundary

A fresh Repair task must address only TREND002-REV-016 and TREND002-REV-017.
Add both exact public D=1 regressions before the smallest residual-aware
non-regional weight repair and complete-term fixed-Gaussian reciprocal-scale
repair. Run focused checks and one final standard workspace gate after the last
code change, update evidence, push, and stop for another fresh independent
re-review. Do not mark PR #109 Ready, merge it, or begin another requirement.

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
