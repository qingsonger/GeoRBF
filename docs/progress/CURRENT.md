# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review complete / fresh Repair required for PR #106
- Requirement: REQ-ANISO-002, Issue #105
- Branch: `codex/req-aniso-002-orientation-tensor`
- Implementation pull request: #106 (Draft)
- Stable ANISO002-REV-003 repair and complete-gate head: `7d38a45`
- Latest independently re-reviewed head: `8e467c8`
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

- A fresh isolated read-only `math_reviewer` inspected exact PR head `8e467c8`.
  It independently closed ANISO002-REV-001, ANISO002-REV-002, and
  ANISO002-REV-003, but identified P2 ANISO002-REV-004.
- ANISO002-REV-004 shows that a valid D=2 direction proportional to `[1,30]`
  forms an exact PSD rank-one outer product but a slightly indefinite
  componentwise-rounded tensor. Nalgebra returns a minimum eigenvalue near
  `-1.1089908126111444e-16`, and the estimator rejects the valid sample as
  `NegativeEigenvalue`.
- The parent Review task independently reproduced that public-API failure with
  unit weight and fixed ratios `[1,1]`; the temporary regression was removed
  and the clean worktree restored.
- The ANISO002-REV-003 regression still selects `[3,2,1]` and checks the
  independently derived scaled scores `1913/2646 < 1654/1323`. All 14 focused
  public orientation-tensor tests pass, including the earlier rotation and
  influence regressions.
- Warning-denying georbf all-target/all-feature Clippy, the D=4 compile-fail
  Rustdoc contract, example execution, and optimized benchmark smoke pass. The
  smoke retained checksum `1.00428812046557887e4` at approximately 5.41 us per
  estimate locally.
- Exact repair head `7d38a45` passed the complete standard workspace gate:
  format, warning-denying all-target/all-feature Clippy, all workspace tests
  with all features, workspace Rustdoc, all 58 requirement checks, and
  complete diff whitespace validation.
- The subsequent review-evidence and bounded-handoff changes are Markdown
  only. They change no production, test, manifest, schema, CI, build, API,
  numerical, registry, or dependency input and do not invalidate the stable-
  head gate.
- Local `actionlint` and the unavailable later tools listed below remain
  unexecuted and are not claimed as passed.

## Next task boundary

Open a fresh bounded Repair task for only PR #106, REQ-ANISO-002, and
ANISO002-REV-004. First add the public D=2 one-sample regression recorded in
the review evidence and prove the current `NegativeEigenvalue` failure for
direction `[1,30]`, unit weight, and fixed ratios `[1,1]`. Then make valid
orientation-tensor construction/eigendecomposition preserve the documented
PSD and normalization policy without eigenvalue clipping, jitter, hidden
regularization, or invalid-input fallback. Run focused checks during repair
and one complete standard workspace gate after the final code change. Update
the review evidence and this bounded handoff, commit, push, and stop for a
fresh independent re-review. Do not revisit the closed ANISO002-REV-001/002/003
repairs or begin another requirement.

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
