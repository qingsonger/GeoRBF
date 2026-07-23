# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Eleventh Repair complete; fresh independent re-review required
- Requirement: REQ-TREND-002, Issue #108
- Branch: `codex/req-trend-002-region-controls`
- Draft pull request: #109
- Latest independently re-reviewed code/test/contract head: `2a25f44`
- Latest independent-review evidence head: `f5228b5`
- Eleventh Repair code/test head: `0952729`
- Stable full-gate head: `0952729`
- Dependencies: REQ-TREND-001, REQ-PROJECT-001, and REQ-NORMAL-001 are
  integrated
- Registry state in this change: `implemented`

## Eleventh Repair result

- The exact public D=2 compiled-control regression first reproduced
  TREND002-REV-018 on the pre-Repair head: expected approximately
  `-6.035055754270406e-183` for the second gradient, actual exact zero.
- `RadialSeparation` now retains its original represented transformed
  displacement internally. The stable fixed-Gaussian path uses those
  components directly instead of reconstructing them through a normalized
  unit vector whose transverse component can underflow.
- The repaired regression retains the independent gradient within
  `1024 * EPSILON` relative tolerance. Public APIs, formulae, fixed-SPD
  structure, demand bounds, and non-Gaussian paths are unchanged.
- This Repair records evidence but does not independently close
  TREND002-REV-018. PR #109 remains Draft and the requirement remains
  `implemented`.

## Validation state

- Focused validation passed the exact regression, all twenty public
  `trend_controls` tests, all fifteen public `local_trend` integration tests,
  all five private local-trend regressions, all eleven kernel-calculus tests,
  all thirteen anisotropy tests, and complete diff whitespace validation.
- Exact stable head `0952729` passed workspace format, warning-denying workspace
  all-target/all-feature Clippy, all-feature workspace tests, all 35 workspace
  Rustdoc tests, all 58 requirement checks, and complete diff whitespace
  validation.
- The evidence tail after `0952729` changes only the requirement change
  fragment, independent-review record, and bounded Markdown handoff. Ready-only
  Windows/Ubuntu/macOS and benchmark-smoke CI remain intentionally unexecuted
  while PR #109 is Draft.

## Next task boundary

A fresh Review/re-review task must create an isolated read-only project
`math_reviewer` to independently verify TREND002-REV-018 against exact Repair
head `0952729` and inspect the complete PR diff for new P0-P3 findings. It must
not inherit this Repair reasoning or repair production code. If findings
remain, record them and stop. Only a clean re-review may continue through the
repository's mandatory Ready, exact-head three-platform and benchmark-smoke CI,
and single-merge integration sequence. Do not begin another requirement.

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
