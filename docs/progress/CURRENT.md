# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review / REQ-THICK-002 findings recorded
- Requirement: REQ-THICK-002, Issue #96
- Branch: `codex/req-thick-002-sampled-validation`
- Draft pull request: #97
- Reviewed head: `5878055d7ccbc7250aad3e6837f00375161e0052`
- Registry state: `implemented`, not `integrated`
- Dependencies: REQ-THICK-001 and REQ-MODEL-001 are integrated

## Independent review result

- No P0 finding was reported.
- P1 THICK002-REV-001: search evaluates rounded points but reports the nominal
  line parameters. At `x=1e16`, nominal span three can correspond to stored
  intersections four units apart, producing a false violation and proposal.
- P2 THICK002-REV-002: the public validation operation has caller-unbounded
  location and iteration counts but no controlled entry point or cancellation
  checkpoints.
- P3 THICK002-REV-003: the change evidence claims violation and proposal
  provenance coverage, but tests assert only counts and determinism.
- P3 THICK002-REV-004: the documented off-grid tangential-contact `NotFound`
  behavior has no regression test.
- Normal orientation, bracket and bisection signs, original-coordinate gradient
  chain rule, type-7 quantiles, diagnostic separation, explicit proposal/refit
  boundary, dimension bounds, and deferred interfaces were otherwise verified.

## Validation state

- Exact implementation and reviewed head `5878055d` passed the complete local
  standard gate recorded by the Implement task.
- Draft CI run 29758953857 passed its configured Ubuntu correctness job on that
  exact head. The Ready-only three-platform and benchmark-smoke matrix was
  skipped as designed and is not claimed as passed.
- The independent reviewer and parent Review task each passed all four focused
  integration tests, all four module truth/numerical tests, and the complete PR
  diff whitespace check. The parent also passed both validation Rustdoc tests
  and all 58 requirement checks.
- Both independently reproduced the P1 case: nominal reported span three versus
  Euclidean separation four between the returned points.
- The optimized benchmark evidence remains 2588.35 microseconds per 32-location
  validation with checksums `16000` and `1000.0`; smoke remains 1951.10
  microseconds with checksums `32` and `2.0`.
- The final staged Review evidence diff passed `git diff --cached --check` and
  changes only this handoff and the independent review record.

## Next task

Open a fresh Repair task for only THICK002-REV-001 through THICK002-REV-004 on
Draft PR #97. Reproduce each finding with an independent regression, repair
reported Euclidean distance/rounding semantics and cancellation without
crossing the requirement boundary, add the missing provenance and tangential
evidence, run focused checks and the final standard gate after the last code
change, update review evidence and this bounded handoff, commit and push, then
stop for a fresh independent re-review. Do not begin REQ-PROJECT-001.

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
