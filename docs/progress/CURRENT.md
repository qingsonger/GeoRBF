# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Integration state / REQ-INFEAS-001 complete
- Requirement: REQ-INFEAS-001, Issue #84 (closed)
- Implementation pull request: #85, squash-merged as `0262a04`
- Integration-state branch: `codex/req-infeas-001-integration-state`
- Integration-state pull request: pending
- Review record: `docs/reviews/PR-85-INDEPENDENT-REVIEW.md`
- Registry state in this change: `integrated`
- Next eligible requirement: REQ-NORMAL-001 (`planned`)

## Integration result

- A fresh read-only project `math_reviewer` independently reviewed the exact
  complete PR and R85-003 repair using only bounded requirement, dependency,
  normative, diff, test, benchmark, registry, handoff, and validation evidence.
- R85-001, R85-002, and R85-003 are closed. Exact proportionality and interval
  decisions remain binary-exact and scale/sign invariant; approximate rows
  remain warning-only; hard rows remain unchanged; soft objectives and cones
  remain separate; the Ready-only constraint-diagnostics smoke spans Windows,
  Ubuntu, and macOS.
- No P0, P1, P2, or P3 finding remained before integration; the independent
  review requirement is complete.
- Exact implementation Ready head `4fec622` passed Windows, Ubuntu, and macOS
  with every configured backend and benchmark-smoke workload, including
  `constraint_diagnostics`, in CI run 29719420711.
- PR #85 squash-merged exactly once as `0262a04`; Issue #84 closed as
  completed.
- Post-merge `main` run 29719948949 passed the same complete three-platform
  correctness, backend, benchmark-smoke, and requirement-registry gate on
  `0262a04`.
- This isolated integration-state change updates only the registry, review
  evidence, history index, and bounded handoff. It changes no production code,
  test, manifest, schema, CI, build input, API, normative contract, numerical
  behavior, dependency, tag, or release.

## Validation state

- The focused `constraint_diagnostics --smoke` command passed locally with
  checksum `768`; `git diff --check` passed.
- After the last CI/build-input change, stable repair head `1982d89` passed the
  complete standard gate: format, workspace all-targets/all-features Clippy,
  workspace all-features tests, workspace doctests, and all 58 requirement
  checks.
- The final reviewer passed all eight infeasibility tests, all eight linear-
  constraint tests, four provenance-allocation fault tests, the benchmark smoke
  with checksum `768`, the example, GeoRBF all-target/all-feature Clippy,
  format, all 58 requirement checks, and both requested diff checks.
- Draft CI run 29718863367 passed the configured Ubuntu gate on exact reviewed
  head `0c465e7`. The Ready-only Windows/Ubuntu/macOS matrix correctly did not
  run while the PR was Draft.
- Exact implementation Ready-head and post-merge `main` three-platform gates
  are green as recorded above.
- The isolated integration-state tree passed the complete local standard gate:
  workspace format, warning-denying all-target/all-feature Clippy, all-feature
  workspace tests, workspace Rustdoc, all 58 requirement checks, and
  `git diff --check`.
- Local `actionlint` is unavailable and is not claimed as passed. The later
  unavailable-tool dispositions below are unchanged.

## Next task

After the isolated integration-state pull request is green and merged, open a
fresh task and perform the mandatory preflight. Use
`cargo xtask requirements next`; do not start another requirement in this
task.

## Durable evidence

- Acceptance criteria and exclusions: closed GitHub Issue #84
- Merged implementation and repairs: GitHub PR #85
- Integration-state pull request: pending
- Independent review: `docs/reviews/PR-85-INDEPENDENT-REVIEW.md`
- Requirement summary: `changes/REQ-INFEAS-001.md`
- Focused tests: `crates/georbf/tests/infeasibility.rs`
- Normative behavior: `docs/math/CONSTRAINT_SEMANTICS.md`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable. No unavailable check is claimed as passed.
