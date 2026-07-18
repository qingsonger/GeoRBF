# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Implement / REQ-SOFT-001 complete; fresh independent Review required
- Requirement: REQ-SOFT-001, Issue #72 (open)
- Branch: `codex/req-soft-001-per-constraint-soft-losses`
- Draft implementation pull request: #73
- Registry state in this change: `implemented`
- Dependencies: REQ-IR-001 and REQ-SOLVE-001 are `integrated`
- No later requirement may start until REQ-SOFT-001 is reviewed, merged, and
  truthfully integrated in later fresh tasks

## Implementation result

- Added provenance-bearing canonical soft objectives for equality, linear
  bound/interval, and cone relations with independent positive scales and
  SquaredL2, AbsoluteL1, or Huber loss.
- Preserved every hard equality, bound, and cone in its existing feasibility
  collection while mixed soft objectives coexist in the same canonical
  problem and follow deterministic semantic insertion order.
- Defined the exact `rho(violation / scale)` contract, including equality,
  interval, and cone violation semantics and Huber's scaled-residual delta.
- Added objective capability/scaling metadata and checked memory estimates;
  level priors now compile to the shared soft-equality objective form while
  retaining stable level identity.
- Added five independent soft-loss tests, focused problem-IR/level regressions,
  and a 96-constraint D=1/D=2/D=3 mixed-objective compilation benchmark.
- Updated mathematical semantics, ADR-0003 consequences, architecture,
  Rustdoc, registry evidence, and the requirement change fragment.

## Validation state

- Focused soft-loss, problem-IR, and level tests passed.
- Focused mixed-objective benchmark smoke passed across D=1, D=2, and D=3.
- The complete standard workspace gate passed on the stable final
  implementation tree: format, warning-denying workspace Clippy, all-feature
  workspace tests, workspace Rustdoc, all 58 requirement checks, and
  `git diff --check`.
- No Ready CI, independent review, merge, integration, tag, or release is
  claimed.

## Next task

Open a fresh read-only Review task for the Draft implementation PR. Supply only
the REQ-SOFT-001 show/deps summary, Issue #72 acceptance criteria,
`docs/math/CONSTRAINT_SEMANTICS.md`, ADR-0003, the relevant architecture
contract, the complete PR diff, and recorded test/benchmark evidence to the
project `math_reviewer`. Review loss formulae, signs, dimensions and scale
units, equality/bound/cone violation definitions, hard-family preservation,
level-prior composition, provenance, deterministic ordering, memory estimates,
allocation paths, hidden optimization/regularization, interfaces, benchmark
scope, and registry truth. Record findings without repairing production code
or beginning another requirement in that task.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #72
- Draft implementation pull request: GitHub PR #73
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
