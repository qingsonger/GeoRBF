# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review / fresh re-review required for repaired PR #52
- Requirement: REQ-IR-001, Issue #51
- Branch: `codex/req-ir-001-semantic-canonical-ir`
- Draft pull request: #52
- Original reviewed head: `dc88b999f02e31934dc1daa06a4909a87aed69ab`
- Repair code/test head: `4562a20d565bc541ffd06a37220378c41229a627`
- Review record: `docs/reviews/PR-52-INDEPENDENT-REVIEW.md`
- Registry state: `documented` (not `integrated`)
- Dependencies: complete closure through REQ-FUNC-001 is `integrated`

## Repair result pending fresh independent re-review

- P2-1 was addressed only in `crates/georbf/tests/problem_ir.rs`. The exact
  canonical-mapping regression now compares every equality, bound, SOC-left,
  and SOC-right `(variable, coefficient)` sequence supplied by the distinct
  existing fixture.
- The same regression now compares complete canonical cone provenance:
  observation identifier, source path and line, original units, field path,
  and constraint group.
- No production code, API, manifest, schema, build input, dependency,
  benchmark, or interface disposition changed. Repair evidence is not an
  independent finding closure.

## Next task

Open a fresh Review/re-review task for only PR #52. Supply the independent
reviewer the bounded requirement/dependency summaries, normative documents,
complete repaired diff, and validation evidence without this Repair task's
reasoning transcript. Verify P2-1 and check for new findings. If clean, follow
the mandatory ready-head integration sequence; otherwise record findings and
stop for another bounded Repair. Do not begin REQ-FIELD-001.

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

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, allocation instrumentation, and API/ABI/schema
snapshot checks are tracked by later requirements and release gates. Local
`actionlint` is unavailable.
