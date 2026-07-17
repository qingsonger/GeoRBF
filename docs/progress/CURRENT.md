# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair / R70-010 repaired; fresh independent re-review required
- Requirement: REQ-LEVEL-001, Issue #69 (open)
- Branch: `codex/req-level-001-explicit-level-variables`
- Draft implementation pull request: #70
- Registry state in this change: `implemented`
- Dependencies: REQ-IR-001 and REQ-MODEL-001 are `integrated`
- Repair implementation head: `612aa0d34f2c75740cb0d26cb57392249d31a892`
- R70-001 through R70-009 are independently closed. R70-010 has repair
  evidence but remains open until a fresh independent re-review.
- Next eligible requirement remains blocked until REQ-LEVEL-001 is freshly
  re-reviewed and integrated in a later fresh task.

## R70-010 repair

- The independently sourced direct-path regression first failed because
  `LevelProblem::try_new` accepted fixed `A`, unknown `B`, identical unit Value
  evaluations at one point, and a positive `A -> B` order edge.
- Validation now rejects a positive direct or transitive order path between
  mathematically identical memberships with `MembershipOrderConflict`.
- Diagnostic evidence retains the lower membership, every selected order edge
  in path order, and the upper membership. Functional provenance does not alter
  mathematical equality, and no hard row is changed, dropped, softened, or
  regularized.
- The regression asserts the exact three-source direct path required by
  R70-010 and a two-edge transitive path with both order sources.

## Validation state

- Repair implementation head `612aa0d` passed the focused level suite (17
  tests), core all-target/all-feature Clippy, all 29 core Rustdoc tests, and the
  64-level benchmark smoke.
- After the final production, test, registry, and normative-document change,
  the complete stable-tree standard gate passed: formatting, warning-denying
  workspace Clippy, all-feature workspace tests, workspace Rustdoc, all 58
  requirement checks, and `git diff --check`.
- The review evidence and bounded handoff after `612aa0d` are documentation
  only. No fresh independent re-review, Ready three-platform CI, merge,
  integration, tag, or release is claimed.

## Next task

Open a fresh Review/re-review task for Draft PR #70. Supply a read-only project
`math_reviewer` only the bounded requirement/dependency summary, normative
documents, full PR and R70-010 repair diffs, tests, benchmark, and validation
evidence. Independently confirm R70-010 closure and check for new findings. If
any P0-P3 finding remains, record it and stop without repairing production
code. If the review is clean, follow the mandatory integration sequence on the
exact head: mark the PR ready, wait for complete Windows/Ubuntu/macOS and all
benchmark-smoke CI, merge exactly once only when green, record truthful
integration state, and stop. Do not begin another requirement.

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
