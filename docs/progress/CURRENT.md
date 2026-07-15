# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review complete / bounded Repair required for PR #52 finding P2-2
- Requirement: REQ-IR-001, Issue #51
- Branch: `codex/req-ir-001-semantic-canonical-ir`
- Draft pull request: #52
- Original reviewed head: `dc88b999f02e31934dc1daa06a4909a87aed69ab`
- Repair code/test head: `4562a20d565bc541ffd06a37220378c41229a627`
- Fresh re-reviewed head: `133c8680cbd32e539dd855b7c59e1f374cc15f43`
- Review record: `docs/reviews/PR-52-INDEPENDENT-REVIEW.md`
- Registry state: `documented` (not `integrated`)
- Dependencies: complete closure through REQ-FUNC-001 is `integrated`

## Fresh re-review result

- A fresh read-only independent `math_reviewer` confirmed P2-1 is closed on
  exact head `133c868`: the equality, bound, both SOC-left, and SOC-right
  sparse mappings and complete cone provenance are now asserted exactly.
- The same reviewer found P2-2. Canonicalization at
  `crates/georbf/src/problem_ir.rs:544`, `:580`, and `:609` clones provenance
  strings with ordinary infallible allocation. OOM therefore cannot return the
  documented structured `AllocationFailed` error.
- No P0, P1, or P3 finding was found. PR #52 must remain Draft.

## Next task

Open a fresh Repair task for only PR #52 finding P2-2. Add a fallible
provenance deep-copy path and use it for equality, linear-bound, and SOC
canonicalization. Add isolated allocation-failure regressions for all three
paths that require structured `AllocationFailed` errors without abort, panic,
or partial result. Run focused checks and the complete standard gate after the
last code change, update evidence, push, and stop for fresh re-review. Do not
begin REQ-FIELD-001.

## Validation evidence

- The exact regression and the complete focused problem-IR file passed; the
  latter ran all 11 tests on repair code/test head `4562a20`.
- The complete stable-head standard gate passed formatting, warning-denying
  workspace Clippy for all targets and features, all-feature workspace tests,
  workspace doctests, all 58 requirement checks, and `git diff --check` on
  exact repair code/test head `4562a20`.
- The first gate attempt found only Clippy's test-function length limit. A test
  helper removed the repetition, focused tests and focused Clippy passed, the
  repair commit was amended, and the complete gate was rerun from the start.
- The subsequent evidence update changes only this bounded handoff and the
  independent-review record, so the immutable code/test-head gate remains
  applicable under the repository's documentation-only evidence rule.
- Fresh re-review at `133c868` made no repository or remote changes. It closed
  P2-1 and found P2-2; Ready-only three-platform and benchmark-smoke CI remains
  intentionally unexecuted while the PR is Draft.

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, allocation instrumentation, and API/ABI/schema
snapshot checks are tracked by later requirements and release gates. Local
`actionlint` is unavailable.
