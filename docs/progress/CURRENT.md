# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair complete / fresh independent re-review required
- Requirement: REQ-INFEAS-001, Issue #84
- Branch: `codex/req-infeas-001-diagnostics`
- Pull request: #85 (Draft)
- Repair code/test head: `680d497d424fe3a611376b6bc415173ff9d2f6e2`
- Registry state: `implemented`
- Dependencies: REQ-CONVEX-001 and REQ-DIAG-001 are `integrated`

## Repair result

- R85-001: exact row proportionality now uses exact binary cross-products, so
  the one-ULP nonparallel pair remains warning-only while the integer 49x pair
  is classified and conflict-checked exactly.
- R85-002: proportional interval ordering uses exact binary products without
  materializing a quotient or transformed endpoint, so finite overflow and
  underflow inputs cannot skip or collapse a conflict.
- Three new public-canonicalization regressions cover all four reviewed
  counterexamples with complete source evidence and strictly ordered conflict
  diagnostics. Canonical rows and solver policy remain unchanged.

## Validation state

- All eight infeasibility tests, all 11 problem-IR tests, and all eight
  linear-constraint tests passed; focused `georbf` all-targets/all-features
  Clippy passed with warnings denied.
- Stable repair head `680d497` passed the complete standard gate: format,
  workspace all-targets/all-features Clippy, workspace all-features tests,
  workspace doctests, and all 58 requirement checks. `git diff --check` passed.
- The prior Draft CI evidence remains historical only. Fresh Draft CI for the
  pushed repair head may run, but this Repair task does not mark the PR Ready
  or perform the mandatory ready-head integration sequence.

## Next task

Open a fresh Review/re-review task for PR #85. Give the independent
`math_reviewer` only the bounded requirement/dependency summary, normative
documents, original findings, exact repaired diff, regressions, and validation
evidence; do not pass Repair reasoning. Independently verify that R85-001 and
R85-002 are closed and check for new P0-P3 findings. If any finding remains,
record it and stop. Only after a clean re-review and confirmed complete local
gate may that fresh task mark the PR Ready, wait for exact ready-head Windows,
Ubuntu, macOS, and benchmark-smoke CI, merge once when green, and record
integration state. Do not begin another requirement.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #84
- Draft implementation pull request: GitHub PR #85
- Independent review: `docs/reviews/PR-85-INDEPENDENT-REVIEW.md`
- Requirement summary: `changes/REQ-INFEAS-001.md`
- Focused tests: `crates/georbf/tests/infeasibility.rs`
- Normative behavior: `docs/math/CONSTRAINT_SEMANTICS.md`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable. No unavailable check is claimed as passed.
