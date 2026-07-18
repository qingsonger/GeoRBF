# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Integration state / REQ-LEVEL-001 complete
- Requirement: REQ-LEVEL-001, Issue #69 (closed)
- Implementation pull request: #70, squash-merged as `11e0659`
- Integration-state branch: `codex/req-level-001-integration-state`
- Integration-state pull request: pending
- Review record: `docs/reviews/PR-70-INDEPENDENT-REVIEW.md`
- Registry state in this change: `integrated`
- Next eligible requirement: REQ-SOFT-001 (`planned`)

## Integration result

- A fresh read-only project `math_reviewer` independently reconfirmed that
  R70-001 through R70-014 are closed and no P0-P3 finding remains.
- Exact Ready head `5bfa52f81f31785a660d7446c55099e570e29521`
  passed the complete Windows, Ubuntu, and macOS matrix with every backend and
  benchmark-smoke workload in CI run 29646041086.
- PR #70 squash-merged exactly once as
  `11e0659319ae08731f083749974d9ad6fb316616`; Issue #69 closed as
  completed.
- Post-merge `main` run 29646382654 passed the same complete three-platform
  correctness, benchmark-smoke, and requirement-registry gate on `11e0659`.
- This isolated integration-state change updates only the registry, review
  evidence, history index, and bounded handoff. It changes no production code,
  test, manifest, schema, CI, build input, API, normative contract, numerical
  behavior, dependency, tag, or release.

## Validation state

- Exact implementation tree `61fa6d3` passed the complete standard workspace
  gate: format, warning-denying workspace Clippy, all-feature workspace tests,
  workspace Rustdoc, all 58 requirement checks, and `git diff --check`.
- The clean reviewers independently repeated focused level and diagnostic
  tests, core Rustdoc, benchmark smoke, requirement and whitespace checks, and
  exact-rational scaled-arithmetic probes.
- Exact implementation Ready-head and post-merge `main` three-platform gates
  are green as recorded above.
- The isolated integration-state tree passed the complete local standard gate:
  format, warning-denying workspace Clippy, all-feature workspace tests,
  workspace Rustdoc, all 58 requirement checks, and `git diff --check`.

## Next task

After the isolated integration-state pull request is green and merged, open a
fresh task and perform the mandatory preflight. Use
`cargo xtask requirements next`; do not start REQ-SOFT-001 or another
requirement in this task.

## Durable evidence

- Acceptance criteria and exclusions: closed GitHub Issue #69
- Merged implementation and repairs: GitHub PR #70
- Integration-state pull request: pending
- Independent review and repair evidence:
  `docs/reviews/PR-70-INDEPENDENT-REVIEW.md`
- Requirement summary: `changes/REQ-LEVEL-001.md`
- Mathematical semantics: `docs/math/CONSTRAINT_SEMANTICS.md`
- Accepted design: `docs/adr/ADR-0003-explicit-level-variables.md`
- Architecture: `docs/architecture/ARCHITECTURE.md`
- Focused tests: `crates/georbf/tests/levels.rs`
- Benchmark: `crates/georbf/benches/level_compilation.rs`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable.
