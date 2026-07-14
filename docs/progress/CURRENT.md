# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review / REQ-SPIKE-002 fresh re-review found P3-1; Repair required
- Requirement: REQ-SPIKE-002, Issue #40
- Branch: `codex/req-spike-002-rrqr-svd-backend`
- Draft pull request: #41
- Freshly re-reviewed PR head: `431da7f`
- Actual repair code/test head: `30bd49520131ff085fd538c93ad767455cdade43`
- Review record: `docs/reviews/PR-41-INDEPENDENT-REVIEW.md`
- Registry state in this change: `documented`
- Dependency: REQ-BOOTSTRAP-001 is `integrated`
- Production dependency state: unchanged; the comparison crate is excluded
  from the production workspace

## Fresh re-review result

- A fresh read-only `math_reviewer` inspected the complete PR diff without
  inheriting the Repair reasoning. It found no P0, P1, or P2 issue.
- P2-1 is closed. Independent analytic singular values put the 12-ULP case at
  approximately `0.8888888888888884` times the SVD threshold with rank two and
  the 15-ULP case at approximately `1.1111111111111103` times the threshold
  with rank three. Both candidate backends match that independent truth.
- P2-2 is closed. The zero-backend configuration is rejected at compile time,
  its exact diagnostic is checked, and all-feature plus both single-backend
  positive configurations remain covered.
- New P3-1: the review record and PR body name nonexistent full repair commit
  `30bd4952105acc6a04a7dcaff72493692f29d051`. The actual object is
  `30bd49520131ff085fd538c93ad767455cdade43`. Exact durable evidence is not
  reproducible until those two locations are corrected in a Repair task.

## Validation state

- On stable repair head `30bd495`, spike formatting, warning-denying Clippy,
  all three 6/6 feature-test configurations, the expected zero-backend compile
  failure, the release smoke workload, workspace formatting, warning-denying
  Clippy, all 139 tests, all 25 doctests and compile-fail tests, all 58
  requirement checks, and `git diff --check` passed.
- Draft Ubuntu CI run 29373908569 passed on exact PR head `431da7f`, including
  the three positive feature configurations, exact negative configuration,
  and spike smoke workload.
- The ready-head Windows/Ubuntu/macOS and benchmark-smoke gate has not run and
  must wait for P3-1 repair plus another clean fresh re-review.

## Next task

Open a fresh Repair task for only PR #41 finding P3-1. Correct the nonexistent
full repair hash in `docs/reviews/PR-41-INDEPENDENT-REVIEW.md` and the PR body to
`30bd49520131ff085fd538c93ad767455cdade43`; verify that exact object with
`git cat-file -e <hash>^{commit}`, run focused documentation/registry checks,
update the bounded handoff, commit, push, and stop. Do not change production
code, mark the PR ready, merge it, or implement REQ-CPD-001. A fresh independent
re-review must follow in a new task.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #40
- Decision: `docs/adr/ADR-0009-nalgebra-rank-review-backend.md`
- Independent review: `docs/reviews/PR-41-INDEPENDENT-REVIEW.md`
- Change summary: `changes/REQ-SPIKE-002.md`
- Benchmark and size baseline: `docs/benchmarks/REQ-SPIKE-002.md`
- Reproducible harness: `spikes/rank-backends/`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, allocation instrumentation, and API/ABI/schema
snapshot checks are tracked by later requirements and release gates. Local
`actionlint` is unavailable.
