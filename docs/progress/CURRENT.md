# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Implement complete; fresh independent Review required
- Requirement: REQ-LINEQ-001, Issue #75 (open)
- Branch: `codex/req-lineq-001-linear-bounds`
- Draft implementation pull request: #76
- Exact implementation and complete local gate head:
  `8931260b6d37aa87bd82fa9416bd97d119c6d134`
- Registry state: `implemented`
- Dependencies: REQ-IR-001 and REQ-LEVEL-001 are `integrated`
- No later requirement may start until REQ-LINEQ-001 is independently reviewed,
  repaired if needed, passed through exact Ready CI, merged, and recorded as
  `integrated`

## Implementation result

- Added immutable D=1/D=2/D=3 Rust constructors for lower, upper, closed
  interval, explicitly oriented closed inside/outside, scalar-gap, and
  directional-monotonicity semantics.
- Inside/outside requires `InsideAtOrBelow` or `InsideAtOrAbove`; scalar gaps
  use `upper - lower >= minimum_gap`; monotonicity accepts exactly one
  coefficient-one directional derivative and explicit increasing/decreasing
  sense.
- All forms lower to existing semantic and canonical linear bounds with stable
  source provenance and insertion order. Existing explicit `LevelOrder` remains
  the sole layer-order path.
- A field-only canonical bound problem can compose with a compiled level
  problem when its named blocks match the field prefix. Membership/fixed rows,
  order bounds, priors, variable identities, and all provenance remain
  unchanged; field bounds/objectives append and canonical metadata is rebuilt.
- Canonical construction rejects infeasible constant rows and disjoint exact
  equal/sign-reversed hard-row intervals with complete source evidence. It does
  not claim general LP feasibility or compare approximate rows.
- Added seven independent tests, a runnable example, a deterministic mixed
  96-constraint benchmark, normative documentation, a benchmark report, and
  the requirement change fragment.
- No solver backend, dependency, schema, fitting support, adapter
  reimplementation, unsafe code, hidden scaling, jitter, regularization,
  pseudoinverse, relaxation, deletion, or automatic repair was introduced.

## Validation state

- Focused linear-constraint tests pass: 7 passed, 0 failed.
- The runnable example passes and prints the expected inside upper bound and
  increasing lower rate.
- The benchmark smoke and 2,000-iteration baseline pass with deterministic
  6,400-byte-per-iteration canonical memory estimates.
- Exact implementation head `8931260` passes the complete stable-head standard
  workspace gate: format,
  warning-denying all-target/all-feature Clippy, all-feature workspace tests,
  workspace Rustdoc, all 58 requirement checks, and `git diff --check`.
- The tail after `8931260` changes only this bounded handoff. It changes no
  production code, test, manifest, schema, CI, build input, registry, API,
  numerical behavior, or dependency, so the exact-head gate remains valid.
- Draft CI is not claimed as reviewed. No independent review, Ready transition,
  merge, integration, tag, or release is claimed.

## Next task

Open a fresh independent Review task limited to the Draft REQ-LINEQ-001 pull
request #76. Review every sign and unit,
inside/outside boundary convention, scalar-gap order, monotonicity direction,
exact row normalization, hard infeasibility evidence, D=1/D=2/D=3 behavior,
allocation paths, interface exclusions, benchmark, registry, and documentation.
Record findings without repairing production code in that Review task. Stop
without beginning another requirement.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #75
- Draft implementation pull request: GitHub PR #76
- Requirement summary: `changes/REQ-LINEQ-001.md`
- Mathematical semantics: `docs/math/CONSTRAINT_SEMANTICS.md`
- Architecture: `docs/architecture/PROBLEM_IR.md` and
  `docs/architecture/ARCHITECTURE.md`
- Focused tests: `crates/georbf/tests/linear_constraints.rs` plus retained
  layer-order coverage in `crates/georbf/tests/levels.rs`
- Example: `crates/georbf/examples/linear_constraints.rs`
- Benchmark and report:
  `crates/georbf/benches/linear_constraint_compilation.rs` and
  `docs/benchmarks/REQ-LINEQ-001.md`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable.
