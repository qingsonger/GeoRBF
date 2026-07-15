# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active scope

- Mode: Repair complete; fresh independent re-review required next
- Requirement: REQ-CPD-001, Issue #45
- Draft pull request: #46
- Branch: `codex/req-cpd-001-rank-nullspace`
- Repair code/test head: `d5c6a89eaa9045f5ec8f7bf6548f1b82eea21a71`
- Registry state: `documented` (not integrated)
- Dependencies: REQ-KERNEL-002, REQ-POLY-001, REQ-FUNC-001, and
  REQ-SPIKE-002 are integrated

## Repair result

- P1-1 is repaired by rejecting any row or column scaling operation that
  rounds a nonzero entry to zero. The regression assembles the exact full-rank
  action `[[1e308,1e-16],[1e308,2e-16]]` from same-unit value functionals and
  forbids `RankDeficient`.
- P2-1 is repaired with a forced-SVD-non-convergence seam and structured
  incomplete diagnostics. Original and scaled norms, scales, zero indices,
  RRQR evidence, and the iteration limit remain available; every SVD-derived
  field and the final decision is explicitly unavailable.
- The review record contains exact code references, focused regression
  evidence, and the complete local validation record.
- PR #46 remains Draft. Repair does not independently close the findings.

## Validation state

- Both focused regressions pass on the repair tree.
- On exact repair code/test head `d5c6a89`, the complete standard gate passed:
  formatting, warning-denying workspace Clippy, all-feature workspace tests,
  workspace doc tests, and all 58 requirement checks.
- `git diff --check` passes. The subsequent handoff/review-evidence update is
  documentation-only, so the stable code/test-head gate remains applicable.
- Ready three-platform and benchmark-smoke CI is intentionally deferred until
  a fresh independent re-review is clean and the PR is marked ready.

## Next task

- Open a fresh Review/re-review task for PR #46 only. Explicitly create and
  wait for the read-only project `math_reviewer`, supplying the requirement and
  dependency summaries, normative documents and ADRs, complete PR diff, and
  validation evidence without the Repair reasoning transcript.
- Independently confirm whether P1-1 and P2-1 are closed and inspect the whole
  repaired diff for new P0-P3 findings. If any finding remains, record it and
  stop without repairing production code in the Review task.
- If the re-review is clean, follow the mandatory sequence exactly: mark PR
  #46 ready, wait for complete Windows/Ubuntu/macOS and benchmark-smoke CI on
  that exact ready head, merge exactly once only if all jobs pass, then record
  truthful integration state in an isolated integration-state change.
- Stop after PR #46 integration handling. Do not begin another requirement.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #45
- Draft implementation: GitHub PR #46
- Independent findings and Repair evidence:
  `docs/reviews/PR-46-INDEPENDENT-REVIEW.md`
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
