# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Implement / REQ-TREND-002 complete
- Requirement: REQ-TREND-002, Issue #108
- Branch: `codex/req-trend-002-region-controls`
- Draft pull request: #109
- Implementation code/test/documentation head: `1291c6d`
- Dependencies: REQ-TREND-001, REQ-PROJECT-001, and REQ-NORMAL-001 are integrated
- Registry state in this change: `implemented`

## Implemented scope

- Ordered explicit and reference-gradient controls compile into the existing
  strict-background `LocalTrendMixture<D>` for exactly D=1, D=2, and D=3.
- Spheroidal and ellipsoidal inputs use fixed `GlobalAnisotropy` metrics under
  caller condition policy. No axis repair, length inference, or arbitrary
  location-dependent metric is introduced.
- Optional axis-aligned regions multiply Gaussian influence by a compact
  quintic C2 gate that is exactly zero with zero gradient and Hessian at every
  boundary.
- Immutable fitted project gradients are sampled once in their original-
  coordinate convention, normalized only above explicit policy, and retain
  field identifier, original norm, confidence, evaluation failures, and no
  fallback direction.
- Diagnostics retain resolved axes/lengths, provenance, strengths, radii,
  regions, condition numbers, sign-invariant direction jumps, low-confidence
  counts, and the primitive background/coverage evidence.
- Rust is implemented. CLI/schema work is N/A until M8; C/C++/Python are N/A
  until M9. Field refit and persistence are outside this compiler requirement.

## Validation state

- Focused `trend_controls` integration tests pass.
- The runnable `trend_controls` example passes.
- The release-mode focused benchmark smoke passes at approximately 10.7 us for
  four controls and 38.7 us for sixteen controls on this development machine.
- Warning-denying `georbf` all-target/all-feature Clippy passes.
- The complete standard gate is pending on the stable PR-linked evidence head.
  No pending check is claimed as passed.

## Next task boundary

Run the complete standard gate once on the stable PR-linked evidence head,
record the exact successful head, push, and stop. Independent mathematical/
numerical Review of Draft PR #109 must start in a fresh task; do not repair or
integrate in this Implement task.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #108
- Draft implementation: GitHub PR #109
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
