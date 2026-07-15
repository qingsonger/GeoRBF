# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair / Issue #43 bounded-handoff correction
- Requirement: REQ-SPIKE-002, Issue #40 (closed)
- Implementation pull request: #41, squash-merged as `4c1ddeb`
- Integration-state pull request: #42, squash-merged as
  `d8ce7508c51f77b8d50245a8d1255ffad2d44c92`
- Repair branch: `codex/issue-43-correct-integration-handoff`
- Repair pull request: pending creation
- Review record: `docs/reviews/PR-41-INDEPENDENT-REVIEW.md`
- Registry state: `integrated`
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
- Exact integration-state ready head `efd2221` passed the complete Windows,
  Ubuntu, and macOS CI matrix in run 29380377235.
- PR #42 squash-merged exactly once as
  `d8ce7508c51f77b8d50245a8d1255ffad2d44c92`; post-merge `main` run
  29380658715 passed the complete three-platform correctness, benchmark-smoke,
  and requirement-registry gate on that exact commit.
- The REQ-SPIKE-002 implementation and integration-state sequence is complete.

## Validation state

- On stable implementation code/test head `30bd495`, the focused spike matrix
  and complete standard workspace gate passed. Later implementation-branch
  commits changed documentation only.
- Exact implementation and integration-state ready-head and post-merge `main`
  three-platform gates are green as recorded above.
- The complete local integration-state standard gate passed: formatting,
  warning-denying workspace Clippy with all targets and features, workspace
  tests with all features, workspace rustdoc, all 58 requirement checks, and
  `git diff --check`.
- This Issue #43 repair changes only this bounded handoff. The diff against
  post-merge `main` proves that no production code, test, manifest, schema,
  build input, API, numerical behavior, dependency, tag, or release changed,
  so the immutable full-gate evidence above is reused.

## Next task

After this bounded-handoff repair pull request is reviewed and merged, open a
fresh Implement task. Perform the mandatory preflight and use
`cargo xtask requirements next`; do not start REQ-CPD-001 in this Repair task.

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
