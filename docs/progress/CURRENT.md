# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Implement complete / awaiting independent Review
- Requirement: REQ-SOLVE-001, Issue #57
- Branch: `codex/req-solve-001-dense-equality-solvers`
- Draft pull request: #58
- Registry state: `implemented`
- Dependencies: REQ-SPIKE-001, REQ-SPIKE-002, and REQ-FIELD-001 are integrated

## Implemented scope

- GeoRBF-owned finite symmetric `DenseEqualitySystem`, explicit
  `DenseSolveOptions`, immutable `DenseSolution`, and structured errors and
  diagnostics; nalgebra types remain private.
- Explicit checked Cholesky or symmetric-pivoted Bunch--Kaufman LBLT without
  fallback, plus eight-pass equilibration, RRQR screening, bounded SVD review,
  ambiguity detection, norms, rank classification, and condition policy.
- `None` and positive finite `Explicit(value)` regularization, recording both
  original and effective rank and residual evidence. Automatic regularization,
  jitter, substitute diagonals, pseudoinverse, and constraint relaxation are
  absent.
- Symmetric congruence factorization scaling and bounded one-factorization
  iterative refinement. Corrections are accepted only when the fixed-stack
  exact-binary residual in original units strictly decreases.
- Independent analytic SPD, mandatory 2-by-2 indefinite pivot, rank failure,
  row/unit scaling, condition, exact residual, explicit regularization,
  malformed input, SVD non-convergence, and assembled-field boundary tests.
- Rustdoc, runnable example, 64-by-64 Cholesky/LBLT benchmark, three-platform
  smoke route, benchmark evidence, dependency re-audit, and change fragment.
- CLI, C, C++, and Python are N/A until immutable fitted models, versioned
  schemas, and frozen binding surfaces exist.

## Validation state

- Focused solver integration tests and forced-SVD unit tests pass.
- Focused `cargo clippy -p georbf --all-targets -- -D warnings` passes.
- The runnable example and two-iteration benchmark smoke pass.
- Four consecutive 100-iteration benchmark runs have bit-identical checksums;
  timing and binary-size evidence is in `docs/benchmarks/REQ-SOLVE-001.md`.
- The final content including the PR #58 registry link and `implemented` state
  passed the complete standard gate: formatting, workspace
  all-target/all-feature Clippy, workspace tests, doctests, all 58 requirement
  checks, and `git diff --check`. This subsequent sentence changes handoff
  evidence only, so that immutable-content gate remains applicable.
- Draft PR #58 CI was triggered after the initial push. Its result is not an
  Implement-mode prerequisite and must be checked during the fresh Review task.

## Next task

Open a fresh Review task that independently reviews only PR #58 and
REQ-SOLVE-001 with the project `math_reviewer`. Do not repair production code
or begin REQ-MODEL-001 in that task. If findings exist, record them and stop for
a separate Repair task; if none exist, follow the fresh re-review/integration
sequence in `docs/CODEX_WORKFLOW.md`.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #57
- Implementation: GitHub PR #58
- Requirement summary: `changes/REQ-SOLVE-001.md`
- Solver policy: `docs/architecture/SOLVER_POLICY.md`
- Backend decision and production re-audit:
  `docs/adr/ADR-0010-nalgebra-dense-factorization-backend.md`
- Benchmark: `docs/benchmarks/REQ-SOLVE-001.md`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable.
