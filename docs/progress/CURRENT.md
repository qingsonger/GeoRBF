# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair complete; fresh Review required
- Requirement: REQ-TREND-001, Issue #102
- Branch: `codex/req-trend-001-positive-definite-local-trends`
- Draft pull request: #103
- Latest independently re-reviewed head:
  `ff2f4812e09a7334418ba13232fa7af8f3d607ed`
- F7-F8 repair code/test head: `2b5189d624045c16f2ca7a55b73ee6f24960e999`
- F9 repair code/test head: `4753abf248132c8745a99b493b24dc58738b4f02`
- Dependencies: REQ-KERNEL-003, REQ-ANISO-001, and REQ-MODEL-001 are integrated
- Registry state: `implemented`; F9 re-review and integration remain

## Repair result

- A fresh isolated `math_reviewer` independently closed F7 and F8. Their
  public regressions and independent high-precision truths pass on repair head
  `2b5189d`.
- F9's required public D=1 regression failed against the pre-repair
  implementation, then passed on repair head `4753abf` at the independent
  truth `1.2101577062956176e-17`.
- The repair replaces cancellation-prone `scaled^2 - 1` with the equivalent
  `(delta-radius)(delta+radius)/radius^2` coefficient before the existing
  inverse-radius-square and stable Gaussian product are applied. The successor
  distance from radius `3` therefore remains representable.
- This Repair does not independently close F9. PR #103 remains Draft.

## Repair validation state

- All 15 focused local-trend tests, georbf Rustdoc, the runnable example, and
  D=1/D=2/D=3 release benchmark smoke passed. The smoke retained the
  established deterministic checksums at approximately 211 ns, 458 ns, and
  1.12 us per Hessian evaluation.
- Exact repair code/test head `4753abf` passed the complete stable-head
  standard gate: workspace format, warning-denying workspace all-target/all-
  feature Clippy, all workspace tests with all features, workspace Rustdoc,
  all 58 requirement checks, and complete diff whitespace validation.
- The previously reported Draft CI run is not evidence for the repaired head.
  Ready-only Windows/Ubuntu/macOS and benchmark-smoke CI have not run on the
  repaired head and are not claimed as passed.

## Next task boundary

A fresh Review task must independently re-review F9 and the complete repaired
PR diff without inheriting this Repair reasoning. If any P0-P3 finding remains,
record evidence and stop without repairing it. Only if the re-review is clean
may that task follow the mandatory ready-head CI and integration sequence. Do
not begin another requirement.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #102
- Independent findings and required regressions:
  `docs/reviews/PR-103-INDEPENDENT-REVIEW.md`
- Requirement summary and benchmark baseline: `changes/REQ-TREND-001.md`
- Independent property/error tests: `crates/georbf/tests/local_trend.rs`
- Public implementation and Rustdoc: `crates/georbf/src/local_trend.rs`
- Runnable example: `crates/georbf/examples/local_trend_mixture.rs`
- Focused benchmark: `crates/georbf/benches/local_trend_mixture.rs`
- Mathematical contract: `docs/architecture/ANISOTROPY.md`, ADR-0005, ADR-0008

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable. No unavailable check is claimed as passed.
