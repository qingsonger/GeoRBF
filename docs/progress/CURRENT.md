# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Clean independent re-review complete; fresh integration Review required
- Requirement: REQ-SOFT-001, Issue #72 (open)
- Branch: `codex/req-soft-001-per-constraint-soft-losses`
- Draft implementation pull request: #73
- Original reviewed head: `978e400b2f9b25b9f84ac3102ff40388c44b42d8`
- Exact repair head: `530f6fd817dabcae70a304e3db2430211692615f`
- Cleanly re-reviewed repair head: `530f6fd817dabcae70a304e3db2430211692615f`
- Pre-re-review evidence head: `6687402e7ab42508637460ddbe3d7a156a45cac6`
- Registry state remains `implemented`
- Dependencies: REQ-IR-001 and REQ-SOLVE-001 are `integrated`
- No later requirement may start until REQ-SOFT-001 is independently
  integrated after Ready-only CI and merge in a later fresh task

## Re-review result

- A new read-only project `math_reviewer` independently reviewed exact repair
  head `530f6fd` without inheriting implementation or Repair reasoning and made
  no repository or remote change.
- R73-001 is closed. No P0, P1, P2, or P3 finding remains.
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
- The parent Review task independently passed all 6 soft-loss tests, 11
  problem-IR tests, 21 level tests, all 29 georbf Rustdoc tests, and the
  D=1/D=2/D=3 96-constraint soft-objective compilation benchmark smoke.
- The reviewer independently passed the focused repaired regression, compact
  requirement and dependency checks, and whitespace checks across the complete
  implementation, exact repair, and evidence-only diffs.
- The complete stable-head standard workspace gate passed: format,
  warning-denying all-target/all-feature Clippy, all-feature workspace tests,
  workspace Rustdoc, all 58 requirement checks, and `git diff --check`.
- That complete gate remains valid for immutable repair head `530f6fd`; the
  commits after it and this re-review change only review and bounded-handoff
  documentation, not production, test, manifest, schema, or build input.
- Draft Ubuntu correctness CI passed on pre-re-review evidence head `6687402`.
  CI has not yet run on the re-review evidence commit; Ready-only Windows,
  Ubuntu, macOS, and benchmark-smoke CI remains intentionally unexecuted.
- No Ready transition, merge, integration, tag, or release is claimed.

## Next task

Open a fresh integration Review task limited to PR #73. Confirm that the new PR
head differs from cleanly re-reviewed repair head `530f6fd` only by review and
handoff evidence, synchronize the PR evidence, and mark it Ready. Wait for the
complete Windows, Ubuntu, and macOS correctness matrix plus every benchmark
smoke workload on that exact Ready head. Merge exactly once only if all required
CI is green, then record truthful REQ-SOFT-001 integration state in a separate
integration-state change and bounded handoff. Stop without beginning another
requirement.

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
