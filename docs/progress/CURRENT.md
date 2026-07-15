# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active scope

- Mode: Repair complete; fresh independent re-review required next
- Requirement: REQ-CPD-001, Issue #45
- Draft pull request: #46
- Branch: `codex/req-cpd-001-rank-nullspace`
- Prior reviewed head: `687731f0807e7c541123ae1c419d724b458546d0`
- Repair code/test head: `10d3892381356ed5453e1c58b5daceefee037dda`
- Registry state: `documented` (not integrated)
- Dependencies: REQ-KERNEL-002, REQ-POLY-001, REQ-FUNC-001, and
  REQ-SPIKE-002 are integrated

## Repair result

- P1-2 repaired: null-space QR now uses the same safe equilibrated action as
  rank diagnosis, maps `u` back with `z = D_row u`, and deterministically
  reorthogonalizes with a stable norm. Independent `1e200` value-row and
  `1e-308` derivative-row regressions preserve `Q^T Z = 0` and `Z^T Z = I`.
- P2-2 repaired: side-condition and orthonormality quality values are actual
  matrix infinity norms, including the required two-entry aggregate residual
  regression.
- P2-3 repaired: original-unit weight residuals are restored from scaled
  arithmetic without forming overflowing products or folding NaN into zero;
  an unrepresentable restored value is a structured error.
- No unrelated requirement, adapter, solver, or production dependency changed.
  PR #46 remains Draft.

## Validation state

- All three focused regressions and the complete `georbf` CPD test target pass.
- On exact repair code/test head `10d3892`, the complete standard gate passed:
  formatting, warning-denying workspace Clippy, all-feature workspace tests,
  workspace doc tests, and all 58 requirement checks.
- `git diff --check` passed. The subsequent handoff/review-evidence update is
  documentation-only, so the stable code/test-head gate remains applicable.
- Four consecutive repaired benchmark runs were deterministic and established
  the updated 0.706--1.125 ms local complete-assembly baseline.
- Exact pre-repair Draft CI run 29388300906 passed Ubuntu on head `0140176`;
  CI for the pushed repair head is pending. The ready three-platform and
  benchmark-smoke matrix must remain skipped until a clean re-review.

## Next task

- Open a fresh Review task for PR #46 only and supply the independent
  `math_reviewer` with bounded requirement/dependency summaries, normative
  documents and ADRs, the complete PR diff, and validation evidence without
  inheriting this Repair reasoning.
- Independently confirm P1-2, P2-2, and P2-3 are closed and inspect the complete
  repaired diff for new P0-P3 findings. Record the exact reviewed head.
- If any finding remains, record it and stop for another fresh Repair task.
  Do not repair production code in Review.
- Only if the re-review is clean may that fresh task synchronize PR evidence,
  mark PR #46 Ready, wait for the exact-head Windows/Ubuntu/macOS plus all
  benchmark-smoke CI, merge once when green, and record integration state.
- Do not begin another requirement in this task.

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
