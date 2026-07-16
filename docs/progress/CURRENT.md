# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review complete / awaiting Repair
- Requirement: REQ-SOLVE-001, Issue #57
- Branch: `codex/req-solve-001-dense-equality-solvers`
- Draft pull request: #58
- Registry state: `implemented`
- Dependencies: REQ-SPIKE-001, REQ-SPIKE-002, and REQ-FIELD-001 are integrated

## Independent review finding

- P1-1: `solver.rs` constructs and clones nalgebra matrices and allocates
  RRQR, SVD, factorization, and solve workspaces without checked peak-memory
  estimation or an explicit limit. Those backend allocations bypass the
  structured allocation errors, so a finite user-sized system can pass the
  fallible GeoRBF-owned copies and then abort or panic in the backend.
- Required repair: add checked peak-working-set arithmetic, enforce an explicit
  solver memory limit including applicable field execution limits, and add
  pre-dispatch limit and estimate-overflow regressions.
- No other P0-P3 finding was found. PR #58 remains Draft and the registry stays
  `implemented` pending Repair and fresh re-review.

## Validation state

- Draft Ubuntu CI run 29467512174 passed on exact reviewed head `0b3ae41`.
  The Ready-only three-platform and benchmark-smoke matrix did not run.
- Nine public solver tests, the forced-SVD regression, runnable example,
  two-iteration Cholesky/LBLT benchmark smoke, all 58 requirement checks, and
  `git diff --check` passed during Review.
- The implementation head already passed the complete standard gate recorded
  in the requirement evidence. This task changes only review and bounded-
  handoff documentation, so that immutable-code gate remains applicable.

## Next task

Open a fresh Repair task for PR #58 and address only P1-1 from
`docs/reviews/PR-58-INDEPENDENT-REVIEW.md`. Reproduce the missing memory-policy
boundary, add the required regressions, implement the smallest complete repair,
run focused checks and the final stable-head standard gate, update review
evidence and this bounded handoff, push, and stop for a fresh independent
re-review. Do not begin REQ-MODEL-001.

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
