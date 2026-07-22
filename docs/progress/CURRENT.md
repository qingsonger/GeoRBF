# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review complete / fresh Repair required for PR #106
- Requirement: REQ-ANISO-002, Issue #105
- Branch: `codex/req-aniso-002-orientation-tensor`
- Implementation pull request: #106 (Draft)
- ANISO002-REV-005/006/007 repair and complete-gate head:
  `b634751d6545957d0d65039fb344108ad67169df`
- Independently re-reviewed final Repair handoff head:
  `ae7983c8d13b3ab3c5a44cc0aa9b3c60ee7a0008`
- Review state: ANISO002-REV-001 through ANISO002-REV-006 are closed;
  ANISO002-REV-007 production behavior is repaired but its allocation
  regression evidence remains incomplete as P3 ANISO002-REV-009; P2
  ANISO002-REV-008 also requires Repair
- Dependencies: REQ-ORIENT-001 and REQ-ANISO-001 are integrated
- Registry state in this change: `implemented`

## Implemented scope

- Added sign-invariant normalized weighted orientation tensors for exactly
  D=1, D=2, and D=3, with stable relative-weight normalization and compensated
  symmetric accumulation.
- Added represented trace normalization, complete-exponent exact dyadic
  D=2/D=3 principal-minor review, and a uniform off-diagonal retention policy.
  ANISO002-REV-008 shows that its fixed 64-step search does not cover the full
  accepted binary64 domain.
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

- Fresh isolated re-review independently closes ANISO002-REV-005 and
  ANISO002-REV-006. Source inspection confirms the ANISO002-REV-007 production
  path now has sample-count-independent fixed fold state and fixed-count owned
  result vectors.
- ANISO002-REV-008 independently reproduces a valid D=2 `[1,2^-538]` sample
  rejected by the fixed 64-step PSD-retention search. The first accepted
  represented scale is `2^-537`, outside the attempted range.
- ANISO002-REV-009 finds that the allocation regression counts manual
  `record_allocation_attempt` annotations instead of actual allocator calls,
  so it cannot protect the repaired production property from unannotated heap
  allocation.
- The reviewer executed the existing focused binaries: all 16 public tests and
  both private exact-dyadic/allocation tests passed. The example and optimized
  benchmark smoke passed; independent IEEE-754 probes reproduced
  ANISO002-REV-008 and verified the repaired REV-005/006 boundaries.
- Exact repair head `b634751` passed the complete standard workspace gate:
  format, warning-denying all-target/all-feature Clippy, all workspace tests
  with all features, workspace Rustdoc, all 58 requirement checks, and
  complete diff whitespace validation.
- Exact final Repair handoff head `ae7983c` passed the Draft Ubuntu correctness
  CI. Its tail after `b634751` is Markdown only. This Review evidence/handoff
  change is also Markdown only and does not invalidate the exact code-head
  local gate.
- Ready-only Windows, Ubuntu, macOS, and benchmark-smoke CI has not run and is
  not claimed as passed. Local `actionlint` and the unavailable later tools
  listed below remain unexecuted and are not claimed as passed.

## Next task boundary

Open a fresh bounded Repair task for only PR #106, ANISO002-REV-008, and
ANISO002-REV-009. First add the public D=2 `[1,2^-538]` regression and an actual
allocator-observing serial regression around `try_estimate`; then implement the
smallest repairs. The allocation regression must compare four and sixteen
samples for fixed and cross-validated policies and complete the remaining
ANISO002-REV-007 evidence obligation. Run focused checks during repair and one
complete standard gate after the final code change, update Repair evidence and
this handoff, commit, push, and stop for a fresh independent re-review. Do not
mark PR #106 Ready, merge it, or begin another requirement.

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
  feature change is introduced by this Review

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, and API/ABI/schema snapshot checks are tracked by
later requirements and release gates. Local `actionlint` is unavailable. No
unavailable check is claimed as passed.
