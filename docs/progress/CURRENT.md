# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair complete; fresh independent re-review required for R76-001
- Requirement: REQ-LINEQ-001, Issue #75 (open)
- Branch: `codex/req-lineq-001-linear-bounds`
- Draft implementation pull request: #76
- Exact repaired implementation and complete local-gate head:
  `b1f15d547333e17b8c8462014046a7b93e5ece00`
- Registry state remains `implemented`
- Dependencies: REQ-IR-001 and REQ-LEVEL-001 are `integrated`
- No later requirement may start until REQ-LINEQ-001 is independently
  re-reviewed, passed through exact Ready CI, merged, and recorded as
  `integrated`

## Repair result

- R76-001 is repaired in production code and regression coverage, but has not
  yet received the required fresh independent re-review.
- Composition now compares stable observation identifiers across every hard
  equality, hard bound, hard cone, and soft objective in the level and field
  canonical inputs before appending any record.
- The new regression reuses level-definition observation ID 100 in an otherwise
  valid field hard bound. It failed before the production repair and now returns
  `ProblemIrError::DuplicateObservationId` with the duplicated identifier.
- No relation is changed, dropped, reordered, softened, scaled, regularized, or
  repaired automatically.

## Validation state

- On exact repaired head `b1f15d5`, all eight linear-constraint tests and all 21
  level tests passed.
- The complete standard gate passed on exact repaired head `b1f15d5`:
  formatting, warning-denying workspace/all-target/all-feature Clippy,
  all-feature workspace tests, workspace Rustdoc, all 58 requirement checks,
  and `git diff --check`.
- The following evidence-only commit changes only this bounded handoff and
  `docs/reviews/PR-76-INDEPENDENT-REVIEW.md`; no production, test, manifest,
  schema, CI, build, registry, API, numerical, or dependency input changed after
  the complete gate.
- No fresh re-review, Ready transition, three-platform/benchmark-smoke CI,
  merge, integration-state change, tag, or release is claimed.

## Next task

Open a fresh Review/re-review task limited to PR #76 and REQ-LINEQ-001. Create
and wait for the read-only project `math_reviewer`, providing only the bounded
requirement/dependency context, normative documents, PR diff, original finding,
repair, and validation evidence. The reviewer must independently confirm that
R76-001 is closed and check for new P0-P3 findings. If clean, synchronize review
evidence, mark PR #76 Ready, wait for complete Windows/Ubuntu/macOS and all
benchmark-smoke CI on that exact ready head, merge exactly once only when green,
and record truthful integration state in an isolated follow-up change. Then
stop; do not begin another requirement in the same task.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #75
- Draft implementation pull request: GitHub PR #76
- Independent review and repair evidence:
  `docs/reviews/PR-76-INDEPENDENT-REVIEW.md`
- Requirement summary: `changes/REQ-LINEQ-001.md`
- Mathematical semantics: `docs/math/CONSTRAINT_SEMANTICS.md`
- Architecture: `docs/architecture/PROBLEM_IR.md` and
  `docs/architecture/ARCHITECTURE.md`
- Focused tests: `crates/georbf/tests/linear_constraints.rs` plus retained
  layer-order coverage in `crates/georbf/tests/levels.rs`
- Example: `crates/georbf/examples/linear_constraints.rs`
- Benchmark and report:
  `crates/georbf/benches/linear_constraint_compilation.rs` and
  `docs/benchmarks/REQ-LINEQ-001.md`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable.
