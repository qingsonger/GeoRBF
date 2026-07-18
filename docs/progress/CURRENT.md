# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Independent Review complete; fresh integration Review required
- Requirement: REQ-LEVEL-001, Issue #69 (open)
- Branch: `codex/req-level-001-explicit-level-variables`
- Draft implementation pull request: #70
- Registry state in this change: `implemented`
- Dependencies: REQ-IR-001 and REQ-MODEL-001 are `integrated`
- Exact independently reviewed local and remote head: `6a03fe6`
- R70-001 through R70-014 are independently closed; no P0-P3 finding remains.
- PR #70 remains Draft; Ready CI, merge, and integration have not occurred.
- The next eligible requirement remains blocked until REQ-LEVEL-001 is
  integrated in a later fresh task.

## Review result

- The project `math_reviewer` independently confirmed that exact-zero fixed
  availability rejects every positive required gap before exponent comparison.
- Nonzero fixed-gap review uses a dimensionless relative comparison scaled only
  from the actual compared gaps, without a dimensioned `1.0` floor; positive
  scalar-unit rescaling preserves the verdict.
- The `1e-20` and `1e20`-rescaled regression retains the exact lower-definition,
  order-edge, and upper-definition evidence sequence.
- The reviewer reconfirmed all earlier membership, equality-closure, order,
  cycle, gauge, contrast, provenance, canonical-row, and interface contracts.
- No hard row is changed, dropped, softened, regularized, or repaired.

## Validation state

- The independent reviewer passed 21 level tests, 6 diagnostics tests, all 29
  core Rustdoc tests, the 64-level benchmark smoke, complete PR and repair
  whitespace checks, and an exact-rational scaled-arithmetic probe covering
  representations, ordering pairs, feasible accumulated paths, and unit
  rescaling boundaries.
- The parent Review task independently passed the same focused Rust tests and
  Rustdoc, the benchmark smoke at approximately 192 microseconds per validation
  and compile iteration, all 58 requirement checks, and the complete PR
  whitespace check.
- Exact implementation tree `61fa6d3` passed the complete standard workspace
  gate. The reviewed head adds only review evidence and this bounded handoff;
  exact-head Draft Ubuntu CI passed.
- Ready-only Windows, Ubuntu, macOS, and benchmark-smoke CI has not run. No
  merge, integration, tag, or release is claimed.

## Next task

Open a fresh integration Review task for PR #70. Perform mandatory preflight
and verify that the PR head still matches the clean reviewed head. If unchanged,
synchronize the clean review evidence, mark the PR ready, and wait for the
complete Windows, Ubuntu, macOS, and benchmark-smoke CI triggered on that exact
ready head. Merge exactly once only when every required check is green, then
record the truthful integration state in an isolated change. Stop after
integration; do not begin the next requirement in that task.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #69
- Draft implementation pull request: GitHub PR #70
- Independent review and repair evidence:
  `docs/reviews/PR-70-INDEPENDENT-REVIEW.md`
- Requirement summary: `changes/REQ-LEVEL-001.md`
- Mathematical semantics: `docs/math/CONSTRAINT_SEMANTICS.md`
- Accepted design: `docs/adr/ADR-0003-explicit-level-variables.md`
- Architecture: `docs/architecture/ARCHITECTURE.md`
- Focused tests: `crates/georbf/tests/levels.rs`
- Benchmark: `crates/georbf/benches/level_compilation.rs`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable.
