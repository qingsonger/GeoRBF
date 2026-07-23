# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Integration state / REQ-SPARSE-001 complete
- Requirement: REQ-SPARSE-001, closed Issue #117
- Implementation pull request: #118, squash-merged as `ce93a98`
- Integration-state branch: `codex/req-sparse-001-integration-state`
- Integration-state pull request: pending creation
- Exact Ready head: `4c436b6`
- Clean independent re-review head:
  `917e6b3b5a12f48588cb5a34676cb2093988a8db`
- Third Repair implementation and stable gate head:
  `85d7e99356fe3790f2a0d63430b78a4f68ad8a0f`
- Closed findings: P1 SPARSE001-REV-001, P2 SPARSE001-REV-002, P2
  SPARSE001-REV-003, P3 SPARSE001-REV-004, and P1 SPARSE001-REV-005
- Open P0-P3 findings: none
- Dependencies: REQ-SPIKE-003, REQ-FIELD-001, and REQ-KERNEL-004 are integrated
- Registry status in this change: `integrated`

## Integration result

- An isolated read-only `math_reviewer` inspected the complete base-to-head
  diff and independently closed SPARSE001-REV-005.
- The faer 0.24.4 AMD and complete symbolic-analysis request reconstruction,
  retained symbolic and numeric bounds, separate live peaks, and enforcement
  before backend CSC construction are conservative and match pinned backend
  truth.
- The 64-by-64 all-supported between-limit regression independently obtains
  faer's AMD request and rejects after only the `Started` progress event.
- No new P0-P3 finding was identified.
- Exact Ready head `4c436b6` passed complete Windows, Ubuntu, and macOS CI run
  29998141180, including every configured backend combination, benchmark
  smoke, and requirement validation.
- PR #118 squash-merged exactly once as `ce93a98`; Issue #117 closed as
  completed. Post-merge `main` CI run 29999737528 passed the same complete
  three-platform gate on exact merge commit `ce93a98`.
- This isolated integration-state change updates only the registry, review
  evidence, completed-history index, and bounded handoff. It changes no
  production code, test, manifest, schema, CI, build input, API, numerical
  behavior, dependency, tag, or release.

## Validation state

- Exact stable implementation head
  `85d7e99356fe3790f2a0d63430b78a4f68ad8a0f` passed all 44 all-feature core
  unit tests, all ten all-feature sparse integration tests, format, and
  warning-denying all-target/all-feature Clippy.
- After the last production or test change, that same exact head passed the
  complete standard workspace gate: format, warning-denying
  workspace/all-target/all-feature Clippy, all-feature workspace tests,
  workspace doctests, and the 58-requirement registry check.
- The independent reviewer passed all ten sparse integration tests, the
  canonical-capacity regression, and the 64-point release benchmark smoke.
  The parent Review task independently passed the sparse tests,
  canonical-capacity regression, registry check, and PR whitespace check.
- Draft CI run 29997122789 passed on exact reviewed head `917e6b3`.
- Exact Ready-head run 29998141180 and post-merge `main` run 29999737528 are
  both green on Windows, Ubuntu, and macOS, including every configured
  benchmark smoke.
- The isolated integration-state tree must pass the complete local standard
  gate and exact Ready-head CI before it merges.

## Next task boundary

Create the isolated integration-state pull request, record its number in the
review, history index, and this handoff, then run the complete local standard
gate. Mark that PR Ready, wait for exact Ready-head Windows, Ubuntu, macOS, and
benchmark-smoke CI, merge only if green, and stop. Do not begin
REQ-CENTER-001.

## Durable evidence

- Acceptance criteria and exclusions: closed GitHub Issue #117
- Merged implementation: GitHub PR #118
- Integration-state pull request: pending creation
- Independent review and Repair evidence:
  `docs/reviews/PR-118-INDEPENDENT-REVIEW.md`
- Requirement summary: `changes/REQ-SPARSE-001.md`
- Architecture: `docs/architecture/ARCHITECTURE.md`
- Numerical policy: `docs/architecture/SOLVER_POLICY.md`
- Backend selection: `docs/adr/ADR-0012-rstar-faer-compact-sparse-backends.md`
- Benchmark: `docs/benchmarks/REQ-SPARSE-001.md`
- Production implementation: `crates/georbf/src/problem_ir.rs` and
  `crates/georbf/src/sparse.rs`
- Independent tests: canonical-IR unit tests and
  `crates/georbf/tests/sparse.rs`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, and API/ABI/schema snapshot checks are tracked by
later requirements and release gates. Local `actionlint` is unavailable. No
unavailable check is claimed as passed.
