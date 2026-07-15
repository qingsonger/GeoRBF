# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active scope

- Mode: Repair complete; fresh Review required next
- Requirement: REQ-CPD-001, Issue #45
- Draft pull request: #46
- Branch: `codex/req-cpd-001-rank-nullspace`
- Reviewed head: `9d7177eb034ae07e8ef04a915a0fa06664b8450e`
- Repair code/test head: `6af215f2758360513fce2b2cdf0d63914dd11bc7`
- Registry state: `documented` (not integrated)
- Dependencies: REQ-KERNEL-002, REQ-POLY-001, REQ-FUNC-001, and
  REQ-SPIKE-002 are integrated

## Repair result

- P2-4 is repaired on exact code/test head `6af215f`. Mapped basis products are
  exponent-aware, and original-unit residuals use product-wise binary-exponent
  accumulation rather than reconstructing from an underflowed column
  normalization.
- The independent D=1 order-one regression for
  `Q=[1e308,1e-308,1e-308]^T` directly recomputes every `Q^T Z` entry and all
  unit-coordinate expanded-weight residuals. The reported original-unit
  matrix-infinity residual is finite, nonzero, and matches independent truth.
- Unrepresentable restored residuals are explicit structured errors and cannot
  be discarded through `f64::max`. PR #46 remains Draft pending fresh review.

## Validation state

- The complete public `georbf` CPD target and all four private CPD diagnostic
  regressions pass.
- On exact repair code/test head `6af215f`, the complete standard gate passed:
  formatting, warning-denying workspace Clippy, all-feature workspace tests,
  workspace doc tests, and all 58 requirement checks.
- `git diff --check` passed. The subsequent handoff/review-evidence update is
  documentation-only, so the stable code/test-head gate remains applicable.
- The optimized one-iteration benchmark smoke retained checksum
  `-4.97657470788226419e-14` and completed in 1.1175 ms locally; the previously
  recorded 0.706--1.125 ms complete-assembly baseline remains the timing record.
- Exact reviewed-head Draft CI run 29390599350 passed Ubuntu on head `9d7177e`.
  The ready three-platform and benchmark-smoke matrix must remain skipped
  until a later clean re-review.

## Next task

- Open a fresh Review/re-review task for PR #46 only and use the independent
  project `math_reviewer` with bounded requirement, dependency, normative
  document, complete-diff, and validation context.
- Confirm P2-4 is closed and inspect the complete repaired head for new P0-P3
  findings. Record evidence without repairing production code in that task.
- If the review is clean, follow the mandatory integration sequence: mark the
  PR ready, wait for Windows/Ubuntu/macOS and benchmark-smoke CI on that exact
  ready head, merge only when all are green, then record truthful integration
  state and stop.
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
