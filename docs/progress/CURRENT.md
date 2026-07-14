# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review / PR #35 fresh re-review
- Requirement: REQ-POLY-001, Issue #34
- Branch: `codex/req-poly-001-polynomial-spaces`
- Draft pull request: #35
- Previous reviewed head: `acc65c667932c14d461e2bedd028eea5f1d2bfd8`
- Repair code/test head: `3a538d8a5673b49548f49c86ab0587563bd08405`
- Review record: `docs/reviews/PR-35-INDEPENDENT-REVIEW.md`
- Registry state: `documented`; REQ-DIM-001 is `integrated`

## Repair result

- The fresh read-only `math_reviewer` found no P0, P1, or P2 issue in the
  implementation, mathematics, safety, interfaces, performance, or scope.
- Repair head `3a538d8` adds the one required P3-1 joint-output atomicity
  regression. It asserts the structured gradient length error and proves that
  correctly sized values and undersized gradients both retain their distinct
  sentinels.
- The regression passes against the existing implementation, so production
  code did not change. P3-1 awaits fresh independent re-review.
- PR #35 remains Draft. This task did not begin REQ-FUNC-001.

## Next task

Open a fresh Review task for only PR #35. Supply the independent reviewer the
bounded requirement and dependency summaries, normative documents, complete PR
diff, original P3-1 finding, and repair validation evidence. Independently
verify that P3-1 is closed and check for new findings. If clean, follow the
ready-head CI and integration sequence in `docs/CODEX_WORKFLOW.md`; otherwise
record findings and stop. Do not begin REQ-FUNC-001.

## Validation evidence

- Stable code/test head `8369aac` passed formatting, warning-denying workspace
  Clippy, 129 workspace tests, 24 doctests/compile-fail tests, all 58 requirement
  checks, ten focused Release polynomial tests, strict rustdoc, the runnable
  example, and polynomial benchmark smoke.
- Four full local polynomial benchmark runs had identical generation and
  evaluation checksums; `docs/benchmarks/REQ-POLY-001.md` records the baseline.
- Reviewed head `acc65c6` changed only the registry PR link/status and this
  bounded handoff after the stable full gate. Draft CI run 29329182602 passed
  the Ubuntu correctness gate on that exact head.
- Repair code/test head `3a538d8` passed the exact focused regression, formatting,
  warning-denying workspace Clippy, 129 workspace tests, 24 doctests and
  compile-fail tests, and all 58 requirement checks.
- The repair changed one regression test only; no production, manifest, schema,
  build input, registry state, or interface changed. The later review-record
  and bounded-handoff update is documentation-only.

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, allocation instrumentation, and API/ABI/schema
snapshot checks are tracked by later requirements and release gates. Local
`actionlint` is unavailable.
