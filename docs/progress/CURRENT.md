# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair complete; fresh independent re-review required
- Requirement: REQ-SOFT-001, Issue #72 (open)
- Branch: `codex/req-soft-001-per-constraint-soft-losses`
- Draft implementation pull request: #73
- Original reviewed head: `978e400b2f9b25b9f84ac3102ff40388c44b42d8`
- Exact repair head: `530f6fd817dabcae70a304e3db2430211692615f`
- Registry state remains `implemented`
- Dependencies: REQ-IR-001 and REQ-SOLVE-001 are `integrated`
- No later requirement may start until REQ-SOFT-001 is independently
  re-reviewed, merged, and truthfully integrated in later fresh tasks

## Repair result

- R73-001 is implemented on the exact repair head but is not yet independently
  closed.
- `CanonicalCapabilities` now includes relation geometry retained by soft
  objectives as well as geometry in the hard equality, linear-bound, and cone
  collections.
- A new regression compiles isolated soft-only equality, linear-bound, and cone
  problems, proves the corresponding hard collections remain empty, and
  requires exactly the matching public capability flag.
- Public Rustdoc and the requirement change fragment now define the geometry
  flags consistently. Loss capability behavior and every hard relation remain
  unchanged.
- No backend, optimizer, dependency, regularization, hard-to-soft conversion,
  interface expansion, or unrelated requirement work was introduced.

## Validation state

- The new soft-only capability regression failed before the repair at the
  equality assertion and passed after the repair.
- Focused validation passed all 6 soft-loss tests, 11 problem-IR tests, 21 level
  tests, all 29 georbf Rustdoc tests, and the D=1/D=2/D=3 96-constraint
  soft-objective compilation benchmark smoke.
- The complete stable-head standard workspace gate passed: format,
  warning-denying all-target/all-feature Clippy, all-feature workspace tests,
  workspace Rustdoc, all 58 requirement checks, and `git diff --check`.
- Draft Ubuntu correctness CI passed on the original reviewed head. CI has not
  yet run on the repair head; Ready-only Windows, Ubuntu, macOS, and
  benchmark-smoke CI remains intentionally unexecuted.
- No re-review, Ready transition, merge, integration, tag, or release is
  claimed.

## Next task

Open a fresh Review task limited to an independent re-review of R73-001 on
exact repair head `530f6fd817dabcae70a304e3db2430211692615f`. Give a new read-only project
`math_reviewer` only the bounded REQ-SOFT-001 summary and integrated dependency
closure, Issue #72 criteria and exclusions, the M4 plan, scoped normative
documents, the complete PR and repair diffs, the original finding, and recorded
validation evidence. Record whether R73-001 is closed and whether any P0-P3
finding remains, update this review record and bounded handoff, commit, push,
and stop. Do not mark the PR Ready, merge, integrate, or begin another
requirement in that re-review task.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #72
- Draft implementation pull request: GitHub PR #73
- Independent review and repair evidence:
  `docs/reviews/PR-73-INDEPENDENT-REVIEW.md`
- Requirement summary: `changes/REQ-SOFT-001.md`
- Mathematical semantics: `docs/math/CONSTRAINT_SEMANTICS.md`
- Accepted level-prior design: `docs/adr/ADR-0003-explicit-level-variables.md`
- Architecture: `docs/architecture/ARCHITECTURE.md`
- Focused tests: `crates/georbf/tests/soft_losses.rs`,
  `crates/georbf/tests/problem_ir.rs`, and `crates/georbf/tests/levels.rs`
- Benchmark: `crates/georbf/benches/soft_objective_compilation.rs`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable.
