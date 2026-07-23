# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair complete; fresh independent re-review required
- Requirement: REQ-SPARSE-001
- Issue: #117
- Branch: `codex/req-sparse-001-compact-support`
- Draft pull request: #118
- Original reviewed head: `806bbff`
- Repair implementation and stable gate head: `a246995`
- Original findings repaired: P1 SPARSE001-REV-001; P2 SPARSE001-REV-002
  and SPARSE001-REV-003
- Dependencies: REQ-SPIKE-003, REQ-FIELD-001, and REQ-KERNEL-004 are integrated
- Registry status: `planned` until review, Ready CI, merge, and integration

## Repair result

- SPARSE001-REV-001: assembly diagnostics now expose and sum retained index,
  canonical capacities, CSC, right-hand side, pair, entry, row, and bulk-load
  components per checked stage. Solve includes the complete retained borrowed
  system beside backend CSC, dense-fill bound, vectors, and exact residual
  work. Limits at retained, assembly-peak, and solve-peak boundaries have
  direct regressions.
- SPARSE001-REV-002: support coverage is derived from accepted exact-support
  pairs before numeric-zero filtering. A co-located Value and
  DirectionalDerivative regression requires the zero action to retain two
  support neighbors per row with no isolated center.
- SPARSE001-REV-003: direct regressions now cover complete CSC/diagnostic
  repeat determinism, hard canonical conflict, solve cancellation and memory
  rejection, an unrepresentable candidate radius, and fixed 64- versus
  512-point bounded-neighbor growth. The change fragment is narrowed to direct
  evidence.

## Evidence state

- Exact repair head `a24699525aa811f2a55b3eecf880eb64e685ee76` passed all
  nine all-feature sparse integration tests and warning-denying all-target
  Clippy.
- The 64-point release benchmark smoke passed after the final production
  change with 352 stored nonzeros and finite checksums for assembly, solve,
  and local evaluation.
- The same exact head passed the complete five-check standard gate: format,
  warning-denying workspace/all-target/all-feature Clippy, all-feature
  workspace tests, workspace doctests, and all 58 requirement checks.
- The subsequent handoff commit changes only review evidence and this bounded
  handoff; it does not change production, tests, manifests, schema, CI, or
  build inputs and therefore does not invalidate the stable gate.

## Next task boundary

Open a fresh Review/re-review task for Draft PR #118. Supply a new isolated
read-only `math_reviewer` with the bounded requirement summary, dependency
closure, normative documents and ADR, complete repaired PR diff, and exact
validation evidence. Verify all three original findings are closed and inspect
for new P0-P3 findings. If any finding remains, record it and stop without
repairing production code. If the re-review is clean, follow the repository's
separate ready-head CI and integration sequence. Do not begin REQ-CENTER-001.

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
