# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review complete / clean re-review; integration sequence required
- Requirement: REQ-SOLVE-001, Issue #57
- Branch: `codex/req-solve-001-dense-equality-solvers`
- Draft pull request: #58
- Registry state: `implemented`
- Dependencies: REQ-SPIKE-001, REQ-SPIKE-002, and REQ-FIELD-001 are integrated

## Review result

- A fresh read-only `math_reviewer` inspected exact repair head `5639762`
  against base `66d9796` and found no P0, P1, P2, or P3 issue.
- P1-1 is closed: `DenseEqualitySystem::try_from_field` is private, and public
  `try_solve_field` enforces the smaller retained field or solver limit before
  the private copy while counting the still-live field matrix and right-hand
  side.
- The compile-fail Rustdoc proves external callers cannot invoke the former
  conversion. The repair changed no solver mathematics or peak estimate.
- Complete independent evidence and residual non-finding risks are recorded in
  `docs/reviews/PR-58-INDEPENDENT-REVIEW.md`.

## Validation state

- The fresh reviewer independently passed all eleven public solver tests, all
  three private solver regressions, workspace Rustdoc including the new
  compile-fail case, the runnable example, two-iteration 64-by-64 Cholesky/LBLT
  benchmark smoke, all 58 requirement checks, and `git diff --check`.
- Exact repair head `5639762` had already passed the complete local standard
  gate after its final production and Rustdoc change. This Review task changes
  only the review record and bounded handoff.
- Draft Ubuntu CI run 29470504173 passed on exact repair head `5639762`. The
  Ready-only Windows, Ubuntu, macOS, and benchmark-smoke matrix has not run.

## Next task

Open a fresh Review/integration task for PR #58. Confirm the post-review head
changes only review evidence and this bounded handoff, synchronize PR evidence,
mark the PR Ready, and wait for the complete Windows, Ubuntu, macOS, and
benchmark-smoke CI on that exact ready head. Merge exactly once only when that
CI is green, then record truthful integration state in an isolated change.
Stop without beginning REQ-MODEL-001.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #57
- Implementation: GitHub PR #58
- Requirement summary: `changes/REQ-SOLVE-001.md`
- Independent review: `docs/reviews/PR-58-INDEPENDENT-REVIEW.md`
- Solver policy: `docs/architecture/SOLVER_POLICY.md`
- Backend decision and production re-audit:
  `docs/adr/ADR-0010-nalgebra-dense-factorization-backend.md`
- Benchmark: `docs/benchmarks/REQ-SOLVE-001.md`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable.
