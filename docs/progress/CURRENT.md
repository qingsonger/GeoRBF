# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Clean final re-review complete; Ready CI and integration required
- Requirement: REQ-PROJECT-001, Issue #99
- Branch: `codex/req-project-001-independent-multi-field-projects`
- Draft pull request: #100
- Independently reviewed implementation head: `16c8001`
- Cleanly re-reviewed evidence head: `417eb6e`
- Independent review record: `docs/reviews/PR-100-INDEPENDENT-REVIEW.md`
- Final re-review result: no P0-P3 finding; no Repair required
- Registry state in this change: `implemented`, not `integrated`
- Dependencies: REQ-MODEL-001 and REQ-LEVEL-001 are integrated

## Implemented scope

- `GeoProject<D>` owns one or more immutable `ProjectField<D>` entries for
  exactly D=1, D=2, and D=3 while preserving caller insertion order.
- Stable caller-controlled `FieldId` values support deterministic entry and
  fitted-field lookup. Construction returns structured empty, duplicate, and
  allocation failures without partial success.
- Every entry owns an existing `FittedField<D>` produced by the same
  `FieldProblem<D>` core. The project layer adds no assembly, solver, kernel,
  evaluation, or cross-field coupling implementation.
- `ReferenceFieldInput` resolves only an existing field identifier and
  delegates value, gradient, and Hessian evaluation in that retained field's
  own original-coordinate convention. It defines no local-mixture weighting,
  coordinate reprojection, topology, persistence, or adapter behavior.

## Independent review and validation

- A fresh isolated `math_reviewer` independently inspected the exact seven-file
  PR diff and found no P0-P3 issue. It changed no repository or remote state.
- Both reviewer and parent passed all six focused project tests and the D=4
  compile-fail Rustdoc test; complete diff whitespace checks are green.
- Exact reviewed implementation head `16c8001` retains the complete standard
  local gate: workspace formatting, warning-denying all-target/all-feature
  Clippy, all-feature workspace tests, workspace Rustdoc, all 58 requirement
  checks, and `git diff --check`.
- Draft CI run 29796378377 passed Ubuntu correctness on `16c8001`. The Ready-
  only three-platform and benchmark-smoke matrix was skipped as designed and
  is not claimed as passed.
- This Review adds only the independent review record and this bounded handoff;
  production, tests, manifests, schemas, CI, build inputs, API behavior,
  numerical behavior, registry state, and dependency inputs are unchanged.
- A new isolated read-only `math_reviewer` re-reviewed exact evidence head
  `417eb6e` and found no P0-P3 issue. It passed all six project tests, the D=4
  compile-fail Rustdoc test, original-coordinate derivative and exact-center
  capability/error regressions, all 58 requirement checks, and complete PR and
  evidence-tail whitespace checks.
- The parent re-review passed the six project tests, D=4 compile-fail Rustdoc,
  all 58 requirement checks, and complete PR whitespace validation. Exact-head
  Draft CI run 29796926734 passed Ubuntu correctness on `417eb6e`.

## Next task boundary

Synchronize this clean re-review evidence and mark PR #100 ready. Wait for the
complete Windows, Ubuntu, macOS, and benchmark-smoke CI on the exact Ready
evidence head. Merge exactly once only when every required check is green, then
record truthful integration state in an isolated change. Do not begin another
requirement.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #99
- Draft implementation pull request: GitHub PR #100
- Independent review: `docs/reviews/PR-100-INDEPENDENT-REVIEW.md`
- Requirement summary: `changes/REQ-PROJECT-001.md`
- Independent property/error tests: `crates/georbf/tests/project.rs`
- Public implementation and Rustdoc: `crates/georbf/src/project.rs`
- Architecture boundary: `docs/architecture/ARCHITECTURE.md`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable. No unavailable check is claimed as passed.
