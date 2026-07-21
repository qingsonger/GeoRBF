# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review / REQ-THICK-002 re-review found one P2; Repair required
- Requirement: REQ-THICK-002, Issue #96
- Branch: `codex/req-thick-002-sampled-validation`
- Draft pull request: #97
- Original reviewed head: `5878055d7ccbc7250aad3e6837f00375161e0052`
- Repair code/test head: `6bc6c1dc53bdd093110858cbf5d0787e97c702e9`
- Independently re-reviewed head: `aa6c134d68ddabd6750220dcca1c158ea81e3bc4`
- Registry state: `implemented`, not `integrated`
- Dependencies: REQ-THICK-001 and REQ-MODEL-001 are integrated

## Independent re-review result

- A fresh read-only `math_reviewer` independently closed THICK002-REV-001
  through THICK002-REV-004 on exact head `aa6c134`.
- P2 THICK002-REV-005 remains open: the public controlled sampled-validation
  entry point fixes `ExecutionOptions` to its default instead of accepting the
  caller's explicit thread count and determinism choice.
- Required regressions must prove that thread count two returns typed
  `UnsupportedThreadCount` before any fitted-field evaluation and that one
  thread with determinism false succeeds while every progress event reports
  those exact options.
- No other P0-P3 finding was reported. The PR must remain Draft.

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
- Draft CI run 29787331468 passed Ubuntu correctness on exact independently
  re-reviewed head `aa6c134`. The Ready-only three-platform and benchmark-smoke
  matrix remains unexecuted.
- The evidence commit following `6bc6c1d` changes only this handoff and the
  independent review record and therefore does not invalidate the stable
  code/test-head standard gate.

## Next task

Open a fresh Repair task for only Draft PR #97 and THICK002-REV-005. Add the two
independent execution-options regressions, implement the smallest public API
repair without changing sampled-thickness mathematics, run focused checks and
the complete final standard gate after the last code change, update the review
evidence and bounded handoff, commit, push, and stop for another fresh
independent re-review. Do not begin REQ-PROJECT-001.

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
