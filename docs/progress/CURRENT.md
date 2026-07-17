# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair complete / bounded independent re-review required
- Requirement: REQ-DIAG-001, Issue #63
- Branch: `codex/req-diag-001-structured-diagnostics`
- Draft pull request: #64
- Repair implementation head: `193ee44`
- Original reviewed head: `872837e`
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
  observation provenance and from a validated source location plus field path
  with independently optional observation, level, and constraint-group
  identifiers.
- The repair retains source-plus-field paths before either identifier exists
  and source-located level declarations without an observation ID; empty field
  and supplied group text remain rejected.
- Preserved backend-owned detailed CPD, dense solver, assembly, and model
  diagnostics; the new layer is a common orchestration/adapter taxonomy.
- Added independent tests for every error category, exact numeric and symbolic
  codes, identifier uniqueness, complete deterministic display strings,
  table-driven independent source-path components, invalid evidence,
  `Send + Sync`, and absence of core output macros.
- Updated Rustdoc, architecture, registry interface dispositions, and
  `changes/REQ-DIAG-001.md`.

## Validation state

- All six focused diagnostics integration tests pass.
- Focused warning-denying Clippy passes.
- Focused Rustdoc passes.
- `git diff --check` passes.
- Exact repair implementation head `193ee44` passed the complete local standard
  gate: format, warning-denying workspace Clippy for all targets and features,
  all-feature workspace tests, workspace Rustdoc, all 58 requirement checks,
  and `git diff --check`.
- The later review-record and bounded-handoff updates change only
  `docs/reviews/PR-64-INDEPENDENT-REVIEW.md` and this file. They change no
  production code, test, manifest, schema, CI, build input, registry, API,
  numerical behavior, dependency, tag, or release state.
- Exact pre-repair branch head `13bee8c` passed Draft Ubuntu CI run
  29508139350. The pushed repair head still requires fresh Draft CI observation
  in the next independent re-review task.
- The original review's P1-1 and P2-1 have repair evidence but are not closed
  until a fresh independent reviewer confirms the fixes and checks for new
  findings.

## Next task

Open a fresh independent re-review task for PR #64. Supply the requirement and
dependency summaries, normative documents, original findings, repair diff
`13bee8c..193ee44`, validation evidence, and fresh Draft CI state to the
read-only reviewer. Confirm P1-1 and P2-1 are closed and check for new P0-P3
findings. If any finding remains, record it and stop without repair. If clean,
follow the repository's ready-CI-integration sequence in that fresh Review
task. Do not begin REQ-EXEC-001 or another requirement.

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
