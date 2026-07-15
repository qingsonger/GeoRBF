# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active scope

- Mode: Review complete; Repair required next
- Requirement: REQ-CPD-001, Issue #45
- Draft pull request: #46
- Branch: `codex/req-cpd-001-rank-nullspace`
- Reviewed head: `687731f0807e7c541123ae1c419d724b458546d0`
- Repair code/test head: `d5c6a89eaa9045f5ec8f7bf6548f1b82eea21a71`
- Registry state: `documented` (not integrated)
- Dependencies: REQ-KERNEL-002, REQ-POLY-001, REQ-FUNC-001, and
  REQ-SPIKE-002 are integrated

## Review result

- A second fresh read-only `math_reviewer` independently inspected the complete
  repaired diff at exact head `687731f` without inheriting Repair reasoning.
- P1-1 is closed: each scaling operation now rejects nonzero-to-zero rounding,
  and the exact extreme-scale regression covers the failure.
- P2-1 is closed: bounded SVD non-convergence retains completed equilibration
  and RRQR evidence while leaving every SVD-derived field and the final
  decision unavailable.
- New P1-2: rank is diagnosed on equilibrated `Q`, but null-space QR runs on
  original `Q`; representable actions near `1e200` or `1e-308` can overflow or
  underflow the backend norm and violate nonzero row-scale invariance.
- New P2-2: binding residuals labeled as matrix infinity norms are maximum
  entries, so aggregate row residual can be understated by up to the nullity.
- New P2-3: original-unit weight residual products can overflow to NaN, and the
  current maximum fold can silently turn that NaN into a fabricated zero.
- No P0 or P3 finding was identified. PR #46 must remain Draft.

## Validation state

- Both focused regressions pass on the repair tree.
- On exact repair code/test head `d5c6a89`, the complete standard gate passed:
  formatting, warning-denying workspace Clippy, all-feature workspace tests,
  workspace doc tests, and all 58 requirement checks.
- `git diff --check` passes. The subsequent handoff/review-evidence update is
  documentation-only, so the stable code/test-head gate remains applicable.
- Exact final-head Draft CI run 29387532506 passed Ubuntu; the ready
  three-platform and benchmark-smoke matrix was correctly skipped.
- Ready transition and integration are blocked until P1-2, P2-2, and P2-3 are
  repaired and another fresh independent re-review is clean.

## Next task

- Open a fresh Repair task for PR #46 only. Reproduce and add independent
  regressions for P1-2, P2-2, and P2-3 as specified in the review record.
- Implement only the smallest complete repairs: construct the null space from
  safe equilibrated data with the mathematically correct map back, compute the
  documented matrix infinity residuals, and prevent non-finite original-unit
  residual arithmetic from fabricating a finite result.
- Run focused checks during repair and the complete standard gate after the
  final code change. Update repair evidence and this bounded handoff, commit
  and push, then stop for a fresh independent re-review.
- Do not mark PR #46 ready, merge it, or begin another requirement in Repair.

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
