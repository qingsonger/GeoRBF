# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review / REQ-FIELD-001 findings recorded
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
- CLI, C, C++, and Python are N/A until solver, model, schema, and binding
  requirements establish a stable fitting surface.

## Validation state

- Focused field tests: 5 passed.
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

## Next task

Open a fresh Repair task for Draft PR #55. Address only P2-1 through P2-4 and
P3-1 in `docs/reviews/PR-55-INDEPENDENT-REVIEW.md`: remove the infallible
kernel-error allocation, reuse CPD observation polynomial scratch, complete
D=1/D=2/D=3 invalid-capability coverage, add independent CPD augmented-matrix
truth, and correct the variable-block allocation diagnostic. Add the specified
regressions, run focused checks and the final stable-head standard gate, update
review evidence and this bounded handoff, commit, push, and stop for a fresh
independent re-review. Do not begin REQ-SOLVE-001.

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
fuzzing, mutation testing, allocation instrumentation, and API/ABI/schema
snapshot checks are tracked by later requirements and release gates. Local
`actionlint` is unavailable.
