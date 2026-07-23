# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Clean independent re-review; Ready CI and integration required
- Requirement: REQ-SPIKE-003, Issue #114
- Branch: `codex/req-spike-003-sparse-backends`
- Draft pull request: #115
- Repair implementation and stable full-gate head: `83ba364`
- Closed findings: P1 SPIKE003-REV-001, P2 SPIKE003-REV-002, and
  P2 SPIKE003-REV-003
- Review state: the complete repaired PR has no remaining P0-P3 finding
- Dependencies: REQ-KERNEL-004 and its complete closure are integrated
- Registry state: `implemented`

## Independent re-review result

- The preceding isolated `math_reviewer` closed SPIKE003-REV-001. The hand-derived
  three-point truth checks both candidates' actual CSC arrays, storage-level
  matrix-vector result, and recovered solution independently of the harness
  kernel, assembly, and row-major matrix-vector helpers.
- The reviewer also closed SPIKE003-REV-002. Solver rows truthfully describe
  construct/factor/solve/review/checksum end-to-end totals, and no isolated
  factorization-speed conclusion remains.
- A fresh isolated `math_reviewer` closed SPIKE003-REV-003. Index rows and
  every evidence surface use the explicit
  construct/query/filter/canonicalize/checksum end-to-end phase, and the schema
  regression requires that exact construction-bearing label.
- The timed control flow and boundary are unchanged, so the existing fixed
  three-trial Windows ranges remain valid. No query-only performance claim
  remains.
- Exact comparison with the unique brute-force pair oracle, pair sorting and
  deduplication, and sorted-unique CSC checks satisfy duplicate prevention. No
  new P0-P3 finding was identified in the complete PR diff.

## Evidence state

- Exact Repair head `83ba364` passed sparse-harness formatting, warning-denying
  all-target/all-feature Clippy, all 10 combined-feature tests, all four minimal
  feature cross-products, both negative configurations, and the optimized
  release smoke workload with the corrected index phase label.
- The construction, query, filter, canonicalization, checksum, fixture, and
  iteration timing region is unchanged; no benchmark rerun was required.
- Exact Repair head `83ba364` passed the complete standard local gate and all
  58 requirement checks.
- The reviewer independently passed all 10 locked all-feature tests, the
  optimized locked smoke workload, exact dependency review, compact
  requirement/dependency checks, and whitespace validation.
- The parent Review independently passed the same 10 all-feature tests, the
  optimized release smoke workload, all 58 requirement checks, and whitespace
  validation.
- The tail after `83ba364` changes only this bounded handoff and the
  review-evidence Markdown; it does not invalidate the stable code/test/build
  gate.
- Ready-only Windows, Ubuntu, macOS, and benchmark-smoke CI remains unexecuted
  and is not claimed. PR #115 remains Draft.

## Next task boundary

Commit and push this evidence-only clean re-review conclusion, synchronize PR
evidence, and mark PR #115 Ready. Wait for the complete Windows, Ubuntu, and
macOS matrix with every benchmark smoke on that exact Ready head. Merge exactly
once only if it is green, wait for the exact merge commit's complete `main` CI,
then record truthful integration through an isolated integration-state change.
Do not begin REQ-SPARSE-001.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #114
- Draft implementation: GitHub PR #115
- Requirement summary: `changes/REQ-SPIKE-003.md`
- Independent review: `docs/reviews/PR-115-INDEPENDENT-REVIEW.md`
- Reproducible harness: `spikes/sparse-backends/`
- Selection decision:
  `docs/adr/ADR-0012-rstar-faer-compact-sparse-backends.md`
- Scaling and size evidence: `docs/benchmarks/REQ-SPIKE-003.md`
- Numerical policy: `docs/architecture/SOLVER_POLICY.md`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, and API/ABI/schema snapshot checks are tracked by
later requirements and release gates. Local `actionlint` is unavailable. No
unavailable check is claimed as passed.
