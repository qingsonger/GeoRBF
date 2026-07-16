# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair complete / fresh independent re-review required
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
  affine normalization, centers, coefficients, capabilities, general
  assembly/solve diagnostics, and complete CPD polynomial-action, RRQR/SVD
  rank, verified null-space, quality, provenance, and projected-energy evidence.
- Added one high-level fit boundary that consumes the normalized-coordinate
  `FieldProblem<D>`, assembles and solves with the same retained kernel, and
  discards builder, canonical, dense-system, and factorization state.
- Added original-coordinate scalar value, Cartesian gradient, and
  capability-gated Hessian evaluation, including exact center-variable signs,
  mixed second/third derivative contractions, anisotropy chain rules, and
  normalization chain rules.
- Added complete polynomial Hessians, deterministic borrowed model-record
  inputs including complete CPD assembly evidence, structured fit/evaluation
  failures, exact center capability rejection, and immutable `Send + Sync`
  multithread reads.
- Clarified that global anisotropy consumes points and returns pre-transform
  derivatives in its caller's current coordinate system. In fitted models that
  caller system is normalized model coordinates; external original-coordinate
  derivatives are produced afterward by the retained affine normalization.
- Added six independent analytic/property tests, Rustdoc, a runnable example,
  deterministic D=1/D=2/D=3 benchmark and baseline, CI benchmark smoke routing,
  architecture/model-format updates, and a requirement change fragment.
- CLI, C, C++, and Python are N/A until M8 schema/CLI and M9 binding
  requirements define stable external model surfaces.

## Validation state

- Focused model tests: six passed, covering Gaussian original-coordinate truth,
  combined anisotropy/normalization derivatives, directional-center signs
  through third order, CPD quadratic truth plus deterministic complete
  four-center/three-term rank, 4-by-1 null-space, quality, and nonempty
  projected-energy evidence, Matérn capability boundaries, and concurrent
  bit-identical reads.
- All five focused field-assembly tests and all thirteen focused
  global-anisotropy tests passed.
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
- A fresh read-only `xhigh` mathematical review found P2-1 and P3-1. The
  bounded repair now retains and exposes the discarded CPD evidence and
  clarifies the anisotropy caller-coordinate contract without changing
  numerical behavior.
- The complete local standard gate passed on the final repair tree: format,
  warning-denying workspace Clippy, all-feature workspace tests, workspace
  Rustdoc including compile-fail dimension boundaries, all 58 requirement
  checks, and `git diff --check`.

## Next task

Open a fresh read-only independent Review task for the exact pushed PR #61
repair head. Supply only the REQ-MODEL-001 summary and dependency closure,
Issue #60 criteria, M3 plan, scoped normative documents, PR diff, the original
review findings, and the repair validation evidence. Verify only P2-1 and P3-1
plus regression risk, update the independent review record, and stop. Keep the
PR Draft during this re-review; do not mark it ready, merge it, or begin
REQ-EXEC-001 in the same task.

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
