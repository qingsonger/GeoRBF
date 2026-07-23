# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Issue #123 handoff repair integrated; this branch records its
  merge-stable integration state
- Requirement: REQ-CENTER-001, closed Issue #120
- Implementation pull request: #121, squash-merged as `474988b`
- Integration-state pull request: #122, merged as `c40b97d`
- Post-merge handoff repair: closed Issue #123, PR #124 squash-merged as
  `2eb618b`
- Handoff-repair exact Ready head: `ca9dacd`
- Handoff-repair exact Ready-head CI: run 30049344424
- Handoff-repair post-merge `main` CI: run 30050847919
- Merge-stable handoff branch:
  `codex/issue-123-center-handoff-integration-state`
- Dependencies: REQ-SOLVE-001 and REQ-MODEL-001 are integrated
- Registry status: `integrated`

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
- Integration-state PR #122 exact Ready head `4f4e897` passed complete
  Windows, Ubuntu, and macOS CI run 30017196433, including every configured
  backend combination, benchmark smoke, and requirement validation.
- PR #122 merged exactly once as `c40b97d`. Post-merge `main` CI run
  30019137463 passed the same complete three-platform gate on that exact merge
  commit.
- Issue #123 corrected only the stale bounded handoff left by PR #122. PR #124
  changes no production code, tests, manifests, schemas, CI, build inputs,
  APIs, numerical behavior, dependencies, tags, releases, or the integrated
  requirement registry.
- A fresh Review task independently examined exact repair head `2790ac9`
  against `main` at `c40b97d` and found no P0--P3 issue. A fresh isolated
  `math_reviewer` then re-reviewed the evidence-only delta through exact final
  head `ca9dacd` and likewise found no P0--P3 issue.
- Exact Ready head `ca9dacd` passed complete Windows, Ubuntu, and macOS CI run
  30049344424, including all configured backend combinations, benchmark smoke,
  and requirement validation. PR #124 then squash-merged exactly once as
  `2eb618b`; Issue #123 closed as completed.
- Post-merge `main` CI run 30050847919 passed the same complete three-platform
  gate on exact merge commit `2eb618b`.
- This isolated follow-up changes only this bounded handoff. Its conditional
  next-task boundary remains truthful while its pull request is open and after
  that pull request merges.

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
- Integration-state exact Ready-head run 30017196433 and post-merge `main` run
  30019137463 are both green on Windows, Ubuntu, and macOS, including every
  configured benchmark smoke.
- PR #124 Draft CI run 30020972952 passed the Ubuntu correctness gate on exact
  reviewed repair head `2790ac9`; Draft CI run 30023689651 passed the complete
  Ubuntu correctness job on exact evidence head `ca9dacd`.
- Both `git diff --check c40b97d...ca9dacd` and
  `git diff --check 2790ac9..ca9dacd` passed. The recorded complete local
  standard gate remains applicable because that Review changed only Markdown
  review and handoff evidence.
- Exact Ready-head run 30049344424 and post-merge `main` run 30050847919 are
  both green on Windows, Ubuntu, and macOS, including every configured
  benchmark smoke.

## Next task boundary

Perform the mandatory preflight first.

- If GitHub has an open pull request for
  `codex/issue-123-center-handoff-integration-state`, start a fresh Review and
  handle only that pull request. Independently review its one-file
  documentation-only delta against `2eb618b`. If no P0--P3 finding remains,
  mark it Ready, wait for complete Windows, Ubuntu, and macOS benchmark-smoke
  CI on that exact Ready head, and merge exactly once only when green.
- If that pull request is already merged and no higher-priority Repair or
  Review work exists, start a new Implement task for the single atomic
  requirement returned by `cargo xtask requirements next`; at the time of this
  handoff that command returns REQ-TUNE-001.

Do not begin REQ-TUNE-001 in the integration-state Review task.

## Durable evidence

- Acceptance criteria and exclusions: closed GitHub Issue #120
- Merged implementation: GitHub PR #121
- Merged integration state: GitHub PR #122
- Post-merge handoff correction: GitHub Issue #123
- Merged handoff repair: GitHub PR #124
- Merge-stable handoff branch:
  `codex/issue-123-center-handoff-integration-state`
- Handoff repair review: `docs/reviews/PR-124-INDEPENDENT-REVIEW.md`
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
