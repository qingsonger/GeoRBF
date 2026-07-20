# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Implement complete; independent Review required
- Requirement: REQ-THICK-002, Issue #96
- Branch: `codex/req-thick-002-sampled-validation`
- Draft pull request: #97
- Registry state: `implemented`, not `integrated`
- Dependencies: REQ-THICK-001 and REQ-MODEL-001 are integrated

## Implementation result

- Added an immutable fitted-field validation API for exactly D=1, D=2, and
  D=3 with explicit adjacent level values, selected original-coordinate
  locations, minimum thickness, search/refinement limits, tolerances,
  quantiles, provenance, and proposal opt-in.
- Lower and upper intersections use deterministic uniform bracketing along the
  negative and positive fitted-gradient normal followed by bounded bisection.
  Low gradients, no intersection, and refinement exhaustion remain explicit
  per-location failures.
- Reports preserve measurements, failures, minimum, deterministic type-7
  quantiles, violations, and optional proposed local constraints. Proposals do
  not mutate a problem or trigger solving/refitting.
- Sampled validation has the distinct
  `SampledGeometricValidation / SampledGeometricEvidence` label and makes no
  global minimum-distance claim.

## Validation state

- Focused integration and module tests pass for fitted analytic parallel
  levels, independently evaluated curved levels, no intersection, quantiles,
  invalid inputs, deterministic reports, proposals, extreme gradient scales,
  and supported-dimension type bounds.
- Warning-denying all-target/all-feature Clippy passes for the crate.
- The optimized 32-location benchmark measured 2588.35 microseconds per
  validation with checksums `16000` and `1000.0`; its smoke workload passed
  at 1951.10 microseconds with checksums `32` and `2.0`.
- The stable implementation source head passed the complete local standard
  gate: workspace format, warning-denying all-target/all-feature Clippy,
  all-feature workspace tests, workspace Rustdoc, all 58 requirement checks,
  and `git diff --check`.

## Next task

Open a fresh read-only Review task for REQ-THICK-002 and the Draft PR. Supply
only the requirement summary, dependency closure, Issue #96 criteria and
exclusions, M5 plan, relevant thickness/model contracts and ADRs, complete PR
diff, test and benchmark evidence, registry/handoff state, and exact validation
results. Mathematical review must cover normal orientation, bracketing and
bisection behavior, dimensions and coordinate units, curved/tangential cases,
quantile convention, hard-constraint separation, proposal/refit boundaries,
finite arithmetic, allocations, determinism, diagnostics, interface
disposition, and requirement truthfulness. Do not repair production code in
that Review task.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #96
- Implementation pull request: GitHub PR #97
- Requirement summary: `changes/REQ-THICK-002.md`
- Independent truth/error tests: `crates/georbf/tests/thickness_validation.rs`
- Normative behavior: `docs/math/THICKNESS.md`
- Benchmark: `docs/benchmarks/REQ-THICK-002.md`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable. No unavailable check is claimed as passed.
