# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Clean independent re-review complete; Ready integration sequence in
  progress / REQ-INFEAS-001
- Requirement: REQ-INFEAS-001, Issue #84
- Implementation branch: `codex/req-infeas-001-diagnostics`
- Implementation pull request: #85 (Draft until evidence synchronization)
- Stable CI-repair head: `1982d89af58344e3150cd7e547c8ac0b30ddab02`
- Clean re-reviewed head: `0c465e7e869d1118a56c39cabc73c2cf1b29cf92`
- Registry state: `implemented`; it is not `integrated`
- Dependencies: REQ-CONVEX-001 and REQ-DIAG-001 are `integrated`

## Final re-review disposition

- A fresh read-only project `math_reviewer` independently reviewed the exact
  complete PR and R85-003 repair using only bounded requirement, dependency,
  normative, diff, test, benchmark, registry, handoff, and validation evidence.
- R85-001, R85-002, and R85-003 are closed. Exact proportionality and interval
  decisions remain binary-exact and scale/sign invariant; approximate rows
  remain warning-only; hard rows remain unchanged; soft objectives and cones
  remain separate; the Ready-only constraint-diagnostics smoke spans Windows,
  Ubuntu, and macOS.
- No P0, P1, P2, or P3 finding remains. PR #85 may proceed to Ready CI;
  REQ-INFEAS-001 remains `implemented` until the full integration sequence
  completes.

## Validation state

- The focused `constraint_diagnostics --smoke` command passed locally with
  checksum `768`; `git diff --check` passed.
- After the last CI/build-input change, stable repair head `1982d89` passed the
  complete standard gate: format, workspace all-targets/all-features Clippy,
  workspace all-features tests, workspace doctests, and all 58 requirement
  checks.
- The final reviewer passed all eight infeasibility tests, all eight linear-
  constraint tests, four provenance-allocation fault tests, the benchmark smoke
  with checksum `768`, the example, GeoRBF all-target/all-feature Clippy,
  format, all 58 requirement checks, and both requested diff checks.
- Draft CI run 29718863367 passed the configured Ubuntu gate on exact reviewed
  head `0c465e7`. The Ready-only Windows/Ubuntu/macOS matrix correctly did not
  run while the PR was Draft.
- This re-review task changes only the review record and bounded handoff. The
  complete stable-head gate remains applicable because no production, test,
  manifest, schema, CI, build, API, numerical, registry, or dependency input
  changes.
- Local `actionlint` is unavailable and is not claimed as passed. The later
  unavailable-tool dispositions below are unchanged.

## Next task

Synchronize this evidence-only re-review and handoff commit on PR #85, mark the
PR Ready, and wait for the complete Windows, Ubuntu, and macOS correctness
matrix plus every benchmark-smoke workload on that exact final evidence head.
Merge exactly once only when green, wait for green post-merge `main` CI, then
record truthful integration state in an isolated follow-up change. If this task
is interrupted, resume only this integration sequence; do not begin another
requirement.

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
