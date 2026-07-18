# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair complete; fresh Review / re-review required
- Requirement: REQ-LEVEL-001, Issue #69 (open)
- Branch: `codex/req-level-001-explicit-level-variables`
- Draft implementation pull request: #70
- Registry state in this change: `implemented`
- Dependencies: REQ-IR-001 and REQ-MODEL-001 are `integrated`
- R70-001 through R70-010 are independently closed.
- R70-011 repair implementation:
  `914c1eaf6b85991c2bb2f3d51c99bcf4e29de6c3`
- R70-011 is repaired but not independently re-reviewed or closed.
- The next eligible requirement remains blocked until REQ-LEVEL-001 is freshly
  re-reviewed and integrated in a later fresh task.

## R70-011 repair

- The new fixed/prior same-point regression failed before the production
  change because construction returned success.
- Distinct fixed/prior anchors now prove contrast only when no mathematically
  identical cross-level Value membership hard-couples those levels.
- The regression uses distinct functional and semantic provenance and requires
  `MissingContrast` naming A and B; a paired positive case preserves distinct
  membership behavior.
- Priors remain soft objective metadata. No hard row is changed, dropped,
  softened, regularized, or converted from a prior.

## Validation state

- All 18 focused level tests passed.
- Core all-target/all-feature Clippy, all 29 core Rustdoc tests, and the
  64-level benchmark smoke passed.
- After the final production, test, registry, and normative-document change,
  exact implementation head `914c1ea` passed the complete standard workspace
  gate: formatting, warning-denying all-target/all-feature workspace Clippy,
  all-feature workspace tests, workspace Rustdoc, all 58 requirement checks,
  and `git diff --check`.
- The subsequent repair-evidence and bounded-handoff change is documentation
  only. No Ready three-platform CI, merge, integration, tag, or release is
  claimed.

## Next task

Open a fresh Review / re-review task for Draft PR #70. Supply a read-only
project `math_reviewer` only the bounded REQ-LEVEL-001 summary and dependency
closure, M4 plan, relevant normative documents and ADR, complete PR and
R70-011 repair diffs, tests, benchmark, prior review record, and validation
evidence. Independently confirm R70-011 is closed, reconfirm R70-001 through
R70-010, and check for new P0-P3 findings. Do not repair production code in the
same task. If the review is clean and the final head has complete local checks,
follow the mandatory ready-CI-integration sequence from `AGENTS.md`. Do not
begin another requirement.

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
