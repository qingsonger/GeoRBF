# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Implement
- Requirement: REQ-TANGENT-001, Issue #90
- Branch: `codex/req-tangent-001-tangent-constraints`
- Pull request: pending Draft creation after the first pushed implementation commit
- Registry state: `planned` until the stable implementation gate and Draft PR exist
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
- The complete stable-head standard gate is still pending and must run once
  after the Draft PR number is recorded.

## Next task boundary

Finish the same Implement mode only: create and push the first implementation
commit, open the Draft PR, record its number and set the registry status to
`implemented`, run the complete stable-head standard gate, commit/push the
final evidence, and stop. Independent mathematical review must occur in a
fresh task; do not begin REQ-THICK-001.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #90
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
