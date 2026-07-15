# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review / fresh independent re-review of PR #49 P3-2 repair
- Requirement: REQ-SPIKE-001, Issue #48
- Branch: `codex/req-spike-001-dense-factorization`
- Draft pull request: #49
- P3-2 repair base: `c9d5fa96b3c75679ce497873f88c530cc85a9480`
- Review record: `docs/reviews/PR-49-INDEPENDENT-REVIEW.md`
- Registry state: `documented` (not integrated)
- Dependency: REQ-BOOTSTRAP-001 is `integrated`

## Repair result

- P3-2 is implemented: the review record's top-level result and repair-status
  fields now say that P3-1 is implemented and that P3-2 requires a fresh
  independent re-review.
- The review record and this bounded handoff now agree on the next task.
- Only the review record and this handoff changed. Production code, tests,
  manifests, schemas, and build inputs are unchanged.

## Next task

Open a fresh Review task and supply a read-only independent reviewer with the
bounded requirement context, normative documents, complete PR diff, validation
evidence, and exact repaired PR head. Confirm P3-2 is closed and check for new
findings. Follow the repository's re-review and integration sequence only if
the review is clean. Do not begin REQ-IR-001 in this task.

## Validation evidence

- The independent reviewer reran spike formatting, warning-denying all-feature
  Clippy, combined and both single-backend configurations with 8/8 tests each,
  optimized smoke, all 58 requirement checks, and `git diff --check`; all
  passed.
- The primary task ran the complete standard workspace gate on exact reviewed
  head `1ca34634aab1a46ee482b1d0737119c0327123db`; all five required checks and
  `git diff --check` passed.
- The documentation-only P3-2 repair reran all 58 requirement checks and
  `git diff --check`; both passed.
- Exact-head Draft CI run 29403767685 passed on Ubuntu. The reviewer relied on
  that run for the required no-backend rejection and did not rerun it locally.
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
