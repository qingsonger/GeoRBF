# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair complete; fresh re-review required
- Requirement: REQ-TUNE-001, deterministic parameter tuning
- Issue: #126
- Branch: `codex/req-tune-001-deterministic-tuning`
- Draft pull request: #127
- Independently reviewed implementation head: `555157c`
- Repair implementation head: `ae570a5`
- Dependency: REQ-MODEL-001 is integrated
- Registry status: `in_progress`

## Repair result

The fresh Repair task addressed only independent findings TUNE001-REV-001
through TUNE001-REV-005:

- GCV uses canonical `n * RSS / (n - effective_dof)^2` and rejects a
  candidate observation-count mismatch explicitly.
- Extreme positive distance ratios use a finite log difference without an
  intermediate quotient.
- Cross-validation requires `2 <= folds <= observations` before evaluator
  dispatch.
- CV diagnostics retain exact per-fold weighted squared errors and weights.
- Nearest-distance and fold-order sorting is in-place, deterministic, and
  allocation-free after work-vector reservation.

The required formula, extreme-scale, one-fold, unequal-weight evidence,
common-count, and allocation-counting regressions are present. No other
requirement, interface milestone, solver policy, or dependency changed.

## Validation state

- Focused validation passed all 14 tuning integration tests, the isolated
  zero-allocation ordering unit regression, warning-denying georbf all-target/
  all-feature Clippy, and smoke plus complete five-strategy release benchmarks.
- Exact repair implementation head `ae570a5` passed the complete standard
  local gate after the last production, test, and Rust documentation change:
  format, warning-denying workspace/all-target/all-feature Clippy, all-feature
  workspace tests, workspace doctests, and the 58-requirement registry check.
- The complete PR whitespace check also passed on that head.
- Only Markdown review and bounded-handoff evidence follows the validated
  implementation head, so its immutable complete gate remains applicable.
- PR #127 remains Draft. The next task must inspect CI on the pushed repair
  evidence head; no post-repair remote CI result is claimed here.

## Next task boundary

Start a fresh Review/re-review task for only PR #127. Supply an isolated
read-only project `math_reviewer` with the bounded requirement context, original
findings, repair diff, and validation evidence. Verify that TUNE001-REV-001
through TUNE001-REV-005 are closed and check for new P0--P3 findings.

If any finding remains, record evidence and stop without repairing production
code. If the re-review is clean, follow the mandatory integration sequence:
mark PR #127 Ready, wait for complete Windows/Ubuntu/macOS and benchmark-smoke
CI on that exact Ready head, merge exactly once only when it is green, and
record truthful integration state. Then stop.

Do not begin REQ-PERF-001 in the re-review task.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #126
- Independent review and repair evidence:
  `docs/reviews/PR-127-INDEPENDENT-REVIEW.md`
- Requirement summary: `changes/REQ-TUNE-001.md`
- Numerical policy: `docs/architecture/SOLVER_POLICY.md`
- Benchmark: `docs/benchmarks/REQ-TUNE-001.md`
- Production implementation: `crates/georbf/src/tuning.rs`
- Independent tests: `crates/georbf/tests/tuning.rs`
- Release benchmark: `crates/georbf/benches/parameter_tuning.rs`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, and API/ABI/schema snapshot checks are tracked by
later requirements and release gates. Local `actionlint` is unavailable. No
unavailable check is claimed as passed.
