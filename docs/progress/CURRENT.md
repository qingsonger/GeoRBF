# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review complete / Repair required for PR #106
- Requirement: REQ-ANISO-002, Issue #105
- Branch: `codex/req-aniso-002-orientation-tensor`
- Implementation pull request: #106 (Draft)
- Stable implementation and complete-gate head: `2d65666`
- Independently reviewed head: `2c33c3f`
- Dependencies: REQ-ORIENT-001 and REQ-ANISO-001 are integrated
- Registry state in this change: `implemented`

## Implemented scope

- Added sign-invariant normalized weighted orientation tensors for exactly
  D=1, D=2, and D=3, with stable relative-weight normalization and compensated
  symmetric accumulation.
- Added private bounded symmetric eigendecomposition, deterministic
  canonical-sign principal axes, ordered eigenvalues and normalized shares,
  explicit isotropy thresholding, adjacent eigengaps, and per-axis confidence.
- Added validated scale-free principal-axis ratios with fixed selection or a
  finite explicitly bounded candidate set and deterministic leave-one-out
  squared-share cross-validation.
- Added complete candidate-score and per-sample tensor-influence evidence,
  maximum outlier influence, weight concentration, selection-kind, and sample
  count diagnostics.
- Added independent public property/error tests, compile-fail dimension
  coverage, synchronized Rustdoc and architecture mathematics, a runnable
  example, changelog fragment, and deterministic CI smoke benchmark.
- Rust: implemented. CLI: N/A until M8 schemas/complete CLI. C/C++/Python: N/A
  until M9 API freeze and adapters. Persistence/model integration: excluded.

## Validation state

- Focused orientation-tensor tests pass (11 tests).
- Warning-denying georbf all-target Clippy, georbf Rustdoc, example execution,
  and the optimized benchmark smoke pass.
- Exact implementation head `2d65666` passed the complete standard workspace
  gate: format, warning-denying all-target/all-feature Clippy, all workspace
  tests with all features, workspace Rustdoc, all 58 requirement checks, and
  complete diff whitespace validation.
- Independent Review reproduced P1 ANISO002-REV-001: leave-one-out candidate
  scores and selected ratios can change under a global rotation when a
  training fold has a repeated eigenspace, even when the full tensor has
  positive reported eigengaps.
- Independent Review reproduced P3 ANISO002-REV-002: valid extreme finite
  weights can produce an influence of `1.0000000000000002`, violating the
  documented `[0,1]` public range.
- The Review evidence change adds only `docs/reviews/PR-106-INDEPENDENT-REVIEW.md`
  and updates this bounded handoff. It changes no production, test, manifest,
  schema, CI, build, API, numerical, registry, or dependency input.
- Local `actionlint` and the unavailable later tools listed below remain
  unexecuted and are not claimed as passed.

## Next task boundary

Open a fresh Repair task for only PR #106 findings ANISO002-REV-001 and
ANISO002-REV-002. First add the recorded global-rotation cross-validation and
extreme-weight influence regressions, then implement the smallest explicit and
documented fixes. Run focused checks during repair and the complete standard
workspace gate once after the final code change on the stable head. Update the
review evidence and this handoff, push, and stop for a fresh independent
re-review. Keep PR #106 Draft and do not start another requirement.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #105
- Draft implementation: GitHub PR #106
- Independent findings: `docs/reviews/PR-106-INDEPENDENT-REVIEW.md`
- Requirement summary and benchmark baseline: `changes/REQ-ANISO-002.md`
- Public implementation and Rustdoc: `crates/georbf/src/orientation_tensor.rs`
- Independent property/error tests: `crates/georbf/tests/orientation_tensor.rs`
- Runnable example: `crates/georbf/examples/orientation_tensor.rs`
- Focused benchmark: `crates/georbf/benches/orientation_tensor.rs`
- Mathematical contract: `docs/architecture/ANISOTROPY.md`
- Numerical dependency decisions: ADR-0009 and ADR-0010; no dependency or
  feature change is introduced by this requirement

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable. No unavailable check is claimed as passed.
