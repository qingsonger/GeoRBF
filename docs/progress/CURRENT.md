# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review / REQ-EXEC-001 findings recorded
- Requirement: REQ-EXEC-001, Issue #66
- Branch: `codex/req-exec-001-deterministic-execution-controls`
- Draft pull request: #67
- Reviewed head: `1b2325b`
- Stable implementation commit: `ef16599`
- Review record: `docs/reviews/PR-67-INDEPENDENT-REVIEW.md`
- Registry state: `implemented`
- Required next mode: fresh Repair of R67-001, R67-002, and R67-003 only

## Review result

- P2 R67-001: a sink can cancel on `Completed`, producing a successful terminal
  event followed by a cancellation error.
- P2 R67-002: a failing SVD, factorization, or related fallible numerical call
  can return before the promised post-call cancellation checkpoint.
- P2 R67-003: early refinement completion credits skipped refinement slots as
  completed work, and current tests do not lock exact counts.
- The independent reviewer found no changed matrix formula, sign, dimension,
  unit, CPD null-space, rank threshold, factorization, regularization, residual,
  hard-constraint, hidden recovery, or interface-disposition defect.

## Validation state

- The stable implementation tree passed the complete local standard gate:
  format, warning-denying workspace Clippy, all-feature workspace tests,
  workspace Rustdoc, all 58 requirement checks, and `git diff --check`.
- The read-only reviewer independently passed all six execution-control tests,
  all-feature `georbf` tests, `georbf` Rustdoc, and the complete PR
  `git diff --check` on reviewed head `1b2325b`.
- Draft Ubuntu CI run 29550596570 passed on the reviewed head. Ready-only
  Windows, Ubuntu, macOS, and benchmark-smoke CI has not run.
- This review task changes only the independent review record and bounded
  handoff; it does not repair production code or change tests, manifests,
  schemas, CI, build inputs, registry state, or numerical behavior.

## Next task

Open a fresh Repair task for PR #67 and address only R67-001, R67-002, and
R67-003. Add the independent terminal-state, failing-backend cancellation, and
exact progress-count regressions before or alongside the smallest fixes. Run
focused checks while repairing and the complete standard gate after the last
code change, then update the review evidence and bounded handoff, commit, push,
and stop for a fresh independent re-review. Do not begin another requirement.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #66
- Draft implementation: GitHub PR #67
- Independent review: `docs/reviews/PR-67-INDEPENDENT-REVIEW.md`
- Requirement summary: `changes/REQ-EXEC-001.md`
- Architecture: `docs/architecture/ARCHITECTURE.md`
- Relevant numerical policy: `docs/architecture/SOLVER_POLICY.md` and ADR-0010

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable.
