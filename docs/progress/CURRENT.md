# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Implement / REQ-LEVEL-001 complete; fresh independent Review required
- Requirement: REQ-LEVEL-001, Issue #69 (open)
- Branch: `codex/req-level-001-explicit-level-variables`
- Draft implementation pull request: #70
- Registry state in this change: `implemented`
- Dependencies: REQ-IR-001 and REQ-MODEL-001 are `integrated`
- Next eligible requirement after eventual integration: determined by a fresh
  `cargo xtask requirements next`; do not start it in the Review task

## Implementation result

- Added explicit fixed, unknown, and prior level definitions with stable IDs,
  complete provenance, validated prior mean/scale/loss, and no implicit
  objective or solver claim.
- Added hard field memberships `f(x_i) - h_k = 0`, order DAG edges
  `h_upper - h_lower >= minimum_gap`, one deterministic canonical `levels`
  variable block, fixed equality rows, order bounds, and explicit prior
  metadata outside the hard canonical problem.
- Added deterministic cycle, undefined-reference, isolated-unknown,
  per-component gauge, field-connected contrast, same-functional fixed
  conflict, and transitive fixed-gap conflict checks with source paths.
- Added 10 independent behavior/error tests and a deterministic 64-level
  validation-plus-canonicalization benchmark smoke workload.
- Updated the mathematical contract, ADR-0003 implementation consequences,
  architecture, Rust crate/API docs, requirement registry, and change fragment.

## Validation state

- Focused `cargo test -p georbf --test levels`: 10 passed.
- Focused `cargo clippy -p georbf --all-targets --all-features -- -D warnings`:
  passed.
- `cargo bench -p georbf --bench level_compilation -- --smoke`: passed for a
  64-level D=1 validation and canonicalization workload.
- Complete standard gate passed on the final implementation tree linking Draft
  PR #70: format, warning-denying workspace Clippy, all-feature workspace
  tests, workspace Rustdoc, all 58 requirement checks, and `git diff --check`.
  The subsequent validation-note edit changes documentation only.

## Next task

Open a fresh read-only Review task for the Draft implementation PR. Supply only
the REQ-LEVEL-001 show/deps summary, `docs/math/CONSTRAINT_SEMANTICS.md`,
ADR-0003, the architecture level-layer contract, the complete PR diff, and the
recorded validation evidence to the project `math_reviewer`. Review level-row
signs and indices, fixed/prior semantics, cycle and connected-component logic,
transitive gap conflict arithmetic, conservative floating-point boundary
handling, hard-constraint preservation, source provenance, allocations,
interface dispositions, benchmark scope, and registry truth. Do not repair
production code or begin another requirement in that task.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #69
- Draft implementation pull request: GitHub PR #70
- Requirement summary: `changes/REQ-LEVEL-001.md`
- Mathematical semantics: `docs/math/CONSTRAINT_SEMANTICS.md`
- Accepted design: `docs/adr/ADR-0003-explicit-level-variables.md`
- Architecture: `docs/architecture/ARCHITECTURE.md`
- Focused tests: `crates/georbf/tests/levels.rs`
- Benchmark: `crates/georbf/benches/level_compilation.rs`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable.
