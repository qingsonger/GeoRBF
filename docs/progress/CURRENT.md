# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair complete; fresh Review/re-review required
- Requirement: REQ-SPARSE-001
- Issue: #117
- Branch: `codex/req-sparse-001-compact-support`
- Draft pull request: #118
- Re-reviewed evidence head: `244a04a`
- Second Repair implementation and stable gate head:
  `eca914287138baa42fddd09313596be60aa4a681`
- Findings addressed pending independent confirmation: P1
  SPARSE001-REV-001 and P3 SPARSE001-REV-004
- Previously closed findings: P2 SPARSE001-REV-002 and P2
  SPARSE001-REV-003
- Dependencies: REQ-SPIKE-003, REQ-FIELD-001, and REQ-KERNEL-004 are integrated
- Registry status: `planned`; Ready CI and integration remain forbidden until
  a fresh independent re-review is clean

## Second Repair result

- Every equality-only canonical relation vector's allocated capacity now
  contributes to the retained payload. This includes the logically empty but
  reserved linear-bound, cone, and soft-objective buffers.
- Sparse canonicalization preflight conservatively counts all four relation
  element types. Assembly diagnostics and solve retained-system diagnostics
  inherit the corrected canonical payload.
- An internal canonical-IR regression independently sums all relation vectors,
  equality terms, provenance strings, variable blocks and names, and scaling
  vectors. Sparse regressions reject limits between the old and corrected
  assembly peaks and between the old and corrected solve peaks.
- Review evidence now truthfully states that residual accumulation is
  `b-A*x`; the exposed infinity norm and backward error remain sign invariant.
- No support, matrix, factorization, solution, residual, tolerance, fallback,
  or regularization behavior changed.

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
- Draft CI run 29992714121 passed its Ubuntu correctness job on the previous
  re-reviewed head `244a04a`. Ready-only Windows/Ubuntu/macOS and
  benchmark-smoke CI has not run on the repaired head and is not claimed.

## Next task boundary

Open a fresh Review/re-review task for PR #118. Perform the mandatory preflight
and use an isolated project `math_reviewer` supplied only the bounded
requirement summary and dependency closure, Issue #117 criteria, M7 plan,
architecture and solver policy, ADR-0012, prior findings, the complete fixed
PR diff, and exact validation evidence. Independently confirm
SPARSE001-REV-001 and SPARSE001-REV-004 are closed and check for new P0-P3
findings. If any finding remains, record it and stop without repairing
production code. If the review is clean and the final head's complete local
gate is green, follow the repository's Review/re-review integration sequence:
sync PR evidence, mark Ready, wait for exact-ready-head Windows/Ubuntu/macOS
and benchmark-smoke CI, merge only when all are green, and record truthful
integration state. Do not begin REQ-CENTER-001.

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
