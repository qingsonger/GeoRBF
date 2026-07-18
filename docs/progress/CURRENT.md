# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair complete; fresh independent Review required
- Requirement: REQ-LEVEL-001, Issue #69 (open)
- Branch: `codex/req-level-001-explicit-level-variables`
- Draft implementation pull request: #70
- Registry state in this change: `implemented`
- Dependencies: REQ-IR-001 and REQ-MODEL-001 are `integrated`
- R70-001 through R70-013 are independently closed at reviewed head `49998ef`.
- Repair implementation commit `61fa6d3` addresses only R70-014; the finding is
  not independently closed yet.
- PR #70 must remain Draft; Ready CI and integration are not authorized.
- The next eligible requirement remains blocked until REQ-LEVEL-001 is freshly
  re-reviewed and integrated in a later fresh task.

## Repair result

- `ScaledMagnitude` now orders every positive magnitude above its exact zero
  representation before comparing exponents.
- Fixed-order roundoff review handles an exact-zero available gap directly and
  otherwise scales tolerance from the compared gaps without a dimensioned
  `1.0` floor.
- The required regression first failed on the prior production tree for the
  `1e-20` gap, then passed before and after a positive `1e20` unit rescaling with
  the exact lower-definition, order, and upper-definition source sequence.
- No hard row is changed, dropped, softened, regularized, or repaired.

## Validation state

- Focused: 21 level tests, 6 diagnostics tests, core all-target/all-feature
  Clippy, all 29 core Rustdoc tests, and the 64-level benchmark smoke passed.
  The benchmark completed at approximately 247 microseconds per validation and
  compile iteration.
- Exact implementation tree `61fa6d3` passed the complete standard workspace
  gate: formatting, warning-denying all-target/all-feature workspace Clippy,
  all-feature workspace tests, workspace Rustdoc, all 58 requirement checks,
  and `git diff --check`.
- The subsequent handoff and repair-evidence commit changes only this file and
  `docs/reviews/PR-70-INDEPENDENT-REVIEW.md`; it relies on that immutable
  implementation-tree gate and receives scoped whitespace and requirement
  verification.
- No Ready three-platform CI, merge, integration, tag, or release is claimed.

## Next task

Open a fresh read-only independent Review task for Draft PR #70. Review the
exact remote head against base `main`, confirm R70-014 closure, reconfirm
R70-001 through R70-013, and check for new findings using only the bounded
requirement/dependency summary, Issue #69 criteria, M4 plan, normative level
documents, PR diff, tests, benchmark, prior review record, and validation
evidence. Do not repair production code, mark the PR ready, merge, integrate the
requirement, or begin another requirement in that task.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #69
- Draft implementation pull request: GitHub PR #70
- Independent review and repair evidence:
  `docs/reviews/PR-70-INDEPENDENT-REVIEW.md`
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
