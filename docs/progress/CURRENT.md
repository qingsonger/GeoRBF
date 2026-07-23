# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Implement / REQ-SPARSE-001 complete; independent Review required
- Requirement: REQ-SPARSE-001
- Issue: #117
- Branch: `codex/req-sparse-001-compact-support`
- Draft pull request: #118
- Dependencies: REQ-SPIKE-003, REQ-FIELD-001, and REQ-KERNEL-004 are integrated
- Registry status: `planned` until review, Ready CI, merge, and integration

## Implemented scope

- The existing `FieldProblem<D>` assembles strictly positive-definite Wendland
  systems directly from exact support-neighbor pairs into GeoRBF-owned
  sorted-unique full symmetric CSC for D=1, D=2, and D=3.
- A private immutable rstar index retains stable center/term identities. D=1
  and D=2 are zero-padded into three index coordinates to avoid rstar's
  one-dimensional panic; exact support truth remains dimension-specific.
- Candidate hits are independently recomputed with stable isotropic or global-
  anisotropy separation and the strict `radius < support_radius` rule.
- Private faer 0.24.4 lower LLT with AMD ordering solves without densification,
  fallback, jitter, regularization, pseudoinverse, or constraint relaxation.
  Exact original-unit residual review uses tolerance `128*n*epsilon`.
- `FittedField<D>` gained sparse fitting without a second model hierarchy and
  retains the index for local-center value, gradient, and Hessian evaluation.
- Diagnostics cover nonzeros, density, support coverage, memory, backend,
  ordering, residuals, and visited-versus-total evaluation centers.

## Validation state

- Focused sparse, field, model, and execution tests pass.
- The sparse test suite covers a hand-derived CSC and analytic solution,
  support boundaries, D=1/D=2/D=3 dense parity, mixed Value and
  DirectionalDerivative representers, local evaluation, anisotropy, 512-point
  scaling, cancellation, memory limits, and singular rejection.
- Warning-denying focused all-feature Clippy passes for the sparse test and
  production compact-sparse benchmark.
- The release benchmark smoke passed at 64 points; four full 512-point runs
  retained 3,200 nonzeros and bit-identical phase checksums.
- The final stable implementation state passes all five standard workspace
  checks: format, warning-denying all-target/all-feature Clippy, all-feature
  workspace tests, workspace doctests, and the 58-requirement registry check.
  The only later changes record validation and pull-request evidence.

## Next task boundary

Open a fresh Review task for the Draft PR. Supply only the requirement
show/dependency summaries, Issue #117 criteria, scoped architecture/solver
documents and ADR-0012, the complete PR diff, tests, dependency audit, and
benchmark evidence to the isolated `math_reviewer`. Do not repair findings or
begin REQ-CENTER-001 in that Review task.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #117
- Draft implementation: GitHub PR #118
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
