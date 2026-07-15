# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Implement / REQ-IR-001 complete
- Requirement: REQ-IR-001, Issue #51
- Branch: `codex/req-ir-001-semantic-canonical-ir`
- Draft pull request: pending creation after the implementation commit is pushed
- Registry state before Draft PR creation: `in_progress` (not `integrated`)
- Dependencies: complete closure through REQ-FUNC-001 is `integrated`

## Implemented scope

- Added immutable `SemanticProblemIr<D>` for D=1/D=2/D=3 with full source,
  units, field-path, group, compiled-functional, relation, enforcement/loss,
  and execution provenance.
- Added validated sparse affine expressions and named canonical variable blocks.
- Added deterministic hard equality, linear-bound, interval, and SOC mapping
  through an explicit caller linearizer; basis/kernel assembly remains outside
  this IR requirement.
- Added solver-neutral canonical rows/cones with complete provenance, explicit
  identity scaling, required solver capabilities, and checked numeric memory
  estimates.
- Soft loss metadata is retained, but objective/epigraph compilation returns a
  source-aware unsupported-path error until later approved requirements.
- Added independent tests, compile-fail rustdoc, runnable example, architecture
  docs, changelog fragment, deterministic benchmark, and three-platform CI smoke
  routing.

## Validation state

- Focused problem-IR tests: 11 passed.
- Focused warning-denying Clippy: passed for the library, test, example, and
  benchmark targets after the final focused changes.
- Runnable example and unsupported-dimension doctest: passed.
- Benchmark smoke passed; the checked estimate is dimension-specific because
  the explicit variable-scaling vector has D entries.
- Four local 1,000-iteration baselines retained checksums `5384000`, `5392000`,
  and `5400000` for D=1/D=2/D=3; timing ranges are recorded in
  `docs/benchmarks/REQ-IR-001.md`.
- One complete final stable-head gate passed: format, warning-denying workspace
  Clippy, all-feature workspace tests, workspace doctests, the 58-requirement
  registry check, and `git diff --check`.

## Next task

Open a fresh independent Review task for the Draft PR created from this branch.
Review only REQ-IR-001 against Issue #51, its dependency closure, normative IR
and constraint documents, the complete PR diff, and validation evidence. Use a
fresh read-only project `math_reviewer`; do not repair production code or begin
REQ-FIELD-001 in that Review task.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #51
- Requirement summary: `changes/REQ-IR-001.md`
- Architecture contract: `docs/architecture/PROBLEM_IR.md`
- Benchmark: `docs/benchmarks/REQ-IR-001.md`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, allocation instrumentation, and API/ABI/schema
snapshot checks are tracked by later requirements and release gates. Local
`actionlint` is unavailable.
