# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review complete; Repair required
- Requirement: REQ-CONTOUR-001, one-dimensional level points
- Issue: #132
- Branch: `codex/req-contour-001-level-points`
- Draft pull request: #133
- Reviewed head: `323fcd9`
- Stable implementation gate head: `b41e482`
- Dependency: REQ-MODEL-001 is integrated
- Registry status: `in_progress`

## Independent review result

The fresh isolated read-only `math_reviewer` found three defects on exact head
`323fcd9`:

- P1 CONTOUR001-REV-001: an away-from-centers-only gradient can change sign
  across an unsampled nondifferentiable center. Coordinate-width termination
  can then fabricate a stationary candidate and at-level stationary root.
- P2 CONTOUR001-REV-002: `stationary_brackets()` records a near-zero scan node
  with neighboring nonzero derivatives without requiring an endpoint sign
  change, contradicting the API and guide's sign-bracket claim.
- P3 CONTOUR001-REV-003: the transformed independent truth test verifies
  original-coordinate locations but not the returned original-coordinate
  derivative values or reflection sign.

No other P0--P3 finding was identified. Complete counterexamples, exact lines,
mathematical reasoning, and required regressions are in
`docs/reviews/PR-133-INDEPENDENT-REVIEW.md`.

## Validation state

- The isolated reviewer and parent Review task independently passed all seven
  all-feature contour integration tests, the focused Rustdoc example, release
  benchmark smoke with checksum `2.50500000000000000e2`, the 58-requirement
  registry check, and the complete PR whitespace check.
- Draft CI run 30077398167 passed Ubuntu on exact reviewed head `323fcd9`.
  The Ready-only Windows, Ubuntu, and macOS benchmark matrix was skipped as
  designed and is not claimed.
- Stable implementation head `b41e482` passed the complete standard local gate
  after the final production, test, manifest, benchmark, CI, and registry
  change.
- This Review changes only Markdown review and bounded-handoff evidence, so
  the immutable implementation-head gate remains applicable.

## Next task boundary

Start a fresh Repair task for only PR #133 findings
CONTOUR001-REV-001 through CONTOUR001-REV-003. Add the specified independent
regressions before the smallest production repair. Run focused checks during
iteration, then one complete standard workspace gate after the last production
or test change. Update the review record and this bounded handoff, commit,
push, and stop for a fresh isolated re-review.

Do not mark PR #133 Ready, merge it, change REQ-CONTOUR-001 to `integrated`, or
begin REQ-CONTOUR-002 in the Repair task.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #132
- Draft implementation: GitHub PR #133
- Independent review: `docs/reviews/PR-133-INDEPENDENT-REVIEW.md`
- Core implementation: `crates/georbf/src/contour.rs`
- Independent tests: `crates/georbf/tests/contour.rs`
- User guide: `docs/user-guide/LEVEL_POINTS.md`
- Requirement summary: `changes/REQ-CONTOUR-001.md`
- Benchmark evidence: `docs/benchmarks/REQ-CONTOUR-001.md`
- Release benchmark: `crates/georbf/benches/level_points.rs`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, and API/ABI/schema snapshot checks are tracked by
later requirements and release gates. Local `actionlint` is unavailable. No
unavailable check is claimed as passed.
