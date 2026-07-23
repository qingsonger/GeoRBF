# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review/re-review complete; fresh Repair required
- Requirement: REQ-SPARSE-001
- Issue: #117
- Branch: `codex/req-sparse-001-compact-support`
- Draft pull request: #118
- Freshly re-reviewed evidence head:
  `f7c832181ff6529ca554fd212afe60580f7d7633`
- Second Repair implementation and stable gate head:
  `eca914287138baa42fddd09313596be60aa4a681`
- Open finding: P1 SPARSE001-REV-005
- Closed findings: P1 SPARSE001-REV-001, P2 SPARSE001-REV-002, P2
  SPARSE001-REV-003, and P3 SPARSE001-REV-004
- Dependencies: REQ-SPIKE-003, REQ-FIELD-001, and REQ-KERNEL-004 are integrated
- Registry status: `planned`; Ready CI and integration are forbidden while
  SPARSE001-REV-005 remains open

## Fresh re-review result

- The isolated project `math_reviewer` independently confirmed
  SPARSE001-REV-001 and SPARSE001-REV-004 are closed; the two previously
  closed findings remain closed.
- New P1 SPARSE001-REV-005: the explicit sparse-solve peak estimate omits
  faer AMD and symbolic-analysis scratch, retained symbolic structures, and
  numeric scratch.
- For a permitted 64-by-64 all-supported system, pinned faer 0.24.4 requests
  approximately 118,072 bytes of AMD scratch alone on the reviewed 64-bit
  targets. The current dense-factor and working-vector allowance totals only
  70,656 bytes.
- A caller limit between the reported and actual logical peak can pass
  preflight and enter backend allocation above the explicit limit. The
  architecture, solver-policy, and change-fragment complete-peak claims are
  therefore not yet true.

## Evidence state

- Exact stable implementation head
  `eca914287138baa42fddd09313596be60aa4a681` passed all 44 all-feature core
  unit tests, all nine all-feature sparse integration tests, and
  warning-denying all-target/all-feature Clippy.
- After the last production or test change, that same exact head passed the
  complete standard workspace gate: format, warning-denying
  workspace/all-target/all-feature Clippy, all-feature workspace tests,
  workspace doctests, and the 58-requirement registry check.
- The subsequent handoff update changes only review evidence and
  `docs/progress/CURRENT.md`; it does not invalidate the stable implementation
  gate.
- The independent reviewer passed all nine all-feature sparse integration
  tests, the canonical reserved-capacity unit regression, and the complete PR
  whitespace check. Exact faer 0.24.4 and rstar 0.13.0 feature resolution was
  confirmed.
- Draft CI run 29994904719 passed its Ubuntu correctness job on exact
  re-reviewed head `f7c8321`. Ready-only Windows/Ubuntu/macOS and
  benchmark-smoke CI has not run and is not claimed.

## Next task boundary

Open a fresh Repair task for PR #118 and address only
SPARSE001-REV-005. Account with checked conservative arithmetic for pinned
faer 0.24.4 AMD and symbolic-analysis scratch, retained symbolic structures,
numeric-factor storage, and numeric scratch before backend dispatch. Add an
internal 64-by-64 all-supported Wendland regression that independently obtains
or reproduces the AMD scratch request, places a limit strictly between the old
and corrected solve peaks, permits assembly, and requires
`SparseSolveError::MemoryLimitExceeded` before any factorization progress
event. Run focused checks during repair, then the complete standard workspace
gate once on the stable head after the final production or test change.
Update the review evidence and bounded handoff, commit, push, and stop for a
fresh independent re-review. Do not mark the PR Ready or begin
REQ-CENTER-001.

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
