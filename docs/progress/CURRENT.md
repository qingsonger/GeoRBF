# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review / fresh independent re-review of PR #49 P3-1 repair
- Requirement: REQ-SPIKE-001, Issue #48
- Branch: `codex/req-spike-001-dense-factorization`
- Draft pull request: #49
- P3-1 repair base: `c7d8d43aaa3b837f56af1fe9084ce388d3099dd6`
- Review record: `docs/reviews/PR-49-INDEPENDENT-REVIEW.md`
- Registry state: `documented` (not integrated)
- Dependency: REQ-BOOTSTRAP-001 is `integrated`

## Repair result

- P3-1 is implemented: ADR-0010 now says that all eight independent harness
  tests passed, consistent with the repaired harness and 8/8 review evidence.
- The review record now distinguishes the re-review finding from its bounded
  documentation-only repair.
- Only the ADR, review record, and this handoff changed. Production code,
  tests, manifests, schemas, and build inputs are unchanged.

## Next task

Open a fresh Review task and supply a read-only independent reviewer with the
bounded requirement context, normative documents, complete PR diff, validation
evidence, and exact repaired PR head. Confirm P3-1 is closed and check for new
findings. Follow the repository's re-review and integration sequence only if
the review is clean. Do not begin REQ-IR-001 in this task.

## Validation evidence

- The independent reviewer reran combined, faer-only, and nalgebra-only tests;
  each passed 8/8. Spike formatting, warning-denying all-feature Clippy, the
  required no-backend rejection, optimized smoke, all 58 requirement checks,
  and `git diff --check` passed.
- The documentation-only P3-1 repair reran all 58 requirement checks and
  `git diff --check`; both passed.
- The stable repair head retains the recorded complete local workspace gate and
  passed exact-head Draft CI run 29402438886. The independent reviewer did not
  rerun the complete workspace gate.
- No three-platform or benchmark-smoke ready-head CI is claimed while the PR
  remains Draft.

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, allocation instrumentation, and API/ABI/schema
snapshot checks are tracked by later requirements and release gates. Local
`actionlint` is unavailable. The performed advisory review was an OSV batch API
query of every exact selected package; it is not a claim that unavailable
audit tools ran.
