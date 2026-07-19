# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair complete / independent re-review required
- Requirement: REQ-SPIKE-004, Issue #78
- Branch: `codex/req-spike-004-qp-socp-backends`
- Draft pull request: #79
- Registry state in this change: `implemented`
- Integrated dependency: REQ-BOOTSTRAP-001
- Review record: `docs/reviews/PR-79-INDEPENDENT-REVIEW.md`

## Repair result

- R79-001: certificate review now normalizes every nonzero vector by its
  infinity norm before checking dual-cone membership, stationarity, and the
  strict separating inequality. Zero and nonfinite certificates fail.
- Certificate regressions reject the reported near-zero nonstationary
  Clarabel and OSQP vectors and accept positive rescalings of valid
  certificates.
- R79-002: both OSQP QP identity matrices now use direct O(n) CSC construction
  instead of n-squared dense iteration. Backend-specific regressions verify
  dimensions, semantics, and O(n) stored nonzeros for both QP fixtures.
- Three repaired release benchmark runs had bit-identical per-backend
  checksums. The regenerated QP ranges overlap and make no backend-ordering
  claim.

## Validation state

- Focused spike formatting and warning-denying all-target/all-feature Clippy
  passed.
- Combined-feature tests passed 11 cases; Clarabel-only passed 8 and OSQP-only
  passed 6. Empty-backend selection was rejected with the required compile
  error.
- The repaired release smoke workload and three complete release benchmark
  runs passed.
- After the final code change, the stable repair tree passed workspace format,
  warning-denying all-target/all-feature Clippy, all-feature workspace tests,
  workspace Rustdoc, all 58 requirement checks, and `git diff --check`. The
  later edit records only this validation evidence; no production, test,
  manifest, schema, or build input changed.
- PR #79 remains Draft. Ready-only Windows/Ubuntu/macOS and benchmark-smoke CI
  remain pending until a clean fresh re-review.

## Next task

Open a fresh Review/re-review task for only PR #79. Supply the project
`math_reviewer` with the bounded requirement and dependency summaries,
normative documents, exact PR diff, original findings, repaired benchmark
evidence, focused checks, final standard-gate evidence, and Draft CI. Confirm
independently that R79-001 and R79-002 are closed and check for new P0-P3
findings. Do not repair production code in that task. If clean, follow the
mandatory ready, exact-head three-platform plus benchmark-smoke CI, merge, and
isolated integration-state sequence. Do not begin REQ-CONVEX-001.

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable. Exact OSV and GitHub advisory API queries
from the implementation task remain the performed dependency review;
unavailable tools are not claimed.
