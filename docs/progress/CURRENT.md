# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review / PR #35 ready-head integration sequence
- Requirement: REQ-POLY-001, Issue #34
- Branch: `codex/req-poly-001-polynomial-spaces`
- Draft pull request: #35
- Original reviewed head: `acc65c667932c14d461e2bedd028eea5f1d2bfd8`
- Repair code/test head: `3a538d8a5673b49548f49c86ab0587563bd08405`
- Clean fresh re-review head: `e1ae7a67dd71a99d52993af0b5b4ac2f3d388c45`
- Review record: `docs/reviews/PR-35-INDEPENDENT-REVIEW.md`
- Registry state: `documented`; REQ-DIM-001 is `integrated`

## Fresh re-review result

- A fresh read-only `math_reviewer` inspected the complete PR diff through
  `e1ae7a6` without inheriting implementation reasoning and found no P0, P1,
  P2, or P3 issue.
- P3-1 is closed. The reviewer independently verified the structured gradient
  length error and the distinct value/gradient sentinels in the joint-output
  atomicity regression; the exact focused regression passed at the reviewed
  head.
- The implementation, mathematics, safety, interfaces, performance, scope,
  benchmark wiring, and requirement evidence are clean for ready-head CI.
- PR #35 remains Draft until the final local standard gate and PR evidence are
  synchronized. This task did not begin REQ-FUNC-001.

## Next task

Finish only PR #35's ready-head integration sequence. Run the complete local
standard gate on the final review-evidence head, synchronize the PR evidence,
mark the PR ready, and wait for the Windows, Ubuntu, macOS, and all benchmark
smoke CI on that exact head. Merge exactly once only when that CI is green,
then record truthful integration state in an isolated change. Do not begin
REQ-FUNC-001.

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
- Draft CI run 29330040940 passed the Ubuntu correctness gate on exact fresh
  re-review head `e1ae7a6`; the Ready-only matrix was correctly skipped.

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, allocation instrumentation, and API/ABI/schema
snapshot checks are tracked by later requirements and release gates. Local
`actionlint` is unavailable.
