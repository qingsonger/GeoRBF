# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review complete; Repair required
- Requirement: REQ-NORMAL-001, Issue #87
- Branch: `codex/req-normal-001-observations`
- Pull request: #88 (Draft)
- Reviewed head: `8724f288b1415b95492e2195a2f72e2032d1b9b1`
- Registry state: `implemented`, not `integrated`
- Dependencies: REQ-ORIENT-001 and REQ-CONVEX-001 are `integrated`

## Independent review findings

- P1 R88-001: a positive degree angle can underflow to zero during radians
  conversion, silently strengthening the requested hard angular cone.
- P2 R88-002: componentwise L1 and Huber losses on D=3 complement rows depend
  on the arbitrary complement basis and are not rotation invariant.
- P2 R88-003: the final AngularCone roles and constraints use infallible `vec!`
  allocations despite the structured allocation-failure contract.
- Full evidence and required regressions are in
  `docs/reviews/PR-88-INDEPENDENT-REVIEW.md`.

## Validation state

- A fresh read-only project `math_reviewer` independently reviewed exact base
  `0ae9714` and head `8724f28` from bounded requirement, dependency, normative,
  diff, test, benchmark, registry, handoff, and validation evidence.
- The reviewer and parent task each passed all 8 focused normal-observation
  tests; the current tests do not cover R88-001, R88-002, or R88-003.
- The parent task passed the example and benchmark smoke; checksum was `11088`.
- Draft CI run 29723009629 passed the configured Ubuntu correctness job on exact
  reviewed head `8724f28`. Ready-only three-platform and benchmark smoke jobs
  correctly did not run.
- The complete standard gate recorded for exact implementation head `8724f28`
  remains valid. This Review task changes only review and bounded-handoff
  documents; it changes no production, test, manifest, schema, CI, build input,
  API, or numerical behavior.

## Next task boundary

Open a fresh Repair task limited to R88-001, R88-002, and R88-003. Add the
specified independent regressions before or alongside the smallest complete
repairs, run focused checks during repair and the complete standard gate after
the final code change, update review evidence and this bounded handoff, push,
and stop for a fresh independent re-review. Do not begin REQ-TANGENT-001.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #87
- Draft implementation pull request: GitHub PR #88
- Independent review: `docs/reviews/PR-88-INDEPENDENT-REVIEW.md`
- Requirement summary: `changes/REQ-NORMAL-001.md`
- Focused tests: `crates/georbf/tests/normal_observations.rs`
- Normative behavior: `docs/math/NORMAL_AND_TANGENT.md`
- Benchmark: `docs/benchmarks/REQ-NORMAL-001.md`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable. No unavailable check is claimed as passed.
