# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Implement
- Requirement: REQ-DIAG-001, Issue #63
- Branch: `codex/req-diag-001-structured-diagnostics`
- Pull request: pending Draft creation
- Registry state: `in_progress`
- Dependencies: REQ-IR-001 and its complete closure are `integrated`

## Scope

Implement one GeoRBF-owned diagnostic boundary for input, capability, rank,
gauge, contrast, infeasibility, conditioning, memory, cancellation, and version
failures. Retain stable machine-readable codes and observation/level source
paths without freezing Rust memory layout or implementing later adapter,
schema, level-DAG, convex-solver, or execution-control requirements.

## Implementation state

- Added `diagnostics` core types for stable error categories and codes,
  validated category-specific evidence, deterministic display, and
  `GeoRbfError`.
- Added fallible `DiagnosticPath` construction from complete semantic
  observation provenance, independent level paths, and combined
  observation-level paths with stable `LevelId`.
- Preserved backend-owned detailed CPD, dense solver, assembly, and model
  diagnostics; the new layer is a common orchestration/adapter taxonomy.
- Added independent tests for every error category, stable codes, complete
  source paths, invalid evidence, `Send + Sync`, and absence of core output
  macros.
- Updated Rustdoc, architecture, registry interface dispositions, and
  `changes/REQ-DIAG-001.md`.

## Validation state

- Focused diagnostics integration tests pass.
- Focused warning-denying Clippy passes.
- Focused Rustdoc passes.
- `git diff --check` passes.
- The complete standard workspace gate is still pending on the final stable
  implementation head.

## Next task

Finish focused workspace regression checks, create the Draft implementation PR,
record its real number, run the complete standard gate once on the final stable
head, update this bounded handoff, commit, push, and stop. Independent Review
must run in a fresh task and must not inherit this implementation reasoning.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #63
- Requirement summary: `changes/REQ-DIAG-001.md`
- Architecture: `docs/architecture/ARCHITECTURE.md`
- Implementation and tests: `crates/georbf/src/diagnostics.rs` and
  `crates/georbf/tests/diagnostics.rs`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable.
