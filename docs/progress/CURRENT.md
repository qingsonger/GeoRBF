# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Integration state / REQ-SPIKE-003 complete
- Requirement: REQ-SPIKE-003, closed Issue #114
- Implementation pull request: #115, squash-merged as `97fabfa`
- Integration-state branch: `codex/req-spike-003-integration-state`
- Integration-state pull request: #116 (Draft until exact Ready CI is green)
- Repair implementation and stable full-gate head: `83ba364`
- Exact Ready head: `6052712`
- Closed findings: P1 SPIKE003-REV-001, P2 SPIKE003-REV-002, and
  P2 SPIKE003-REV-003
- Review state: the complete repaired PR has no remaining P0-P3 finding
- Dependencies: REQ-KERNEL-004 and its complete closure are integrated
- Registry state in this change: `integrated`

## Integration result

- The isolated `math_reviewer` independently closed all three findings and
  found no new P0-P3 issue in the complete repaired PR.
- Exact comparison with the unique brute-force pair oracle, pair sorting and
  deduplication, and sorted-unique CSC checks satisfy neighborhood and storage
  truth. The hand-derived three-point fixture independently verifies both
  candidate CSC arrays, storage products, symmetry, and recovered solution.
- Index rows truthfully cover construct/query/filter/canonicalize/checksum
  end-to-end work. Solver rows truthfully cover
  construct/factor/solve/review/checksum end-to-end work. No query-only or
  isolated-factorization performance claim remains.
- Exact Ready head `6052712` passed complete Windows, Ubuntu, and macOS CI run
  29984412613, including every configured backend combination, benchmark
  smoke, and requirement validation.
- PR #115 squash-merged exactly once as `97fabfa`; Issue #114 closed as
  completed. Post-merge `main` CI run 29985301861 passed the same complete
  three-platform gate on exact merge commit `97fabfa`.
- This isolated integration-state change updates only the registry, review
  evidence, completed-history index, and bounded handoff. It changes no
  production code, test, manifest, schema, CI, build input, API, numerical
  behavior, dependency, tag, or release.

## Validation state

- Exact Repair head `83ba364` passed sparse-harness formatting,
  warning-denying all-target/all-feature Clippy, all 10 combined-feature tests,
  all four minimal feature cross-products, both negative configurations, the
  optimized release smoke workload, the complete local standard gate, all 58
  requirement checks, and whitespace validation.
- The isolated re-review independently passed all 10 locked all-feature tests,
  the optimized locked smoke workload, exact dependency review, compact
  requirement/dependency checks, and whitespace validation.
- Exact Ready-head run 29984412613 and post-merge `main` run 29985301861 are
  both green on Windows, Ubuntu, and macOS, including every configured
  benchmark smoke.
- The isolated integration-state tree must pass the complete local standard
  gate and exact Ready-head CI before it merges.

## Next task boundary

Run the complete local standard gate on the final integration-state head. Mark
PR #116 Ready, wait for exact Ready-head Windows, Ubuntu, macOS, and
benchmark-smoke CI, merge only if green, and stop. Do not start REQ-SPARSE-001.

## Durable evidence

- Acceptance criteria and exclusions: closed GitHub Issue #114
- Merged implementation: GitHub PR #115
- Integration-state pull request: GitHub PR #116
- Requirement summary: `changes/REQ-SPIKE-003.md`
- Independent review: `docs/reviews/PR-115-INDEPENDENT-REVIEW.md`
- Reproducible harness: `spikes/sparse-backends/`
- Selection decision:
  `docs/adr/ADR-0012-rstar-faer-compact-sparse-backends.md`
- Scaling and size evidence: `docs/benchmarks/REQ-SPIKE-003.md`
- Numerical policy: `docs/architecture/SOLVER_POLICY.md`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, and API/ABI/schema snapshot checks are tracked by
later requirements and release gates. Local `actionlint` is unavailable. No
unavailable check is claimed as passed.
