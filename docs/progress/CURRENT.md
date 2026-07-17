# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair / REQ-LEVEL-001 findings addressed; fresh re-review required
- Requirement: REQ-LEVEL-001, Issue #69 (open)
- Branch: `codex/req-level-001-explicit-level-variables`
- Draft implementation pull request: #70
- Registry state in this change: `implemented`
- Dependencies: REQ-IR-001 and REQ-MODEL-001 are `integrated`
- Repaired implementation head: `a56e7ad24a9eaa4768534d3cd897ee74a6355659`
- Repair scope: R70-001 through R70-009; not yet independently re-reviewed
- Next eligible requirement remains blocked until REQ-LEVEL-001 is freshly
  re-reviewed, integrated, and selected by a later fresh task

## Repair result

- R70-001: memberships accept only one coefficient-1 Value atom and reject
  derivatives, scaled values, and multi-atom expressions structurally.
- R70-002 and R70-003: scaled path magnitudes preserve extreme conflicts and
  accept the feasible `-MAX -> 0 -> MAX` hard system without altering rows.
- R70-004 through R70-006: cycle evidence excludes downstream edges, fixed
  membership conflicts include all four sources, and contrast diagnostics cite
  the failing field component.
- R70-007 and R70-008: deterministic tie-breaking and exact-source regressions
  are explicit, and mathematical Value equality ignores provenance.
- R70-009: positive gaps or anchors on membershipless levels no longer
  manufacture scalar-field contrast.

## Validation state

- Repair head `a56e7ad` passed the focused level suite (16 tests), core
  all-target/all-feature Clippy, core Rustdoc, and the 64-level benchmark smoke.
- After the final production, test, registry, and normative-document change,
  the complete stable-tree standard gate passed: formatting, warning-denying
  workspace Clippy, all-feature workspace tests, workspace Rustdoc, all 58
  requirement checks, and `git diff --check`.
- This evidence update and bounded handoff are documentation-only. No fresh
  independent mathematical re-review or repair-head Draft CI result is claimed.

## Next task

Open a fresh Review/re-review task for Draft PR #70. Supply a new read-only
project `math_reviewer` only the bounded REQ-LEVEL-001 context, original review,
repair diff, and validation evidence. Independently confirm R70-001 through
R70-009 are closed and check for new P0-P3 findings. If any finding remains,
record it and stop without repair. If clean, follow the mandatory sequence:
mark ready, wait for complete Windows/Ubuntu/macOS and benchmark-smoke CI on the
exact ready head, merge exactly once only if green, record truthful integration
state, and stop. Do not start another requirement in that task.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #69
- Draft implementation pull request: GitHub PR #70
- Independent review: `docs/reviews/PR-70-INDEPENDENT-REVIEW.md`
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
