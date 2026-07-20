# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review clean; Ready CI pending
- Requirement: REQ-TANGENT-001, Issue #90
- Branch: `codex/req-tangent-001-tangent-constraints`
- Pull request: #91 (Draft)
- Reviewed implementation head: `86d1d3dcc948d70f6825822d1efe94b92b8b4f5b`
- Repair implementation head: `5e99aa629118ca4b4c81927d31adf67f19822b58`
- Independently re-reviewed head: `ab84fda560229fcb8e8c2ccf0e0361bba3751f30`
- Review result: R91-001 closed; no P0-P3 findings remain
- Registry state: `implemented`, not `integrated`
- Dependencies: REQ-FUNC-001, REQ-SOFT-001, and REQ-DIAG-001 are `integrated`

## Independent re-review result

- A fresh read-only project `math_reviewer` independently confirmed that the
  no-gauge path consumes exactly one item before any size inspection,
  reservation, or collection. R91-001 is closed.
- The paired unbounded-iterator regressions prove source-aware
  `GEORBF-E4001` without a gauge and retain `CountOverflow` with an explicit
  gauge.
- Tangent formula, sign, reversal invariance, units, hard/soft semantics,
  value-gauge mathematics, deterministic provenance, finite input handling,
  adapter dispositions, benchmark and CI wiring, and registry truth are clean.
- No P0, P1, P2, or P3 finding remains. Kernel, center, polynomial, rank,
  SPD/CPD, anisotropy, and Hessian concerns are unchanged and out of scope.

## Validation state

- Exact independently re-reviewed head `ab84fda` passed workspace format,
  warning-denying all-target/all-feature Clippy, all-feature workspace tests,
  workspace doctests, all 58 requirement checks, and `git diff --check`.
- The same head passed all eight focused tangent integration tests, both
  tangent module regressions, the runnable example, and benchmark smoke
  checksum `3824`.
- Draft CI run 29731323902 passed the configured Ubuntu correctness gate on
  exact head `ab84fda`. The Ready-only matrix correctly did not run.
- This final evidence update changes only the independent review record and
  this bounded handoff. It changes no production code, test, manifest, schema,
  CI, build, API, numerical, registry, or dependency input.

## Next task boundary

Commit and push this clean re-review evidence, mark PR #91 ready, and wait for
the complete Windows/Ubuntu/macOS and benchmark-smoke CI on that exact Ready
head. Merge exactly once only when it is green, then record truthful registry
and handoff state through an isolated integration-state change. Do not begin
REQ-THICK-001 or any other requirement.

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
