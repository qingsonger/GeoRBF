# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review complete; repair required
- Requirement: REQ-TANGENT-001, Issue #90
- Branch: `codex/req-tangent-001-tangent-constraints`
- Pull request: #91 (Draft)
- Reviewed implementation head: `86d1d3dcc948d70f6825822d1efe94b92b8b4f5b`
- Review result: P2 R91-001
- Registry state: `implemented`, not `integrated`
- Dependencies: REQ-FUNC-001, REQ-SOFT-001, and REQ-DIAG-001 are `integrated`

## Review result

- A fresh read-only project `math_reviewer` independently reviewed exact PR
  head `86d1d3d` using only bounded requirement, dependency, normative, diff,
  test, benchmark, registry-hunk, handoff, and validation evidence.
- P2 R91-001: `TangentProblem::try_new` reserves and consumes the entire
  observation iterator before checking the known absence of a gauge. A missing
  gauge can therefore return `CountOverflow` or `AllocationFailed`, or never
  return for an unbounded iterator, instead of source-aware `GEORBF-E4001`.
- Required regression: `std::iter::repeat(valid_tangent)` plus no gauge must
  return `MissingGauge`, `GEORBF-E4001`, and the first tangent identifier;
  the same iterator with an explicit gauge must retain `CountOverflow`.
- The tangent formula, sign, units, reversal invariance, explicit hard and
  scalar soft semantics, value-gauge mathematics, provenance, dimension bounds,
  adapter dispositions, and absence of hidden regularization are otherwise
  consistent. No P0, P1, P3, or additional P2 finding was reported.

## Validation state

- Draft CI run 29729498305 passed its configured Ubuntu correctness job on
  exact implementation head `86d1d3d`; Ready-only matrix jobs did not run.
- The parent Review task passed all six focused integration tests, both unit
  regressions, the example, benchmark smoke checksum `3824`, all 58 requirement
  checks, and the complete PR diff whitespace check.
- The Review evidence tree passed the complete standard gate: workspace format,
  warning-denying all-target/all-feature Clippy, all-feature workspace tests,
  workspace doctests, all 58 requirement checks, and `git diff --check`.
- Exact implementation head `86d1d3d` retains its recorded complete local gate.
  This final evidence wording touches only the independent review record and
  this bounded handoff, not production, tests, manifests, schema, CI, registry,
  numerical behavior, dependencies, or build inputs.

## Next task boundary

Open a fresh Repair task limited to R91-001. Add the independent missing-gauge
precedence regressions, implement the smallest repair, run focused checks and
the final stable-head standard gate, update review evidence and this handoff,
push, and stop for a fresh independent re-review. Do not begin REQ-THICK-001 or
any other requirement.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #90
- Draft implementation pull request: GitHub PR #91
- Independent review: `docs/reviews/PR-91-INDEPENDENT-REVIEW.md`
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
