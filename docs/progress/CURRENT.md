# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active scope

- Mode: Review complete; fresh Repair required next
- Requirement: REQ-CPD-001, Issue #45
- Draft pull request: #46
- Branch: `codex/req-cpd-001-rank-nullspace`
- Reviewed head: `cf4976ee7e575da1856d5871f6f6f744fccd43d4`
- Repair code/test head: `6af215f2758360513fce2b2cdf0d63914dd11bc7`
- Registry state: `documented` (not integrated)
- Dependencies: REQ-KERNEL-002, REQ-POLY-001, REQ-FUNC-001, and
  REQ-SPIKE-002 are integrated

## Review result

- A fourth fresh read-only `math_reviewer` inspected the exact repaired head
  without inheriting Repair reasoning. P2-4 is closed, and there are no P0,
  P1, or P3 findings.
- New P2-5: `original_dot_abs` returns `Some(0.0)` for a nonzero result below
  the minimum subnormal exponent instead of surfacing structured
  unrepresentable arithmetic. It can also round each mantissa product before
  summation and erase the finite representable residual `2^-104` in the exact
  near-cancellation `((1+2^-52)(1-2^-52)) - 1`.
- The helper feeds both null-space and expanded-weight original-unit
  diagnostics, so PR #46 remains Draft for a Repair limited to P2-5.

## Validation state

- The complete public `georbf` CPD target and all four private CPD diagnostic
  regressions pass.
- On exact repair code/test head `6af215f`, the complete standard gate passed:
  formatting, warning-denying workspace Clippy, all-feature workspace tests,
  workspace doc tests, and all 58 requirement checks.
- `git diff --check` passed. The subsequent handoff/review-evidence update is
  documentation-only, so the stable code/test-head gate remains applicable.
- The optimized one-iteration benchmark smoke retained checksum
  `-4.97657470788226419e-14` and completed in 1.1175 ms locally; the previously
  recorded 0.706--1.125 ms complete-assembly baseline remains the timing record.
- Exact reviewed-head Draft CI run 29392843498 passed Ubuntu on head
  `cf4976e`. The ready three-platform and benchmark-smoke matrix must remain
  skipped until a later clean re-review.

## Next task

- Open a fresh Repair task for PR #46 limited to P2-5. Add independent exact
  regressions for the representable `2^-104` cancellation residual and the
  unrepresentable `2^-1075` product through both public diagnostic paths.
- Implement the smallest accumulation repair, run focused checks and the final
  standard gate after the last code change, update repair evidence and this
  bounded handoff, commit, push, and stop for fresh independent re-review.
- Do not mark PR #46 ready, merge it, or begin another requirement in that
  Repair task.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #45
- Draft implementation: GitHub PR #46
- Independent findings and Repair evidence:
  `docs/reviews/PR-46-INDEPENDENT-REVIEW.md`
- Mathematical contract: `docs/math/CPD_AND_POLYNOMIALS.md`
- Solver policy: `docs/architecture/SOLVER_POLICY.md`
- Backend decision and production re-audit:
  `docs/adr/ADR-0009-nalgebra-rank-review-backend.md`
- Change summary: `changes/REQ-CPD-001.md`
- Benchmark and size baseline: `docs/benchmarks/REQ-CPD-001.md`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, allocation instrumentation, and API/ABI/schema
snapshot checks are tracked by later requirements and release gates. Local
`actionlint` is unavailable.
