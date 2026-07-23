# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review complete; fresh Repair required
- Requirement: REQ-SPIKE-003, Issue #114
- Branch: `codex/req-spike-003-sparse-backends`
- Draft pull request: #115
- Re-reviewed Repair implementation and stable full-gate head: `7257e67`
- Closed findings: P1 SPIKE003-REV-001 and P2 SPIKE003-REV-002
- Open finding: P2 SPIKE003-REV-003
- Dependencies: REQ-KERNEL-004 and its complete closure are integrated
- Registry state: `implemented`

## Review result

- The isolated `math_reviewer` closed SPIKE003-REV-001. The hand-derived
  three-point truth checks both candidates' actual CSC arrays, storage-level
  matrix-vector result, and recovered solution independently of the harness
  kernel, assembly, and row-major matrix-vector helpers.
- The reviewer closed SPIKE003-REV-002. Solver rows truthfully describe
  construct/factor/solve/review/checksum end-to-end totals, and no isolated
  factorization-speed conclusion remains.
- New P2 SPIKE003-REV-003: the index timer also constructs the complete Kiddo
  or Rstar index, but the explicit phase name and ADR/benchmark/README prose
  describe only query/filter/canonicalize/checksum work. The unchanged timing
  region must be relabeled to include construction, and the schema regression
  must require that component explicitly.
- No other P0-P3 finding was identified. The PR remains Draft.

## Evidence state

- Exact Repair head `7257e67` passed sparse-harness formatting, warning-denying
  all-target/all-feature Clippy, all 10 combined-feature tests, all four minimal
  feature cross-products, both negative configurations, and the optimized
  release smoke workload.
- Three consecutive optimized Windows runs refreshed the complete 216-, 512-,
  and 1,000-point end-to-end benchmark evidence.
- Exact Repair head `7257e67` passed the complete standard local gate and all
  58 requirement checks.
- The isolated reviewer passed all 10 locked all-feature tests, a locked smoke
  run, exact direct-version review, and the complete exact-head whitespace
  check. The parent Review task independently passed the same 10 tests, the
  optimized locked release smoke workload, all 58 requirement checks, and the
  exact-head whitespace check.
- The Review tail changes only this bounded handoff and the review-evidence
  Markdown; it does not invalidate the stable code/test/build gate.
- Ready-only Windows, Ubuntu, macOS, and benchmark-smoke CI remains unexecuted
  and is not claimed. PR #115 remains Draft.

## Next task boundary

Open a fresh Repair task for Draft PR #115 limited to SPIKE003-REV-003. Relabel
the unchanged index timing region everywhere as an explicit
construct/query/filter/canonicalize/checksum end-to-end phase, and strengthen
the benchmark schema regression to require the construction component. Retain
the existing timing ranges only if the timed code and phase boundary remain
unchanged; otherwise rerun the fixed three-trial workload. Run focused checks
and one stable-head standard gate, update Repair evidence, push, and stop for a
fresh independent re-review. Do not mark the PR ready, merge it, or begin
REQ-SPARSE-001.

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
