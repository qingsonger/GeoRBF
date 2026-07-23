# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair complete; fresh independent re-review required
- Requirement: REQ-SPIKE-003, Issue #114
- Branch: `codex/req-spike-003-sparse-backends`
- Draft pull request: #115
- Reviewed implementation head: `2ad68e5`
- Repair implementation and stable full-gate head: `7257e67`
- Repair scope: P1 SPIKE003-REV-001 and P2 SPIKE003-REV-002
- Dependencies: REQ-KERNEL-004 and its complete closure are integrated
- Registry state: `implemented`

## Repair result

- SPIKE003-REV-001: a hand-derived `0`, `R/2`, `R` fixture now checks
  `phi = [1, 3/16, 0]`, the exact expected CSC arrays, sorted uniqueness,
  symmetry, candidate-storage matrix-vector results, and recovery of
  `[1, 2, 3]` for both faer and sprs without deriving expected values from the
  harness kernel, assembly, or row-major matrix-vector helpers.
- SPIKE003-REV-002: benchmark CSV rows now carry explicit end-to-end phase
  names. Solver results are truthfully labeled as
  construct/factor/solve/review/checksum totals, the three-trial Windows
  evidence was refreshed, and the ADR no longer makes an isolated
  factorization-speed inference.
- The PR remains Draft and both findings remain pending independent closure.

## Evidence state

- Exact Repair head `7257e67` passed sparse-harness formatting, warning-denying
  all-target/all-feature Clippy, all 10 combined-feature tests, all four minimal
  feature cross-products, both negative configurations, and the optimized
  release smoke workload.
- Three consecutive optimized Windows runs refreshed the complete 216-, 512-,
  and 1,000-point end-to-end benchmark evidence.
- Exact Repair head `7257e67` passed the complete standard local gate and all
  58 requirement checks. The subsequent handoff commit changes only this
  bounded handoff and the review-evidence Markdown.
- Ready-only Windows, Ubuntu, macOS, and benchmark-smoke CI remains unexecuted
  and is not claimed. PR #115 remains Draft.

## Next task boundary

Open a fresh Review task for Draft PR #115. Use a new isolated read-only
`math_reviewer` to re-review exact Repair head `7257e67` and the complete PR
diff against base `244e887`, with special attention to SPIKE003-REV-001 and
SPIKE003-REV-002, the hand-derived CSC truth, actual candidate storage, and
benchmark interpretation. Record findings and stop. Do not repair code in that
task, mark the PR ready, merge it, or begin REQ-SPARSE-001.

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
