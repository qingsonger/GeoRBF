# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair complete / fresh independent re-review required for PR #106
- Requirement: REQ-ANISO-002, Issue #105
- Branch: `codex/req-aniso-002-orientation-tensor`
- Implementation pull request: #106 (Draft)
- ANISO002-REV-005/006/007 repair and complete-gate head:
  `b634751d6545957d0d65039fb344108ad67169df`
- Pre-repair independently re-reviewed head:
  `45645cb1e47e9f09fc7cae3215ea136ae643a4e1`
- Review state: ANISO002-REV-001 through ANISO002-REV-004 are closed;
  ANISO002-REV-005/006/007 have Repair evidence but require fresh independent
  closure
- Dependencies: REQ-ORIENT-001 and REQ-ANISO-001 are integrated
- Registry state in this change: `implemented`

## Implemented scope

- Added sign-invariant normalized weighted orientation tensors for exactly
  D=1, D=2, and D=3, with stable relative-weight normalization and compensated
  symmetric accumulation.
- Added represented trace normalization, complete-exponent exact dyadic
  D=2/D=3 principal-minor review, and a bounded maximum uniform off-diagonal
  retention policy.
- Added a primary bounded symmetric eigendecomposition and an explicit bounded
  PSD-SVD path when eigensolver roundoff is negative for a certified matrix;
  diagnostics record both the spectral path and correlation-retention scale.
- Added deterministic canonical-sign principal axes, ordered eigenvalues and
  normalized shares, explicit isotropy thresholding, adjacent eigengaps, and
  per-axis confidence.
- Added validated scale-free principal-axis ratios with fixed selection or a
  finite explicitly bounded candidate set and deterministic leave-one-out
  squared-share cross-validation. Positive normalized shares may not underflow.
- Added complete candidate-score and per-sample tensor-influence evidence,
  maximum outlier influence, weight concentration, selection kind, and sample
  count diagnostics. Leave-one-out folds use fixed-size stack spectral state
  and no sample-sized scratch allocation.
- Added independent public property/error tests, compile-fail dimension
  coverage, synchronized Rustdoc and architecture mathematics, a runnable
  example, changelog fragment, and deterministic CI smoke benchmark.
- Rust: implemented. CLI: N/A until M8 schemas/complete CLI. C/C++/Python: N/A
  until M9 API freeze and adapters. Persistence/model integration: excluded.

## Validation state

- Before repair, the permanent public minimum-subnormal regression reproduced
  `[[1,5e-324],[5e-324,0]]`; the ratio underflow regression showed that
  `[2^537,2^537,1]` was accepted; and the serial fixed-ratio allocation audit
  grew from 9 attempts for four samples to 21 for sixteen samples.
- On exact repair head `b634751`, the minimum-subnormal tensor is exact-dyadic
  PSD, the positive normalized share underflow is rejected structurally, and
  explicit allocation counts are sample-count-independent for fixed and
  cross-validated estimation.
- All 16 public orientation-tensor tests and both exact-dyadic/allocation unit
  tests pass, including all regressions for ANISO002-REV-001 through 007.
- Warning-denying georbf all-target/all-feature Clippy, the D=4 compile-fail
  Rustdoc contract, example, optimized benchmark smoke, all 58 requirement
  checks, and diff whitespace pass.
- The optimized 100,000-estimate smoke reported checksum
  `5.02144060231886397e5`, preserving the prior per-estimate contribution, at
  approximately 5.15 us per estimate locally.
- Exact repair head `b634751` passed the complete standard workspace gate:
  format, warning-denying all-target/all-feature Clippy, all workspace tests
  with all features, workspace Rustdoc, all 58 requirement checks, and
  complete diff whitespace validation.
- This bounded-handoff and Repair-evidence update is Markdown only. It changes
  no production, test, manifest, schema, CI, build, API, numerical, registry,
  or dependency input and does not invalidate the exact-head gate.
- Ready-only Windows, Ubuntu, macOS, and benchmark-smoke CI has not run and is
  not claimed as passed. Local `actionlint` and the unavailable later tools
  listed below remain unexecuted and are not claimed as passed.

## Next task boundary

Open a fresh bounded Review/re-review task for only PR #106 and REQ-ANISO-002.
Create the isolated project `math_reviewer` and independently verify closure of
ANISO002-REV-005/006/007 plus the complete PR diff using exact head `b634751`
and its gate evidence. If any P0-P3 finding remains, record it and stop without
repair. Only after a clean re-review may that fresh task follow the mandatory
Ready -> exact-head Windows/Ubuntu/macOS plus benchmark-smoke CI -> single merge
-> truthful integration-state sequence. Do not begin another requirement.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #105
- Draft implementation: GitHub PR #106
- Independent findings and Repair evidence:
  `docs/reviews/PR-106-INDEPENDENT-REVIEW.md`
- Requirement summary and benchmark baseline: `changes/REQ-ANISO-002.md`
- Public implementation and Rustdoc: `crates/georbf/src/orientation_tensor.rs`
- Independent property/error tests: `crates/georbf/tests/orientation_tensor.rs`
- Runnable example: `crates/georbf/examples/orientation_tensor.rs`
- Focused benchmark: `crates/georbf/benches/orientation_tensor.rs`
- Mathematical contract: `docs/architecture/ANISOTROPY.md`
- Numerical dependency decisions: ADR-0009 and ADR-0010; no dependency or
  feature change is introduced by this repair

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable. No unavailable check is claimed as passed.
