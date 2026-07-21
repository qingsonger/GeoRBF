# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair complete / fresh independent re-review required for PR #106
- Requirement: REQ-ANISO-002, Issue #105
- Branch: `codex/req-aniso-002-orientation-tensor`
- Implementation pull request: #106 (Draft)
- Stable repair and complete-gate head: `1f1fdc6`
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
- The subsequent repair-evidence and bounded-handoff commit changes only these
  Markdown records. It changes no production, test, manifest, schema, CI,
  build, API, numerical, registry, or dependency input and does not invalidate
  the stable-head gate.
- Local `actionlint` and the unavailable later tools listed below remain
  unexecuted and are not claimed as passed.

## Next task boundary

Open a fresh independent Review/re-review task for only PR #106 and
REQ-ANISO-002. Supply the project `math_reviewer` only the bounded requirement
summary and dependency closure, normative ANISOTROPY and ADR-0009/ADR-0010
contracts, Issue #105 criteria, complete PR diff, tests, benchmark, repair
evidence, and validation state; do not inherit this Repair reasoning. Verify
ANISO002-REV-001 and ANISO002-REV-002 are closed and check for new P0-P3
findings. If any finding remains, record it and stop without repair. If clean,
follow the mandatory ready-head sequence: mark PR #106 ready, wait for the
complete Windows/Ubuntu/macOS and benchmark-smoke CI on that exact head, merge
only when green, record truthful integration state, and stop. Do not begin
another requirement.

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
