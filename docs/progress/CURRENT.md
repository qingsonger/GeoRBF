# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review / ready-head integration sequence for PR #49
- Requirement: REQ-SPIKE-001, Issue #48
- Branch: `codex/req-spike-001-dense-factorization`
- Draft pull request: #49
- Clean re-review head: `7e365514c5541b6b3e8b594f6fc8ef18bbc90851`
- Review record: `docs/reviews/PR-49-INDEPENDENT-REVIEW.md`
- Registry state: `documented` (not integrated)
- Dependency: REQ-BOOTSTRAP-001 is `integrated`

## Re-review result

- A fresh read-only `math_reviewer` confirmed P3-2 is closed on exact head
  `7e365514c5541b6b3e8b594f6fc8ef18bbc90851` and found no remaining P0, P1,
  P2, or P3 issue in the complete PR diff.
- Independent truth review reconfirmed the SPD and mandatory two-by-two-pivot
  indefinite cases, finite original-unit backward-error evidence, one-factor
  refinement, explicit failure, no hidden adjustment, dependency isolation,
  truthful interfaces, CI claims, and requirement state.
- The review evidence update changes only the review record and this handoff;
  production code, tests, manifests, schemas, CI, and build inputs are
  unchanged.

## Integration sequence

Synchronize the clean review evidence, mark PR #49 ready, and wait for the
complete Windows, Ubuntu, and macOS correctness matrix and every benchmark
smoke workload on that exact Ready head. Merge exactly once only if the full
gate is green, then record truthful integration state in an isolated change.
Do not begin REQ-IR-001 in this task.

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
- Exact repaired-head Draft CI run 29404916642 passed on Ubuntu.
- The primary task ran the complete standard workspace gate and
  `git diff --check` on exact clean-review head
  `7e365514c5541b6b3e8b594f6fc8ef18bbc90851`; all passed.
- No three-platform or benchmark-smoke Ready-head CI is claimed before that
  event-triggered run completes.

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, allocation instrumentation, and API/ABI/schema
snapshot checks are tracked by later requirements and release gates. Local
`actionlint` is unavailable. The performed advisory review was an OSV batch API
query of every exact selected package; it is not a claim that unavailable
audit tools ran.
