# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair (P1-1 repaired; fresh Review is next)
- Requirement: REQ-FUNC-001, Issue #37
- Branch: `codex/req-func-001-atomic-functionals`
- Draft pull request: #38
- Registry state in this change: `in_progress`
- Dependencies: REQ-DIM-001, REQ-KCALC-001, and REQ-POLY-001 are `integrated`

## Independent review result

- A fresh read-only `math_reviewer` reviewed exact head `6dcbb9f` without the
  implementation reasoning and found one P1 defect and no P0, P2, or P3 issue.
- P1-1 found that `ObservationFunctional::try_apply_kernel` required a complete
  third-order `SpatialKernelJet` even though atom pairs demand only order zero,
  one, or two.
- Durable review evidence and the required regressions are recorded in
  `docs/reviews/PR-38-INDEPENDENT-REVIEW.md`.
- The Repair task now passes the exact demand to the evaluator and consumes a
  `SpatialKernelJetPrefix` that exposes only its supported order. Coincident
  Matérn 1/2 value/value and Matérn 3/2 derivative/derivative regressions pass
  without fabricating higher derivatives. An insufficient prefix is a
  structured error retaining both term provenances.
- This repair has not been independently re-reviewed. PR #38 remains Draft and
  REQ-FUNC-001 remains `in_progress`.

## Validation state

- Focused functional tests pass 10/10; focused Clippy, the runnable example,
  benchmark smoke in D=1/D=2/D=3, and `git diff --check` pass.
- After the final production and test change, the complete stable-code standard
  gate passed: formatting, workspace Clippy with all targets/features,
  workspace tests with all features, workspace rustdoc, and all 58 requirement
  checks. Subsequent changes are review evidence and this bounded handoff only.
- Draft Ubuntu CI is green on pre-repair head `acaf0dc`; the pushed repair head
  still requires Draft CI and a fresh independent re-review.

## Next task

Open a fresh Review/re-review task for only Draft PR #38 and REQ-FUNC-001. Give
the read-only `math_reviewer` the bounded requirement/dependency summaries,
normative documents, complete PR diff, original P1-1 evidence, regressions, and
validation evidence without this Repair task's reasoning transcript. Verify
that P1-1 is closed and check for new P0-P3 findings. If any finding remains,
record it and stop without repair. If clean, follow the mandatory integration
sequence: mark the PR ready, wait for complete Windows/Ubuntu/macOS and all
benchmark-smoke CI on that exact ready head, merge only when green, then record
truthful integration state. Do not start another requirement.

## Durable evidence

- Requirement summary: `changes/REQ-FUNC-001.md`
- Benchmark baseline: `docs/benchmarks/REQ-FUNC-001.md`
- Independent review: `docs/reviews/PR-38-INDEPENDENT-REVIEW.md`
- Acceptance criteria and exclusions: GitHub Issue #37

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, allocation instrumentation, and API/ABI/schema
snapshot checks are tracked by later requirements and release gates. Local
`actionlint` is unavailable.
