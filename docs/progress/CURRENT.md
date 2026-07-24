# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Implement complete; fresh isolated Review required
- Requirement: REQ-CONTOUR-001, one-dimensional level points
- Issue: #132
- Branch: `codex/req-contour-001-level-points`
- Draft pull request: #133
- Stable implementation gate head: `b41e482`
- Dependency: REQ-MODEL-001 is integrated
- Registry status: `in_progress`

## Implemented scope

- `FittedField<1>` accepts a finite target level, finite original-coordinate
  domain, explicit scan/refinement limits, and positive finite value,
  coordinate, and derivative tolerances.
- Every requested scan interval is deterministically midpoint-split. Analytic
  fitted-value and original-coordinate derivative sign brackets are retained
  and refined by bracket-preserving bisection.
- Isolated points are sorted and tolerance-deduplicated with fitted value,
  residual, analytic derivative, and boundary/crossing/stationary
  classification. Non-level stationary candidates remain separately visible.
- Adjacent scan segments satisfying both value and derivative tolerances are
  merged into degenerate level intervals. The API emits no arbitrary isolated
  point for a non-isolated solution set and reports that finite scan resolution
  is not a global root-count proof.
- The controlled serial path rejects unsupported thread counts before fitted
  evaluation, reports deterministic progress, checks cancellation around every
  analytic query, and returns no partial report on failure.
- Rust and benchmark surfaces are implemented. CLI is N/A until an M8
  versioned model/project input can supply a fitted field. C, C++, and Python
  remain M9 work.

## Validation state

- Seven independent integration tests pass for transformed
  crossing roots, tangency, non-level stationarity, exact boundaries,
  degenerate intervals, invalid settings, work overflow, center-limited
  evaluation failure, refinement exhaustion, cancellation, serial policy, and
  progress.
- The focused contour rustdoc example and warning-denying all-target Clippy
  pass.
- Release benchmark smoke passes with two roots, one non-level stationary
  point, and checksum `2.50500000000000000e2`.
- The normal 2,000-iteration release benchmark records
  125,356.75 ns/extraction and checksum `2.50500000000000000e5` on the documented
  local environment.
- Exact stable implementation head `b41e482` passed formatting, all-target and
  all-feature workspace Clippy with warnings denied, the complete all-feature
  workspace test suite, all workspace Rustdoc tests, and the 58-requirement
  registry check after the final production, test, manifest, benchmark, and
  CI-routing change.
- The subsequent handoff-link change modifies only the requirement pull-request
  number and this bounded handoff. It changes no production code, test,
  manifest, schema, benchmark, CI, API, or numerical behavior.

## Next task boundary

Start a fresh isolated mathematical/numerical Review task for only Draft PR
#133 and REQ-CONTOUR-001. Supply the reviewer the bounded requirement/dependency
summary, relevant architecture contracts, base-to-head diff, seven independent
tests, rustdoc, and benchmark evidence without this implementation reasoning
transcript. Record findings but do not repair production code in that Review
task. Do not begin REQ-CONTOUR-002.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #132
- Draft implementation: GitHub PR #133
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
