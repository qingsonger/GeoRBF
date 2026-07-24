# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review complete; Repair required
- Requirement: REQ-TUNE-001, deterministic parameter tuning
- Issue: #126
- Branch: `codex/req-tune-001-deterministic-tuning`
- Draft pull request: #127
- Reviewed implementation head: `555157c`
- Dependency: REQ-MODEL-001 is integrated
- Registry status: `in_progress`

## Independent review result

The fresh isolated read-only `math_reviewer` found five defects on exact head
`555157c`:

- P1 TUNE001-REV-001: GCV implements `RSS/(n-df)^2` instead of canonical
  `n*RSS/(n-df)^2`, and no common candidate observation count is enforced.
- P1 TUNE001-REV-002: `ln(value/target)` overflows or underflows for legal
  extreme positive values even when the equivalent log difference is finite.
- P2 TUNE001-REV-003: one-fold cross-validation leaves an empty training
  complement and must fail before evaluator dispatch.
- P2 TUNE001-REV-004: CV diagnostics retain only fold quotients, losing the
  raw weighted squared errors and weights required to audit the total score.
- P3 TUNE001-REV-005: stable sorting performs hidden allocations outside the
  advertised structured allocation-failure path.

No other P0--P3 finding was identified. The complete evidence, independent
truth examples, exact lines, and required regressions are in
`docs/reviews/PR-127-INDEPENDENT-REVIEW.md`.

## Validation state

- The isolated reviewer and parent Review task independently passed all 11
  tuning integration tests, the tuning rustdoc example, the five-strategy
  optimized benchmark smoke, the 58-requirement registry check, and the
  complete PR whitespace check.
- Draft CI run 30058923924 passed the Ubuntu correctness job on exact reviewed
  head `555157c`. The Ready-only Windows, Ubuntu, and macOS benchmark matrix
  was skipped as designed and is not claimed.
- The stable implementation head passed the complete standard local gate after
  the final production, test, manifest, CI, and registry change: format,
  warning-denying workspace/all-target/all-feature Clippy, all-feature
  workspace tests, workspace doctests, and registry validation.
- This Review changes only Markdown review and bounded-handoff evidence, so the
  immutable implementation-head gate remains applicable.

## Next task boundary

Start a fresh Repair task for only PR #127 findings TUNE001-REV-001 through
TUNE001-REV-005. Reproduce each issue and add the specified regression before
the smallest production repair. Run focused checks during iteration, then one
complete standard workspace gate after the last production or test change.
Update the review record and this bounded handoff, commit, push, and stop for a
fresh independent re-review.

Do not mark PR #127 Ready, merge it, change REQ-TUNE-001 to `integrated`, or
begin REQ-PERF-001 in the Repair task.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #126
- Independent review: `docs/reviews/PR-127-INDEPENDENT-REVIEW.md`
- Requirement summary: `changes/REQ-TUNE-001.md`
- Numerical policy: `docs/architecture/SOLVER_POLICY.md`
- Benchmark: `docs/benchmarks/REQ-TUNE-001.md`
- Production implementation: `crates/georbf/src/tuning.rs`
- Independent tests: `crates/georbf/tests/tuning.rs`
- Release benchmark: `crates/georbf/benches/parameter_tuning.rs`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, and API/ABI/schema snapshot checks are tracked by
later requirements and release gates. Local `actionlint` is unavailable. No
unavailable check is claimed as passed.
