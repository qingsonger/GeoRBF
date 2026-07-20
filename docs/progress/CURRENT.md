# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Implement / REQ-INFEAS-001 complete locally
- Requirement: REQ-INFEAS-001, Issue #84
- Branch: `codex/req-infeas-001-diagnostics`
- Pull request: pending initial push; create as Draft
- Registry state in this change: `in_progress` until the Draft PR number exists
- Dependencies: REQ-CONVEX-001 and REQ-DIAG-001 are `integrated`

## Implementation result

- Added immutable source-aware exact-duplicate and scale-aware near-duplicate
  review for hard canonical affine equality and linear-bound functionals.
- Rows are independently infinity-normalized and compared in both sign
  orientations. The explicit warning-only threshold is `128 * epsilon`.
- Diagnostics retain relation kinds, both complete provenances, orientation,
  normalized distance, and threshold in deterministic equality-then-bound pair
  order. Canonical rows remain byte-for-byte unchanged.
- Exact conflict review now covers constant equalities and exactly proportional
  equality/equality, equality/bound, and bound/bound intervals. Only exact
  proportionality can reject a pair; near duplicates never prove infeasibility.
- Soft objectives remain outside hard-feasibility decisions. Ordered cones and
  general multi-row infeasibility remain on the independently reviewed convex
  certificate path.
- Rust, example, benchmark, Rustdoc, normative constraint/architecture docs,
  changelog, and explicit CLI/C/C++/Python dispositions are complete.

## Validation state

- Five independent `infeasibility` tests pass: exact duplicates, sign-reversed
  near duplicates, source/order preservation, unchanged canonical input,
  equality/bound and constant-equality conflicts at row scales `1e-12`, `1`,
  and `1e12`, soft exclusion, D=1/D=2/D=3, `Send + Sync`, and a reviewed
  three-row primal-infeasibility certificate.
- The adjacent problem-IR, linear-constraint, and convex-solver suites pass.
- The runnable example reports one exact and two near-duplicate pairs.
- The 5,000-iteration 96-constraint benchmark measured 145.75 microseconds per
  review with checksum `480000`; the eight-iteration smoke measured 178.09
  microseconds with checksum `768`. Timings are not performance promises.
- After the final production/test change, the stable tree passed the complete
  standard gate: workspace format, warning-denying all-target/all-feature
  Clippy, all-feature workspace tests, workspace Rustdoc, all 58 requirement
  checks, and `git diff --check`.

## Next task

After the branch is pushed and the Draft PR number is linked in this handoff and
the registry, open a fresh Review task. It must review only REQ-INFEAS-001 and
the Draft PR, create and wait for the project `math_reviewer`, record findings,
and stop without repairing production code or starting another requirement.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #84
- Requirement summary: `changes/REQ-INFEAS-001.md`
- Focused tests: `crates/georbf/tests/infeasibility.rs`
- Example: `crates/georbf/examples/constraint_diagnostics.rs`
- Benchmark and report: `crates/georbf/benches/constraint_diagnostics.rs` and
  `docs/benchmarks/REQ-INFEAS-001.md`
- Normative behavior: `docs/math/CONSTRAINT_SEMANTICS.md`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable. No unavailable check is claimed as passed.
