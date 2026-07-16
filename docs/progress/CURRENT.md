# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Integration state / REQ-MODEL-001 complete
- Requirement: REQ-MODEL-001, Issue #60 (closed)
- Implementation pull request: #61, squash-merged as `6a12aeb`
- Integration-state branch: `codex/req-model-001-integration-state`
- Integration-state pull request: #62 (Draft until final evidence is green)
- Review record: `docs/reviews/PR-61-INDEPENDENT-REVIEW.md`
- Registry state in this change: `integrated`
- Next eligible requirement: REQ-DIAG-001 (`planned`)

## Integration result

- A fresh independent re-review closed P2-1 and P3-1 and found no P0, P1, P2,
  or P3 issue in the complete implementation and repair diff.
- Exact Ready head `da24b0a` passed the complete Windows, Ubuntu, and macOS CI
  matrix with every benchmark-smoke workload in run 29497804566.
- PR #61 squash-merged exactly once as `6a12aeb`; Issue #60 closed as
  completed.
- Post-merge `main` run 29498504443 passed the same complete three-platform
  correctness, benchmark-smoke, and requirement-registry gate.
- This isolated integration-state change updates only registry and progress
  evidence. It changes no production code, tests, manifest, schema, CI, build
  input, API, numerical behavior, dependency, tag, or release.

## Validation state

- Implementation and repair heads passed focused checks and the complete local
  standard gate; the clean reviewer independently repeated the model,
  anisotropy, field-assembly, and Rustdoc regressions.
- Exact implementation Ready-head and post-merge `main` three-platform gates
  are green as recorded above.
- The isolated integration-state registry tree passed the complete local
  standard gate: format, warning-denying workspace Clippy, all-feature
  workspace tests, workspace Rustdoc, all 58 requirement checks, and
  `git diff --check`. This validation-note update is documentation-only.

## Next task

After the isolated integration-state pull request is green and merged, open a
fresh task and perform the mandatory preflight. Use
`cargo xtask requirements next`; do not start the next requirement in this
task.

## Durable evidence

- Acceptance criteria and exclusions: closed GitHub Issue #60
- Merged implementation: GitHub PR #61
- Integration-state pull request: GitHub PR #62
- Independent review: `docs/reviews/PR-61-INDEPENDENT-REVIEW.md`
- Requirement summary: `changes/REQ-MODEL-001.md`
- Architecture: `docs/architecture/ARCHITECTURE.md`
- Deterministic model inputs: `docs/architecture/MODEL_FORMAT.md`
- Mathematical field representation: `docs/math/MATH_SPEC.md`
- Benchmark: `docs/benchmarks/REQ-MODEL-001.md`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable.
