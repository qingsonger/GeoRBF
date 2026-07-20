# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Integration state / REQ-CONVEX-001 complete
- Requirement: REQ-CONVEX-001, Issue #81 (closed)
- Implementation pull request: #82, squash-merged as `742ee57`
- Integration-state branch: `codex/req-convex-001-integration-state`
- Integration-state pull request: pending
- Review record: `docs/reviews/PR-82-INDEPENDENT-REVIEW.md`
- Registry state in this change: `integrated`
- Next eligible requirement: REQ-INFEAS-001 (`planned`)

## Integration result

- A fresh read-only project `math_reviewer` independently reviewed the exact
  repaired PR head using only bounded requirement, dependency, normative-
  document, diff, test, benchmark, registry, handoff, and CI evidence. It did
  not inherit implementation or Repair reasoning.
- R82-001 through R82-008 are closed. No P0, P1, P2, or P3 finding remains.
- Structurally zero objectives now use a recorded dimensionless objective-unit
  reference. Original row values convert it to componentwise gradient units;
  no raw dimensioned floor or tolerance multiplier was added.
- The adapter positively infinity-normalizes independent zero/nonnegative rows
  and whole Lorentz blocks before dispatch, maps backend slack and dual values
  back to original units, and records the complete scaling vector.
- The public hard-only `x >= 1` regression succeeds at row scales `1e-12`, `1`,
  and `1e12`, with every normalized KKT and hard-relation review at or below the
  exact requested `1e-9` tolerance. A synthetic nonstationary dual is still
  rejected.
- Exact implementation Ready head `e6a3621` passed Windows, Ubuntu, and macOS
  with every configured backend and benchmark-smoke workload in CI run
  29711773645.
- PR #82 squash-merged exactly once as `742ee57`; Issue #81 closed as
  completed.
- Post-merge `main` run 29712183062 passed the same complete three-platform
  correctness, backend, benchmark-smoke, and requirement-registry gate on
  `742ee57`.
- This isolated integration-state change updates only the registry, review
  evidence, history index, and bounded handoff. It changes no production code,
  test, manifest, schema, CI, build input, API, normative contract, numerical
  behavior, dependency, tag, or release.

## Validation state

- Focused warning-denying all-target/all-feature Clippy, all six private convex
  tests, all ten convex integration tests, the runnable example, the 8/16
  benchmark smoke workload, and `git diff --check` passed after the final
  production/test change. Smoke checksums remained
  `4.00000000000000444` and `7.99999999999999911`; timings are not performance
  promises.
- Exact code/test head `c1753bdb98e6abec69486c36713d887491204f67`
  passed the complete standard gate: workspace format, warning-denying
  all-target/all-feature Clippy, all-feature workspace tests, workspace
  Rustdoc, all 58 requirement checks, and `git diff --check`.
- The later solver-policy, requirement change fragment, review evidence, and
  registry test-name update change no production, test, manifest, schema, CI,
  build, API, numerical, or dependency input. The updated registry separately
  passed all 58 requirement checks and exact whitespace checks.
- Exact implementation Ready-head and post-merge `main` three-platform gates
  are green as recorded above.
- The isolated integration-state tree passed the complete local standard gate:
  workspace format, warning-denying all-target/all-feature Clippy, all-feature
  workspace tests, workspace Rustdoc, all 58 requirement checks, and
  `git diff --check`.

## Next task

After the isolated integration-state pull request is green and merged, open a
fresh task and perform the mandatory preflight. Use
`cargo xtask requirements next`; do not start REQ-INFEAS-001 or another
requirement in this task.

## Durable evidence

- Acceptance criteria and exclusions: closed GitHub Issue #81
- Merged implementation and repairs: GitHub PR #82
- Integration-state pull request: pending
- Independent review and repair evidence:
  `docs/reviews/PR-82-INDEPENDENT-REVIEW.md`
- Requirement summary: `changes/REQ-CONVEX-001.md`
- Solver policy and backend decision: `docs/architecture/SOLVER_POLICY.md` and
  `docs/adr/ADR-0011-clarabel-convex-backend.md`
- Focused tests: `crates/georbf/tests/convex_solver.rs`
- Example and benchmark: `crates/georbf/examples/convex_solver.rs`,
  `crates/georbf/benches/convex_solver.rs`, and
  `docs/benchmarks/REQ-CONVEX-001.md`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable. Exact OSV and repository advisory API
queries are the performed dependency review; unavailable tools are not claimed.
