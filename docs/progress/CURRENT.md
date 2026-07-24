# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair complete; fresh isolated re-review required
- Requirement: REQ-CONTOUR-001, one-dimensional level points
- Issue: #132
- Branch: `codex/req-contour-001-level-points`
- Draft pull request: #133
- Reviewed head: `323fcd9`
- Repair implementation head: `1280cd2`
- Dependency: REQ-MODEL-001 is integrated
- Registry status: `in_progress`

## Repair result

Exact repair head `1280cd2c772d2e049eb1e28203077f711fb16036`
addresses only the three confirmed PR #133 findings:

- CONTOUR001-REV-001: extraction rejects gradients that are only supported
  away from centers before evaluation, preventing a nondifferentiable center
  from fabricating stationary evidence.
- CONTOUR001-REV-002: tolerance-small derivative nodes remain candidate
  evidence but become diagnostic brackets only with a real neighboring sign
  change; exact-zero nodes use an explicit zero-width bracket.
- CONTOUR001-REV-003: independent transformed truth now checks
  original-coordinate derivative values and a negative-scale reflection.

Complete counterexamples, repair details, and regression evidence are in
`docs/reviews/PR-133-INDEPENDENT-REVIEW.md`.

## Validation state

- Before repair, the focused contour suite reproduced the first two defects as
  two failures. After repair, all eight all-feature contour integration tests
  passed.
- The focused contour Rustdoc example passed.
- Release benchmark smoke passed with deterministic checksum
  `2.50500000000000000e2`.
- The 58-requirement registry check and complete PR whitespace check passed.
- Exact repair implementation tree `1280cd2` passed the complete standard
  local gate: formatting, all-target/all-feature Clippy with warnings denied,
  all-feature workspace tests, workspace Rustdoc tests, and the registry check.
- Draft CI run 30079007316 passed Ubuntu on pre-repair documentation head
  `a101f65`. It is not repair validation. The Ready-only Windows, Ubuntu, macOS,
  and benchmark-smoke matrix remains unexecuted and is not claimed.

## Next task boundary

Start a fresh Review/re-review task for only PR #133 and REQ-CONTOUR-001. Give
the isolated read-only `math_reviewer` the bounded requirement/dependency
summary, normative documents, original findings, exact repair diff, tests,
benchmark evidence, and validation results. It must independently confirm
CONTOUR001-REV-001 through CONTOUR001-REV-003 are closed and inspect for new
P0--P3 findings.

If findings remain, record them and stop without repair. If no findings remain
and the exact final head has a complete green local gate, the fresh Review task
may follow the required integration sequence: mark PR #133 Ready, wait for the
complete Windows/Ubuntu/macOS and benchmark-smoke CI on that exact Ready head,
merge exactly once only when it is green, and then record truthful integration
state. Do not begin REQ-CONTOUR-002.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #132
- Draft implementation: GitHub PR #133
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
