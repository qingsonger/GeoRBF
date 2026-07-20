# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair / REQ-THICK-002 repair complete; fresh re-review required
- Requirement: REQ-THICK-002, Issue #96
- Branch: `codex/req-thick-002-sampled-validation`
- Draft pull request: #97
- Reviewed head: `5878055d7ccbc7250aad3e6837f00375161e0052`
- Repair code/test head: `6bc6c1dc53bdd093110858cbf5d0787e97c702e9`
- Registry state: `implemented`, not `integrated`
- Dependencies: REQ-THICK-001 and REQ-MODEL-001 are integrated

## Repair result pending independent re-review

- THICK002-REV-001: reports now use the Euclidean separation of the returned
  intersections. The `x=1e16` analytic regression proves stored distance four
  and no false violation/proposal at threshold 3.5.
- THICK002-REV-002: the new controlled public entry point uses a checked maximum
  evaluation budget and pre/post-evaluation cancellation checkpoints. A typed
  cancellation after evaluation three returns no report or later event.
- THICK002-REV-003: integration tests compare every provenance field on each
  measurement, violation, and proposed constraint with its input location.
- THICK002-REV-004: the analytic `x=0.5` curved-field tangent on an off-root grid
  is required to return lower-side `NotFound` while the upper side succeeds.
- These are Repair claims only. No finding is independently closed until a
  fresh read-only re-review examines the repaired head.

## Validation state

- Exact repair code/test head `6bc6c1d` passed the complete standard local gate:
  workspace format, warning-denying all-target/all-feature Clippy, all-feature
  workspace tests, workspace Rustdoc, all 58 requirement checks, and the
  complete diff whitespace check.
- Focused checks passed five public integration tests, six module
  truth/numerical tests, eight execution-control tests, both relevant Rustdoc
  tests, and warning-denying `georbf` Clippy.
- The repaired optimized benchmark measured 2299.12 microseconds per validation
  with unchanged checksums `16000` and `1000.0`; smoke measured 1793.80
  microseconds with unchanged checksums `32` and `2.0`.
- Prior Draft CI run 29784308094 passed Ubuntu correctness on pre-repair evidence
  head `473ac29`. CI on the pushed Repair evidence head is not yet claimed; the
  Ready-only three-platform and benchmark-smoke matrix remains unexecuted.
- The evidence commit following `6bc6c1d` changes only this handoff and the
  independent review record and therefore does not invalidate the stable
  code/test-head standard gate.

## Next task

Open a fresh Review/re-review task for only Draft PR #97. Supply the independent
reviewer the bounded requirement/dependency summary, normative documents, exact
diff through the Repair evidence head, and validation evidence without this
Repair reasoning. Independently confirm THICK002-REV-001 through
THICK002-REV-004 and check for new P0-P3 findings. If any finding remains,
record it and stop without repair. Only if re-review is clean may that fresh
task mark the PR ready, wait for exact ready-head Windows/Ubuntu/macOS and all
benchmark-smoke CI, merge once when green, and record isolated truthful
integration state. Do not begin REQ-PROJECT-001.

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
