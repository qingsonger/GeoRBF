# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review complete; fresh Repair required
- Requirement: REQ-LEVEL-001, Issue #69 (open)
- Branch: `codex/req-level-001-explicit-level-variables`
- Draft implementation pull request: #70
- Registry state in this change: `implemented`
- Dependencies: REQ-IR-001 and REQ-MODEL-001 are `integrated`
- R70-001 through R70-013 are independently closed at reviewed head `49998ef`.
- Fresh independent review found one new P1 finding, R70-014: the fixed-order
  conflict tolerance depends on the caller's scalar unit.
- PR #70 must remain Draft; Ready CI and integration are not authorized.
- The next eligible requirement remains blocked until REQ-LEVEL-001 is freshly
  re-reviewed and integrated in a later fresh task.

## Review result

- A fresh read-only `math_reviewer` reviewed exact local and remote head
  `49998ef` against base and merge base `2904c64` and confirmed R70-012 and
  R70-013 are closed. R70-001 through R70-011 also remain closed.
- R70-014 is exhibited by two distinct membership points, fixed values both
  exactly zero, and a positive `1e-20` order gap. The hard system is infeasible,
  but the `1.0` comparison-scale floor makes construction accept it; rescaling
  the scalar unit by `1e20` changes the verdict.
- Repair must base roundoff allowance on actual problem magnitudes with an
  exact-zero case and retain the lower-definition, order, and upper-definition
  sources. It must not change, drop, soften, regularize, or repair a hard row.

## Validation state

- Focused: 20 level tests, 6 diagnostics tests, core all-target/all-feature
  Clippy, all 29 core Rustdoc tests, and the 64-level benchmark smoke passed.
  The benchmark completed at approximately 369 microseconds per validation and
  compile iteration.
- Exact implementation tree `0df0550` passed the complete standard workspace
  gate: formatting, warning-denying all-target/all-feature workspace Clippy,
  all-feature workspace tests, workspace Rustdoc, all 58 requirement checks,
  and `git diff --check`.
- The subsequent handoff and repair-evidence commit changes only
  `docs/progress/CURRENT.md` and
  `docs/reviews/PR-70-INDEPENDENT-REVIEW.md`; it relies on the immutable
  implementation-tree gate and receives scoped whitespace verification.
- The fresh reviewer passed all 20 focused level tests, all 6 diagnostics
  tests, all 29 core Rustdoc tests, the level benchmark smoke at approximately
  253 microseconds per validation and compile iteration, both scoped whitespace
  reviews, and the requirement show/dependency review.
- Exact-head Draft Ubuntu CI passed at `49998ef`; the Ready-only Windows,
  Ubuntu, macOS, and benchmark-smoke matrix remained skipped as expected.
- After recording the review, all 58 requirement checks, the complete PR
  whitespace check, and the scoped review-evidence whitespace check passed;
  only this review record and the bounded handoff changed.
- No Ready three-platform CI, merge, integration, tag, or release is claimed.

## Next task

Open a fresh Repair task for Draft PR #70 and address only R70-014. First add a
regression with two distinct membership points, fixed levels both exactly zero,
and a direct `1e-20` order gap; require `FixedOrderConflict` and the exact
lower-definition/order/upper-definition source sequence, then repeat after a
positive unit rescaling and require the same result. Implement the smallest
unit-invariant fixed-order comparison, run focused checks during development
and the complete standard gate on the stable final implementation head, update
repair evidence and this bounded handoff, commit, push, and stop for a fresh
independent re-review. Do not broaden the requirement, mark the PR ready, merge,
integrate the requirement, or begin another requirement in that task.

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
