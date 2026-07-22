# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review complete, fresh Repair required
- Requirement: REQ-TREND-002, Issue #108
- Branch: `codex/req-trend-002-region-controls`
- Draft pull request: #109
- Latest reviewed code/test/contract head: `42c5686`
- Exact reviewed evidence head: `b0ff092`
- Stable full-gate head: `42c5686`
- Dependencies: REQ-TREND-001, REQ-PROJECT-001, and REQ-NORMAL-001 are
  integrated
- Registry state in this change: `implemented`

## Review result

- A fresh isolated read-only `math_reviewer` closed TREND002-REV-013 for its
  published D=1 regression. Gaussian evaluation now enters the demand-bounded
  stable jet without a generic represented-derivative preflight, and retains
  the independently derived approximately `-6.62e199` complete Hessian.
- New P1 TREND002-REV-014 remains. For a D=1 regional control on the plateau of
  `[-2, 2]`, control `-2^-53`, query/center one, unit strength/radius/metric,
  fixed Gaussian length `1e100`, and negligible strict background, binary64
  subtraction rounds the displacement to one. The weight Hessian then loses
  the exact subtraction residual and returns approximately `-3.68e-201`
  instead of the independent positive truth `8.168564517495419e-17`.
- No other P0-P3 finding remains.
- PR #109 remains Draft and the requirement remains `implemented`.

## Validation state

- This Review passed all fifteen public `trend_controls` tests, all fifteen
  `local_trend` integration tests, all five private local-trend regressions,
  and complete PR diff whitespace validation. The isolated reviewer separately
  reran the exact REV-013 regression.
- Exact stable head `42c5686` passed workspace format, warning-denying workspace
  all-target/all-feature Clippy, all-feature workspace tests, workspace Rustdoc,
  all 58 requirement checks, and complete diff whitespace validation.
- The evidence tail after `42c5686` changes only this review record and bounded
  Markdown handoff. Ready-only Windows/Ubuntu/macOS and benchmark-smoke CI
  remain intentionally unexecuted while PR #109 is Draft.

## Next task boundary

Open a fresh Repair task for TREND002-REV-014 only. First add the exact public
D=1 plateau regression recorded in the review and reproduce the tiny negative
Hessian against the independent positive approximately `8.16856e-17` truth.
Then implement the smallest residual-aware Gaussian displacement repair, run
focused checks and one complete stable-head standard gate after the last code
change, update review evidence and this bounded handoff, push, and stop for a
fresh independent re-review. Do not mark PR #109 ready, merge it, or begin
another requirement.

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
