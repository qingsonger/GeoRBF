# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review complete / fresh Repair required for PR #106
- Requirement: REQ-ANISO-002, Issue #105
- Branch: `codex/req-aniso-002-orientation-tensor`
- Implementation pull request: #106 (Draft)
- Stable repair and complete-gate head: `1f1fdc6`
- Independently re-reviewed head: `627d360`
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

- A fresh isolated read-only `math_reviewer` re-reviewed exact head `627d360`.
  ANISO002-REV-001 and ANISO002-REV-002 are closed, but new P2
  ANISO002-REV-003 prevents Ready/integration.
- ANISO002-REV-003 shows that independently rounded expected shares need not
  sum to exactly one. For an all-axis unresolved fold this creates
  candidate-dependent artificial loss; with a `[1,2,2]` direction weighted
  by `f64::MAX`, unit-weight coordinate axes, and candidates `[3,2,1]` and
  `[4,2,1]`, it overwhelms the real approximately `1e-309` minor-fold evidence
  and reverses the exact selection from `[3,2,1]` to `[4,2,1]`.
- Repair head `1f1fdc6` groups unresolved training-fold eigenspaces at
  `64 D epsilon` for basis-independent candidate loss and applies the explicit
  `64 D^2 epsilon` influence upper roundoff policy.
- The new global-rotation and extreme-weight regressions failed against the
  pre-repair implementation and pass at the repair head. All 13 focused public
  orientation-tensor tests pass.
- Warning-denying georbf all-target/all-feature Clippy, georbf Rustdoc
  including the D=4 compile-fail contract, example execution, and the optimized
  benchmark smoke pass. The smoke retained checksum
  `1.00428812046557887e4` at approximately 4.58 us per estimate locally.
- Exact repair head `1f1fdc6` passed the complete standard workspace gate:
  format, warning-denying all-target/all-feature Clippy, all workspace tests
  with all features, workspace Rustdoc, all 58 requirement checks, and
  complete diff whitespace validation.
- The commits after repair head `1f1fdc6`, including this re-review evidence,
  change only these Markdown records. They change no production, test,
  manifest, schema, CI, build, API, numerical, registry, or dependency input
  and do not invalidate the stable-head gate.
- Local `actionlint` and the unavailable later tools listed below remain
  unexecuted and are not claimed as passed.

## Next task boundary

Open a fresh bounded Repair task for only PR #106, REQ-ANISO-002, and
ANISO002-REV-003. First add the public four-sample D=3 extreme-weight
regression recorded in the review evidence and prove that `[3,2,1]` wins over
`[4,2,1]`; then make grouped expected mass honor the normalized-share
invariant, especially for an all-axis unresolved group. Run focused checks
during repair and one complete standard workspace gate after the final code
change. Update the review evidence and this bounded handoff, commit, push, and
stop for a fresh independent re-review. Do not revisit the closed
ANISO002-REV-001/002 repairs or begin another requirement.

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
