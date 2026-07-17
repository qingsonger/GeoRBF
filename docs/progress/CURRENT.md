# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review/re-review / new P1 R70-010 recorded; fresh Repair required
- Requirement: REQ-LEVEL-001, Issue #69 (open)
- Branch: `codex/req-level-001-explicit-level-variables`
- Draft implementation pull request: #70
- Registry state in this change: `implemented`
- Dependencies: REQ-IR-001 and REQ-MODEL-001 are `integrated`
- Re-reviewed head: `93f85dd17e145042f4282208c361c9aac95b8181`
- R70-001 through R70-009 are closed; R70-010 remains open
- Next eligible requirement remains blocked until REQ-LEVEL-001 is freshly
  re-reviewed, integrated, and selected by a later fresh task

## Fresh re-review result

- A fresh read-only project `math_reviewer` independently reviewed the bounded
  requirement, dependency, normative-document, original-review, full-diff,
  repair-diff, test, benchmark, and validation evidence.
- R70-001 through R70-009 are closed. No additional P0, P2, or P3 finding
  remains.
- P1 R70-010: two independently sourced memberships can evaluate the same
  mathematical Value at one point while belonging to different levels. A
  positive order path between those levels is accepted as contrast even though
  the two membership equalities force the level values equal and make the hard
  order bound infeasible.
- Required regression: fixed `A`, unknown `B`, identical unit Value evaluations
  with distinct provenance, and direct positive `A -> B`; require structured
  infeasibility with both membership sources and the order-edge source.

## Validation state

- Repair head `a56e7ad` passed the focused level suite (16 tests), core
  all-target/all-feature Clippy, core Rustdoc, and the 64-level benchmark smoke.
- After the final production, test, registry, and normative-document change,
  the complete stable-tree standard gate passed: formatting, warning-denying
  workspace Clippy, all-feature workspace tests, workspace Rustdoc, all 58
  requirement checks, and `git diff --check`.
- Exact re-reviewed head `93f85dd` passed Draft Ubuntu correctness CI run
  29563643533. The reviewer confirmed the full PR diff passes
  `git diff --check`; `a56e7ad..93f85dd` is documentation-only.
- This finding record and bounded handoff are documentation-only. No Ready
  three-platform CI, merge, integration, tag, or release is claimed.

## Next task

Open a fresh Repair task for Draft PR #70 and address only R70-010. Reproduce
the identical-membership positive-gap infeasibility with distinct provenance,
add the required source-aware regression, and implement the smallest complete
semantic check without altering hard rows. Run focused checks during repair and
the complete standard gate after the final production or test change. Update
review evidence and this bounded handoff, commit, push, and stop for fresh
independent re-review. Do not begin another requirement.

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
