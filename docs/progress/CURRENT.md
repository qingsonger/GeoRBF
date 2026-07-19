# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Clean independent re-review complete / Ready integration sequence in progress
- Requirement: REQ-SPIKE-004, Issue #78
- Branch: `codex/req-spike-004-qp-socp-backends`
- Draft pull request: #79
- Exact repaired implementation and complete local-gate head:
  `4b57e72c04e4e8dd7d5ce2c819ca2a02495cdf2c`
- Cleanly re-reviewed repair head:
  `4b57e72c04e4e8dd7d5ce2c819ca2a02495cdf2c`
- Registry state in this change: `implemented`
- Integrated dependency: REQ-BOOTSTRAP-001
- Review record: `docs/reviews/PR-79-INDEPENDENT-REVIEW.md`

## Re-review result

- A fresh read-only project `math_reviewer` independently reviewed exact repair
  head `4b57e72c` against base `5b5db20f` using only bounded requirement,
  dependency, normative-document, diff, test, benchmark, and CI evidence.
- R79-001 and R79-002 are closed. Certificate review is scale invariant, zero
  and nonfinite vectors fail, and both QP benchmark fixtures use directly
  constructed O(n) CSC data with exact semantic regressions.
- The reviewer independently confirmed the QP and SOCP analytic optima,
  objective and row signs, cone ordering, exact statuses, Farkas conditions,
  hard-constraint preservation, explicit settings, and benchmark claims.
- No P0, P1, P2, or P3 finding remains. The exact repaired head is safe to mark
  Ready; merge remains forbidden until its complete Ready CI is green.

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
- Exact repaired Draft head `4b57e72c` passed Ubuntu CI run 29678281944. The
  Ready-only Windows/Ubuntu/macOS and benchmark-smoke matrix remains pending.
- This re-review task changes only review and bounded-handoff documentation;
  no production, test, manifest, schema, CI, build, registry, API, numerical,
  or dependency input changes.

## Next task

Synchronize this evidence-only review and handoff commit on PR #79, mark the PR
Ready, and wait for the complete Windows, Ubuntu, and macOS correctness matrix
plus every benchmark smoke workload on that exact final evidence head. Merge
exactly once only when green, then record truthful integration state in an
isolated follow-up change. If this task is interrupted, resume only this
integration sequence; do not begin REQ-CONVEX-001.

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable. Exact OSV and GitHub advisory API queries
from the implementation task remain the performed dependency review;
unavailable tools are not claimed.
