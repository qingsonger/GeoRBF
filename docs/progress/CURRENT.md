# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Integration state / REQ-SPIKE-002 complete
- Requirement: REQ-SPIKE-002, Issue #40 (closed)
- Implementation pull request: #41, squash-merged as `4c1ddeb`
- Integration-state branch: `codex/req-spike-002-integration-state`
- Integration-state pull request: pending
- Review record: `docs/reviews/PR-41-INDEPENDENT-REVIEW.md`
- Registry state in this change: `integrated`
- Next eligible requirement: REQ-CPD-001 (`planned`)
- Production dependency state: unchanged; the comparison crate is excluded
  from the production workspace

## Integration result

- A second fresh read-only `math_reviewer` inspected the complete PR diff at
  exact head `66ed708` without inheriting Repair reasoning.
- P2-1, P2-2, and P3-1 are independently confirmed closed. No P0, P1, P2, or
  P3 issue remains.
- Exact ready head `3e6f4e1` passed the complete Windows, Ubuntu, and macOS CI
  matrix with every benchmark smoke workload in run 29376057562.
- PR #41 squash-merged exactly once as `4c1ddeb`; Issue #40 closed as
  completed.
- Post-merge `main` run 29376336046 passed the same complete three-platform
  correctness, benchmark-smoke, and requirement-registry gate.
- This isolated integration-state change updates only registry and progress
  evidence. It changes no production code, tests, manifest, schema, build
  input, API, numerical behavior, dependency, tag, or release.

## Validation state

- On stable implementation code/test head `30bd495`, the focused spike matrix
  and complete standard workspace gate passed. Later implementation-branch
  commits changed documentation only.
- Exact ready-head and post-merge `main` three-platform gates are green as
  recorded above.
- The complete local standard gate must pass on the final integration-state
  head before its pull request is marked ready.

## Next task

After the isolated integration-state pull request is green and merged, open a
fresh Implement task. Perform the mandatory preflight and use
`cargo xtask requirements next`; do not start REQ-CPD-001 in this task.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #40
- Decision: `docs/adr/ADR-0009-nalgebra-rank-review-backend.md`
- Independent review: `docs/reviews/PR-41-INDEPENDENT-REVIEW.md`
- Change summary: `changes/REQ-SPIKE-002.md`
- Benchmark and size baseline: `docs/benchmarks/REQ-SPIKE-002.md`
- Reproducible harness: `spikes/rank-backends/`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, allocation instrumentation, and API/ABI/schema
snapshot checks are tracked by later requirements and release gates. Local
`actionlint` is unavailable.
