# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review complete; Repair required for R76-001
- Requirement: REQ-LINEQ-001, Issue #75 (open)
- Branch: `codex/req-lineq-001-linear-bounds`
- Draft implementation pull request: #76
- Reviewed head: `0da5084c3b4f7f909299069c3c8dcf3145d1f282`
- Registry state remains `implemented`
- Dependencies: REQ-IR-001 and REQ-LEVEL-001 are `integrated`
- No later requirement may start until REQ-LINEQ-001 is repaired,
  independently re-reviewed, passed through exact Ready CI, merged, and
  recorded as `integrated`

## Independent review result

- The fresh read-only project `math_reviewer` found no P0, P1, or P3 issue and
  one P2 issue, recorded as R76-001 in
  `docs/reviews/PR-76-INDEPENDENT-REVIEW.md`.
- R76-001: field and level inputs each enforce stable observation-ID uniqueness
  internally, but canonical composition does not recheck IDs across the two
  inputs. It can therefore return one canonical problem with ambiguous duplicate
  provenance identities.
- Formula signs and units, closed region boundaries, scalar and level gap
  orientation, monotonicity direction and functional shape, exact row reversal,
  hard infeasibility evidence, hard/soft separation, D=1/D=2/D=3 behavior,
  allocation paths, interfaces, benchmark, registry, and documentation were
  otherwise consistent with the requirement.

## Validation state

- Exact implementation head `8931260` retains its complete green local standard
  gate; `8931260..0da5084` changes only this bounded handoff.
- Exact reviewed head `0da5084` passed Draft Ubuntu correctness CI.
- The independent reviewer repeated all seven focused tests, the complete
  Rustdoc suite, example, benchmark smoke, all 58 requirement checks, and
  whitespace checks; all passed.
- The parent Review task passed all seven linear-constraint tests, all 21 level
  tests, the example, benchmark smoke, all georbf Rustdoc, all 58 requirement
  checks, and `git diff --check`.
- No repair, re-review, Ready transition, merge, integration, tag, or release
  is claimed.

## Next task

Open a fresh Repair task limited to R76-001. Add a regression in which one field
bound reuses the observation ID of a level record and require structured
rejection during composition. Implement the smallest complete cross-problem
provenance-identity check, run focused checks, then the complete standard gate
once on the stable repaired head. Update review evidence and this bounded
handoff, commit, push, and stop for a fresh independent re-review. Do not begin
another requirement.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #75
- Draft implementation pull request: GitHub PR #76
- Independent review: `docs/reviews/PR-76-INDEPENDENT-REVIEW.md`
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
