# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review complete / repair required
- Requirement: REQ-INFEAS-001, Issue #84
- Branch: `codex/req-infeas-001-diagnostics`
- Pull request: #85 (Draft)
- Reviewed head: `1833b7ea8e8a414fdcb012c399dd1e35e54e6f2a`
- Registry state: `implemented`
- Dependencies: REQ-CONVEX-001 and REQ-DIAG-001 are `integrated`

## Independent review result

- P0 R85-001: `exact_row_scale` uses rounded division followed by rounded
  multiplication as an exact proportionality predicate. It can falsely reject
  a feasible one-ULP nonparallel equality system and can miss the exactly
  proportional integer rows `[1, 7, 13]` and `[49, 343, 637]`.
- P1 R85-002: interval endpoint multiplication silently skips a proportional
  conflict on overflow and collapses distinct endpoints on underflow.
- No other P0-P3 finding was reported. The exact counterexamples, independent
  derivations, file/line evidence, and required regressions are recorded in
  `docs/reviews/PR-85-INDEPENDENT-REVIEW.md`.

## Validation state

- Draft CI run 29714495028 passed the Ubuntu correctness job on exact reviewed
  head `1833b7e`; the Ready-only three-platform and benchmark-smoke matrix was
  correctly skipped.
- Both the independent reviewer and parent task passed all five focused
  infeasibility tests. The parent task also passed all 58 requirement checks
  and `git diff --check`.
- The existing tests do not cover either reviewed counterexample. Their green
  result does not close R85-001 or R85-002.
- Exact implementation commit `63f34ed` retains its recorded complete standard
  gate. The reviewed head changes only registry/handoff evidence after it.

## Next task

Open a fresh Repair task limited to R85-001 and R85-002. Reproduce both defects
through public canonicalization behavior, add the independent regressions
required by the review, implement the smallest exact and representability-safe
repair, run focused checks during development and the complete standard gate on
the final stable head, update the review evidence and this bounded handoff,
commit, push, and stop for a fresh independent re-review. Do not mark PR #85
Ready and do not begin another requirement.

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
