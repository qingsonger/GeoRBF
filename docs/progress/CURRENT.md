# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active scope

- Mode: Implement complete; independent Review required next
- Requirement: REQ-CPD-001, Issue #45
- Draft implementation pull request: #46
- Branch: `codex/req-cpd-001-rank-nullspace`
- Registry state: `documented` (not integrated)
- Dependencies: REQ-KERNEL-002, REQ-POLY-001, REQ-FUNC-001, and
  REQ-SPIKE-002 are integrated

## Implementation result

- Added GeoRBF-owned row-major CPD matrices, rank diagnostics, center and
  atomic-functional provenance, orthonormal polynomial null spaces,
  provenance-bearing `w = Z y`, and finite symmetric `Z^T K Z` projection.
- `Q[j, alpha] = M_j p_alpha` assembly reuses polynomial scratch across
  centers for Value and DirectionalDerivative representers in D=1, D=2, and
  D=3.
- The private nalgebra 0.35.0 adapter applies eight recorded alternating
  infinity-norm equilibration passes, column-pivoted RRQR screening, bounded
  SVD review, and a factor-16 ambiguity guard. Deficiency, threshold
  adjacency, disagreement, non-convergence, and verification failures are
  explicit; no coordinate mutation, jitter, pseudoinverse, or hidden
  regularization is used.
- Rust API, rustdoc, example, diagnostics, independent property tests,
  benchmark baseline, three-platform ready-PR smoke routing, normative docs,
  ADR production re-audit, and change fragment are updated.
- CLI, C, C++, and Python are N/A because no field schema, fitted model, or
  stable binding surface exists yet.

## Validation state

- Focused CPD tests pass for polynomial reproduction in D=1/D=2/D=3,
  null-space residual and orthonormality, value/derivative assembly, exact
  degeneracy, unit and nonzero functional-scale invariance, exact-binary
  analytic threshold cases on both sides of the SVD cutoff, deterministic
  repeatability, structured errors, and projected/KKT equivalence.
- The runnable example and the 64-center D=3 benchmark smoke workload pass.
  Four 100-iteration benchmark runs and the optimized binary size are recorded
  in `docs/benchmarks/REQ-CPD-001.md`.
- The complete local standard gate passed after the final code, test,
  manifest, lockfile, workflow, and registry changes: formatting,
  warning-denying workspace Clippy with all targets and features, workspace
  tests with all features, workspace rustdoc, all 58 requirement checks, and
  `git diff --check`. This bounded-handoff update is documentation-only.
- Initial Draft CI run 29384467033 on implementation commit `b219ec6` stopped
  at one Ubuntu rustfmt line-layout difference. The closure was rewritten into
  a cross-platform-stable two-statement form and the complete local gate then
  passed. The replacement Draft CI belongs to the final pushed head.
- The production dependency graph has 13 unique external packages, permissive
  licenses, maximum declared MSRV 1.89, and no findings from exact OSV and
  GitHub advisory queries. Unsafe-source exposure and size evidence are
  recorded in ADR-0009 and the benchmark report.

## Next task

- Open a fresh Review task for Draft PR #46 and only REQ-CPD-001.
- Supply the reviewer only the requirement show/deps summaries, normative CPD
  and solver documents, ADR-0004/0007/0009, PR diff, and validation/benchmark
  evidence. Use the project read-only `math_reviewer` because this is a
  mathematical and numerical change.
- Record P0-P3 findings and independent truth reasoning. Do not repair
  production code in that Review task and do not start another requirement.
- If the review is clean, a later fresh Review/integration task must mark the
  PR ready, wait for exact-head Windows/Ubuntu/macOS and all benchmark-smoke
  CI, merge exactly once only when green, and then record truthful integration
  state.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #45
- Draft implementation: GitHub PR #46
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
