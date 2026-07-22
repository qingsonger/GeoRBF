# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair complete, fresh independent re-review required
- Requirement: REQ-TREND-002, Issue #108
- Branch: `codex/req-trend-002-region-controls`
- Draft pull request: #109
- Exact independently reviewed head: `3a8ba8f`
- Fourth Repair code/test/contract head: `accad99`
- Stable full-gate head: `accad99`
- Dependencies: REQ-TREND-001, REQ-PROJECT-001, and REQ-NORMAL-001 are
  integrated
- Registry state in this change: `implemented`

## Fourth Repair result

- This Repair addresses only P1 TREND002-REV-008. A public compiled D=1
  regression uses the reviewed strength, radius, fixed-kernel length, query,
  and center in both argument orders, isolates the local term with an
  underflowed background value, and compares against independent logarithmic
  truth.
- Weight jets retain signed logarithmic scale and mathematical exact-zero
  provenance through complete mixture value, gradient, and Hessian products.
  An individually underflowed Gaussian factor therefore no longer erases a
  representable combined contribution.
- Fixed-kernel evaluation is skipped only for a mathematically exact compact
  query jet or center factor. The symmetric TREND002-REV-007 regression remains
  green through Hessian demand.
- PR #109 remains Draft and the registry remains `implemented`; this Repair
  does not close its own finding.

## Validation state

- Before production repair, the new regression reproduced expected
  `1.87835170036433494e-172` versus actual zero. After repair, all eleven public
  `trend_controls` tests, all fifteen `local_trend` integration tests, and the
  five private local-trend unit regressions passed.
- Warning-denying focused Clippy, the runnable example, and release benchmark
  smoke passed. The benchmark measured approximately 12.4 us for four controls
  and 43.9 us for sixteen controls on this development machine.
- Exact stable head `accad99` passed workspace format, warning-denying workspace
  all-target/all-feature Clippy, all-feature workspace tests, workspace Rustdoc,
  all 58 requirement checks, and complete diff whitespace validation.
- The evidence tail after `accad99` changes only the review record and bounded
  Markdown handoff. Ready-only Windows/Ubuntu/macOS and benchmark-smoke CI
  remain intentionally unexecuted while PR #109 is Draft.

## Next task boundary

A fresh Review task must give an isolated read-only project `math_reviewer`
only the bounded REQ-TREND-002 summary and integrated dependency closure, Issue
#108 acceptance criteria and exclusions, the M6 plan, ANISOTROPY and
ADR-0005/ADR-0008 contracts, the complete PR and Fourth Repair diffs, directly
relevant source/tests/example/benchmark, and recorded validation evidence. It
must verify exact Repair head `accad99`, independently check TREND002-REV-008
and the retained TREND002-REV-007 compact short-circuit, search for new P0-P3
findings, record the result, push, and stop. Do not repair code, mark the PR
ready, merge, or begin another requirement in that Review task.

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
