# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review / PR #38 ready-head integration sequence
- Requirement: REQ-FUNC-001, Issue #37
- Branch: `codex/req-func-001-atomic-functionals`
- Pull request: #38 (Draft until the final review-evidence head is pushed)
- Original reviewed head: `6dcbb9fa8d874cd5de4217e6f5f1deeac9927e0b`
- Repair and clean fresh re-review head: `264c46a31908a85eb76289ae43e1bad8b5c2ea00`
- Registry state in this change: `documented`
- Dependencies: REQ-DIM-001, REQ-KCALC-001, and REQ-POLY-001 are `integrated`

## Fresh re-review result

- A fresh read-only `math_reviewer` inspected the complete diff at exact repair
  head `264c46a` without inheriting the Repair reasoning and found no P0, P1,
  P2, or P3 issue.
- P1-1 is closed. The evaluator receives the exact zero-, first-, or
  second-order atom-pair demand and consumes a demand-bounded
  `SpatialKernelJetPrefix` without fabricating higher center derivatives.
- The reviewer independently confirmed the coincident Matérn 1/2 value action
  is one and the Matérn 3/2 same-direction mixed action is `3/4` for length
  scale two, with the shared query/center signs applied exactly once.
- The implementation, mathematics, safety, interfaces, performance, benchmark
  wiring, and requirement evidence are clean for ready-head CI.

## Validation state

- The independent reviewer passed formatting, focused warning-denying Clippy,
  all 10 functional tests, the compile-fail doctest, runnable example,
  benchmark smoke, all 58 requirement checks, `git diff --check`, and scoped
  safety/allocation checks.
- The complete final review-evidence standard gate passes: formatting,
  workspace Clippy with all targets/features, workspace tests with all
  features, workspace rustdoc, and all 58 requirement checks.
- Draft Ubuntu CI run 29339066111 is green on exact repair head `264c46a`.

## Next task

Finish only PR #38's ready-head integration sequence. Run the complete local
standard gate on the final review-evidence head, synchronize the PR evidence,
mark the PR ready, and wait for the Windows, Ubuntu, macOS, and all benchmark
smoke CI on that exact head. Merge exactly once only when that CI is green,
then record truthful integration state in an isolated change. Do not begin
another requirement.

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
