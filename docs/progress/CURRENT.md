# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair / PR #49 P3-2 review-header status
- Requirement: REQ-SPIKE-001, Issue #48
- Branch: `codex/req-spike-001-dense-factorization`
- Draft pull request: #49
- P3-2 finding head: `1ca34634aab1a46ee482b1d0737119c0327123db`
- Review record: `docs/reviews/PR-49-INDEPENDENT-REVIEW.md`
- Registry state: `documented` (not integrated)
- Dependency: REQ-BOOTSTRAP-001 is `integrated`

## Re-review result

- P1-1, P1-2, P1-3, and P3-1 are closed. No new P0, P1, or P2 finding was
  identified.
- P3-2 remains open: the review record's top-level result and repair-status
  fields still request the already completed P3-1 Repair, contradicting the
  later review sections and prior handoff.
- This Review task records the finding only. It does not implement the repair.

## Next task

Open a fresh Repair task for P3-2 only. Update the review record's top-level
result and repair status so they truthfully say that P3-1 is implemented and
that P3-2 requires a fresh re-review. Update this handoff consistently, rerun
all 58 requirement checks and `git diff --check`, commit and push, then stop
for another fresh independent re-review. Do not change production or test code
and do not begin REQ-IR-001.

## Validation evidence

- The independent reviewer reran spike formatting, warning-denying all-feature
  Clippy, combined and both single-backend configurations with 8/8 tests each,
  optimized smoke, all 58 requirement checks, and `git diff --check`; all
  passed.
- The primary task ran the complete standard workspace gate on exact reviewed
  head `1ca34634aab1a46ee482b1d0737119c0327123db`; all five required checks and
  `git diff --check` passed.
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
