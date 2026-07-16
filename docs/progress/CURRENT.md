# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review complete / bounded Repair required
- Requirement: REQ-MODEL-001, Issue #60
- Branch: `codex/req-model-001-immutable-fitted-field`
- Draft pull request: #61
- Reviewed head: `14d21d1`
- Review record: `docs/reviews/PR-61-INDEPENDENT-REVIEW.md`
- Registry state: `implemented`
- Dependencies: REQ-SOLVE-001 and REQ-COORD-001 are `integrated`

## Implemented scope

- Added immutable `FittedField<D>` for D=1/D=2/D=3 with one concrete retained
  kernel definition, optional constant global anisotropy, coordinate metadata,
  affine normalization, centers, coefficients, capabilities, and general
  assembly/solve diagnostics. The review found that CPD-specific rank,
  null-space, and projected-energy assembly evidence is not yet retained.
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
- Exact implementation head `8652bb4` passed one complete local
  standard gate: format, warning-denying workspace Clippy, all-feature
  workspace tests, workspace Rustdoc including the D=4 compile-fail boundary,
  all 58 requirement checks, and `git diff --check`.
- The subsequent PR-link handoff update changes only the completion registry
  and this bounded evidence file; it changes no production code, tests,
  manifest, schema, CI, build input, benchmark input, or numerical behavior.
- Exact reviewed head `14d21d1` passed Draft Ubuntu CI run 29480459334:
  format, warning-denying workspace Clippy, all-feature workspace tests,
  workspace Rustdoc, spike gates, and all 58 requirement checks.
- A fresh read-only `xhigh` mathematical review found one P2 and one P3
  finding. A separately adjudicated proposed normalization-semantics P1 was
  rejected because Issue #60 explicitly establishes normalized-model-coordinate
  kernel evaluation followed by the `S` derivative chain rule.

## Next task

Open a fresh Repair task for only PR #61 findings P2-1 and P3-1. Retain and
expose complete CPD rank, verified null-space, and projected-energy assembly
evidence in immutable fitted-model diagnostics, with the required deterministic
regression. Clarify that global anisotropy consumes points in its caller's
current coordinate system and distinguish those derivatives from external
original-coordinate fitted-model outputs. Run focused checks during repair,
then the complete standard gate on the stable final head, update evidence,
push, and stop for a fresh independent re-review. Do not begin REQ-EXEC-001.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #60
- Implementation: GitHub PR #61
- Independent review: `docs/reviews/PR-61-INDEPENDENT-REVIEW.md`
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
