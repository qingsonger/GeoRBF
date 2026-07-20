# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair complete; fresh independent re-review required
- Requirement: REQ-TANGENT-001, Issue #90
- Branch: `codex/req-tangent-001-tangent-constraints`
- Pull request: #91 (Draft)
- Reviewed implementation head: `86d1d3dcc948d70f6825822d1efe94b92b8b4f5b`
- Repair implementation head: `5e99aa629118ca4b4c81927d31adf67f19822b58`
- Review result: P2 R91-001 repaired; independent closure not yet confirmed
- Registry state: `implemented`, not `integrated`
- Dependencies: REQ-FUNC-001, REQ-SOFT-001, and REQ-DIAG-001 are `integrated`

## Repair result

- Repair head `5e99aa6` checks the already-known absence of a gauge before
  inspecting iterator size, reserving storage, or collecting observations. It
  reads only the first item to distinguish an empty problem from a source-aware
  `GEORBF-E4001`; no-gauge infinite iterators now terminate immediately.
- Two independent regressions use `std::iter::repeat(valid_tangent)`: without a
  gauge it returns `MissingGauge`, `GEORBF-E4001`, and the first tangent ID;
  with an explicit gauge it retains `CountOverflow`.
- Empty-input behavior and the explicit-gauge collection path remain unchanged.
  No formula, sign, units, hard/soft semantics, canonical relation, adapter,
  dependency, registry status, or later-requirement scope changed.

## Validation state

- Repair implementation head `5e99aa6` passed all eight focused tangent
  integration tests, both tangent module regressions, the runnable example,
  and benchmark smoke checksum `3824`.
- The same immutable repair head passed the complete standard gate: workspace
  format, warning-denying all-target/all-feature Clippy, all-feature workspace
  tests, workspace doctests, all 58 requirement checks, and `git diff --check`.
- The evidence update after `5e99aa6` changes only the requirement change
  fragment, independent review record, and this bounded handoff. It changes no
  production code, test, manifest, schema, CI, registry, dependency, API, or
  numerical behavior. Draft Ready-only matrix jobs remain intentionally absent.

## Next task boundary

Open a fresh independent re-review task for PR #91. Review exact PR head and
confirm whether R91-001 is closed without new P0-P3 findings. If clean, mark the
PR ready, wait for the complete Windows/Ubuntu/macOS and benchmark-smoke CI on
that exact ready head, merge only when green, and record integration state.
Do not begin REQ-THICK-001 or any other requirement.

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
