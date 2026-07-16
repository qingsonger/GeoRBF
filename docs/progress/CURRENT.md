# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair complete / fresh REQ-FIELD-001 re-review required
- Requirement: REQ-FIELD-001, Issue #54
- Branch: `codex/req-field-001-hard-equality-assembly`
- Draft pull request: #55
- Review record: `docs/reviews/PR-55-INDEPENDENT-REVIEW.md`
- Registry state: `documented` (not `integrated`)
- Dependencies: complete closure through REQ-ANISO-001, REQ-CPD-001, and
  REQ-IR-001 is `integrated`

## Implemented scope

- Added public `FieldProblem<D>` for D=1/D=2/D=3 with distinct observation and
  center collections, exact all-representer alignment, and hard-equality-only
  validation.
- Added upper-triangle mixed value/directional-derivative kernel assembly through
  an explicit evaluator callback and metadata capability checks.
- Added canonical center-weight/polynomial variable blocks and immutable
  GeoRBF-owned symmetric dense matrix, right-hand side, and diagnostics.
- Added automatic complete polynomial augmentation, CPD RRQR/SVD rank review,
  verified null space, side-condition rows, and projected `Z^T K Z` evidence.
- Added independent analytic/property/error tests, rustdoc, runnable example,
  normative math/architecture updates, changelog fragment, deterministic
  benchmark, and three-platform CI smoke routing.
- Repair head `b8c1367` removes allocation from kernel-error mapping, reuses
  observation polynomial scratch with constant-allocation evidence, completes
  invalid-capability coverage in D=1/D=2/D=3, independently verifies the full
  CPD augmented fixture and `Z^T K Z=4/3`, and precisely diagnoses variable-
  block collection reservation failure.
- CLI, C, C++, and Python are N/A until solver, model, schema, and binding
  requirements establish a stable fitting surface.

## Validation state

- Focused field tests: 5 public integration tests and 3 private
  allocation/error-path regressions passed.
- Focused all-target warning-denying Clippy: passed.
- Runnable example and D=1/D=2/D=3 benchmark smoke: passed.
- Four local 100-iteration benchmark runs retained dimension-specific bit-stable
  checksums; timing ranges are recorded in
  `docs/benchmarks/REQ-FIELD-001.md`.
- Exact implementation head `6687631` passed one complete stable-code-head
  gate: format, warning-denying workspace
  Clippy, all-feature workspace tests, workspace doctests, the 58-requirement
  registry check, and `git diff --check`.
- A fresh read-only `math_reviewer` reviewed exact head `ab28b3c` and found no
  P0 or P1 issue, four P2 findings, and one P3 finding. It independently passed
  the five field tests, runnable example, D=1/D=2/D=3 benchmark smoke, and all
  58 requirement-registry checks. Draft Ubuntu CI run 29422460418 passed on the
  same head.
- Exact repair code/test head `b8c1367` passed the complete stable-head gate:
  format, warning-denying workspace Clippy, all-feature workspace tests,
  workspace doctests, all 58 requirement checks, and `git diff --check`.
  The runnable example and D=1/D=2/D=3 optimized benchmark smoke also passed
  with the previously recorded bit-stable checksums.

## Next task

Open a fresh Review task for Draft PR #55. Supply a new read-only
`math_reviewer` only the bounded requirement/dependency summaries, Issue #54,
the scoped normative documents, complete PR diff, and validation evidence.
Independently confirm whether P2-1 through P2-4 and P3-1 are closed and check
for new P0-P3 findings without inheriting this Repair reasoning. If findings
remain, record them and stop. If the re-review is clean, follow the mandatory
sequence on the exact reviewed head: synchronize evidence, mark the PR ready,
wait for the complete Windows/Ubuntu/macOS and benchmark-smoke CI, merge only
when all required jobs are green, and truthfully record integration state in an
isolated change. Do not begin REQ-SOLVE-001.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #54
- Requirement summary: `changes/REQ-FIELD-001.md`
- Mathematical contract: `docs/math/MATH_SPEC.md`
- Architecture contract: `docs/architecture/ARCHITECTURE.md`
- Benchmark: `docs/benchmarks/REQ-FIELD-001.md`
- Independent review: `docs/reviews/PR-55-INDEPENDENT-REVIEW.md`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable.
