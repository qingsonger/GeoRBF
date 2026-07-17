# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair completed; fresh independent re-review required
- Requirement: REQ-EXEC-001, Issue #66
- Branch: `codex/req-exec-001-deterministic-execution-controls`
- Draft pull request: #67
- Original repair commit: `947888a`
- R67-004 repair commit: `33cf9def4a418970281b3ad130dcf58ec1b29074`
- Review and repair record: `docs/reviews/PR-67-INDEPENDENT-REVIEW.md`
- Registry state: `implemented`
- Required next mode: Review/re-review of PR #67 only

## R67-004 repair result

- The prior direct-`ProgressTracker` rank and factorization tests were replaced
  with tests that invoke `DenseEqualitySystem::try_solve_with_control`.
- One-shot `cfg(test)` hooks fail the actual original-rank-review and
  factorization calls. They are absent from non-test builds.
- Each hook coordinates a separate cancellation thread through a two-phase
  barrier while the injected backend call is active, ensuring cancellation is
  observable before the failing result reaches the production checkpoint.
- Both tests return structured cancellation at the expected stage and lock the
  exact successful-event prefix, which excludes the failed stage and
  `Completed`.
- Production execution semantics, formulas, numerical policy, dependencies,
  manifests, schemas, and the requirement registry were not changed.

## Validation state

- Both public-path failure-priority regressions passed.
- All 8 execution-control integration tests passed.
- All-feature `georbf` and warning-denying all-target core Clippy passed.
- Stable repair commit `33cf9de` passed the complete local standard gate:
  format, warning-denying workspace Clippy, all-feature workspace tests,
  workspace Rustdoc, all 58 requirement checks, and `git diff --check`.
- Draft Ubuntu CI passed on the preceding review-evidence head `0bb7fac`.
  CI on the new repair head had not run when this handoff was written.
- Ready-only Windows, Ubuntu, macOS, and benchmark-smoke CI has not run.

## Next task

Open a fresh Review/re-review task for PR #67. Explicitly create the project
`math_reviewer` agent with only the bounded requirement and dependency
summaries, Issue #66, M3 plan, scoped architecture and solver contracts, PR
diffs, review record, and validation evidence. Independently confirm R67-004 is
closed and check for new P0-P3 findings; do not repair code in that task.

If any finding remains, record it and stop. If the re-review is clean and the
final head retains a complete green local gate, synchronize PR evidence, mark
the PR ready, wait for the complete Windows/Ubuntu/macOS and benchmark-smoke CI
on that exact ready head, merge only when all required CI is green, and record
truthful integration state in an isolated change. Then stop without beginning
another requirement.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #66
- Draft implementation and repairs: GitHub PR #67
- Independent review and repair evidence:
  `docs/reviews/PR-67-INDEPENDENT-REVIEW.md`
- Requirement summary: `changes/REQ-EXEC-001.md`
- Architecture: `docs/architecture/ARCHITECTURE.md`
- Relevant numerical policy: `docs/architecture/SOLVER_POLICY.md` and ADR-0010

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable.
