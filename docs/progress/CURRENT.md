# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review / R70-011 found; fresh Repair required
- Requirement: REQ-LEVEL-001, Issue #69 (open)
- Branch: `codex/req-level-001-explicit-level-variables`
- Draft implementation pull request: #70
- Registry state in this change: `implemented`
- Dependencies: REQ-IR-001 and REQ-MODEL-001 are `integrated`
- Independently reviewed head: `3b6cf1366f30b9285c1023e5b2c73810c8c1b282`
- R70-001 through R70-010 are independently closed. R70-011 is open.
- Next eligible requirement remains blocked until REQ-LEVEL-001 is repaired,
  freshly re-reviewed, and integrated in a later fresh task.

## R70-011 finding

- The distinct-anchor contrast branch compares fixed values and prior means
  without proving the anchored membership evaluations are mathematically
  distinct.
- Fixed A at `0`, prior B with mean `1`, identical Value memberships at one
  point, and no order edges are accepted even though the hard rows force
  `h_A = h_B = f(x) = 0`; B's prior is soft and cannot create nonzero field
  contrast.
- The required regression uses distinct functional and semantic provenance for
  the identical Value points and must return `MissingContrast` naming A and B.
- Repair must not turn the prior into a hard equality or change, drop, soften,
  or regularize any hard row.

## Review and validation state

- A fresh read-only project `math_reviewer` independently closed R70-010,
  reconfirmed R70-001 through R70-009, and found only P1 R70-011. No P0, P2,
  or P3 finding remains.
- Exact reviewed head `3b6cf13` passed Draft Ubuntu correctness CI run
  29565567615.
- The parent Review task independently reran all 17 focused level tests, all 29
  core Rustdoc tests, and the complete PR whitespace check; all passed.
- This task changes only the review record and bounded handoff after the
  reviewed head. No Ready three-platform CI, merge, integration, tag, or
  release is claimed.

## Next task

Open a fresh Repair task for Draft PR #70. Address only R70-011. First add the
independently specified same-point fixed/prior regression and confirm it fails,
then implement the smallest contrast-validation repair without changing hard
row semantics. Run focused checks during development and the complete standard
workspace gate after the final production or test change. Update the review
repair evidence and this bounded handoff, commit, push, and stop for a fresh
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
