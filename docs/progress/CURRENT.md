# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active scope

- Mode: Repair complete; fresh independent re-review required next
- Requirement: REQ-CPD-001, Issue #45
- Draft pull request: #46
- Branch: `codex/req-cpd-001-rank-nullspace`
- Reviewed head: `cf4976ee7e575da1856d5871f6f6f744fccd43d4`
- Repair code/test head: `06ad419c06fd4c887c32be8a8dcd6ff9e1061c68`
- Registry state: `documented` (not integrated)
- Dependencies: REQ-KERNEL-002, REQ-POLY-001, REQ-FUNC-001, and
  REQ-SPIKE-002 are integrated

## Repair result

- P2-5 is repaired with a fixed-stack exact binary accumulator that retains
  the representable `2^-104` near-cancellation residual and does not fabricate
  zero for the exact `2^-1075` product.
- Both original-unit diagnostic callers map a nonzero result that would round
  to zero to their existing structured unrepresentable-residual errors.
- Independent regressions cover exact cancellation, the subnormal boundary,
  and a public D=1 order-one null-space and unit-coordinate expanded-weight
  case checked against fused double-double truth.
- PR #46 remains Draft pending a fresh independent complete-diff re-review.

## Validation state

- The complete public `georbf` CPD target passes all 13 tests, and all six
  private CPD diagnostic regressions pass.
- On exact repair code/test head `06ad419`, the complete standard gate passed:
  formatting, warning-denying workspace Clippy, all-feature workspace tests,
  workspace doc tests, and all 58 requirement checks.
- `git diff --check` passed. The subsequent handoff/review-evidence update is
  documentation-only, so the stable code/test-head gate remains applicable.
- The optimized one-iteration benchmark smoke retained checksum
  `-4.97657470788226419e-14` and completed in 0.7554 ms locally, within the
  recorded 0.706--1.125 ms complete-assembly baseline.
- Exact reviewed-documentation-head Draft CI run 29393699380 passed Ubuntu on
  `6d82acf`. New Draft CI for the pushed repair head is not yet evidence; the
  ready three-platform and benchmark-smoke matrix remains deferred until a
  clean re-review.

## Next task

- Open a fresh Review/re-review task for PR #46. Supply the independent
  `math_reviewer` only the bounded requirement/dependency summaries, normative
  documents and ADRs, complete PR diff, and validation evidence.
- Independently confirm P2-5 is closed and inspect the complete repaired diff
  for new P0-P3 findings. Do not repair production code in that Review task.
- If any finding remains, record it and stop. If the re-review is clean and
  the exact final head has valid local evidence, follow the mandatory sequence:
  mark PR #46 ready, wait for complete Windows/Ubuntu/macOS and benchmark-smoke
  CI on that exact ready head, merge only when green, and record truthful
  integration state. Then stop without beginning another requirement.

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
