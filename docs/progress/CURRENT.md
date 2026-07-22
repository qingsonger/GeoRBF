# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review complete, fresh Repair required
- Requirement: REQ-TREND-002, Issue #108
- Branch: `codex/req-trend-002-region-controls`
- Draft pull request: #109
- Exact independently reviewed head: `3a8ba8f`
- Third Repair code/test/evidence head: `8203876`
- Stable full-gate head: `8203876`
- Dependencies: REQ-TREND-001, REQ-PROJECT-001, and REQ-NORMAL-001 are
  integrated
- Registry state in this change: `implemented`

## Fresh re-review result

- TREND002-REV-007 is independently closed. The compact center-factor
  short-circuit is algebraically correct, and the public regression covers both
  argument orders through Hessian demand.
- New P1 TREND002-REV-008 remains. Represented-zero query or center factors do
  not distinguish exact compact support from Gaussian underflow. For a valid
  D=1 non-regional control with strength `1e154`, radius one, fixed Gaussian
  length `100`, query zero, and center `47`, each argument order short-circuits
  even though the independently combined local value is the finite,
  representable `1.878351700364362e-172`.
- No additional P0, P2, or P3 finding was identified. PR #109 remains Draft
  and the registry remains `implemented`.

## Validation state

- The isolated reviewer passed all ten public `trend_controls` tests, all five
  private local-trend regressions, the exact TREND002-REV-007 regression, and
  complete diff whitespace validation.
- The parent Review task independently passed the same public and private
  focused tests, workspace format, all 58 requirement checks, and complete
  diff whitespace on exact reviewed head `3a8ba8f`.
- Draft Ubuntu CI run 29902996233 passed its configured correctness gate on
  exact reviewed head `3a8ba8f`. Ready-only Windows/Ubuntu/macOS and benchmark
  smoke remain intentionally unexecuted.
- Exact Repair head `8203876` retains the complete recorded standard gate. The
  tail afterward changes only the review record and bounded Markdown handoff.

## Next task boundary

A fresh Repair task must address only TREND002-REV-008. First add a public
compiled D=1 regression using the reviewed strength, radius, fixed-kernel
length, query, and center in both argument orders, isolate the local term with
a zero-valued background, and compare against independent logarithmic-domain
truth. Then retain zero provenance or logarithmic scale far enough to skip only
mathematically exact compact-support zeros without regressing TREND002-REV-007.
Run focused checks and one complete stable-head standard gate after the final
code change, update review evidence and this bounded handoff, push, and stop
for a fresh independent re-review. Do not begin another requirement.

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
