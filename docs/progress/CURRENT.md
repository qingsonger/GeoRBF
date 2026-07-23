# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review clean; mandatory Ready CI and integration sequence in progress
- Requirement: REQ-SPARSE-001
- Issue: #117
- Branch: `codex/req-sparse-001-compact-support`
- Draft pull request: #118
- Clean independent re-review head:
  `917e6b3b5a12f48588cb5a34676cb2093988a8db`
- Third Repair implementation and stable gate head:
  `85d7e99356fe3790f2a0d63430b78a4f68ad8a0f`
- Closed findings: P1 SPARSE001-REV-001, P2 SPARSE001-REV-002, P2
  SPARSE001-REV-003, P3 SPARSE001-REV-004, and P1 SPARSE001-REV-005
- Open P0-P3 findings: none
- Dependencies: REQ-SPIKE-003, REQ-FIELD-001, and REQ-KERNEL-004 are integrated
- Registry status: `planned`; Ready CI, merge, and isolated integration-state
  evidence remain required

## Clean re-review result

- An isolated read-only `math_reviewer` inspected the complete base-to-head
  diff and independently closed SPARSE001-REV-005.
- The faer 0.24.4 AMD and complete symbolic-analysis request reconstruction,
  retained symbolic and numeric bounds, separate live peaks, and enforcement
  before backend CSC construction are conservative and match pinned backend
  truth.
- The 64-by-64 all-supported between-limit regression independently obtains
  faer's AMD request and rejects after only the `Started` progress event.
- No new P0-P3 finding was identified.

## Evidence state

- Exact stable implementation head
  `85d7e99356fe3790f2a0d63430b78a4f68ad8a0f` passed all 44 all-feature core
  unit tests, all ten all-feature sparse integration tests, format, and
  warning-denying all-target/all-feature Clippy.
- After the last production or test change, that same exact head passed the
  complete standard workspace gate: format, warning-denying
  workspace/all-target/all-feature Clippy, all-feature workspace tests,
  workspace doctests, and the 58-requirement registry check.
- The independent reviewer passed all ten sparse integration tests, the
  canonical-capacity regression, and the 64-point release benchmark smoke.
  The parent Review task independently passed the sparse tests,
  canonical-capacity regression, registry check, and PR whitespace check.
- Draft CI run 29997122789 passed on exact reviewed head `917e6b3`.
- This review evidence and handoff change only documentation and do not
  invalidate the stable implementation gate.
- Ready-only Windows/Ubuntu/macOS and benchmark-smoke CI has not run and is not
  claimed.

## Next task boundary

Commit and push the synchronized clean-review evidence, update PR #118, and
mark it ready. Wait for the complete Windows/Ubuntu/macOS and benchmark-smoke
CI on that exact Ready head. Merge exactly once only if every required check
is green, then record truthful integration state in an isolated change. Do not
begin REQ-CENTER-001 in this task.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #117
- Draft implementation: GitHub PR #118
- Independent review and Repair evidence:
  `docs/reviews/PR-118-INDEPENDENT-REVIEW.md`
- Requirement summary: `changes/REQ-SPARSE-001.md`
- Architecture: `docs/architecture/ARCHITECTURE.md`
- Numerical policy: `docs/architecture/SOLVER_POLICY.md`
- Backend selection: `docs/adr/ADR-0012-rstar-faer-compact-sparse-backends.md`
- Benchmark: `docs/benchmarks/REQ-SPARSE-001.md`
- Production implementation: `crates/georbf/src/problem_ir.rs` and
  `crates/georbf/src/sparse.rs`
- Independent tests: canonical-IR unit tests and
  `crates/georbf/tests/sparse.rs`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, and API/ABI/schema snapshot checks are tracked by
later requirements and release gates. Local `actionlint` is unavailable. No
unavailable check is claimed as passed.
