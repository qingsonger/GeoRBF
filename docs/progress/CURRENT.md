# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review complete; Repair required for R73-001
- Requirement: REQ-SOFT-001, Issue #72 (open)
- Branch: `codex/req-soft-001-per-constraint-soft-losses`
- Draft implementation pull request: #73
- Reviewed head: `978e400b2f9b25b9f84ac3102ff40388c44b42d8`
- Registry state remains `implemented`
- Dependencies: REQ-IR-001 and REQ-SOLVE-001 are `integrated`
- No later requirement may start until REQ-SOFT-001 is repaired, independently
  re-reviewed, merged, and truthfully integrated in later fresh tasks

## Independent review result

- The fresh read-only project `math_reviewer` found no P0, P1, or P3 issues
  and one P2 issue, recorded as R73-001 in
  `docs/reviews/PR-73-INDEPENDENT-REVIEW.md`.
- R73-001: `CanonicalCapabilities` derives equality, linear-bound, and cone
  geometry flags only from hard-family collections. A soft-only cone therefore
  reports that second-order-cone support is unnecessary even though exact
  lowering requires that geometry.
- Loss formulae, scale units, relation violations, hard-family preservation,
  level-prior composition, provenance, ordering, memory/allocation paths,
  hidden-regularization exclusions, interfaces, benchmark scope, and registry
  truth were otherwise consistent with the requirement.

## Validation state

- The reviewed implementation head retained its complete green local standard
  gate and focused test/benchmark evidence.
- Draft Ubuntu correctness CI passed on the exact reviewed head.
- Ready-only Windows, Ubuntu, macOS, and benchmark-smoke CI has not run.
- No repair, re-review, Ready transition, merge, integration, tag, or release
  is claimed.

## Next task

Open a fresh Repair task limited to R73-001. Reproduce the defect with a
soft-only second-order-cone capability regression and add soft-only equality
and linear-bound cases that define the intended public flag semantics. Apply
the smallest complete capability-metadata repair, run focused checks, then run
the complete standard workspace gate once on the stable repaired head. Update
the review evidence and bounded handoff, commit, push, and stop for a fresh
independent re-review. Do not begin another requirement.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #72
- Draft implementation pull request: GitHub PR #73
- Independent review: `docs/reviews/PR-73-INDEPENDENT-REVIEW.md`
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
