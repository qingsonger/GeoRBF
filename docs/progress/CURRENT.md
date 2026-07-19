# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review complete / Repair required
- Requirement: REQ-SPIKE-004, Issue #78
- Branch: `codex/req-spike-004-qp-socp-backends`
- Draft pull request: #79
- Registry state in this change: `implemented`
- Integrated dependency: REQ-BOOTSTRAP-001
- Review record: `docs/reviews/PR-79-INDEPENDENT-REVIEW.md`

## Review result

- A fresh read-only project `math_reviewer` reviewed exact PR head `10e0266`
  against base `5b5db20` using only the bounded requirement, dependency,
  normative-document, diff, test, benchmark, and CI evidence.
- R79-001 (P1): absolute certificate thresholds can accept near-zero vectors
  with unit relative stationarity error. Repair must use normalized or scale-
  aware certificate review and add invalid-scale and valid-rescaling tests.
- R79-002 (P2): the timed QP fixture builds Clarabel inputs in O(n) work but
  emits two n-squared OSQP identities inside the timed region. Repair must make
  construction equivalent or separate setup/solve timing, add sparse-fixture
  regressions, and regenerate the affected benchmark table and prose.
- The reviewer independently confirmed the QP and SOCP analytic optima,
  canonical signs and cones, exact-status paths, and dual-cone conventions.
  No other P0-P3 finding was reported.

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
- Exact reviewed Draft head `10e0266` passed Ubuntu CI run 29674034129. The
  Ready-only Windows/Ubuntu/macOS and benchmark-smoke matrix remains pending.
- This Review task changes only review and handoff documentation. Workspace
  formatting, all 58 requirement checks, and staged whitespace checks passed.

## Next task

Open a fresh Repair task for only PR #79 findings R79-001 and R79-002. Reproduce
both findings, add the required certificate-scale and sparse-fixture
regressions, implement the smallest repairs, regenerate the affected benchmark
evidence, run focused checks and the final standard workspace gate, update the
review record and this bounded handoff, commit, push, and stop for a fresh
independent re-review. Do not begin REQ-CONVEX-001.

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable. Exact OSV and GitHub advisory API queries
were used for the dependency review; unavailable tools are not claimed.
