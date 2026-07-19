# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Integration state in progress / REQ-SPIKE-004
- Requirement: REQ-SPIKE-004, Issue #78 (closed)
- Implementation pull request: #79, squash-merged as `60f9bb6e`
- Integration-state branch: `codex/req-spike-004-integration-state`
- Review record: `docs/reviews/PR-79-INDEPENDENT-REVIEW.md`
- Registry state in this change: `integrated`
- Next eligible requirement: REQ-CONVEX-001 (`planned`)

## Integration result

- A fresh read-only project `math_reviewer` independently reviewed exact repair
  head `4b57e72c` against base `5b5db20f` using only bounded requirement,
  dependency, normative-document, diff, test, benchmark, and CI evidence.
- R79-001 and R79-002 are closed and no P0-P3 finding remains.
- Exact implementation Ready head
  `7e17e546c5378efbce6b7a325dd61e8c21bd5c48` passed Windows, Ubuntu, and
  macOS with every configured backend and benchmark-smoke workload in CI run
  29679134481.
- PR #79 squash-merged exactly once as
  `60f9bb6e8755b6457a8b509b0357d8ba5ad07551`; Issue #78 closed as completed.
- Post-merge `main` run 29679504405 passed the same complete three-platform
  correctness, backend, benchmark-smoke, and requirement-registry gate on
  exact `60f9bb6e`.
- This isolated integration-state change updates only the registry, review
  evidence, history index, and bounded handoff. It changes no production code,
  test, manifest, schema, CI, build input, API, normative contract, numerical
  behavior, dependency, tag, or release.

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
- The clean reviewer independently confirmed the analytic truth, exact status
  and certificate paths, hard-constraint preservation, explicit settings,
  deterministic evidence, linear-sparse fixtures, interface dispositions, and
  complete-diff whitespace checks.
- Exact implementation Ready-head and post-merge `main` three-platform gates
  are green as recorded above.
- The isolated integration-state tree has not yet run its final complete local
  standard gate; it must pass before the integration-state PR becomes Ready.

## Next task

Publish the isolated integration-state branch as a Draft PR, add its assigned
PR number to the durable index and handoff, run the complete local standard
gate on the final tree, and mark the PR Ready. Wait for exact-head Windows,
Ubuntu, and macOS correctness plus every benchmark smoke, merge exactly once
only when green, then stop. Do not begin REQ-CONVEX-001 in this task.

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable. Exact OSV and GitHub advisory API queries
from the implementation task remain the performed dependency review;
unavailable tools are not claimed.
