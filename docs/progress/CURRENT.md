# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review / REQ-SPIKE-002 findings recorded
- Requirement: REQ-SPIKE-002, Issue #40
- Branch: `codex/req-spike-002-rrqr-svd-backend`
- Draft pull request: #41
- Reviewed implementation head: `9cd0c30`
- Review record: `docs/reviews/PR-41-INDEPENDENT-REVIEW.md`
- Registry state in this change: `documented`
- Dependency: REQ-BOOTSTRAP-001 is `integrated`
- Production dependency state: unchanged; the comparison crate is excluded
  from the production workspace

## Independent review result

- No P0, P1, or P3 finding was identified. Two P2 findings block ready status.
- P2-1: the `f64::EPSILON / 4.0` unresolved near-threshold perturbation rounds
  away when added to `2.0`, so that case duplicates exact rank deficiency and
  does not test either side of the adopted SVD threshold.
- P2-2: `--no-default-features` leaves `Backend::ALL` empty, so all six tests
  pass vacuously and the smoke command prints only a header before succeeding.
- PR #41 remains Draft. No production or spike implementation was repaired in
  this Review task.

## Validation state

- A fresh read-only independent `math_reviewer` confirmed both P2 findings.
- All-feature, faer-only, and nalgebra-only focused tests pass all six current
  cases; warning-denying all-target/all-feature spike Clippy passes.
- The zero-backend false success was reproduced with both `cargo test` and the
  release smoke command using `--no-default-features`.
- Draft Ubuntu CI run 29343523143 passed on exact head `9cd0c30`.
- The implementation head is unchanged, so its recorded complete local
  standard gate remains valid. The ready-head three-platform and benchmark-
  smoke gate has not run and must wait for repair and clean re-review.

## Next task

Open a fresh Repair task for only PR #41 findings P2-1 and P2-2. Add independent
threshold-boundary truth regressions and reject the zero-backend configuration,
then run focused checks and one final standard gate on the repaired stable
head. Update review evidence and this bounded handoff, commit, push, and stop
for a fresh independent re-review. Do not implement REQ-CPD-001.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #40
- Decision: `docs/adr/ADR-0009-nalgebra-rank-review-backend.md`
- Independent review: `docs/reviews/PR-41-INDEPENDENT-REVIEW.md`
- Change summary: `changes/REQ-SPIKE-002.md`
- Benchmark and size baseline: `docs/benchmarks/REQ-SPIKE-002.md`
- Reproducible harness: `spikes/rank-backends/`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, allocation instrumentation, and API/ABI/schema
snapshot checks are tracked by later requirements and release gates. Local
`actionlint` is unavailable.
