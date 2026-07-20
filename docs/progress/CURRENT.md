# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Implement
- Requirement: REQ-TANGENT-001, Issue #90
- Branch: `codex/req-tangent-001-tangent-constraints`
- Pull request: #91 (Draft)
- Registry state: `implemented`, not `integrated`
- Dependencies: REQ-FUNC-001, REQ-SOFT-001, and REQ-DIAG-001 are `integrated`

## Implemented scope

- `TangentObservation<D>` lowers one validated D=1/D=2/D=3 tangent to the
  scalar equality `t^T grad f(x) = 0` with hard or explicit scalar
  SquaredL2/L1/Huber enforcement.
- Multiple unique tangent observations may share a point and preserve input
  order and full semantic provenance.
- `TangentProblem<D>` requires and records one caller-owned hard
  `DerivativeGaugeAnchor<D>` after every tangent. Missing gauge returns the
  first tangent source through stable `GEORBF-E4001`; no anchor is inferred.
- Six independent integration tests and two allocation/diagnostic unit tests
  cover analytic rows, multiple tangents, all soft losses, gauge recording,
  missing gauge, invalid metadata, duplicate IDs, dimensions, and storage
  failure.
- Rustdoc, normative math, runnable example, deterministic benchmark, Ready-CI
  smoke wiring, benchmark record, and requirement fragment are present.

## Validation state

- Focused six-test integration suite and two unit regressions pass.
- Warning-denying all-target/all-feature Clippy passes for `georbf`.
- The example prints two tangent rows and explicit gauge ID 3/value 125.
- Benchmark smoke checksum is `3824`; the 2,000-iteration checksum is
  `3824000` at 32.43 microseconds per build+compile on the recorded machine.
- After the Draft PR number and implemented registry state were recorded, the
  stable implementation tree passed the complete standard gate: workspace
  format, warning-denying all-target/all-feature Clippy, all-feature workspace
  tests, workspace Rustdoc, all 58 requirement checks, and `git diff --check`.
- This final handoff wording changes documentation only. It does not change
  production code, tests, manifests, schema, CI, registry, numerical behavior,
  dependencies, or any validated build input.

## Next task boundary

Commit and push this final implementation evidence, update Draft PR #91, and
stop. A fresh task must independently review only PR #91 and REQ-TANGENT-001,
using the project `math_reviewer`; it must not repair production code or begin
REQ-THICK-001 in the same task.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #90
- Draft implementation pull request: GitHub PR #91
- Requirement summary: `changes/REQ-TANGENT-001.md`
- Focused tests: `crates/georbf/tests/tangent_observations.rs`
- Normative behavior: `docs/math/NORMAL_AND_TANGENT.md`
- Benchmark: `docs/benchmarks/REQ-TANGENT-001.md`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable. No unavailable check is claimed as passed.
