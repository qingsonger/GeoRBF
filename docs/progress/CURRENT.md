# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Implement
- Requirement: REQ-PROJECT-001, Issue #99
- Branch: `codex/req-project-001-independent-multi-field-projects`
- Draft pull request: pending creation
- Registry state: `planned` until the Draft PR exists; implementation is present
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

## Validation state

- Focused project integration tests passed before the documentation update.
- The project Rustdoc compile-fail dimension bound passed.
- Formatting, Clippy, the complete standard gate, and the final diff review
  remain to be run on the stable implementation tree.

## Next task boundary

After the complete standard local gate is green and the Draft PR is updated,
stop. A fresh Review task must inspect only REQ-PROJECT-001 and its PR, create
an independent reviewer, record findings, and must not repair production code
or begin another requirement in the same task.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #99
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
