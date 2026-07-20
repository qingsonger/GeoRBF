# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Implement / REQ-THICK-001 complete pending Draft PR publication
- Requirement: REQ-THICK-001, Issue #93
- Branch: `codex/req-thick-001-local-thickness`
- Draft pull request: pending first push
- Registry state in this change: `in_progress` until the Draft PR number is recorded
- Dependencies: REQ-LEVEL-001, REQ-NORMAL-001, and REQ-CONVEX-001 are integrated

## Implemented scope

- Scalar `LevelOrder` gaps remain canonical linear bounds and are labeled
  `ScalarLevelGap / ScalarOnly`; no scalar gap is presented as a geometric
  distance.
- Immutable D=1/D=2/D=3 `LocalNormalThickness` values compile the hard sampled
  first-order relation `T_min ||grad f(x)|| <= h_upper - h_lower` over explicit
  level variables.
- Compilation requests Cartesian derivatives in deterministic axis order,
  owns the positive thickness scale and both level signs, preserves provenance,
  and exposes only affine rows and ordered cones to the canonical solver.
- Diagnostics label the cone `SampledLocalNormalCone /
  SampledLocalFirstOrder`; sampled geometric validation remains deferred to
  REQ-THICK-002 and is not implied.
- Structured failures cover invalid thickness and endpoints, empty input,
  unknown levels, duplicate IDs, caller linearization, field-variable bounds,
  allocation, and unrepresentable scaled coefficients or constants.
- Rust API, Rustdoc, normative documentation, example, focused benchmark,
  benchmark-smoke CI wiring, tests, and change evidence are implemented. CLI is
  N/A until M8; C, C++, and Python are N/A until M9.

## Validation state

- All eight focused `thickness` integration tests pass.
- The `local_thickness` example runs and reports one scalar bound and one local
  cone with distinct diagnostic classifications.
- Benchmark smoke passes with checksum `8304`; the 2,000-iteration baseline is
  40.17 microseconds per build/compile with checksum `8304000` on the recorded
  Windows environment.
- Warning-denying all-target/all-feature Clippy for `georbf` passes.
- The complete stable-head standard gate remains to be run after the final code
  change and before publication.

## Next task

After this implementation is committed, pushed, and opened as a Draft PR,
record its number and set the requirement to `implemented`. Then open a fresh
Review task for only REQ-THICK-001 and that PR. The reviewer must be independent
and read-only; do not repair findings or begin REQ-THICK-002 in the Review task.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #93
- Requirement summary: `changes/REQ-THICK-001.md`
- Focused tests: `crates/georbf/tests/thickness.rs`
- Normative behavior: `docs/math/THICKNESS.md`
- Benchmark: `docs/benchmarks/REQ-THICK-001.md`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable. No unavailable check is claimed as passed.
