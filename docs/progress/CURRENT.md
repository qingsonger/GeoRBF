# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Implement complete; independent Review next
- Requirement: REQ-SPIKE-001, Issue #48
- Branch: `codex/req-spike-001-dense-factorization`
- Pull request: pending Draft creation
- Registry state: `planned` until the Draft PR number is recorded
- Dependency: REQ-BOOTSTRAP-001 is integrated

## Implementation result

- Added an excluded comparison harness pinned to nalgebra 0.35.0 and faer
  0.24.4 with checked Cholesky, symmetric-pivoted Bunch--Kaufman LBLT, finite
  and original-unit residual review, and at most three explicit refinement
  corrections.
- Six independent cases cover analytic SPD truth, a leading-zero indefinite
  system requiring a 2-by-2 pivot, wrong-Cholesky and singular failures,
  ill-conditioned scaling, deterministic repeats, and invalid input.
- Accepted ADR-0010 selects nalgebra 0.35 for later private production adoption.
  This requirement adds no production solver, dependency, public matrix type,
  or user-facing API.
- CI now lints and tests both single-backend configurations and their combined
  path, rejects an empty selection, and runs the smoke workload. Ready and main
  execute this evidence on Windows, Ubuntu, and macOS.

## Validation state

- Combined and both single-backend focused test configurations pass 6/6 tests.
- Spike Clippy with all targets, all features, and warnings denied passes.
- The negative no-backend configuration fails with the required compile error.
- The optimized smoke and complete 32/64/128 comparison workloads pass.
- The final stable-head standard workspace gate and `git diff --check` must be
  recorded after the last implementation or evidence change.

## Next task

After the Draft PR is created and its exact head is green, open a fresh Review
task. Review only REQ-SPIKE-001 and that PR; use the project `math_reviewer`
read-only agent with the bounded requirement/dependency summary, relevant
solver policy and ADR, complete PR diff, and validation evidence. Do not repair
production code or begin REQ-IR-001 in the Review task.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #48
- Backend decision: `docs/adr/ADR-0010-nalgebra-dense-factorization-backend.md`
- Reproducible harness: `spikes/factorization-backends/`
- Benchmark and size baseline: `docs/benchmarks/REQ-SPIKE-001.md`
- Requirement change summary: `changes/REQ-SPIKE-001.md`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, allocation instrumentation, and API/ABI/schema
snapshot checks are tracked by later requirements and release gates. Local
`actionlint` is unavailable. The performed advisory review was an OSV batch API
query of every exact selected package; it is not a claim that unavailable audit
tools ran.
