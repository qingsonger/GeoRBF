# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair complete / fresh independent re-review required
- Requirement: REQ-SOLVE-001, Issue #57
- Branch: `codex/req-solve-001-dense-equality-solvers`
- Draft pull request: #58
- Registry state: `implemented`
- Dependencies: REQ-SPIKE-001, REQ-SPIKE-002, and REQ-FIELD-001 are integrated

## Repair result

- The remaining P1-1 conversion bypass is repaired by making
  `DenseEqualitySystem::try_from_field` private. Public `try_solve_field` is
  now the only assembled-field solver boundary and enforces the smaller field
  or solver memory limit before copying.
- A compile-fail Rustdoc regression proves external callers cannot invoke the
  direct conversion. It reproduced the bypass by compiling before the repair
  and passes after the visibility change.
- The existing assembled-field truth case and one-byte field-limit regression
  pass unchanged. No solver mathematics, memory estimate, dependency,
  benchmark input, adapter surface, or registry status changed.
- Full response evidence is in
  `docs/reviews/PR-58-INDEPENDENT-REVIEW.md`.

## Validation state

- Focused solver Rustdoc, assembled-field truth, and field-limit pre-copy tests
  pass after the repair.
- After the final production and Rustdoc change, the stable repair worktree
  passed formatting, warning-denying workspace Clippy, all-feature workspace
  tests, workspace doctests, and all 58 requirement checks. The later review-
  record and handoff validation notes are Markdown-only evidence updates.
- Final `git diff --check` passed after those evidence updates.
- The previous Draft Ubuntu CI passed on pre-repair head `9361de7`; no remote
  CI result may be attributed to the new repair head yet. The Ready-only three-
  platform and benchmark-smoke matrix has not run.

## Next task

Open a fresh read-only independent Review task for PR #58. Review the exact
repair head and determine whether P1-1 is closed without new P0-P3 findings;
rerun the bounded solver, requirement, example, benchmark-smoke, and diff
evidence. Do not repair production code, mark the PR Ready, merge, or begin
REQ-MODEL-001 in that task.

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
