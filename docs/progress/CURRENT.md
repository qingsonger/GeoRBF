# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review / repair re-review completed with one P2 finding
- Requirement: REQ-EXEC-001, Issue #66
- Branch: `codex/req-exec-001-deterministic-execution-controls`
- Draft pull request: #67
- Stable repair commit: `947888a`
- Re-reviewed head: `d9cba54`
- Review and repair record: `docs/reviews/PR-67-INDEPENDENT-REVIEW.md`
- Registry state: `implemented`
- Required next mode: Repair of R67-004 only

## Re-review result

- R67-001 is closed in implementation and through the public controlled-solve
  terminal-callback regression.
- R67-003 is closed in implementation and through exact public progress
  sequences for zero residual, early stop, explicit regularization, and full
  refinement-budget use.
- R67-002 is structurally repaired, but its required regression is not adequate.
  The two named tests call `ProgressTracker::finish_work` directly with a
  synthetic error; they do not exercise the public controlled solve, actual rank
  or factorization call sites, a second cancellation thread, or a barrier.
- P2 R67-004 records that those tests would pass even if the production rank or
  factorization call sites regressed to the original early-error behavior. No
  current production semantic defect was identified.
- No formula, sign, dimension, unit, CPD, rank threshold, factorization,
  residual, hard-constraint, hidden recovery, dependency, manifest, CI, or
  schema change was found.

## Validation state

- Re-review execution-control tests: 8 passed.
- The two direct-tracker cancellation tests passed but do not close R67-004.
- All-feature `georbf`: 198 unit/integration tests and 29 doctests passed.
- Separate `georbf` Rustdoc: 29 passed.
- `git diff --check` passed for the complete, repair, and evidence-only ranges;
  no added core output macro was found.
- Stable repair commit `947888a` passed the complete local standard gate:
  format, warning-denying workspace Clippy, all-feature workspace tests,
  workspace Rustdoc, all 58 requirement checks, and `git diff --check`.
- Draft Ubuntu CI passed on re-reviewed PR head `d9cba54`. Ready-only Windows,
  Ubuntu, macOS, and benchmark-smoke CI has not run.

## Next task

Open a fresh Repair task for PR #67 and address only R67-004. Add test-only
failing rank and factorization hooks at the actual controlled-solve call sites,
coordinate cancellation from a separate thread with a barrier while each hook
is active, invoke the public controlled solve, and prove cancellation priority
and absence of a successful event for the failed stage. Do not alter production
execution semantics or numerical policy and do not begin another requirement.
Run focused checks during repair, the complete standard gate on the final code
head, update the review evidence and bounded handoff, push, and stop for a fresh
independent re-review.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #66
- Draft implementation and repair: GitHub PR #67
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
