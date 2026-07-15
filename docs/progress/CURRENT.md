# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair / PR #52 finding P2-1
- Requirement: REQ-IR-001, Issue #51
- Branch: `codex/req-ir-001-semantic-canonical-ir`
- Draft pull request: #52
- Reviewed head: `dc88b999f02e31934dc1daa06a4909a87aed69ab`
- Review record: `docs/reviews/PR-52-INDEPENDENT-REVIEW.md`
- Registry state: `documented` (not `integrated`)
- Dependencies: complete closure through REQ-FUNC-001 is `integrated`

## Independent review result

- P2-1: the exact canonical-mapping test checks constants, counts, scaling,
  capabilities, and partial provenance but no sparse variable index or
  coefficient. Coefficient deletion, sign reversal, index permutation, or
  moving terms between SOC sides could therefore escape the required exact
  equality, bound, and SOC mapping evidence.
- The implementation preserves the authoritative coefficient and constant
  mappings by inspection; this is a test-evidence defect, not a confirmed
  production mapping defect.
- No P0, P1, or P3 issue was identified. Review mode made no production or test
  repair and did not begin REQ-FIELD-001.

## Next task

Open a fresh Repair task for only PR #52 finding P2-1. Extend the existing exact
canonical-mapping test to compare every equality, bound, SOC-left, and SOC-right
`(variable, coefficient)` sequence and complete canonical provenance for at
least one row or cone. Run focused checks, then the complete standard workspace
gate on the stable repair head. Update the review record and this bounded
handoff, commit, push, and stop for a fresh independent re-review. Keep the PR
Draft and do not begin REQ-FIELD-001.

## Validation evidence

- The independent reviewer passed all 11 problem-IR tests, crate doctests, the
  runnable example, and D=1/D=2/D=3 benchmark smoke on exact reviewed head
  `dc88b999f02e31934dc1daa06a4909a87aed69ab`.
- The reviewer also passed the complete stable-head standard gate: formatting,
  warning-denying workspace Clippy, all-feature workspace tests, workspace
  doctests, all 58 requirement checks, and `git diff --check`.
- Draft CI run 29410313417 passed the Ubuntu job on the exact reviewed head; the
  Ready-only three-platform and benchmark-smoke matrix correctly remained
  unexecuted.
- This Review task changes only the independent-review record, the requirement's
  review-document link, and this bounded handoff. It does not change production,
  tests, manifests, schemas, build inputs, APIs, dependencies, or numerical
  behavior.

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, allocation instrumentation, and API/ABI/schema
snapshot checks are tracked by later requirements and release gates. Local
`actionlint` is unavailable.
