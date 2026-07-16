# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review complete / bounded Repair required
- Requirement: REQ-DIAG-001, Issue #63
- Branch: `codex/req-diag-001-structured-diagnostics`
- Draft pull request: #64
- Reviewed head: `872837e`
- Review record: `docs/reviews/PR-64-INDEPENDENT-REVIEW.md`
- Registry state: `implemented`
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
- The review found that public path construction cannot yet retain a source
  location plus field path without an observation ID, or a source-located
  level declaration without an observation ID.
- Preserved backend-owned detailed CPD, dense solver, assembly, and model
  diagnostics; the new layer is a common orchestration/adapter taxonomy.
- Added independent tests for every error category, stable codes, complete
  source paths, invalid evidence, `Send + Sync`, and absence of core output
  macros. The review found that exact symbolic identifiers and complete display
  strings are not yet regression-locked.
- Updated Rustdoc, architecture, registry interface dispositions, and
  `changes/REQ-DIAG-001.md`.

## Validation state

- Focused diagnostics integration tests pass.
- Focused warning-denying Clippy passes.
- Focused Rustdoc passes.
- `git diff --check` passes.
- Exact implementation, test, documentation, and registry head `9ef9a22`
  passed the complete local standard gate: format, warning-denying workspace
  Clippy for all targets and features, all-feature workspace tests, workspace
  Rustdoc, all 58 requirement checks, and `git diff --check`.
- This final validation-note update changes only `docs/progress/CURRENT.md`.
  It changes no production code, test, manifest, schema, CI, build input,
  registry, API, numerical behavior, dependency, tag, or release state.
- Exact reviewed head `872837e` passed Draft Ubuntu CI run 29503574991.
- A fresh read-only `xhigh` independent review found one P1 and one P2
  finding. All current mappings and constructor invariants were otherwise
  internally consistent, and no hidden recovery or hard-constraint relaxation
  was introduced.

## Next task

Open a fresh Repair task for only PR #64 findings P1-1 and P2-1. Add a
fallible public path-construction boundary that can retain source location and
field path independently of observation and level identifiers, including a
source-located level without an observation. Add table-driven path regressions
and exact ten-category symbolic-code and complete-display assertions with
identifier uniqueness. Run focused checks during repair, then the complete
standard gate on the stable final head, update evidence, push, and stop for a
fresh independent re-review. Do not begin REQ-EXEC-001 or another requirement.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #63
- Draft implementation: GitHub PR #64
- Independent review: `docs/reviews/PR-64-INDEPENDENT-REVIEW.md`
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
