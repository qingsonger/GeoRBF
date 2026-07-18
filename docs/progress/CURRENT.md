# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Integration state / REQ-SOFT-001 complete
- Requirement: REQ-SOFT-001, Issue #72 (closed)
- Implementation pull request: #73, squash-merged as `988217c`
- Integration-state branch: `codex/req-soft-001-integration-state`
- Integration-state pull request: pending
- Review record: `docs/reviews/PR-73-INDEPENDENT-REVIEW.md`
- Registry state in this change: `integrated`
- Next eligible requirement: REQ-LINEQ-001 (`planned`)

## Integration result

- A fresh read-only project `math_reviewer` independently confirmed R73-001 is
  closed and no P0-P3 finding remains.
- Exact Ready head `4b6b24d151ec9af3db192b7ff496527d21d2748b`
  passed the complete Windows, Ubuntu, and macOS matrix with every configured
  backend and benchmark-smoke workload in CI run 29664739560.
- PR #73 squash-merged exactly once as
  `988217cdebf4c49f2db893e001acec1c7d6e0923`; Issue #72 closed as
  completed.
- Post-merge `main` run 29665084708 passed the same complete three-platform
  correctness, benchmark-smoke, and requirement-registry gate on `988217c`.
- This isolated integration-state change updates only the registry, review
  evidence, history index, and bounded handoff. It changes no production code,
  test, manifest, schema, CI, build input, API, normative contract, numerical
  behavior, dependency, tag, or release.

## Validation state

- Exact implementation tree `530f6fd` passed the complete standard workspace
  gate: format,
  warning-denying all-target/all-feature Clippy, all-feature workspace tests,
  workspace Rustdoc, all 58 requirement checks, and `git diff --check`.
- The clean reviewer independently repeated the repaired capability regression,
  requirement and dependency checks, and whitespace checks; the parent Review
  task passed all focused soft-loss, problem-IR, level, and Rustdoc tests plus
  the D=1/D=2/D=3 96-constraint benchmark smoke.
- Exact implementation Ready-head and post-merge `main` three-platform gates
  are green as recorded above.
- The isolated integration-state tree passed the complete local standard gate:
  format, warning-denying all-target/all-feature Clippy, all-feature workspace
  tests, workspace Rustdoc, all 58 requirement checks, and `git diff --check`.

## Next task

After the isolated integration-state pull request is green and merged, open a
fresh task and perform the mandatory preflight. Use
`cargo xtask requirements next`; do not start REQ-LINEQ-001 or another
requirement in this task.

## Durable evidence

- Acceptance criteria and exclusions: closed GitHub Issue #72
- Merged implementation and repair: GitHub PR #73
- Integration-state pull request: pending
- Independent review and repair evidence:
  `docs/reviews/PR-73-INDEPENDENT-REVIEW.md`
- Requirement summary: `changes/REQ-SOFT-001.md`
- Mathematical semantics: `docs/math/CONSTRAINT_SEMANTICS.md`
- Accepted level-prior design: `docs/adr/ADR-0003-explicit-level-variables.md`
- Architecture: `docs/architecture/ARCHITECTURE.md`
- Focused tests: `crates/georbf/tests/soft_losses.rs`,
  `crates/georbf/tests/problem_ir.rs`, and `crates/georbf/tests/levels.rs`
- Benchmark: `crates/georbf/benches/soft_objective_compilation.rs`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable.
