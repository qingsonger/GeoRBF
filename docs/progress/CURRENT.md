# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair complete / awaiting fresh independent re-review
- Requirement: REQ-SOLVE-001, Issue #57
- Branch: `codex/req-solve-001-dense-equality-solvers`
- Draft pull request: #58
- Registry state: `implemented`
- Dependencies: REQ-SPIKE-001, REQ-SPIKE-002, and REQ-FIELD-001 are integrated

## Repair state

- The bounded Repair task addressed only P1-1 from the independent review.
- Every solve now requires a nonzero explicit memory limit. Checked peak
  arithmetic covers GeoRBF and nalgebra matrix, decomposition, pivot, solve,
  residual, and refinement storage before backend dispatch; overflow and an
  insufficient limit are structured errors and accepted diagnostics retain
  the estimate and limit.
- `DenseFieldSystem` retains `ExecutionOptions`; `try_solve_field` applies the
  smaller field or solver limit and checks the field-inclusive estimate before
  copying the assembled matrix.
- The implementer has recorded repair evidence but has not independently
  closed P1-1. PR #58 remains Draft and the registry remains `implemented`.

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
- The prior Draft Ubuntu CI was green on the reviewed code. Fresh Draft CI for
  the pushed repair head remains remote evidence for the next task; the Ready-
  only three-platform and benchmark-smoke matrix has not run.

## Next task

Open a fresh Review task for PR #58. Supply the read-only independent reviewer
only the bounded requirement/dependency summaries, normative documents, the
PR diff including this repair, and validation evidence. Independently verify
that P1-1's peak model, limit enforcement, field propagation, and regressions
are complete and check for new P0-P3 findings. Do not repair production code in
that task and do not begin REQ-MODEL-001.

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
