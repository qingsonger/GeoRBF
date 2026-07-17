# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review / REQ-LEVEL-001 findings recorded; fresh Repair required
- Requirement: REQ-LEVEL-001, Issue #69 (open)
- Branch: `codex/req-level-001-explicit-level-variables`
- Draft implementation pull request: #70
- Registry state in this change: `implemented`
- Dependencies: REQ-IR-001 and REQ-MODEL-001 are `integrated`
- Independently reviewed implementation head: `7d8d283`
- Review result: five P1, three P2, and one P3 finding; no P0 finding
- Next eligible requirement remains blocked until REQ-LEVEL-001 is repaired,
  freshly re-reviewed, integrated, and selected by a later fresh task

## Review result

- P1 R70-001: unrestricted observation functionals violate membership units and
  invalidate the unconditional membership gauge edge.
- P1 R70-002 and R70-003: extreme finite endpoint subtraction can hide a hard
  conflict, while accumulated-path overflow can reject a feasible hard system.
- P1 R70-008: provenance-bearing `PartialEq` misses mathematically identical
  fixed-membership conflicts from independent sources.
- P1 R70-009: a positive gap or distinct anchor on a membershipless level does
  not force nonzero scalar-field contrast.
- P2 R70-004, R70-005, and R70-006: cycle sources overreport downstream edges,
  fixed-membership conflicts omit definition sources, and missing-contrast
  diagnostics may cite unrelated levels.
- P3 R70-007: deterministic-DAG, exact-source, and independent-functional test
  evidence is overstated in the registry and change fragment.
- Canonical signs, constants, indices, prior separation, hard-row preservation,
  dimension bounds, and interface dispositions were independently confirmed.

## Validation state

- A fresh read-only project `math_reviewer` received only bounded requirement,
  dependency, normative-document, complete-diff, and validation evidence.
- Exact reviewed head `7d8d283` passed Draft Ubuntu correctness CI run
  29561377945. Ready-only three-platform and benchmark-smoke CI did not run.
- The Review task independently repeated the focused level suite (10 passed),
  core Rustdoc, and the complete PR whitespace check.
- The implementation tree retains its recorded complete local standard gate.
  This Review task changes only review and bounded-handoff documentation; the
  requirement registry check and `git diff --check` passed on that evidence
  tree.

## Next task

Open a fresh Repair task for Draft PR #70. Address only R70-001 through R70-009
from `docs/reviews/PR-70-INDEPENDENT-REVIEW.md`, add every specified independent
regression, run focused checks during development, and run the complete standard
gate after the final production or test change. Update the review evidence and
bounded handoff, commit, push, and stop for a fresh independent re-review. Do
not mark the PR ready, merge, integrate REQ-LEVEL-001, or start another
requirement in the Repair task.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #69
- Draft implementation pull request: GitHub PR #70
- Independent review: `docs/reviews/PR-70-INDEPENDENT-REVIEW.md`
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
