# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Integration state / REQ-FUNC-001 complete
- Requirement: REQ-FUNC-001, Issue #37 (closed)
- Implementation pull request: #38, squash-merged as `2879f13`
- Integration-state branch: `codex/req-func-001-integration-state`
- Integration-state pull request: pending creation after the first push
- Review record: `docs/reviews/PR-38-INDEPENDENT-REVIEW.md`
- Registry state in this integration change: `integrated`
- Next eligible requirement: REQ-SPIKE-002 (`planned`)

## Integration result

- A fresh independent mathematical and numerical re-review found no P0, P1,
  P2, or P3 issue. P1-1 is closed by exact demand-bounded kernel-jet prefixes
  and the coincident Matérn regressions.
- Exact ready head `4bf62ca` passed the complete Windows, Ubuntu, and macOS CI
  matrix with every benchmark smoke workload in run 29340242329.
- PR #38 squash-merged exactly once as `2879f13`; Issue #37 closed as
  completed.
- Post-merge `main` run 29340402183 passed the same complete three-platform
  correctness, benchmark-smoke, and requirement-registry gate.
- The complete local integration-state standard gate passed: formatting,
  workspace Clippy with all targets/features, workspace tests with all
  features, workspace rustdoc, and all 58 requirement checks.
- This isolated integration-state change contains no production, test,
  manifest, schema, build-input, API, or numerical-behavior change.

## Next task

Open a fresh Implement task for only REQ-SPIKE-002. Perform the mandatory
preflight, create its Issue with explicit acceptance criteria, confirm its
dependency closure remains integrated, and read only its listed normative
documents and relevant ADRs before beginning the backend evaluation. Do not
continue directly from this integration task.

## Durable evidence

- Requirement summary: `changes/REQ-FUNC-001.md`
- Benchmark baseline: `docs/benchmarks/REQ-FUNC-001.md`
- Independent review: `docs/reviews/PR-38-INDEPENDENT-REVIEW.md`
- Acceptance criteria and exclusions: GitHub Issue #37
- Exact CI and merge history: GitHub PR #38 and runs 29340242329 and
  29340402183

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, allocation instrumentation, and API/ABI/schema
snapshot checks are tracked by later requirements and release gates. Local
`actionlint` is unavailable.
