# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Implement complete / fresh independent Review required
- Requirement: REQ-MODEL-001, Issue #60
- Branch: `codex/req-model-001-immutable-fitted-field`
- Draft pull request: pending first push
- Registry state: `in_progress` until the Draft PR number is recorded
- Dependencies: REQ-SOLVE-001 and REQ-COORD-001 are `integrated`

## Implemented scope

- Added immutable `FittedField<D>` for D=1/D=2/D=3 with one concrete retained
  kernel definition, optional constant global anisotropy, coordinate metadata,
  affine normalization, centers, coefficients, capabilities, and complete
  assembly/solve diagnostics.
- Added one high-level fit boundary that consumes the normalized-coordinate
  `FieldProblem<D>`, assembles and solves with the same retained kernel, and
  discards builder, canonical, dense-system, and factorization state.
- Added original-coordinate scalar value, Cartesian gradient, and
  capability-gated Hessian evaluation, including exact center-variable signs,
  mixed second/third derivative contractions, anisotropy chain rules, and
  normalization chain rules.
- Added complete polynomial Hessians, deterministic borrowed model-record
  inputs, structured fit/evaluation failures, exact center capability
  rejection, and immutable `Send + Sync` multithread reads.
- Added six independent analytic/property tests, Rustdoc, a runnable example,
  deterministic D=1/D=2/D=3 benchmark and baseline, CI benchmark smoke routing,
  architecture/model-format updates, and a requirement change fragment.
- CLI, C, C++, and Python are N/A until M8 schema/CLI and M9 binding
  requirements define stable external model surfaces.

## Validation state

- Focused model tests: six passed, covering Gaussian original-coordinate truth,
  combined anisotropy/normalization derivatives, directional-center signs
  through third order, CPD quadratic polynomial truth, Matérn capability
  boundaries, deterministic record order, and concurrent bit-identical reads.
- Existing polynomial integration tests: ten passed after adding Hessians.
- Runnable immutable-model example: passed.
- D=1/D=2/D=3 optimized benchmark smoke: passed with deterministic checksums.
- Four 100-iteration local benchmark runs retained dimension-specific
  bit-identical checksums; timings and executable size are recorded in
  `docs/benchmarks/REQ-MODEL-001.md`.
- The final stable code/test/manifest/CI/registry tree passed one complete local
  standard gate: format, warning-denying workspace Clippy, all-feature
  workspace tests, workspace Rustdoc including the D=4 compile-fail boundary,
  all 58 requirement checks, and `git diff --check`.

## Next task

After the implementation commit is pushed and the Draft PR number is recorded,
open a fresh Review task for only REQ-MODEL-001. Supply the bounded requirement
and dependency summaries, Issue #60 acceptance criteria, M3 plan, scoped
architecture/model-format/math contracts, complete PR diff, tests, and
benchmark evidence to a fresh read-only `math_reviewer`. Record findings
without repairing production code in the same task. Do not begin
REQ-EXEC-001.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #60
- Requirement summary: `changes/REQ-MODEL-001.md`
- Architecture: `docs/architecture/ARCHITECTURE.md`
- Deterministic model inputs: `docs/architecture/MODEL_FORMAT.md`
- Mathematical field representation: `docs/math/MATH_SPEC.md`
- Benchmark: `docs/benchmarks/REQ-MODEL-001.md`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable.
