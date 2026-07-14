# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review / PR #41 clean re-review; ready-head integration sequence next
- Requirement: REQ-SPIKE-002, Issue #40
- Branch: `codex/req-spike-002-rrqr-svd-backend`
- Pull request: #41, Draft until clean-review evidence is synchronized
- Exact cleanly re-reviewed head: `66ed708a097bd55235f9a4be012c44870a2ffe33`
- Actual repair code/test head: `30bd49520131ff085fd538c93ad767455cdade43`
- Review record: `docs/reviews/PR-41-INDEPENDENT-REVIEW.md`
- Registry state in this change: `documented`
- Dependency: REQ-BOOTSTRAP-001 is `integrated`
- Production dependency state: unchanged; the comparison crate is excluded
  from the production workspace

## Review result

- A second fresh read-only `math_reviewer` inspected the complete PR diff at
  exact head `66ed708` without inheriting Repair reasoning.
- P2-1, P2-2, and P3-1 are independently confirmed closed. No P0, P1, P2, or
  P3 issue remains.
- The threshold regressions bracket the strict SVD threshold under independent
  analytic truth; the zero-backend path fails at compile time; the valid
  repair object and parent resolve exactly.
- This clean-review record changes documentation only. No production code,
  test, manifest, schema, build input, requirement status, mathematical claim,
  or interface changed.

## Validation state

- Focused `cargo xtask requirements show REQ-SPIKE-002`,
  `cargo xtask requirements deps REQ-SPIKE-002`,
  `cargo xtask requirements check`, and `git diff --check` passed.
- On stable code/test head `30bd495`, spike formatting, warning-denying
  Clippy, all three 6/6 feature-test configurations, the expected zero-backend
  compile failure, the release smoke workload, workspace formatting,
  warning-denying Clippy, all 139 tests, all 25 doctests and compile-fail
  tests, all 58 requirement checks, and `git diff --check` passed.
- Draft Ubuntu CI run 29375239847 passed on exact reviewed head `66ed708`.
- The ready-head Windows/Ubuntu/macOS and benchmark-smoke gate has not run and
  must be triggered only after the clean-review evidence commit is pushed and
  PR #41 is marked ready.

## Next task

Synchronize the clean review evidence to PR #41, mark it ready, and wait for
the complete Windows/Ubuntu/macOS and benchmark-smoke CI on that exact ready
head. Merge exactly once only when that CI is green, wait for post-merge `main`
CI, then record truthful integration state through an isolated change. Do not
start REQ-CPD-001 in this task.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #40
- Decision: `docs/adr/ADR-0009-nalgebra-rank-review-backend.md`
- Independent review: `docs/reviews/PR-41-INDEPENDENT-REVIEW.md`
- Change summary: `changes/REQ-SPIKE-002.md`
- Benchmark and size baseline: `docs/benchmarks/REQ-SPIKE-002.md`
- Reproducible harness: `spikes/rank-backends/`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, allocation instrumentation, and API/ABI/schema
snapshot checks are tracked by later requirements and release gates. Local
`actionlint` is unavailable.
