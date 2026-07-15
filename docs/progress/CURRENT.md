# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active scope

- Mode: Review complete; fresh Repair required next
- Requirement: REQ-CPD-001, Issue #45
- Draft pull request: #46
- Branch: `codex/req-cpd-001-rank-nullspace`
- Reviewed head: `9d7177eb034ae07e8ef04a915a0fa06664b8450e`
- Repair code/test head: `10d3892381356ed5453e1c58b5daceefee037dda`
- Registry state: `documented` (not integrated)
- Dependencies: REQ-KERNEL-002, REQ-POLY-001, REQ-FUNC-001, and
  REQ-SPIKE-002 are integrated

## Review result

- The fresh independent `math_reviewer` closed P1-2, P2-2, and P2-3 and found
  no P0, P1, or P3 issue on exact reviewed head `9d7177e`.
- New P2-4 remains: for D=1, order one, and
  `Q=[1e308,1e-308,1e-308]^T`, mapped row-scale normalization and both
  residual helpers can underflow nonzero terms. Direct evaluation gives the
  finite representable residual `-sqrt(2)*1e-308`, while null-space quality
  and expanded-weight diagnostics fabricate `0.0` in original units.
- This is a diagnostic and provenance failure rather than a rank or
  feasibility misclassification because the scaled residual remains within
  the documented tolerance. PR #46 remains Draft.

## Validation state

- All three focused regressions and the complete `georbf` CPD test target pass.
- On exact repair code/test head `10d3892`, the complete standard gate passed:
  formatting, warning-denying workspace Clippy, all-feature workspace tests,
  workspace doc tests, and all 58 requirement checks.
- `git diff --check` passed. The subsequent handoff/review-evidence update is
  documentation-only, so the stable code/test-head gate remains applicable.
- Four consecutive repaired benchmark runs were deterministic and established
  the updated 0.706--1.125 ms local complete-assembly baseline.
- Exact reviewed-head Draft CI run 29390599350 passed Ubuntu on head `9d7177e`.
  The ready three-platform and benchmark-smoke matrix must remain skipped
  until a later clean re-review.

## Next task

- Open a fresh Repair task for PR #46 only and repair P2-4 without expanding
  REQ-CPD-001 or starting other work.
- Add an independent D=1 order-one regression for
  `Q=[1e308,1e-308,1e-308]^T`. Recompute `Q^T Z` and a unit-coordinate
  expanded-weight residual directly in original units; require the truthful
  finite nonzero matrix-infinity residual or an explicit structured
  unrepresentable-arithmetic error, never a fabricated zero.
- Make normalization exponent-aware or reject nonzero-to-zero normalization,
  run focused checks and the final standard gate after the last code change,
  update repair evidence, commit, push, and stop for a fresh re-review.
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
