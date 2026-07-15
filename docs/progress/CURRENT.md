# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair complete / fresh re-review required for PR #52 finding P2-2
- Requirement: REQ-IR-001, Issue #51
- Branch: `codex/req-ir-001-semantic-canonical-ir`
- Draft pull request: #52
- Original reviewed head: `dc88b999f02e31934dc1daa06a4909a87aed69ab`
- P2-1 repair code/test head: `4562a20d565bc541ffd06a37220378c41229a627`
- Fresh re-reviewed head: `133c8680cbd32e539dd855b7c59e1f374cc15f43`
- P2-2 repair code/test head: `1e782a73ab758ea93f0e71e5dba250cf3a03e7aa`
- Review record: `docs/reviews/PR-52-INDEPENDENT-REVIEW.md`
- Registry state: `documented` (not `integrated`)
- Dependencies: complete closure through REQ-FUNC-001 is `integrated`

## Bounded P2-2 repair result

- Equality, linear-bound, and SOC canonicalization now deep-copy every owned
  provenance string only after a successful fallible reservation. Failure is
  returned as `CanonicalizationError::Ir(ProblemIrError::AllocationFailed)`
  with `CanonicalProvenance` storage and no observable partial result.
- Three isolated thread-local failure regressions exercise the public
  `try_compile` path at the equality, bound, and cone provenance-copy sites.
  They do not replace or modify the global allocator.
- The repair is not independently reviewed. PR #52 remains Draft, and
  REQ-IR-001 remains `documented` rather than `integrated`.

## Next task

Open a fresh Review/re-review task for PR #52. Use an independent read-only
`math_reviewer` to confirm P2-2 is closed on exact repair head `1e782a7` and
check for new findings without inheriting this Repair reasoning. If clean,
follow the mandatory Ready -> exact-head Windows/Ubuntu/macOS and benchmark-
smoke CI -> single merge -> isolated integration-state sequence. If any
finding remains, record it and stop without repair. Do not begin REQ-FIELD-001.

## Validation evidence

- All three isolated provenance-copy allocation regressions passed, the full
  problem-IR integration test passed 11/11, and focused warning-denying Clippy
  passed.
- The complete stable-head standard gate passed formatting, warning-denying
  workspace Clippy for all targets and features, all-feature workspace tests,
  workspace doctests, all 58 requirement checks, and `git diff --check` on
  exact P2-2 repair code/test head `1e782a7`.
- This subsequent evidence update changes only the bounded handoff and review
  record, so the immutable code/test-head gate remains applicable under the
  repository's documentation-only evidence rule.
- Draft CI run 29414844400 passed the Ubuntu correctness job on pre-repair head
  `75d6def`. Ready-only three-platform and benchmark-smoke CI remains
  intentionally unexecuted while the PR is Draft; no CI result is claimed for
  the just-pushed repair head.

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, allocation instrumentation, and API/ABI/schema
snapshot checks are tracked by later requirements and release gates. Local
`actionlint` is unavailable.
