# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Integration state / REQ-CPD-001 complete
- Requirement: REQ-CPD-001, Issue #45 (closed)
- Implementation pull request: #46, squash-merged as `0c19373`
- Integration-state branch: `codex/req-cpd-001-integration-state`
- Integration-state pull request: pending creation
- Review record: `docs/reviews/PR-46-INDEPENDENT-REVIEW.md`
- Registry state in this change: `integrated`
- Next eligible requirement: REQ-SPIKE-001 (`planned`)

## Integration result

- A fifth fresh read-only `math_reviewer` closed P2-5 and found no remaining
  P0, P1, P2, or P3 issue in the complete implementation diff.
- Exact ready head `bf69ed4` passed the complete Windows, Ubuntu, and macOS CI
  matrix with every benchmark smoke workload in run 29396342123.
- PR #46 squash-merged exactly once as `0c19373`; Issue #45 closed as
  completed.
- Post-merge `main` run 29396715017 passed the same complete three-platform
  correctness, benchmark-smoke, and requirement-registry gate.
- This isolated integration-state change updates only registry and progress
  evidence. It changes no production code, tests, manifest, schema, build
  input, API, numerical behavior, dependency, tag, or release.

## Validation state

- On stable implementation code/test head `06ad419`, the focused CPD suite and
  complete standard workspace gate passed. Later implementation-branch commits
  changed only review evidence and the bounded handoff.
- Exact ready-head and post-merge `main` three-platform gates are green as
  recorded above.
- The integration-state branch must pass the complete local standard gate and
  `git diff --check` after its final evidence update.

## Next task

After the isolated integration-state pull request is green and merged, open a
fresh Implement task. Perform the mandatory preflight and use
`cargo xtask requirements next`; do not start REQ-SPIKE-001 in this task.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #45
- Merged implementation: GitHub PR #46
- Integration-state pull request: pending creation
- Independent review and repair evidence:
  `docs/reviews/PR-46-INDEPENDENT-REVIEW.md`
- Mathematical contract: `docs/math/CPD_AND_POLYNOMIALS.md`
- Solver policy: `docs/architecture/SOLVER_POLICY.md`
- Backend decision and production re-audit:
  `docs/adr/ADR-0009-nalgebra-rank-review-backend.md`
- Change summary: `changes/REQ-CPD-001.md`
- Benchmark and size baseline: `docs/benchmarks/REQ-CPD-001.md`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, allocation instrumentation, and API/ABI/schema
snapshot checks are tracked by later requirements and release gates. Local
`actionlint` is unavailable.
