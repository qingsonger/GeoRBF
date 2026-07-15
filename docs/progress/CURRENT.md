# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair / address PR #49 re-review finding P3-1 only
- Requirement: REQ-SPIKE-001, Issue #48
- Branch: `codex/req-spike-001-dense-factorization`
- Draft pull request: #49
- Re-reviewed repair head: `7b9226e656eddbafbc6f5f17e7726fc3f8d4c770`
- Review record: `docs/reviews/PR-49-INDEPENDENT-REVIEW.md`
- Registry state: `documented` (not integrated)
- Dependency: REQ-BOOTSTRAP-001 is `integrated`

## Re-review result

- A fresh read-only `math_reviewer` independently inspected the complete
  14-file diff at the exact repair head and closed original findings P1-1,
  P1-2, and P1-3.
- New P3-1: ADR-0010 says all six independent harness tests passed, but the
  repaired harness and review evidence contain eight tests. The accepted ADR's
  evidence count must change from six to eight.
- No new P0, P1, or P2 finding was identified, and no other P3 finding was
  identified.

## Next task

Open a fresh Repair task for only P3-1. Change the ADR-0010 evidence count from
six to eight, update the review record and bounded handoff, run the applicable
documentation and requirement checks, commit, push, and stop. Do not change
production or test code, do not mark PR #49 ready, and do not begin
REQ-IR-001. A fresh independent re-review must follow.

## Validation evidence

- The independent reviewer reran combined, faer-only, and nalgebra-only tests;
  each passed 8/8. Spike formatting, warning-denying all-feature Clippy, the
  required no-backend rejection, optimized smoke, all 58 requirement checks,
  and `git diff --check` passed.
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
