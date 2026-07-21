# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair complete / fresh independent re-review required for PR #106
- Requirement: REQ-ANISO-002, Issue #105
- Branch: `codex/req-aniso-002-orientation-tensor`
- Implementation pull request: #106 (Draft)
- Stable ANISO002-REV-003 repair and complete-gate head: `7d38a45`
- Last independently re-reviewed head: `627d360`
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

- The fresh isolated read-only re-review of head `627d360` closed
  ANISO002-REV-001 and ANISO002-REV-002 and identified P2
  ANISO002-REV-003 in grouped expected-share normalization.
- The required public four-sample extreme-weight regression failed against
  pre-repair head `d5cd66c`: artificial loss `1.232595164407831e-32` reversed
  the exact candidate ordering and selected `[4,2,1]`.
- Repair head `7d38a45` assigns both observed and expected residual probability
  mass to the final eigenspace group. A group spanning every axis therefore
  has exact represented mass one on both sides and zero candidate loss.
- The repaired regression selects `[3,2,1]` and checks independently derived
  scaled scores `1913/2646 < 1654/1323`. All 14 focused public orientation-
  tensor tests pass, including the earlier rotation and influence regressions.
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

Open a fresh bounded Review/re-review task for only PR #106 and
REQ-ANISO-002. Use a fresh isolated read-only project `math_reviewer` to verify
ANISO002-REV-003 against exact repair head `7d38a45` and inspect the complete
PR diff for new P0-P3 findings. If any finding remains, record it and stop
without repairing production code. If the re-review is clean and the final
head retains a complete local gate, follow the mandatory sequence: mark the
PR Ready, wait for complete Windows/Ubuntu/macOS and every benchmark-smoke CI
on that exact Ready head, merge only when green, and record truthful isolated
integration state. Do not begin another requirement.

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
