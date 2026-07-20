# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair complete; fresh independent re-review required / REQ-CONVEX-001
- Requirement: REQ-CONVEX-001, Issue #81
- Implementation branch: `codex/req-convex-001-canonical-solver`
- Implementation pull request: #82 (Draft)
- R82-008 repair code/test head: `c1753bdb98e6abec69486c36713d887491204f67`
- Repair evidence head: `26da57b86d8865604285e3e1cfeea8f124329763`
- Review record: `docs/reviews/PR-82-INDEPENDENT-REVIEW.md`
- Registry state: `implemented`; it is not `integrated`
- Direct dependencies REQ-SOFT-001, REQ-LINEQ-001, and REQ-SPIKE-004 are all
  `integrated`

## R82-008 repair disposition

- R82-001 and R82-003 through R82-007 remain closed by the prior independent
  re-review. This Repair did not modify their disposition.
- R82-008 is implemented but remains open pending fresh independent re-review.
- Structurally zero objectives now use a recorded dimensionless objective-unit
  reference. Original row values convert it to componentwise gradient units;
  no raw dimensioned floor or tolerance multiplier was added.
- The adapter positively infinity-normalizes independent zero/nonnegative rows
  and whole Lorentz blocks before dispatch, maps backend slack and dual values
  back to original units, and records the complete scaling vector.
- The public hard-only `x >= 1` regression succeeds at row scales `1e-12`, `1`,
  and `1e12`, with every normalized KKT and hard-relation review at or below the
  exact requested `1e-9` tolerance. A synthetic nonstationary dual is still
  rejected.
- PR #82 remains Draft. REQ-INFEAS-001 has not begun.

## Validation state

- Focused warning-denying all-target/all-feature Clippy, all six private convex
  tests, all ten convex integration tests, the runnable example, the 8/16
  benchmark smoke workload, and `git diff --check` passed after the final
  production/test change. Smoke checksums remained
  `4.00000000000000444` and `7.99999999999999911`; timings are not performance
  promises.
- Exact code/test head `c1753bdb98e6abec69486c36713d887491204f67`
  passed the complete standard gate: workspace format, warning-denying
  all-target/all-feature Clippy, all-feature workspace tests, workspace
  Rustdoc, all 58 requirement checks, and `git diff --check`.
- The later solver-policy, requirement change fragment, review evidence, and
  registry test-name update change no production, test, manifest, schema, CI,
  build, API, numerical, or dependency input. The updated registry separately
  passed all 58 requirement checks and exact whitespace checks.
- Draft CI run 29710719948 passed the configured Ubuntu job on pre-repair head
  `3117f874`. Ready-only Windows, Ubuntu, macOS, and benchmark-smoke CI has not
  run for the repaired head. A new Draft CI run is expected after push.

## Next task

Open a fresh independent re-review task for Draft PR #82 and REQ-CONVEX-001,
limited to R82-008 and regression risk from its repair. Supply only the bounded
requirement/dependency summaries, Issue #81 criteria, M4 plan, solver policy,
ADR-0011, exact PR diff, review record, tests, benchmark evidence, registry,
this handoff, and exact-head CI evidence to the project `math_reviewer`; do not
inherit Repair reasoning. Verify the zero-objective reference is dimensionally
coherent, backend row scaling preserves each product cone and original-unit
dual/slack/certificate mapping, the three-scale public regression is genuine,
and the synthetic dual remains rejected. If any P0-P3 finding remains, record
it and stop without repair. If the review is clean, follow the mandatory ready
CI and one-merge sequence in `docs/CODEX_WORKFLOW.md`. Do not begin
REQ-INFEAS-001 in this task.

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable. Exact OSV and repository advisory API
queries are the performed dependency review; unavailable tools are not claimed.
