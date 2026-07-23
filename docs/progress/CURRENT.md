# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review complete; Repair required
- Requirement: REQ-SPARSE-001
- Issue: #117
- Branch: `codex/req-sparse-001-compact-support`
- Draft pull request: #118
- Reviewed implementation head: `806bbff`
- Stable implementation gate head: `a0fd9fe`
- Review findings: P1 SPARSE001-REV-001; P2 SPARSE001-REV-002 and
  SPARSE001-REV-003
- Dependencies: REQ-SPIKE-003, REQ-FIELD-001, and REQ-KERNEL-004 are integrated
- Registry status: `planned` until review, Ready CI, merge, and integration

## Independent review result

- SPARSE001-REV-001 (P1): assembly and solve memory estimates omit
  simultaneously live canonical, neighborhood-index, and temporary payloads.
  The explicit limit and claimed conservative peak can therefore undercount
  the operation's logical live memory.
- SPARSE001-REV-002 (P2): row support coverage increments only for nonzero
  kernel actions rather than accepted exact-support pairs. Co-located Value and
  DirectionalDerivative representers can be falsely reported as isolated.
- SPARSE001-REV-003 (P2): the change fragment overstates canonical-conflict and
  failure coverage; deterministic ordering, solve-stage memory/cancellation,
  a sparse nonfinite boundary, and a multi-size subquadratic comparison lack
  direct regressions.
- The reviewer found no other formula, sign, dimension, SPD/CPD, strict-support,
  anisotropy-bound, residual, hidden fallback/regularization, Hessian-
  capability, or interface-disposition defect.

## Evidence state

- A fresh isolated read-only `math_reviewer` inspected exact head `806bbff`
  against base `c6696f2` and recorded all three findings in
  `docs/reviews/PR-118-INDEPENDENT-REVIEW.md`.
- Both reviewer and parent passed all six all-feature sparse integration tests.
  The parent also passed all 58 requirement checks and the complete PR
  whitespace check.
- Draft CI run 29990525588 passed its configured Ubuntu correctness job on
  exact reviewed head `806bbff`; the Ready-only three-platform and
  benchmark-smoke matrix was skipped as designed and is not claimed.
- Stable implementation head `a0fd9fe` retains the complete five-check local
  gate and release benchmark smoke recorded by Implement. This Review changes
  only the review record, registry document index, and bounded handoff.

## Next task boundary

Open a fresh Repair task for Draft PR #118. Address only SPARSE001-REV-001,
SPARSE001-REV-002, and SPARSE001-REV-003 from the independent review. Add the
specified memory-peak, exact-support coverage, deterministic/conflict/failure,
and multi-size scaling regressions; implement the smallest complete repairs;
rerun focused checks and one complete stable-head standard gate; update review
evidence and this bounded handoff; push; and stop for fresh independent
re-review. Do not mark the PR ready, merge it, or begin REQ-CENTER-001.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #117
- Draft implementation: GitHub PR #118
- Independent review: `docs/reviews/PR-118-INDEPENDENT-REVIEW.md`
- Requirement summary: `changes/REQ-SPARSE-001.md`
- Architecture: `docs/architecture/ARCHITECTURE.md`
- Numerical policy: `docs/architecture/SOLVER_POLICY.md`
- Backend selection: `docs/adr/ADR-0012-rstar-faer-compact-sparse-backends.md`
- Benchmark: `docs/benchmarks/REQ-SPARSE-001.md`
- Production implementation: `crates/georbf/src/sparse.rs`
- Independent tests: `crates/georbf/tests/sparse.rs`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, and API/ABI/schema snapshot checks are tracked by
later requirements and release gates. Local `actionlint` is unavailable. No
unavailable check is claimed as passed.
