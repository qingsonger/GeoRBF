# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair / REQ-SPIKE-002 findings repaired; fresh re-review required
- Requirement: REQ-SPIKE-002, Issue #40
- Branch: `codex/req-spike-002-rrqr-svd-backend`
- Draft pull request: #41
- Reviewed implementation head: `9cd0c30`
- Repair code/test head: `30bd495`
- Review record: `docs/reviews/PR-41-INDEPENDENT-REVIEW.md`
- Registry state in this change: `documented`
- Dependency: REQ-BOOTSTRAP-001 is `integrated`
- Production dependency state: unchanged; the comparison crate is excluded
  from the production workspace

## Repair result

- P2-1 is repaired with distinct already-equilibrated matrices 12 and 15 ULPs
  above `0.5`, bracketing the analytic SVD threshold within 12%. Independent
  closed-form singular values establish expected ranks two and three; both
  candidate backends match and retain explicit threshold-adjacency evidence.
- P2-2 is repaired with a compile-time zero-backend rejection. CI now checks
  the exact negative diagnostic and all-feature, faer-only, and nalgebra-only
  positive paths.
- No production dependency or solver changed. PR #41 remains Draft, and the
  repairs have not yet received the required fresh independent re-review.

## Validation state

- On stable repair head `30bd495`, spike formatting, warning-denying Clippy,
  all three 6/6 feature-test configurations, the expected zero-backend compile
  failure, and the release smoke workload passed.
- The same stable head passed workspace formatting, warning-denying Clippy,
  all 139 tests, all 25 doctests and compile-fail tests, all 58 requirement
  checks, and `git diff --check`.
- Draft Ubuntu CI run 29373204550 passed on the pre-repair review-evidence head
  `e1db349`; repaired-head Draft CI awaits the push/synchronize event.
- The ready-head Windows/Ubuntu/macOS and benchmark-smoke gate has not run and
  must wait for a clean fresh re-review.

## Next task

Open a fresh re-review task for only PR #41. Supply the requirement and
dependency summaries, normative documents, complete PR diff, original P2
findings, repair head `30bd495`, and validation evidence to a fresh read-only
`math_reviewer`. Independently confirm both repairs and inspect for new P0-P3
findings. If clean, follow the mandatory ready -> exact-head three-platform and
benchmark-smoke CI -> single merge -> integration-state sequence. Do not repair
code in the Review task and do not implement REQ-CPD-001.

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
