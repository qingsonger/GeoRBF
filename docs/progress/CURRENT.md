# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair / PR #41 P3-1 completed; fresh Review required
- Requirement: REQ-SPIKE-002, Issue #40
- Branch: `codex/req-spike-002-rrqr-svd-backend`
- Draft pull request: #41
- Freshly re-reviewed implementation/evidence head: `431da7f`
- Review finding record head: `2401d69`
- Actual repair code/test head: `30bd49520131ff085fd538c93ad767455cdade43`
- Review record: `docs/reviews/PR-41-INDEPENDENT-REVIEW.md`
- Registry state in this change: `documented`
- Dependency: REQ-BOOTSTRAP-001 is `integrated`
- Production dependency state: unchanged; the comparison crate is excluded
  from the production workspace

## Repair result

- P3-1 is repaired in both durable evidence locations. The repair-evidence
  paragraph and PR #41 Repair update now name the existing code/test commit
  `30bd49520131ff085fd538c93ad767455cdade43`.
- `git cat-file -e <hash>^{commit}` confirmed that exact commit object. Its
  parent is review-evidence commit
  `e1db3492866da63115784432977f3c1e7d039b56`.
- The invalid value remains only in the P3-1 finding narrative that documents
  the original defect; it is no longer presented as repair evidence.
- No production code, test, manifest, schema, build input, requirement status,
  mathematical claim, or interface changed. PR #41 remains Draft.

## Validation state

- Focused `cargo xtask requirements show REQ-SPIKE-002`,
  `cargo xtask requirements deps REQ-SPIKE-002`,
  `cargo xtask requirements check`, and `git diff --check` passed.
- On stable code/test head `30bd495`, spike formatting, warning-denying
  Clippy, all three 6/6 feature-test configurations, the expected zero-backend
  compile failure, the release smoke workload, workspace formatting,
  warning-denying Clippy, all 139 tests, all 25 doctests and compile-fail
  tests, all 58 requirement checks, and `git diff --check` passed.
- Draft Ubuntu CI run 29374908542 passed on exact pre-repair PR head `2401d69`.
  The new documentation-only head still requires fresh independent re-review.
- The ready-head Windows/Ubuntu/macOS and benchmark-smoke gate has not run and
  must wait for that clean re-review.

## Next task

Open a fresh Review/re-review task for only PR #41 and REQ-SPIKE-002. Perform
the mandatory preflight and use a fresh read-only project `math_reviewer`
without inheriting this Repair reasoning. Confirm P3-1 is closed and inspect
the complete PR diff for new findings. If any P0-P3 finding remains, record it
and stop without repairing. If the re-review is clean and the exact final head
has complete local evidence, synchronize the PR evidence, mark PR #41 ready,
wait for the complete Windows/Ubuntu/macOS and benchmark-smoke CI on that exact
ready head, merge exactly once only when it is green, and record truthful
integration state. Do not start REQ-CPD-001 in that task.

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
