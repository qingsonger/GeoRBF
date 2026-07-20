# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review complete / Repair required
- Requirement: REQ-INFEAS-001, Issue #84
- Branch: `codex/req-infeas-001-diagnostics`
- Pull request: #85 (Draft)
- Re-reviewed head: `a6a5fd825b73a794824861bb32e1754727df386c`
- Registry state: `implemented`
- Dependencies: REQ-CONVEX-001 and REQ-DIAG-001 are `integrated`

## Review result

- A fresh read-only project `math_reviewer` independently reviewed exact head
  `a6a5fd8` with only the bounded requirement, dependency, normative-document,
  diff, regression, benchmark, validation, and CI-workflow evidence.
- R85-001 and R85-002 are closed. Exact proportionality and interval ordering
  use exact binary cross-products; all four original counterexamples retain
  their required warning or source-aware conflict behavior.
- P2 R85-003 remains: the new `constraint_diagnostics` benchmark implements a
  smoke mode and is listed as implemented, but the Ready-only three-platform
  workspace job does not invoke it. Ready CI can pass without exercising the
  requirement's representative benchmark.
- No other P0, P1, P2, or P3 finding was identified. PR #85 remains Draft and
  the requirement remains `implemented`.

## Validation state

- All eight infeasibility tests, all 11 problem-IR tests, and all eight
  linear-constraint tests passed; focused `georbf` all-targets/all-features
  Clippy passed with warnings denied.
- Stable repair head `680d497` passed the complete standard gate: format,
  workspace all-targets/all-features Clippy, workspace all-features tests,
  workspace doctests, and all 58 requirement checks. `git diff --check` passed.
- The evidence-only tail `680d497..a6a5fd8` changed only the review record and
  bounded handoff, so the stable-head gate remains applicable. Exact PR head
  `a6a5fd8` passed Draft Ubuntu CI run 29716310057.
- The fresh Review task reran all eight infeasibility tests and the benchmark
  smoke on `a6a5fd8`; both passed and the smoke checksum was `768`.

## Next task

Open a fresh Repair task limited to R85-003. Add one Ready-workspace CI step
running `cargo bench -p georbf --bench constraint_diagnostics -- --smoke`.
Run the focused workflow check when available and the complete standard gate
on the final stable head; record unavailable local `actionlint` truthfully.
Update the review evidence and bounded handoff, push, and stop for another
fresh independent re-review. Do not repair anything else, mark the PR Ready,
merge, integrate the requirement, or begin another requirement in that task.

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
