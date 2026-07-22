# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review complete, fresh Repair required
- Requirement: REQ-TREND-002, Issue #108
- Branch: `codex/req-trend-002-region-controls`
- Draft pull request: #109
- Latest reviewed code/test/contract head: `d42ccb5`
- Exact reviewed evidence head: `473f831`
- Stable full-gate head: `d42ccb5`
- Dependencies: REQ-TREND-001, REQ-PROJECT-001, and REQ-NORMAL-001 are
  integrated
- Registry state in this change: `implemented`

## Review result

- A fresh isolated read-only `math_reviewer` closed TREND002-REV-014 for its
  exact public D=1 plateau regression. The repaired path returns the independent
  positive `8.168564517495419e-17` truth instead of the prior approximately
  `-3.68e-201` result.
- New P1 TREND002-REV-015 remains. With radius `2^-500`, displacement residual
  `2^-1074`, and both arguments on a region plateau, the residual-aware
  diagonal factor `(d-r)(d+r)` underflows before later `r^-4` scaling. The
  implementation returns approximately `-0.618` instead of the independently
  representable positive approximately `1.27501e128` complete Hessian.
- No other P0-P3 finding remains.
- PR #109 remains Draft and the requirement remains `implemented`.

## Validation state

- This Review passed all sixteen public `trend_controls` tests, all fifteen
  `local_trend` integration tests, all five private local-trend regressions,
  the exact REV-014 regression, compact requirement show/dependency checks,
  and complete PR diff whitespace validation.
- Exact stable head `d42ccb5` passed workspace format, warning-denying workspace
  all-target/all-feature Clippy, all-feature workspace tests, workspace Rustdoc,
  all 58 requirement checks, and complete diff whitespace validation.
- The evidence tail after `d42ccb5` changes only the requirement change
  fragment, independent-review record, and bounded Markdown handoff. Ready-only
  Windows/Ubuntu/macOS and benchmark-smoke CI remain intentionally unexecuted
  while PR #109 is Draft.

## Next task boundary

Open a fresh Repair task for TREND002-REV-015 only. First add the exact public
D=1 regional plateau regression recorded in the review: radius `2^-500`,
control `-2^-1074`, query and center `2^-500`, unit strength/metric/fixed
Gaussian length, region `[-1, 1]`, transition width `0.25`, and constant-`0.5`
strict Gaussian background. Reproduce the incorrect negative Hessian against
the independent positive approximately `1.2750102220326992e128` truth. Then
implement the smallest repair that keeps diagonal curvature recoverable until
the two inverse-radius-square factors are applied, run focused checks and one
complete stable-head standard gate after the last code change, update evidence,
push, and stop for a fresh independent re-review. Do not mark PR #109 ready,
merge it, or begin another requirement.

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
