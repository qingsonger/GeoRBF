# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Integration state / REQ-POLY-001 complete
- Requirement: REQ-POLY-001, Issue #34 (closed)
- Implementation pull request: #35, squash-merged as `18b3e66`
- Integration-state branch: `codex/req-poly-001-integration-state`
- Integration-state pull request: pending
- Review record: `docs/reviews/PR-35-INDEPENDENT-REVIEW.md`
- Registry state in this integration change: `integrated`
- Next eligible requirement: REQ-FUNC-001 (`planned`)

## Integration result

- A fresh independent mathematical and numerical re-review found no P0, P1,
  P2, or P3 issue. P3-1 is closed by the joint-output atomicity regression.
- Exact ready head `4f219fd` passed the complete Windows, Ubuntu, and macOS CI
  matrix with every benchmark smoke workload in run 29330884780.
- PR #35 squash-merged exactly once as `18b3e66`; Issue #34 closed.
- Post-merge `main` run 29330997791 passed the same complete three-platform
  correctness, benchmark-smoke, and requirement-registry gate.
- This isolated integration-state change contains no production, test,
  manifest, schema, build-input, API, or numerical-behavior change.

## Next task

Open a fresh Implement task for only REQ-FUNC-001. Perform the mandatory
preflight, create or read its Issue with explicit acceptance criteria, confirm
its dependency closure remains integrated, and read only its listed normative
documents and relevant ADRs before implementation. Do not continue directly
from this integration task.

## Durable evidence

- Requirement summary: `changes/REQ-POLY-001.md`
- Independent review and repair closure:
  `docs/reviews/PR-35-INDEPENDENT-REVIEW.md`
- Deterministic performance baseline: `docs/benchmarks/REQ-POLY-001.md`
- Exact CI and merge history: GitHub PR #35 and runs 29330884780 and
  29330997791

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, allocation instrumentation, and API/ABI/schema
snapshot checks are tracked by later requirements and release gates. Local
`actionlint` is unavailable.
