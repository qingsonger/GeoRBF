# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review complete; Repair required
- Requirement: REQ-SPIKE-003, Issue #114
- Branch: `codex/req-spike-003-sparse-backends`
- Draft pull request: #115
- Reviewed implementation head: `2ad68e5`
- Stable full-gate head: `255bac8`
- Review findings: P1 SPIKE003-REV-001 and P2 SPIKE003-REV-002
- Dependencies: REQ-KERNEL-004 and its complete closure are integrated
- Registry state: `implemented`

## Independent review result

- SPIKE003-REV-001: the claimed independent matrix-vector truth is circular.
  `SparseCase::from_points` creates the right-hand side with the same helper
  that the only matrix-vector assertion calls again. Neither backend's actual
  CSC column pointers, row indices, values, or storage-level matrix-vector
  result is checked.
- SPIKE003-REV-002: the published factor-and-solve timings include triplet
  allocation, CSC construction, factorization, solve, residual recomputation,
  and analytic-truth review. The benchmark report and ADR label and interpret
  them too narrowly.
- The reviewer otherwise confirmed the D=3 Wendland C2 formula and SPD fixture,
  strict support behavior, pair-count bounds, dimensionless original-unit
  backward error, explicit singular failure, absence of hidden regularization
  or fallback, dependency graph, and truthful interface N/A dispositions.

## Evidence state

- A fresh isolated read-only project `math_reviewer` reviewed exact head
  `2ad68e5` against base `244e887` and recorded both findings in
  `docs/reviews/PR-115-INDEPENDENT-REVIEW.md`.
- The reviewer passed all eight locked all-feature harness tests, the locked
  release smoke workload, exact-version metadata and dependency-tree review,
  requirement show/dependency checks, and the complete PR whitespace check.
- The parent Review task independently passed the eight all-feature harness
  tests, all 58 requirement checks, and the complete PR whitespace check.
- Draft CI run 29979880254 passed the configured Ubuntu gate on exact reviewed
  head `2ad68e5`. Ready-only Windows, Ubuntu, macOS, and benchmark-smoke CI was
  skipped as designed and is not claimed.
- Stable implementation head `255bac8` passed the complete standard local gate.
  Commits after it through this Review evidence change only handoff, review,
  and registry-document-index Markdown/YAML evidence; they do not change code,
  tests, manifests, CI, dependencies, or numerical behavior.

## Next task boundary

Open a fresh Repair task for Draft PR #115. Address only SPIKE003-REV-001 and
SPIKE003-REV-002 from `docs/reviews/PR-115-INDEPENDENT-REVIEW.md`. Add the
hand-derived three-point Wendland/CSC/matrix-vector regression and truthful
benchmark phase labeling or phase separation, rerun the fixed benchmark
evidence, focused checks, and one complete stable-head standard gate, update
review evidence and this bounded handoff, push, and stop for a fresh independent
re-review. Do not mark the PR ready, merge it, or begin REQ-SPARSE-001.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #114
- Draft implementation: GitHub PR #115
- Requirement summary: `changes/REQ-SPIKE-003.md`
- Independent review: `docs/reviews/PR-115-INDEPENDENT-REVIEW.md`
- Reproducible harness: `spikes/sparse-backends/`
- Selection decision:
  `docs/adr/ADR-0012-rstar-faer-compact-sparse-backends.md`
- Scaling and size evidence: `docs/benchmarks/REQ-SPIKE-003.md`
- Numerical policy: `docs/architecture/SOLVER_POLICY.md`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, and API/ABI/schema snapshot checks are tracked by
later requirements and release gates. Local `actionlint` is unavailable. No
unavailable check is claimed as passed.
