# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Independent re-review complete / fresh Repair required for PR #106
- Requirement: REQ-ANISO-002, Issue #105
- Branch: `codex/req-aniso-002-orientation-tensor`
- Implementation pull request: #106 (Draft)
- ANISO002-REV-004 repair and complete-gate head:
  `b591a419095cd4e69043f01773c43b14fd9fc914`
- Latest independently re-reviewed head:
  `cebaefff3bd940fc89c98aca0514c1340bf55c3b`
- Re-review result: ANISO002-REV-001 through ANISO002-REV-004 are closed;
  P2 ANISO002-REV-005 and P3 ANISO002-REV-006/007 require Repair
- Dependencies: REQ-ORIENT-001 and REQ-ANISO-001 are integrated
- Registry state in this change: `implemented`

## Implemented scope

- Added sign-invariant normalized weighted orientation tensors for exactly
  D=1, D=2, and D=3, with stable relative-weight normalization and compensated
  symmetric accumulation.
- Added represented trace normalization, D=2/D=3 principal-minor review, and a
  bounded maximum uniform off-diagonal retention policy. ANISO002-REV-005
  identifies an underflow hole in the current exact-expansion certification.
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
- Fresh isolated re-review confirms ANISO002-REV-004 closure for `[1,30]`, but
  finds three new defects in the complete PR: exact-minor products below the
  minimum subnormal are lost (ANISO002-REV-005, P2); a positive normalized
  ratio share can underflow after construction validation (ANISO002-REV-006,
  P3); and leave-one-out tensor/decomposition scratch allocation grows with
  sample count inside per-sample loops (ANISO002-REV-007, P3).
- The parent Review task temporarily added the two required public numerical
  regressions. Both failed: `[1,f64::from_bits(1)]` returned tensor
  `[[1,5e-324],[5e-324,0]]`, and `[2^537,2^537,1]` ratios were accepted. The
  temporary tests were removed and the worktree restored before evidence
  updates.
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
- This bounded-handoff and final re-review evidence update is Markdown only. It changes
  no production, test, manifest, schema, CI, build, API, numerical, registry,
  or dependency input and does not invalidate the exact-head gate.
- Ready-only Windows, Ubuntu, macOS, and benchmark-smoke CI has not run and is
  not claimed as passed. Local `actionlint` and the unavailable later tools
  listed below remain unexecuted and are not claimed as passed.

## Next task boundary

Open a fresh bounded Repair task for only PR #106, REQ-ANISO-002, and
ANISO002-REV-005/006/007. First preserve permanent public regressions for the
minimum-subnormal D=2 PSD boundary, normalized-share underflow, and sample-
count-independent allocation behavior. Then implement the smallest complete
repairs without changing the estimator's mathematical scope, hidden numerical
adjustments, or interfaces. Run focused checks during repair and one complete
standard workspace gate after the last code change; update review evidence and
this bounded handoff, commit, push, and stop for a fresh independent re-review.
Do not mark PR #106 Ready, integrate the requirement, or begin another task.

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
