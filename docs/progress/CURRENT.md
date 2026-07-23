# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Integration state / REQ-CENTER-001 complete
- Requirement: REQ-CENTER-001, closed Issue #120
- Implementation pull request: #121, squash-merged as `474988b`
- Integration-state branch: `codex/req-center-001-integration-state`
- Integration-state pull request: #122 (Draft until exact Ready CI is green)
- Exact Ready head: `4c9525f`
- Dependencies: REQ-SOLVE-001 and REQ-MODEL-001 are integrated
- Registry status in this change: `integrated`

## Integration result

- An isolated read-only project `math_reviewer` reviewed base `aa128ed8`
  through exact Repair head `75110a5`.
- CENTER001-REV-001 through CENTER001-REV-004 are independently closed.
- The complete repaired 13-file PR diff has no remaining or new P0--P3
  finding.
- Exact Ready head `4c9525f` passed complete Windows, Ubuntu, and macOS CI run
  30013183746, including every configured backend combination, benchmark
  smoke, and requirement validation.
- PR #121 squash-merged exactly once as `474988b`; Issue #120 closed as
  completed. Post-merge `main` CI run 30015158750 passed the same complete
  three-platform gate on exact merge commit `474988b`.
- This isolated integration-state change updates only the registry, review
  evidence, completed-history index, and bounded handoff. It changes no
  production code, test, manifest, schema, CI, build input, API, numerical
  behavior, dependency, tag, or release.

## Review validation

- The isolated reviewer and parent Review task independently passed all 13
  center-selection tests, the center-selection rustdoc example, the
  five-strategy release benchmark smoke, the 58-requirement registry check,
  and the complete PR whitespace check.
- The stable Repair tree passed the complete standard gate: format,
  warning-denying workspace/all-target/all-feature Clippy, all-feature
  workspace tests, workspace doctests, and the 58-requirement registry check.
- Draft CI run 30009925065 passed Ubuntu on exact Repair head `75110a5`.
- Exact Ready-head run 30013183746 and post-merge `main` run 30015158750 are
  both green on Windows, Ubuntu, and macOS, including every configured
  benchmark smoke.
- The isolated integration-state tree must pass the complete local standard
  gate and exact Ready-head CI before it merges.

## Next task boundary

Create the isolated integration-state pull request, link its number in this
handoff and completed-history index, and run the complete standard local gate
on the final head. Mark that PR Ready, wait for exact-head
Windows/Ubuntu/macOS and benchmark-smoke CI, merge only if green, and stop. Do
not begin REQ-TUNE-001.

## Durable evidence

- Acceptance criteria and exclusions: closed GitHub Issue #120
- Merged implementation: GitHub PR #121
- Integration-state pull request: GitHub PR #122
- Independent review: `docs/reviews/PR-121-INDEPENDENT-REVIEW.md`
- Requirement summary: `changes/REQ-CENTER-001.md`
- Architecture: `docs/architecture/ARCHITECTURE.md`
- Numerical policy: `docs/architecture/SOLVER_POLICY.md`
- Benchmark: `docs/benchmarks/REQ-CENTER-001.md`
- Production implementation: `crates/georbf/src/center_selection.rs`
- Independent tests: `crates/georbf/tests/center_selection.rs`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, and API/ABI/schema snapshot checks are tracked by
later requirements and release gates. Local `actionlint` is unavailable. No
unavailable check is claimed as passed.
