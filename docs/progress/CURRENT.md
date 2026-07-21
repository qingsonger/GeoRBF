# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair / THICK002-REV-005 complete; fresh independent re-review required
- Requirement: REQ-THICK-002, Issue #96
- Branch: `codex/req-thick-002-sampled-validation`
- Draft pull request: #97
- Original reviewed head: `5878055d7ccbc7250aad3e6837f00375161e0052`
- First repair code/test head: `6bc6c1dc53bdd093110858cbf5d0787e97c702e9`
- Independently re-reviewed head with THICK002-REV-005: `aa6c134d68ddabd6750220dcca1c158ea81e3bc4`
- THICK002-REV-005 repair code/test head: `438937bd2b2ed715de23e1444a2cf41d71bf44c1`
- Registry state: `implemented`, not `integrated`
- Dependencies: REQ-THICK-001 and REQ-MODEL-001 are integrated

## Repair result

- The public controlled sampled-validation entry point now accepts explicit
  `ExecutionOptions` and passes them unchanged into the shared serial progress
  tracker. The convenience entry point retains default execution metadata.
- A public regression requests two threads and requires typed
  `UnsupportedThreadCount` plus an empty progress sequence, proving rejection
  precedes evaluation storage and every fitted-field evaluation.
- A second public regression requests one thread with determinism false,
  requires the unchanged successful report, and checks every progress event for
  that exact policy and operation.
- Search, refinement, Euclidean distance, quantile, violation, proposal, hard-
  constraint, and refit-boundary mathematics are unchanged.
- THICK002-REV-005 remains pending independent closure. No Repair task may mark
  it closed; PR #97 must remain Draft.

## Validation state

- Exact repair code/test head `438937b` passed the complete standard local gate:
  workspace format, warning-denying all-target/all-feature Clippy, all-feature
  workspace tests, workspace Rustdoc, all 58 requirement checks, and the
  complete diff whitespace check.
- Focused checks passed all seven public sampled-validation integration tests,
  all eight execution-control tests, both relevant Rustdoc tests, and warning-
  denying all-target/all-feature `georbf` Clippy.
- The one-iteration optimized benchmark smoke measured 2471.60 microseconds
  with unchanged checksums `32` and `2.0`.
- This following evidence commit changes only this handoff and the independent
  review record and therefore does not invalidate the stable production/test/
  build-input gate on `438937b`.
- Ready-only Windows/Ubuntu/macOS and benchmark-smoke CI remains unexecuted for
  this repair and is not claimed as passed.

## Next task

Open a fresh Review/re-review task for only Draft PR #97. Supply a fresh project
`math_reviewer` only the bounded requirement/dependency context, normative
documents, exact PR diff, prior finding, repair evidence, and validation state.
Independently determine whether THICK002-REV-005 is closed and whether any new
P0-P3 finding exists. If any finding remains, record it and stop. If clean,
follow the mandatory Ready -> exact-head Windows/Ubuntu/macOS plus benchmark-
smoke CI -> single merge -> truthful integration sequence. Do not begin
REQ-PROJECT-001 in this task.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #96
- Implementation pull request: GitHub PR #97
- Independent review: `docs/reviews/PR-97-INDEPENDENT-REVIEW.md`
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
