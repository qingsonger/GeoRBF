# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active scope

- Mode: Review clean; ready-head integration sequence next
- Requirement: REQ-CPD-001, Issue #45
- Pull request: #46 (Draft until this clean-review evidence is pushed)
- Branch: `codex/req-cpd-001-rank-nullspace`
- Exact cleanly reviewed head: `062bae329bbd2194b93d7708a428852c459eccfd`
- Repair code/test head: `06ad419c06fd4c887c32be8a8dcd6ff9e1061c68`
- Registry state: `documented` (not integrated)
- Dependencies: REQ-KERNEL-002, REQ-POLY-001, REQ-FUNC-001, and
  REQ-SPIKE-002 are integrated

## Review result

- A fifth fresh read-only `math_reviewer` independently inspected the complete
  PR diff at exact head `062bae3` without inheriting Repair reasoning.
- P2-5 is closed: the fixed-stack exact binary accumulator retains the
  representable `2^-104` near-cancellation residual, rejects the exact nonzero
  `2^-1075` result as unrepresentable, and both public diagnostic paths have
  independent regressions.
- No P0, P1, P2, or P3 finding remains. The complete diff is consistent with
  the scoped mathematical, numerical, architecture, interface, diagnostic,
  allocation, benchmark, and requirement contracts.

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
- The independent reviewer reran the public CPD target (13/13), all six private
  CPD regressions, formatting, all 58 requirement checks, and
  `git diff --check`; all passed.
- Exact reviewed-head Draft CI run 29394931421 passed its complete Ubuntu job
  on `062bae3`. The ready Windows/Ubuntu/macOS and benchmark-smoke matrix has
  not run and must be triggered on the clean-evidence head.

## Next task

- Push this documentation-only clean-review evidence and mark PR #46 ready.
- Wait for the ready event to complete Windows, Ubuntu, macOS, and every
  benchmark smoke workload on that exact head. Merge exactly once only when
  the complete matrix is green.
- Wait for the post-merge `main` gate, then record truthful integration state
  through an isolated change. Stop without beginning another requirement.

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
