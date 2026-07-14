# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review (P1 finding recorded; Repair is next)
- Requirement: REQ-FUNC-001, Issue #37
- Branch: `codex/req-func-001-atomic-functionals`
- Draft pull request: #38
- Registry state in this change: `in_progress`
- Dependencies: REQ-DIM-001, REQ-KCALC-001, and REQ-POLY-001 are `integrated`

## Independent review result

- A fresh read-only `math_reviewer` reviewed exact head `6dcbb9f` without the
  implementation reasoning and found one P1 defect and no P0, P2, or P3 issue.
- P1-1: `ObservationFunctional::try_apply_kernel` requires the evaluator to
  return a complete third-order `SpatialKernelJet` before dispatch determines
  that an atom pair needs only order zero, one, or two. This wrongly excludes
  valid coincident actions for kernels with exactly the demanded center
  capability, including Matérn 1/2 value/value and Matérn 3/2
  derivative/derivative.
- Durable review evidence and the required regressions are recorded in
  `docs/reviews/PR-38-INDEPENDENT-REVIEW.md`.
- This Review task changed no production code or tests. PR #38 remains Draft
  and REQ-FUNC-001 remains `in_progress`.

## Validation state

- On reviewed head `6dcbb9f`, focused functional tests pass 9/9, functional
  compile-fail rustdoc passes, benchmark smoke passes in D=1/D=2/D=3, and
  Draft Ubuntu CI run 29334259493 is green.
- The implementation-code standard gate passed on `3203360`; changes through
  `6dcbb9f` after that gate contain only requirement linkage metadata and this
  bounded handoff.
- The Repair task must run focused regressions while iterating and the complete
  standard workspace checks once after its final code change.

## Next task

Open a fresh Repair task for only P1-1 in Draft PR #38 and REQ-FUNC-001. First
add the independent coincident Matérn 1/2 value/value regression and the
Matérn 3/2 derivative/derivative companion. Implement the smallest
demand-aware repair without fabricating unsupported derivatives, widening
kernel capability, adding kernel-family special cases, or changing the shared
query/center sign convention. Run focused checks during repair, then the full
standard gate after the last code change. Update the review evidence and this
handoff, commit, push, and stop for a fresh re-review. Do not mark the PR ready
or start another requirement.

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
