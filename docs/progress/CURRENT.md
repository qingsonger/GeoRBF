# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Implement
- Requirement: REQ-ANISO-003, Issue #111
- Branch: `codex/req-aniso-003-export-diagnostics`
- Draft pull request: #112
- Base: `main` at `37cb91d` after REQ-TREND-002 integration
- Stable full-gate head: `98c72dc`
- Dependencies: REQ-ANISO-002 and REQ-TREND-002 are integrated
- Registry state in this change: `in_progress`

## Scope and implementation

- Adds deterministic owned, renderer-neutral anisotropy diagnostic exports for
  exactly D=1, D=2, and D=3 without changing or refitting the compiled local
  SPD mixture.
- Exports caller-ordered control positions, honest spheroidal or ellipsoidal
  resolved axes and lengths, source confidence, signed strengths, influence
  radii, optional compact regions, condition numbers, and sign-invariant
  adjacent direction jumps.
- Exports strict-background policy/condition evidence and caller-ordered sample
  positions with every signed mixture weight, aggregate squared coverage,
  background contribution, active-component count, and domain membership.
- Exports source-aware low-confidence reference directions as control/axis
  region records. Explicit directions are excluded and an absent compact region
  remains explicit.
- No GUI, VTK, encoder, versioned schema, CLI, binding, solver, or numerical
  dependency is added. Schema/CLI work remains M8 and adapters remain M9.

## Validation state

- Focused `anisotropy_diagnostics` integration tests pass all four schema,
  low-confidence, direction-jump, coverage, dimension, and error-path cases.
- The module's compile-fail D=4 Rustdoc test passes.
- Focused warning-denying Clippy and the runnable example pass.
- Exact stable head `98c72dc` passed workspace format, warning-denying workspace
  all-target/all-feature Clippy, all-feature workspace tests, all workspace
  Rustdoc tests, all 58 requirement checks, and complete diff whitespace
  validation.
- The final evidence-tail update changes only Markdown validation records. It
  changes no production code, test, manifest, registry, schema, CI, build input,
  API, numerical behavior, or dependency, so the immutable `98c72dc` full gate
  remains applicable.

## Next task boundary

After this implementation is committed, pushed, and attached to a Draft PR,
open a fresh Review task for only REQ-ANISO-003. The reviewer must receive the
requirement/dependency summaries, Issue #111, M6 plan, ANISOTROPY and
ADR-0005/ADR-0008 contracts, PR diff, and exact validation evidence. Do not
repair production code or start the next requirement in that Review task.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #111
- Requirement summary: `changes/REQ-ANISO-003.md`
- Public implementation and Rustdoc:
  `crates/georbf/src/anisotropy_diagnostics.rs`
- Independent tests: `crates/georbf/tests/anisotropy_diagnostics.rs`
- Runnable example: `crates/georbf/examples/anisotropy_diagnostics.rs`
- Mathematical contract: `docs/architecture/ANISOTROPY.md`, ADR-0005, ADR-0008

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, and API/ABI/schema snapshot checks are tracked by
later requirements and release gates. Local `actionlint` is unavailable. No
unavailable check is claimed as passed.
