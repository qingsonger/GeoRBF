# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair / PR #35 review finding P3-1
- Requirement: REQ-POLY-001, Issue #34
- Branch: `codex/req-poly-001-polynomial-spaces`
- Draft pull request: #35
- Reviewed head: `acc65c667932c14d461e2bedd028eea5f1d2bfd8`
- Review record: `docs/reviews/PR-35-INDEPENDENT-REVIEW.md`
- Registry state: `documented`; REQ-DIM-001 is `integrated`

## Independent review result

- The fresh read-only `math_reviewer` found no P0, P1, or P2 issue in the
  implementation, mathematics, safety, interfaces, performance, or scope.
- P3-1: the joint `PolynomialSpace::try_evaluate` contract promises both output
  buffers remain unchanged on every error, but failure tests cover only the
  separate value-only and gradient-only methods. Current production code is
  correct; the missing regression would allow a future partial-write bug.
- Required regression: pass correctly sized sentinel values and undersized
  sentinel gradients to the joint method, assert the structured gradient
  length error, and assert that both buffers remain unchanged.
- PR #35 must remain Draft. Review mode did not repair production or test code
  and did not begin REQ-FUNC-001.

## Next task

Open a fresh Repair task for only PR #35 finding P3-1. Add the smallest joint-
output atomicity regression, change production code only if the regression
exposes a defect, run focused checks during repair, then run the complete
standard workspace checks once on the stable repair head. Update the review
record and this handoff, commit, push, and stop for fresh independent re-review.
Do not begin REQ-FUNC-001.

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
- This Review task changes only independent-review evidence, the registry's
  document link, and this bounded handoff. It does not change production, test,
  manifest, schema, or build inputs.

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, allocation instrumentation, and API/ABI/schema
snapshot checks are tracked by later requirements and release gates. Local
`actionlint` is unavailable.
