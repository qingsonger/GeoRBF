# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Integration state / REQ-CONTOUR-001 implementation merged
- Requirement: REQ-CONTOUR-001, one-dimensional level points
- Issue: #132, closed
- Implementation branch: `codex/req-contour-001-level-points`
- Implementation pull request: #133, squash-merged as `2634a7d`
- Integration-state branch: `codex/req-contour-001-integration-state`
- Integration-state pull request: #134 (Draft until final exact-head gates pass)
- Exact implementation Ready head: `25df6e9`
- Repair implementation head: `1280cd2`
- Dependency: REQ-MODEL-001 is integrated
- Registry status in this change: `integrated`

## Fresh re-review and integration result

An isolated read-only project `math_reviewer` reviewed the complete base
`a3e89ee..bc892c3` and focused repair `323fcd9..1280cd2`. It found no
remaining or new P0--P3 issue and independently closed all three original
findings. Complete counterexamples, repair mapping, independent truth, and
validation evidence are in
`docs/reviews/PR-133-INDEPENDENT-REVIEW.md`.

Clean re-review evidence was pushed as exact Ready head `25df6e9`. Ready CI run
30088526342 passed the complete Windows, Ubuntu, and macOS workspace matrix,
including all configured correctness checks, backend combinations, requirement
validation, and every benchmark smoke. PR #133 was squash-merged exactly once
as `2634a7d`; Issue #132 closed as completed. Post-merge `main` CI run
30089747253 passed the same complete three-platform gate on exact merge commit
`2634a7d`.

This isolated integration-state change modifies only the registry, completed
history index, independent review evidence, and bounded handoff. It changes no
production code, test, manifest, schema, CI, build input, API, numerical
behavior, dependency, tag, or release.

## Next task boundary

Run the complete standard local gate on the final integration-state tree,
commit and push it, and open its Draft pull request. Mark that pull request
Ready, wait for exact-head Windows, Ubuntu, and macOS correctness plus complete
benchmark-smoke CI, and merge only if green. Then stop. Do not begin
REQ-CONTOUR-002.

## Durable evidence

- Acceptance criteria and exclusions: closed GitHub Issue #132
- Merged implementation: GitHub PR #133
- Integration-state pull request: GitHub PR #134
- Independent review and repair: `docs/reviews/PR-133-INDEPENDENT-REVIEW.md`
- Core implementation: `crates/georbf/src/contour.rs`
- Independent tests: `crates/georbf/tests/contour.rs`
- User guide: `docs/user-guide/LEVEL_POINTS.md`
- Requirement summary: `changes/REQ-CONTOUR-001.md`
- Benchmark evidence: `docs/benchmarks/REQ-CONTOUR-001.md`
- Release benchmark: `crates/georbf/benches/level_points.rs`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, and API/ABI/schema snapshot checks are tracked by
later requirements and release gates. Local `actionlint` is unavailable. No
unavailable check is claimed as passed.
