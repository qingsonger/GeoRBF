# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair / REQ-CENTER-001 complete; awaiting fresh independent re-review
- Requirement: REQ-CENTER-001, open Issue #120
- Branch: `codex/req-center-001-rank-safe-centers`
- Pull request: #121 (Draft; four recorded findings repaired locally)
- Dependencies: REQ-SOLVE-001 and REQ-MODEL-001 are integrated
- Registry status: `planned`

## Repair outcome

- CENTER001-REV-001: replaced the global-diagonal threshold with the selected
  candidate-local `n * epsilon * abs(K_ii)` rule and added residual/power
  regressions for `diag(1, 2^-100)` and its congruently scaled identity.
- CENTER001-REV-002: made `KernelDefiniteness` explicit at construction,
  narrowed the atomic capability to SPD, and added an order-one `-r` CPD truth
  fixture that requires typed rejection before generic numerical work.
- CENTER001-REV-003: added the symmetric `[-1, 0, 1]` seeded farthest exact-tie
  and repeated-result regression.
- CENTER001-REV-004: added table-driven Gram/target length and nonfinite-input
  regressions and narrowed all evidence claims accordingly.

## Repair validation

- The initial focused run reproduced CENTER001-REV-001 exactly:
  `diag(1, 2^-100)` failed at rank one against the unrelated global threshold.
- After the last production and test change, all 13 center-selection tests,
  the center-selection rustdoc example, the five-strategy release benchmark
  smoke, `cargo xtask requirements check`, and `git diff --check` passed.
- The stable Repair tree passed the complete standard gate: format,
  warning-denying workspace/all-target/all-feature Clippy, all-feature
  workspace tests, workspace doctests, and the 58-requirement registry check.
- Draft CI for the pushed Repair head and Ready-only Windows/Ubuntu/macOS plus
  benchmark-smoke CI are not yet claimed.

## Next task boundary

A fresh Review/re-review task must inspect only PR #121. It must create an
isolated read-only `math_reviewer`, confirm CENTER001-REV-001 through
CENTER001-REV-004 are closed, and review the complete repaired diff for new
findings. If any P0--P3 finding remains, record it and stop. Only a clean
re-review may continue with the repository's Ready -> exact-head
Windows/Ubuntu/macOS and benchmark-smoke CI -> merge -> isolated truthful
integration-state sequence. Do not repair in that task or begin REQ-TUNE-001.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #120
- Draft implementation: GitHub PR #121
- Independent review: `docs/reviews/PR-121-INDEPENDENT-REVIEW.md`
- Requirement summary: `changes/REQ-CENTER-001.md`
- Architecture: `docs/architecture/ARCHITECTURE.md`
- Numerical policy: `docs/architecture/SOLVER_POLICY.md`
- Benchmark: `docs/benchmarks/REQ-CENTER-001.md`
- Production implementation: `crates/georbf/src/center_selection.rs`
- Independent tests: `crates/georbf/tests/center_selection.rs`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, and API/ABI/schema snapshot checks are tracked by
later requirements and release gates. Local `actionlint` is unavailable. No
unavailable check is claimed as passed.
