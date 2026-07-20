# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Independent re-review complete with finding / REQ-CONVEX-001
- Requirement: REQ-CONVEX-001, Issue #81
- Implementation branch: `codex/req-convex-001-canonical-solver`
- Implementation pull request: #82 (Draft)
- Repair code/test head: `55f339c5d80666b089d2e2bdfae03a8b2029ae12`
- Re-review input head: `5cca75668d97d60fa2a8c5c0760bd08713af6c9c`
- Review record: `docs/reviews/PR-82-INDEPENDENT-REVIEW.md`
- Registry state: `implemented`; it is not `integrated`
- Direct dependencies REQ-SOFT-001, REQ-LINEQ-001, and REQ-SPIKE-004 are all
  `integrated`

## Independent re-review disposition

- R82-001 and R82-003 through R82-007 are closed.
- R82-002 remains open through new P1 R82-008. For a supported hard-only
  feasibility problem with `P = q = 0`, the dual-stationarity normalization
  divides a nonzero single-row residual only by that residual's own absolute
  contribution, producing one regardless of its magnitude or requested
  tolerance.
- The reviewer reproduced `minimize 0 subject to x >= 1` through the public API
  at tolerance `1e-9`. Clarabel returned exact `Solved`, but GeoRBF returned
  `SolutionReviewFailed` for dual stationarity with normalized value `1.0`.
- R82-008 requires a dimensionally coherent recorded zero-objective KKT/gap
  reference or equivalent policy, row-scale-invariant end-to-end hard-only
  regressions at `1e-12`, `1`, and `1e12`, and retained rejection of a synthetic
  nonstationary dual.
- No other P0-P3 finding remains. PR #82 stays Draft and REQ-INFEAS-001 has not
  begun.

## Validation state

- The independent reviewer ran `cargo build -p georbf`, all five private repair
  tests, all nine convex integration tests, workspace format, all 58
  requirement checks, exact-range whitespace checks, and the public hard-only
  counterexample. Existing checks passed; the counterexample reproduced
  R82-008.
- After the final production/test change, exact code/test head
  `55f339c5d80666b089d2e2bdfae03a8b2029ae12` passed the complete standard gate:
  workspace format, warning-denying all-target/all-feature Clippy, all-feature
  workspace tests, workspace Rustdoc, all 58 requirement checks, and
  `git diff --check`.
- The subsequent review record, solver policy, requirement change fragment,
  and this bounded handoff change only documentation. They do not change
  production, test, manifest, schema, CI, build, registry, API, numerical, or
  dependency inputs, so the immutable code/test-head gate remains applicable.
- Draft CI run 29689552476 passed the configured Ubuntu job on exact re-review
  input head `5cca7566`. Ready-only Windows, Ubuntu, macOS, and benchmark-smoke
  CI has not run for this Draft PR.

## Next task

Open a fresh Repair task for Draft PR #82 and REQ-CONVEX-001 limited to P1
R82-008. First preserve the public hard-only `x >= 1` failure and scaled-row
variants as regressions, then implement the smallest dimensionally coherent
zero-objective stationarity and gap review policy without a hidden unit floor
or weakened rejection. Run focused checks and the complete stable-head standard
gate, update the review evidence and bounded handoff, push, and stop for another
fresh independent re-review. Do not repair any closed finding and do not begin
REQ-INFEAS-001.

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable. Exact OSV and repository advisory API
queries are the performed dependency review; unavailable tools are not claimed.
