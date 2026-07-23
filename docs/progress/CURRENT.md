# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review/re-review complete; Repair required
- Requirement: REQ-SPARSE-001
- Issue: #117
- Branch: `codex/req-sparse-001-compact-support`
- Draft pull request: #118
- Re-reviewed evidence head: `244a04a`
- Repair implementation and stable gate head: `a246995`
- Open findings: P1 SPARSE001-REV-001 and P3 SPARSE001-REV-004
- Closed findings: P2 SPARSE001-REV-002; the non-memory portions of P2
  SPARSE001-REV-003
- Dependencies: REQ-SPIKE-003, REQ-FIELD-001, and REQ-KERNEL-004 are integrated
- Registry status: `planned`; Ready CI and integration are forbidden while
  findings remain

## Re-review result

- SPARSE001-REV-001 remains open. Equality-only canonical compilation reserves
  capacity for empty linear-bound, cone, and soft-objective vectors, but
  `equality_payload_capacity_bytes`, sparse pre-allocation review, assembly
  diagnostics, and retained solve payload omit those live allocations.
- SPARSE001-REV-004 is a new evidence-only P3. The implementation accumulates
  `b-A*x`; the review record currently states `A*x-b`. The accepted norm is
  sign invariant, so production behavior is unaffected.
- Exact-support coverage, deterministic CSC/diagnostics, hard conflict,
  cancellation, nonrepresentable candidate radius, and two-size sparse-growth
  repairs are independently confirmed.
- No other P0-P3 finding was identified. PR #118 must remain Draft.

## Evidence state

- Exact repair head `a24699525aa811f2a55b3eecf880eb64e685ee76` retains the
  recorded nine-test focused evidence, warning-denying Clippy, 64-point
  benchmark smoke, and complete five-check standard gate.
- Draft CI run 29992714121 passed its Ubuntu correctness job on exact
  re-reviewed head `244a04a`; the Ready-only three-platform and benchmark
  matrix was skipped and is not claimed.
- This Review changes only review evidence and the bounded handoff. It does not
  change production, tests, manifests, schema, CI, or build inputs and
  therefore does not invalidate the stable implementation gate.

## Next task boundary

Open a fresh Repair task for Draft PR #118. Address only
SPARSE001-REV-001 and SPARSE001-REV-004. Count every canonical relation
vector's allocated capacity in equality-only preflight, retained diagnostics,
assembly peaks, and solve peaks. Add an independent canonical-capacity sum plus
assembly- and solve-stage limits between the old and corrected peaks. Correct
the residual-sign evidence without changing behavior. Run focused checks while
iterating and the complete standard gate once after the last production or
test change. Update review evidence and this bounded handoff, commit, push, and
stop for another fresh independent re-review. Do not mark PR #118 Ready, merge
it, or begin REQ-CENTER-001.

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
