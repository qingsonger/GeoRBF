# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Integration state / REQ-SPIKE-001 complete
- Requirement: REQ-SPIKE-001, Issue #48 (closed)
- Implementation pull request: #49, squash-merged as `2e4cfbc`
- Integration-state branch: `codex/req-spike-001-integration-state`
- Integration-state pull request: #50 (Draft until final evidence is green)
- Review record: `docs/reviews/PR-49-INDEPENDENT-REVIEW.md`
- Registry state in this change: `integrated`
- Next eligible requirement: REQ-IR-001 (`planned`)

## Integration result

- A fresh read-only `math_reviewer` closed P3-2 and found no remaining P0,
  P1, P2, or P3 issue in the complete implementation diff.
- Exact ready head `2665f0a` passed the complete Windows, Ubuntu, and macOS CI
  matrix with every benchmark smoke workload in run 29406083900.
- PR #49 squash-merged exactly once as `2e4cfbc`; Issue #48 closed as
  completed.
- Post-merge `main` run 29406759904 passed the same complete three-platform
  correctness, benchmark-smoke, and requirement-registry gate.
- This isolated integration-state change updates only registry and progress
  evidence. It changes no production code, tests, manifest, schema, CI, build
  input, API, numerical behavior, dependency, tag, or release.

## Validation state

- The primary task ran the complete standard workspace gate and
  `git diff --check` on exact clean-review head `7e36551`; all passed. Later
  implementation-branch commits changed only review evidence and the bounded
  handoff.
- Exact ready-head and post-merge `main` three-platform gates are green as
  recorded above.
- The integration-state branch must pass the complete local standard gate and
  `git diff --check` after its final evidence update.

## Next task

After the isolated integration-state pull request is green and merged, open a
fresh Implement task. Perform the mandatory preflight and use
`cargo xtask requirements next`; do not start REQ-IR-001 in this task.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #48
- Merged implementation: GitHub PR #49
- Integration-state pull request: GitHub PR #50
- Independent review and repair evidence:
  `docs/reviews/PR-49-INDEPENDENT-REVIEW.md`
- Solver policy: `docs/architecture/SOLVER_POLICY.md`
- Backend decision: `docs/adr/ADR-0010-nalgebra-dense-factorization-backend.md`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, allocation instrumentation, and API/ABI/schema
snapshot checks are tracked by later requirements and release gates. Local
`actionlint` is unavailable. The performed advisory review was an OSV batch API
query of every exact selected package; it is not a claim that unavailable
audit tools ran.
