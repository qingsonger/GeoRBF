# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair complete; fresh independent re-review required
- Requirement: REQ-CONTOUR-002, two-dimensional isolines
- Issue: #135
- Branch: `codex/req-contour-002-isolines`
- Draft pull request: #136
- Reviewed head: `2b93db4`
- Repair implementation and standard-gate head: `9510b6c`
- Independent review: `docs/reviews/PR-136-INDEPENDENT-REVIEW.md`
- Dependency: REQ-MODEL-001 is integrated
- Registry status: `in_progress`
- Base `main`: `6e622c7`; post-merge CI run 30092831813 passed Windows,
  Ubuntu, and macOS

## Implemented scope

- `FittedField<2>` accepts a finite target level, finite original-coordinate
  rectangle, explicit X/Y grid and refinement limits, positive finite value
  and coordinate tolerances, and an explicit marching method.
- `MarchingSimplices` is the fixed lower-left-to-upper-right reference path.
  `DisambiguatedMarchingSquares` uses a scale-normalized bilinear asymptotic
  decider and deterministic positive-connectivity tie for alternating cells.
- Only exact endpoint hits and true value-sign brackets produce
  intersections. Retained brackets use fitted-field values and
  bracket-preserving bisection with an explicit failure limit.
- Shared intersections use canonical grid-edge or grid-vertex identities.
  Only exact repeated undirected segments are removed; distinct nearby
  components are not spatially merged.
- The unique segment graph is accepted only with degree at most two. Returned
  components are deterministic open or closed polylines, and every open
  endpoint retains explicit requested-rectangle side evidence.
- Edges with two exact target-level endpoint samples, unsupported exact-vertex
  patterns, non-manifold vertices, and interior open endpoints are structured
  failures. Finite samples never claim that a nonlinear edge interior or an
  unseen sub-cell component is known.
- The controlled serial path rejects unsupported thread counts before fitted
  evaluation, reports deterministic progress, checks cancellation around every
  analytic value query, and returns no partial report on failure.
- Rust and benchmark surfaces are implemented. CLI is N/A until an M8
  versioned model/project input can supply a fitted field. C, C++, and Python
  remain M9 work.

## Validation state

- Seven independent exact-CPD-polynomial integration tests pass for a
  transformed open line, closed circle, exactly tied saddle, nonzero
  asymptotic decision, marching-simplices reference, invalid input,
  two-endpoint edge degeneracy, refinement exhaustion, work overflow,
  cancellation, serial policy, and progress.
- The focused isoline Rustdoc example and warning-denying all-target,
  all-feature `georbf` Clippy pass.
- Release benchmark smoke passes with one closed circle and checksum
  `1.83299999999997817e4`.
- The normal 500-iteration release benchmark records 19,386,121.40
  ns/extraction and checksum `4.58249999999998696e6` on the documented local
  environment.
- The 58-requirement registry check passes.
- After the final production, test, manifest, benchmark, CI-routing,
  architecture, user-guide, registry, and handoff changes, the stable
  implementation tree passed formatting, all-target and all-feature workspace
  Clippy with warnings denied, the complete all-feature workspace test suite,
  all workspace Rustdoc tests, and the 58-requirement registry check.
- The isolated `math_reviewer` found two P1 and three P2 defects on
  `2b93db4`:
  - CONTOUR002-REV-001: value-tolerance endpoint acceptance can assign one
    grid-vertex key to two distinct bracketed components.
  - CONTOUR002-REV-002: duplicate edge records for exact grid corners reject
    an ordinary two-unique-vertex square contour.
  - CONTOUR002-REV-003: the final allowed bisection is not checked after it
    first satisfies coordinate tolerance.
  - CONTOUR002-REV-004: stable topology sorts can allocate outside the
    structured allocation-failure path.
  - CONTOUR002-REV-005: cancellation is checked after, but not immediately
    before, each fitted-field value query.
- Repair commit `9510b6c` gives tolerance-accepted sign brackets their
  crossed-edge identities, deduplicates square and simplex intersections by
  canonical key before topology classification, checks the updated bracket
  after the final bisection, uses allocation-free deterministic topology
  ordering, and checkpoints cancellation immediately before every fitted-value
  query while preserving post-query cancellation priority.
- Three new exact-CPD integration regressions cover distinct nearby quadratic
  components, exact affine square-corner topology, and final-iteration
  coordinate tolerance. Three internal regressions cover canonical-key
  deduplication, 4096-element zero-allocation ordering, and synchronized
  pre/post-query cancellation behavior.
- The 10-test isoline integration suite, 3-test internal repair suite, focused
  isoline Rustdoc, warning-denying focused Clippy, and release benchmark smoke
  pass. The smoke checksum remains `1.83299999999997817e4`.
- After the final production and test change, exact committed tree `9510b6c`
  passed formatting, warning-denying all-target and all-feature workspace
  Clippy, the complete all-feature workspace test suite, all workspace Rustdoc
  tests, and the 58-requirement registry check.
- The repair has not been independently re-reviewed. PR #136 remains Draft;
  no Ready Windows, Ubuntu, macOS, or complete benchmark-smoke CI is claimed.

## Next task boundary

Stop this Repair task after pushing the documentation-only handoff commit. The
next fresh task must use Review/re-review mode for Draft PR #136. Supply a fresh
isolated `math_reviewer` only the bounded requirement and dependency summary,
normative documents, original five findings, focused repair diff
`978a445..9510b6c`, complete PR diff, and validation evidence. It must
independently confirm closure of CONTOUR002-REV-001 through
CONTOUR002-REV-005 and check for new P0--P3 findings. If any finding remains,
record it and stop without repair. If the re-review is clean, synchronize the
review evidence, mark PR #136 Ready, wait for the complete exact-head Windows,
Ubuntu, macOS, and benchmark-smoke CI, and merge exactly once only if green.
Then record truthful integration state and stop. Do not begin REQ-CONTOUR-003.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #135
- Draft implementation: GitHub PR #136
- Core implementation: `crates/georbf/src/contour/isoline.rs`
- Independent tests: `crates/georbf/tests/isoline.rs`
- User guide: `docs/user-guide/ISOLINES.md`
- Requirement summary: `changes/REQ-CONTOUR-002.md`
- Benchmark evidence: `docs/benchmarks/REQ-CONTOUR-002.md`
- Release benchmark: `crates/georbf/benches/isolines.rs`
- Independent review: `docs/reviews/PR-136-INDEPENDENT-REVIEW.md`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, and API/ABI/schema snapshot checks are tracked by
later requirements and release gates. Local `actionlint` is unavailable. No
unavailable check is claimed as passed.
