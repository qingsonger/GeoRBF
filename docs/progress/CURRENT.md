# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair complete / fresh independent re-review required for PR #106
- Requirement: REQ-ANISO-002, Issue #105
- Branch: `codex/req-aniso-002-orientation-tensor`
- Implementation pull request: #106 (Draft)
- ANISO002-REV-004 repair and complete-gate head:
  `b591a419095cd4e69043f01773c43b14fd9fc914`
- Latest independently re-reviewed head:
  `8e467c86f01bdfcb3eabe7bcc4b9a89147cfa4c1`
- Dependencies: REQ-ORIENT-001 and REQ-ANISO-001 are integrated
- Registry state in this change: `implemented`

## Implemented scope

- Added sign-invariant normalized weighted orientation tensors for exactly
  D=1, D=2, and D=3, with stable relative-weight normalization and compensated
  symmetric accumulation.
- Added represented trace normalization, exact-expansion D=2/D=3 principal-
  minor certification, and a bounded maximum uniform off-diagonal retention
  policy that preserves PSD without diagonal jitter or eigenvalue clipping.
- Added a primary bounded symmetric eigendecomposition and an explicit bounded
  PSD-SVD path when eigensolver roundoff is negative for a certified matrix;
  diagnostics record both the spectral path and correlation-retention scale.
- Added deterministic canonical-sign principal axes, ordered eigenvalues and
  normalized shares, explicit isotropy thresholding, adjacent eigengaps, and
  per-axis confidence.
- Added validated scale-free principal-axis ratios with fixed selection or a
  finite explicitly bounded candidate set and deterministic leave-one-out
  squared-share cross-validation.
- Added complete candidate-score and per-sample tensor-influence evidence,
  maximum outlier influence, weight concentration, selection kind, and sample
  count diagnostics.
- Added independent public property/error tests, compile-fail dimension
  coverage, synchronized Rustdoc and architecture mathematics, a runnable
  example, changelog fragment, and deterministic CI smoke benchmark.
- Rust: implemented. CLI: N/A until M8 schemas/complete CLI. C/C++/Python: N/A
  until M9 API freeze and adapters. Persistence/model integration: excluded.

## Validation state

- The required public D=2 regression first reproduced ANISO002-REV-004 on the
  pre-repair code: direction `[1,30]`, unit weight, and fixed ratios `[1,1]`
  returned `NegativeEigenvalue(-1.1089908126111444e-16)`.
- On exact repair head `b591a41`, that regression succeeds and verifies trace
  one, a nonnegative represented determinant, nonnegative spectral values, a
  recorded sub-unit correlation scale, and the explicit PSD-SVD path.
- All 15 public orientation-tensor tests pass, including the closed rotation,
  influence, and grouped-mass regressions. Warning-denying georbf all-target/
  all-feature Clippy, the D=4 compile-fail Rustdoc contract, example, optimized
  benchmark smoke, all 58 requirement checks, and diff whitespace pass.
- The optimized smoke retained checksum `1.00428812046557887e4` at
  approximately 9.20 us per estimate locally.
- Exact repair head `b591a41` passed the complete standard workspace gate:
  format, warning-denying all-target/all-feature Clippy, all workspace tests
  with all features, workspace Rustdoc, all 58 requirement checks, and
  complete diff whitespace validation.
- This bounded-handoff and review-evidence update is Markdown only. It changes
  no production, test, manifest, schema, CI, build, API, numerical, registry,
  or dependency input and does not invalidate the exact-head gate.
- Ready-only Windows, Ubuntu, macOS, and benchmark-smoke CI has not run and is
  not claimed as passed. Local `actionlint` and the unavailable later tools
  listed below remain unexecuted and are not claimed as passed.

## Next task boundary

Open a fresh bounded Review/re-review task for only PR #106 and
REQ-ANISO-002. Supply an isolated read-only project `math_reviewer` only the
requirement summary and integrated dependency closure, Issue #105 criteria and
exclusions, M6 plan, ANISOTROPY and ADR-0009/ADR-0010 contracts, complete exact
PR diff, ANISO002-REV-004 finding and repair evidence, directly relevant
source/Rustdoc/tests/example/benchmark/CI wiring, scoped registry entry,
handoff, and validation evidence. It must independently verify
ANISO002-REV-004 closure and inspect the complete PR for new P0-P3 findings.

If any finding remains, record evidence and stop without repairing production
code. Only after a clean re-review and a complete stable-head local gate may
that fresh Review task mark PR #106 ready, wait for exact-ready-head Windows,
Ubuntu, macOS, and benchmark-smoke CI, merge once when all are green, and then
record truthful integration state. Do not begin another requirement here.

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
