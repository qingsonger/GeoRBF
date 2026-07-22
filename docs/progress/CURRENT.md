# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair complete / fresh isolated re-review required for PR #106
- Requirement: REQ-ANISO-002, Issue #105
- Branch: `codex/req-aniso-002-orientation-tensor`
- Implementation pull request: #106 (Draft)
- ANISO002-REV-008 maximal-scale Repair code, tests, change fragment,
  normative document, and complete-gate state:
  `682b55ffe1310dcfafa8d127932fab11f4e9848c`
- Review state: ANISO002-REV-001 through ANISO002-REV-007 and
  ANISO002-REV-009 are closed; ANISO002-REV-008 has Repair evidence but remains
  open pending fresh isolated re-review
- Dependencies: REQ-ORIENT-001 and REQ-ANISO-001 are integrated
- Registry state in this change: `implemented`

## Implemented scope

- Added sign-invariant normalized weighted orientation tensors for exactly
  D=1, D=2, and D=3, with stable relative-weight normalization and compensated
  symmetric accumulation.
- Added represented trace normalization, complete-exponent exact dyadic
  D=2/D=3 principal-minor review, and a uniform off-diagonal retention policy.
  Exact order-two minors use monotone ordered-bit bisection; D=3 uses a finite
  high-scale-first interval search whose exact dyadic determinant upper bound
  permits only proved-negative pruning, so independently rounded nonmonotone
  acceptance still returns the greatest certified scale.
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
  and no sample-sized scratch allocation; a dedicated integration test observes
  actual allocator calls for four and sixteen samples under both policies.
- Added independent public property/error tests, compile-fail dimension
  coverage, synchronized Rustdoc and architecture mathematics, a runnable
  example, changelog fragment, and deterministic CI smoke benchmark.
- Rust: implemented. CLI: N/A until M8 schemas/complete CLI. C/C++/Python: N/A
  until M9 API freeze and adapters. Persistence/model integration: excluded.

## Validation state

- The public `[1,2^-538]` regression failed before repair with the reviewed
  structured numerical error and now succeeds at exact scale `2^-537`, trace
  one, zero represented off-diagonal, exact represented PSD, and nonnegative
  spectrum.
- The reviewed public D=3 counterexample failed before the final Repair by
  returning `1.0.to_bits() - 3`. It now returns and exactly matches the tensor
  at `1.0.to_bits() - 1`; an exact dyadic regression covers the final
  accepted--rejected--accepted--rejected determinant sequence and verifies the
  order-two minors at the greatest accepted scale.
- The actual allocator regression measures only warmed `try_estimate` calls.
  Fixed ratios allocate twice for both four and sixteen samples;
  cross-validation allocates five times for both counts. Manual annotation
  counters were removed.
- All 18 public orientation-tensor tests, the dedicated actual-allocation test,
  both private exact-dyadic tests, georbf strict Clippy, the example, optimized
  benchmark smoke, and diff whitespace passed. The smoke checksum remained
  `1.00428812046557887e4` at approximately 6.11 us per estimate locally.
- Exact Repair head `682b55f` passed the complete standard workspace gate:
  format, warning-denying all-target/all-feature Clippy, all workspace tests
  with all features, workspace Rustdoc, all 58 requirement checks, and complete
  diff whitespace. Only review evidence and this bounded Markdown handoff are
  changed afterward.
- Draft Ubuntu CI run 29885690427 passed on earlier reviewed head `f99be61`.
  CI on the final pushed Repair head has not yet run and is not claimed.
- Ready-only Windows, Ubuntu, macOS, and benchmark-smoke CI has not run and is
  not claimed as passed. Local `actionlint` and the unavailable later tools
  listed below remain unexecuted and are not claimed as passed.

## Next task boundary

Open a fresh isolated Review task for PR #106. Supply the project
`math_reviewer` only the bounded REQ-ANISO-002 summary and dependency closure,
Issue #105 criteria, M6 plan, ANISOTROPY and ADR-0009/ADR-0010 contracts, the
complete PR and latest Repair diffs, directly relevant source/tests, and the
recorded validation evidence. Independently verify ANISO002-REV-008 closure,
the exact determinant interval upper-bound proof, maximality of the returned
scale, and the complete PR. If any finding remains, record evidence and stop;
do not repair in Review. If clean, follow the repository's ready-CI-integration
sequence on the exact reviewed head. Do not begin another requirement.

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
- Numerical dependency decisions: ADR-0009 and ADR-0010; this Repair adds only
  the reviewed, exactly pinned `allocation-counter` 0.8.1 test dependency

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, and API/ABI/schema snapshot checks are tracked by
later requirements and release gates. Local `actionlint` is unavailable. No
unavailable check is claimed as passed.
