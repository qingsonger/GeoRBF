# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review complete / P1-1 remains open
- Requirement: REQ-SOLVE-001, Issue #57
- Branch: `codex/req-solve-001-dense-equality-solvers`
- Draft pull request: #58
- Registry state: `implemented`
- Dependencies: REQ-SPIKE-001, REQ-SPIKE-002, and REQ-FIELD-001 are integrated

## Review result

- A fresh read-only reviewer re-reviewed exact repair head `9361de7`, reran
  the focused solver truth/error tests, example, benchmark smoke, requirement
  checks, and `git diff --check`, and found no new P0, P2, or P3 issue.
- The conservative peak model covers the reviewed nalgebra and GeoRBF live
  storage, and the directly used `try_solve` and `try_solve_field` paths check
  their effective limits before backend dispatch or field copying.
- P1-1 remains open because public
  `DenseEqualitySystem::try_from_field` copies an assembled field matrix and
  right-hand side without checking or retaining its `ExecutionOptions` memory
  limit. Callers can bypass `try_solve_field`, lose the field limit, and later
  solve using only a new solver limit.
- The existing field-limit regression covers only `try_solve_field`; it does
  not exercise the public conversion bypass. Full evidence and required repair
  options are in `docs/reviews/PR-58-INDEPENDENT-REVIEW.md`.

## Validation state

- Eleven public solver tests and all three private solver regressions passed,
  including pre-dispatch peak-limit rejection, field-limit propagation, and
  checked estimate overflow.
- The runnable example, two-iteration 64-by-64 Cholesky/LBLT benchmark smoke,
  all 58 requirement checks, focused warning-denying Clippy, and
  `git diff --check` passed on the repair worktree.
- After the final production, test, registry, and build-input change, the
  stable repair head passed formatting, warning-denying workspace Clippy, all-
  feature workspace tests, workspace doctests, and requirement validation.
- Draft Ubuntu CI run 29469494039 passed on exact repair head `9361de7`. The
  Ready-only three-platform and benchmark-smoke matrix has not run.
- This Review task changes only the review record and bounded handoff.
  Production code, tests, manifests, schemas, CI, benchmark inputs, and
  dependencies remain unchanged from the fully checked repair head.

## Next task

Open a fresh Repair task for PR #58 and address only the remaining P1-1 public
conversion bypass recorded in the review document. Reproduce it and add the
required regression before the smallest repair; rerun focused checks and the
complete stable-head standard gate, update repair evidence and this bounded
handoff, push, and stop for another fresh independent re-review. Do not begin
REQ-MODEL-001.

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
