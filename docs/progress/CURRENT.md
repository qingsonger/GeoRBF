# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair complete; fresh independent re-review required
- Requirement: REQ-NORMAL-001, Issue #87
- Branch: `codex/req-normal-001-observations`
- Pull request: #88 (Draft)
- Reviewed head: `8724f288b1415b95492e2195a2f72e2032d1b9b1`
- Repair implementation head: `e94d19bf8baeb94901686f44499e7fdcf9e503c4`
- Registry state: `implemented`, not `integrated`
- Dependencies: REQ-ORIENT-001 and REQ-CONVEX-001 are `integrated`

## Repair disposition

- R88-001 is repaired with a structured representability error when a positive
  angle loses its positive cone slope during conversion or tangent evaluation;
  the minimum-positive-degree regression cannot compile a zero-angle cone.
- R88-002 is repaired by accepting rotation-invariant SquaredL2 and rejecting
  componentwise L1/Huber for D=3 multi-row complement semantics. The complete
  SquaredL2 canonical objective is rotation-tested; D=2 L1 remains supported.
- R88-003 is repaired with separately fallible final role and constraint
  reservations and storage-targeted allocation-failpoint regressions.
- These are repair assertions, not an independent finding closure. Full review
  and repair evidence is in `docs/reviews/PR-88-INDEPENDENT-REVIEW.md`.

## Validation state

- Exact repair implementation head `e94d19b` contains the production, test,
  Rustdoc, normative-math, and requirement-fragment changes.
- All 10 focused integration tests and both allocation-failpoint unit tests
  pass after the repairs.
- After the final code change, the exact implementation tree passed all five
  standard checks: format, workspace/all-target/all-feature Clippy with
  warnings denied, workspace/all-feature tests, workspace doctests, and the
  58-requirement registry check.
- Draft CI on the earlier reviewed head does not validate this repair. PR #88
  remains Draft pending a fresh independent re-review and new-head Draft CI.
- This handoff change and the appended repair evidence are documentation-only;
  they do not change the validated implementation tree.

## Next task boundary

Open a fresh Review/re-review task for PR #88. Supply the independent reviewer
only the bounded REQ-NORMAL-001 context, original findings, exact repair diff,
tests, normative documents, registry/handoff, and validation evidence. Verify
that R88-001, R88-002, and R88-003 are closed and check for new P0-P3 findings.
If clean, follow the mandatory ready-head CI and integration sequence in that
fresh task. Do not begin REQ-TANGENT-001.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #87
- Draft implementation pull request: GitHub PR #88
- Independent review: `docs/reviews/PR-88-INDEPENDENT-REVIEW.md`
- Requirement summary: `changes/REQ-NORMAL-001.md`
- Focused tests: `crates/georbf/tests/normal_observations.rs`
- Normative behavior: `docs/math/NORMAL_AND_TANGENT.md`
- Benchmark: `docs/benchmarks/REQ-NORMAL-001.md`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable. No unavailable check is claimed as passed.
