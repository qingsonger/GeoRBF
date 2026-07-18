# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review complete; Repair required
- Requirement: REQ-LEVEL-001, Issue #69 (open)
- Branch: `codex/req-level-001-explicit-level-variables`
- Draft implementation pull request: #70
- Registry state in this change: `implemented`
- Dependencies: REQ-IR-001 and REQ-MODEL-001 are `integrated`
- R70-001 through R70-011 are independently closed.
- Fresh re-review of exact head `b11d321` found P1 R70-012 and P2 R70-013.
- PR #70 must remain Draft; Ready CI and integration are not authorized.
- The next eligible requirement remains blocked until REQ-LEVEL-001 is freshly
  re-reviewed and integrated in a later fresh task.

## Open review findings

- R70-012: pairwise membership comparison misses transitive level-equality
  chains. A@x=B@x and B@y=C@y imply h_A=h_B=h_C, but distinct fixed A/C,
  fixed/prior A/C contrast, and a positive A-to-C order are currently accepted.
- R70-013: when the field component has only one membership-bearing level, the
  `MissingContrast` fallback can name an unrelated isolated anchor. Evidence
  must remain inside the failing field component, including a one-level case.
- Required regressions and exact source-evidence obligations are recorded in
  `docs/reviews/PR-70-INDEPENDENT-REVIEW.md`.

## Validation state

- The independent reviewer passed all 18 focused level tests, all 29 core
  Rustdoc tests, the 64-level benchmark smoke, both scoped whitespace checks,
  and the compact requirement/dependency review. The parent Review task
  independently passed the focused level suite, core Rustdoc, and complete PR
  whitespace check.
- Exact-head Draft Ubuntu CI run 29630380600 passed at `b11d321`.
- Exact implementation head `914c1ea` retains its recorded complete standard
  workspace gate. `914c1ea..b11d321` changed only review evidence and the
  bounded handoff; this Review task also changes only those two documents.
- No Ready three-platform CI, merge, integration, tag, or release is claimed.

## Next task

Open a fresh Repair task for Draft PR #70 and address only R70-012 and R70-013.
First add the independently specified transitive equality-chain and one-level
field-component regressions and prove they fail on the current production tree.
Then implement the smallest complete repair without changing or relaxing any
hard row or turning a prior into a hard equality. Run focused checks during the
repair and the complete standard workspace gate after the final production or
test change. Update repair evidence and this bounded handoff, commit, push, and
stop for a fresh independent re-review. Do not begin another requirement.

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
