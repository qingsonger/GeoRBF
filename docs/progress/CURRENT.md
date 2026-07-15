# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active scope

- Mode: Review complete; Repair required next
- Requirement: REQ-CPD-001, Issue #45
- Draft pull request: #46
- Branch: `codex/req-cpd-001-rank-nullspace`
- Reviewed implementation head: `85f2ae3207c8f0677463fc4bd00944e5d71cbd0a`
- Registry state: `documented` (not integrated)
- Dependencies: REQ-KERNEL-002, REQ-POLY-001, REQ-FUNC-001, and
  REQ-SPIKE-002 are integrated

## Independent review result

- P1-1: `equilibrate` checks cumulative multipliers but can silently round a
  nonzero entry to zero during row division. The exactly full-rank action
  matrix `[[1e308,1e-16],[1e308,2e-16]]` becomes rank one before RRQR/SVD and
  can be reported as deficient instead of preserved or rejected as an
  unrepresentable scale.
- P2-1: bounded SVD non-convergence returns only the iteration limit after
  equilibration and RRQR evidence has already been computed. The structured
  error must retain the available norms, scales, zero indices, and RRQR
  evidence while marking SVD-derived fields and the final decision unavailable.
- Full independent reasoning, exact code references, and the minimum required
  regressions are in `docs/reviews/PR-46-INDEPENDENT-REVIEW.md`.
- No P0 or P3 finding was reported. PR #46 must remain Draft until both
  findings are repaired and a later fresh independent re-review is clean.

## Validation state

- Draft CI run 29386068937 passed on exact reviewed head `85f2ae3`; the ready
  three-platform and benchmark-smoke job was correctly skipped.
- The implementation task's complete local standard gate remains applicable
  to the unchanged production, tests, manifests, schemas, and build inputs.
- This Review changes only the review record, its requirement-registry link,
  and this bounded handoff. All 58 requirement checks and `git diff --check`
  pass.

## Next task

- Open a fresh Repair task for PR #46 and only findings P1-1 and P2-1. Do not
  broaden REQ-CPD-001 or begin another requirement.
- First reproduce P1-1 with the exact full-rank extreme-scale action and add a
  regression that forbids `RankDeficient`. Add a forced-SVD-non-convergence
  seam and regression that verifies preservation of all available rank
  evidence for P2-1.
- Implement the smallest fixes, run focused checks while iterating, then run
  the complete standard gate once after the final code change on a stable
  head.
- Update the review evidence and bounded handoff, commit, push, and stop for a
  fresh independent re-review. Do not mark the PR ready or merge it in Repair.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #45
- Draft implementation: GitHub PR #46
- Independent findings: `docs/reviews/PR-46-INDEPENDENT-REVIEW.md`
- Mathematical contract: `docs/math/CPD_AND_POLYNOMIALS.md`
- Solver policy: `docs/architecture/SOLVER_POLICY.md`
- Backend decision and production re-audit:
  `docs/adr/ADR-0009-nalgebra-rank-review-backend.md`
- Change summary: `changes/REQ-CPD-001.md`
- Benchmark and size baseline: `docs/benchmarks/REQ-CPD-001.md`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, allocation instrumentation, and API/ABI/schema
snapshot checks are tracked by later requirements and release gates. Local
`actionlint` is unavailable.
