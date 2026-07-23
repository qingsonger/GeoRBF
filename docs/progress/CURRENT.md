# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review / REQ-CENTER-001 complete with findings
- Requirement: REQ-CENTER-001, open Issue #120
- Branch: `codex/req-center-001-rank-safe-centers`
- Pull request: #121 (Draft; one P1, one P2, and two P3 findings)
- Dependencies: REQ-SOLVE-001 and REQ-MODEL-001 are integrated
- Registry status: `planned`

## Independent findings

- P1 CENTER001-REV-001: the global maximum-diagonal greedy pivot threshold
  rejects a full-rank SPD basis after an equivalent nonzero basis scaling,
  before the scale-aware final rank review can run.
- P2 CENTER001-REV-002: the generic public center-selection contract neither
  implements the CPD `Q`/null-space/projected-positive path nor explicitly
  classifies and rejects CPD input at a typed capability boundary.
- P3 CENTER001-REV-003: farthest-point exact seeded ties and repeated results
  lack the required regression.
- P3 CENTER001-REV-004: Gram/target length and nonfinite-input validations are
  untested, so the malformed-shape evidence is overstated.
- Exact evidence and required regressions:
  `docs/reviews/PR-121-INDEPENDENT-REVIEW.md`.

## Review validation

- An isolated read-only project `math_reviewer` reviewed base
  `aa128ed87236c85aa6d310127ad05c70c0a2092b` through head
  `63a9f9035ec280124ea0fc230692b3c271436f59`.
- Both reviewer and parent passed the nine focused integration tests, the
  five-strategy release benchmark smoke, and the complete PR whitespace check.
- The parent also passed the rustdoc example and 58-requirement registry check.
- Draft CI run 30004560859 passed Ubuntu on the exact reviewed head. Ready-only
  three-platform and benchmark-smoke CI was skipped and is not claimed.
- Stable implementation gate head
  `bf850a8f9a4b673425724e71abc46d955258cd6e` remains the last complete local
  standard gate; this Review changes documentation evidence only.

## Next task boundary

A fresh Repair task must address only CENTER001-REV-001 through
CENTER001-REV-004, add the specified independent regressions, run focused
checks during iteration and one complete standard gate after the last
production or test change, update the review evidence and bounded handoff,
push, and stop for another fresh independent re-review. Do not mark PR #121
ready, merge it, or begin REQ-TUNE-001.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #120
- Draft implementation: GitHub PR #121
- Independent review: `docs/reviews/PR-121-INDEPENDENT-REVIEW.md`
- Requirement summary: `changes/REQ-CENTER-001.md`
- Architecture: `docs/architecture/ARCHITECTURE.md`
- Numerical policy: `docs/architecture/SOLVER_POLICY.md`
- Benchmark: `docs/benchmarks/REQ-CENTER-001.md`
- Production implementation: `crates/georbf/src/center_selection.rs`
- Independent tests: `crates/georbf/tests/center_selection.rs`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, and API/ABI/schema snapshot checks are tracked by
later requirements and release gates. Local `actionlint` is unavailable. No
unavailable check is claimed as passed.
