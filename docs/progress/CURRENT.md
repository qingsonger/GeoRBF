# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Implement
- Requirement: REQ-SPIKE-004, Issue #78
- Branch: `codex/req-spike-004-qp-socp-backends`
- Draft pull request: #79
- Registry state in this change: `implemented`
- Integrated dependency: REQ-BOOTSTRAP-001

## Implemented scope

- Added the excluded `spikes/convex-backends` comparison harness pinned to
  Clarabel 0.11.1 and OSQP 1.0.1; the production graph remains unchanged.
- Both candidates pass the same analytic constrained QP and reviewed
  primal-infeasibility certificate. Clarabel also passes analytic and
  infeasible Lorentz-cone cases.
- Accepted ADR-0011 selecting the Clarabel 0.11 release line for later private
  QP/SOCP adapter adoption, subject to the recorded production re-audit and
  diagnostic conditions.
- Added single-feature, combined-feature, negative-empty-feature, lint, and
  benchmark-smoke CI coverage for the existing three-platform matrix.
- Updated solver policy, benchmark evidence, changelog, and requirement state.

## Validation state

- Focused harness lint passed with warnings denied.
- All seven combined-feature tests passed; Clarabel-only passed six and
  OSQP-only passed four.
- The release smoke workload passed. Three complete release benchmark runs had
  bit-identical per-backend checksums and are recorded in
  `docs/benchmarks/REQ-SPIKE-004.md`.
- Exact single-feature release harness sizes, reachable graphs, licenses,
  declared MSRVs, source exposure, upstream activity, and advisory API results
  are recorded in ADR-0011.
- Exact implementation commit `682c9a6` passed the complete standard workspace
  gate: format, warning-denying all-target/all-feature Clippy, all-feature
  workspace tests, workspace Rustdoc, all 58 requirement checks, and
  `git diff --check`.

## Next task

After this implementation is committed, pushed, and represented by a Draft PR,
open a fresh Review task for only REQ-SPIKE-004 and that PR. Supply the
requirement summary and dependencies, ADR-0007, ADR-0011, solver policy, the PR
diff, and validation evidence to an independent `math_reviewer`. Do not repair
production code or start REQ-CONVEX-001 in that Review task.

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable. Exact OSV and GitHub advisory API queries
were used for the dependency review; unavailable tools are not claimed.
