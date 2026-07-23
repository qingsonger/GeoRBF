# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair complete; fresh independent re-review required
- Requirement: REQ-SPARSE-001
- Issue: #117
- Branch: `codex/req-sparse-001-compact-support`
- Draft pull request: #118
- Third Repair implementation and stable gate head:
  `85d7e99356fe3790f2a0d63430b78a4f68ad8a0f`
- Finding addressed pending fresh re-review: P1 SPARSE001-REV-005
- Closed findings: P1 SPARSE001-REV-001, P2 SPARSE001-REV-002, P2
  SPARSE001-REV-003, and P3 SPARSE001-REV-004
- Dependencies: REQ-SPIKE-003, REQ-FIELD-001, and REQ-KERNEL-004 are integrated
- Registry status: `planned`; Ready CI and integration remain forbidden until
  a fresh independent re-review is clean

## Third Repair result

- Pinned faer 0.24.4 AMD and complete symbolic-analysis `StackReq` requests are
  reproduced from dimension and stored nonzeros before backend dispatch.
- Checked conservative bounds now include retained symbolic structures,
  numeric-factor storage, and numeric scratch.
- Diagnostics and enforcement distinguish symbolic-factorization,
  numeric-factorization, and solve-and-review peaks.
- The 64-by-64 all-supported regression independently obtains faer's AMD
  request for 4,096 entries, permits assembly at a limit between the old and
  corrected solve peaks, and rejects solve after only the `Started` progress
  event.

## Evidence state

- Exact stable implementation head
  `85d7e99356fe3790f2a0d63430b78a4f68ad8a0f` passed all 44 all-feature core
  unit tests, all ten all-feature sparse integration tests, format, and
  warning-denying all-target/all-feature Clippy.
- After the last production or test change, that same exact head passed the
  complete standard workspace gate: format, warning-denying
  workspace/all-target/all-feature Clippy, all-feature workspace tests,
  workspace doctests, and the 58-requirement registry check.
- This subsequent handoff update changes only review evidence and
  `docs/progress/CURRENT.md`; it does not invalidate that stable implementation
  gate.
- Draft CI for the Repair head has not yet completed and is not claimed.
  Ready-only Windows/Ubuntu/macOS and benchmark-smoke CI has not run and is not
  claimed.

## Next task boundary

Open a fresh Review/re-review task for PR #118. Supply the isolated project
`math_reviewer` only the bounded requirement summary and dependency closure,
Issue #117 criteria, M7 plan, applicable architecture and solver policy,
ADR-0012, the complete PR diff, prior finding SPARSE001-REV-005, and exact
validation evidence. Independently verify the new faer workspace bounds and
64-by-64 between-limit regression, confirm whether SPARSE001-REV-005 is closed,
and check for new P0-P3 findings. If any finding remains, record it and stop
without repairing production code. If the review is clean, follow the
mandatory integration sequence: synchronize evidence, mark PR #118 ready,
wait for complete Windows/Ubuntu/macOS and benchmark-smoke CI on that exact
ready head, merge only if all are green, then record integration state in an
isolated change. Do not begin REQ-CENTER-001 in that task.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #117
- Draft implementation: GitHub PR #118
- Independent review and Repair evidence:
  `docs/reviews/PR-118-INDEPENDENT-REVIEW.md`
- Requirement summary: `changes/REQ-SPARSE-001.md`
- Architecture: `docs/architecture/ARCHITECTURE.md`
- Numerical policy: `docs/architecture/SOLVER_POLICY.md`
- Backend selection: `docs/adr/ADR-0012-rstar-faer-compact-sparse-backends.md`
- Benchmark: `docs/benchmarks/REQ-SPARSE-001.md`
- Production implementation: `crates/georbf/src/problem_ir.rs` and
  `crates/georbf/src/sparse.rs`
- Independent tests: canonical-IR unit tests and
  `crates/georbf/tests/sparse.rs`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, and API/ABI/schema snapshot checks are tracked by
later requirements and release gates. Local `actionlint` is unavailable. No
unavailable check is claimed as passed.
