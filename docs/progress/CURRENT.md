# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review clean / Ready-head integration sequence for PR #52
- Requirement: REQ-IR-001, Issue #51
- Branch: `codex/req-ir-001-semantic-canonical-ir`
- Draft pull request: #52
- Original reviewed head: `dc88b999f02e31934dc1daa06a4909a87aed69ab`
- P2-1 repair code/test head: `4562a20d565bc541ffd06a37220378c41229a627`
- Fresh re-reviewed head: `133c8680cbd32e539dd855b7c59e1f374cc15f43`
- P2-2 repair code/test head: `1e782a73ab758ea93f0e71e5dba250cf3a03e7aa`
- Clean P2-2 re-reviewed head: `b6e5e136afab1449497c9058653d793f19c4c63f`
- Review record: `docs/reviews/PR-52-INDEPENDENT-REVIEW.md`
- Registry state: `documented` (not `integrated`)
- Dependencies: complete closure through REQ-FUNC-001 is `integrated`

## Clean re-review result

- A fresh read-only `math_reviewer` independently confirmed P2-2 is closed and
  found no remaining P0, P1, P2, or P3 issue in the complete PR diff.
- Equality, linear-bound, and SOC canonicalization use one fallible deep-copy
  path for every owned provenance string. Its test-only failure hook is
  thread-local, one-shot, RAII-cleaned, and exercised through public
  `try_compile` without any partial canonical result.
- The clean review evidence update changes only the review record and this
  bounded handoff. Production code, tests, manifests, schemas, CI, and build
  inputs are unchanged from the fully checked repair head.

## Integration sequence

Commit and push this documentation-only clean-review evidence, then mark PR #52
Ready. Wait for the complete Windows, Ubuntu, and macOS correctness matrix and
every benchmark-smoke workload on that exact Ready head. Merge exactly once
only when the full gate is green, wait for post-merge `main` CI, and record
truthful integration state in an isolated change. Do not begin REQ-FIELD-001.

## Validation evidence

- All three isolated provenance-copy allocation regressions passed, the full
  problem-IR integration test passed 11/11, and focused warning-denying Clippy
  passed.
- The complete stable-head standard gate passed formatting, warning-denying
  workspace Clippy for all targets and features, all-feature workspace tests,
  workspace doctests, all 58 requirement checks, and `git diff --check` on
  exact P2-2 repair code/test head `1e782a7`.
- The fresh reviewer independently passed the complete standard gate, all
  three provenance allocation regressions, the 11-test problem-IR integration
  file, the runnable example, D=1/D=2/D=3 benchmark smoke, all 58 requirement
  checks, and `git diff --check` on reviewed head `b6e5e13`.
- Draft CI run 29415584108 passed the Ubuntu correctness job on exact reviewed
  head `b6e5e13`. Ready-only three-platform and benchmark-smoke CI remains
  intentionally unexecuted until this clean evidence is pushed and PR #52 is
  marked Ready.

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, allocation instrumentation, and API/ABI/schema
snapshot checks are tracked by later requirements and release gates. Local
`actionlint` is unavailable.
