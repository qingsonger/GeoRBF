# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Implement
- Requirement: REQ-TUNE-001, deterministic parameter tuning
- Issue: #126
- Branch: `codex/req-tune-001-deterministic-tuning`
- Pull request: not yet opened
- Dependency: REQ-MODEL-001 is integrated
- Registry status: `in_progress`

## Implemented scope

- Added a GeoRBF-owned `TuningProblem<D>` for D=1, D=2, and D=3 with explicit
  finite candidates and inclusive physical-domain bounds for length, support
  radius, regularization, axis ratio, and influence radius.
- Implemented exact fixed selection, median-nearest-neighbor distance
  selection, seeded deterministic cross-validation, generalized
  cross-validation, and worst-case squared power-function selection.
- Kept actual fitting behind a caller-owned `TuningEvaluator`. Evaluator
  failure rejects the whole search; there is no candidate skipping, criterion
  fallback, hidden regularization, jitter, or semantic-problem mutation.
- Added complete criterion, seed, bound, fold, candidate-score, tie, and
  criterion-evidence diagnostics.
- Added independent strategy truth, known-optimum, deterministic-seed, bounds,
  malformed-evidence, failure, and D=1/D=2/D=3 tests plus a runnable rustdoc
  example.
- Added a five-strategy release benchmark, Ready/main CI smoke wiring,
  numerical-policy documentation, and the requirement change fragment.

## Validation state

- Focused tuning integration tests pass: 11/11.
- Focused tuning rustdoc passes: 1/1.
- Focused warning-denying Clippy passes for the tuning test and benchmark.
- The optimized 16-candidate benchmark smoke passes for all five strategies.
- The complete standard workspace gate has not yet run on the stable head and
  is not claimed as passed.
- Mathematical/numerical independent Review remains a fresh next task after
  this Implement task opens or updates the Draft PR.

## Next task boundary

Finish only this Implement task: record the full benchmark baseline, run the
complete standard workspace gate after the final code/manifest/registry
change, update this handoff with the stable head and exact validation evidence,
commit, push, open or update a Draft PR, and stop.

The following task must be a fresh independent Review of only REQ-TUNE-001 and
its Draft PR. It must use the isolated project `math_reviewer`, record P0--P3
findings, and must not repair production code or start REQ-PERF-001.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #126
- Requirement summary: `changes/REQ-TUNE-001.md`
- Numerical policy: `docs/architecture/SOLVER_POLICY.md`
- Benchmark: `docs/benchmarks/REQ-TUNE-001.md`
- Production implementation: `crates/georbf/src/tuning.rs`
- Independent tests: `crates/georbf/tests/tuning.rs`
- Release benchmark: `crates/georbf/benches/parameter_tuning.rs`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, and API/ABI/schema snapshot checks are tracked by
later requirements and release gates. Local `actionlint` is unavailable. No
unavailable check is claimed as passed.
