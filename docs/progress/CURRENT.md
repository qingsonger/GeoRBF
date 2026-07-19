# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Integration state / REQ-LINEQ-001 complete
- Requirement: REQ-LINEQ-001, Issue #75 (closed)
- Implementation pull request: #76, squash-merged as `42768a8`
- Integration-state branch: `codex/req-lineq-001-integration-state`
- Integration-state pull request: #77 (Draft until exact Ready CI is green)
- Review record: `docs/reviews/PR-76-INDEPENDENT-REVIEW.md`
- Registry state in this change: `integrated`
- Next eligible requirement: REQ-SPIKE-004 (`planned`)

## Integration result

- A fresh read-only project `math_reviewer` independently confirmed R76-001 is
  closed and no P0-P3 finding remains.
- Exact implementation Ready head
  `1541eb761ce7acf7dec8d4445875f499a6868804` passed Windows, Ubuntu, and
  macOS with every configured backend and benchmark-smoke workload in CI run
  29671462544.
- PR #76 squash-merged exactly once as
  `42768a80cadd261d9d45e35a920e8ac7cc929558`; Issue #75 closed as
  completed.
- Post-merge `main` run 29671754311 passed the same complete three-platform
  correctness, backend, benchmark-smoke, and requirement-registry gate on
  `42768a8`.
- This isolated integration-state change updates only the registry, review
  evidence, history index, and bounded handoff. It changes no production code,
  test, manifest, schema, CI, build input, API, normative contract, numerical
  behavior, dependency, tag, or release.

## Validation state

- Exact repaired implementation tree `b1f15d5` passed the complete standard
  workspace gate: format, warning-denying all-target/all-feature Clippy,
  all-feature workspace tests, workspace Rustdoc, all 58 requirement checks,
  and `git diff --check`.
- The clean reviewer independently passed all eight linear-constraint tests,
  all 21 level tests, four provenance-allocation tests, all 30 georbf doctests,
  the example, benchmark smoke, formatting, all 58 requirement checks, and
  complete-PR whitespace checks.
- Exact implementation Ready-head and post-merge `main` three-platform gates
  are green as recorded above.
- The isolated integration-state tree passed the complete local standard gate:
  format, warning-denying all-target/all-feature Clippy, all-feature workspace
  tests, workspace Rustdoc, all 58 requirement checks, and `git diff --check`.

## Next task

After the isolated integration-state pull request is green and merged, open a
fresh task and perform the mandatory preflight. Use
`cargo xtask requirements next`; do not start REQ-SPIKE-004 or another
requirement in this task.

## Durable evidence

- Acceptance criteria and exclusions: closed GitHub Issue #75
- Merged implementation and repair: GitHub PR #76
- Integration-state pull request: GitHub PR #77
- Independent review and repair evidence:
  `docs/reviews/PR-76-INDEPENDENT-REVIEW.md`
- Requirement summary: `changes/REQ-LINEQ-001.md`
- Mathematical semantics: `docs/math/CONSTRAINT_SEMANTICS.md`
- Architecture: `docs/architecture/PROBLEM_IR.md` and
  `docs/architecture/ARCHITECTURE.md`
- Focused tests: `crates/georbf/tests/linear_constraints.rs` plus retained
  layer-order coverage in `crates/georbf/tests/levels.rs`
- Example: `crates/georbf/examples/linear_constraints.rs`
- Benchmark and report:
  `crates/georbf/benches/linear_constraint_compilation.rs` and
  `docs/benchmarks/REQ-LINEQ-001.md`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable.
