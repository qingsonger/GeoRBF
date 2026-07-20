# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair complete / fresh re-review required
- Requirement: REQ-INFEAS-001, Issue #84
- Branch: `codex/req-infeas-001-diagnostics`
- Pull request: #85 (Draft)
- Stable CI-repair head: `1982d89af58344e3150cd7e547c8ac0b30ddab02`
- Registry state: `implemented`
- Dependencies: REQ-CONVEX-001 and REQ-DIAG-001 are `integrated`

## Review result

- The fresh read-only project `math_reviewer` closed R85-001 and R85-002 on
  exact head `a6a5fd8` and found only P2 R85-003: the representative
  `constraint_diagnostics` smoke was absent from Ready CI.
- Repair commit `1982d89` adds exactly that smoke command to the Ready-only
  `workspace` job. Its existing matrix applies the step to Windows, Ubuntu,
  and macOS; Draft behavior is unchanged.
- R85-003 is repaired but not independently closed. PR #85 remains Draft and
  REQ-INFEAS-001 remains `implemented` pending a fresh re-review.

## Validation state

- The focused `constraint_diagnostics --smoke` command passed locally with
  checksum `768`; `git diff --check` passed.
- After the last CI/build-input change, stable repair head `1982d89` passed the
  complete standard gate: format, workspace all-targets/all-features Clippy,
  workspace all-features tests, workspace doctests, and all 58 requirement
  checks.
- Local `actionlint` is unavailable and is not claimed as passed. The later
  unavailable-tool dispositions below are unchanged.

## Next task

Open a fresh Review/re-review task limited to PR #85. Supply the independent
`math_reviewer` only the bounded requirement and dependency summaries,
normative documents, exact PR diff, R85-003 repair, benchmark evidence, and
validation evidence. If any P0-P3 finding remains, record it and stop without
repairing. If the review is clean and the exact final head retains a complete
green local gate, mark the PR Ready, wait for the full Windows/Ubuntu/macOS and
benchmark-smoke CI on that exact ready head, merge only when it is green, and
record the truthful integration state. Do not begin another requirement.

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
