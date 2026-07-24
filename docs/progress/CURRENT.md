# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review complete; Repair required
- Requirement: REQ-CONTOUR-002, two-dimensional isolines
- Issue: #135
- Branch: `codex/req-contour-002-isolines`
- Draft pull request: #136
- Reviewed head: `2b93db4`
- Stable implementation head: `4e766af`
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
- The isolated `math_reviewer` found two P1 and three P2 defects:
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
- On exact reviewed head `2b93db4`, the parent Review reran all seven focused
  isoline integration tests and the focused Rustdoc example; they passed.
  Draft CI run 30095419189 also passed its configured Ubuntu correctness gate.
  This is not the complete Ready three-platform and benchmark-smoke gate.
- No production, test, manifest, schema, benchmark, registry, CI, or build
  input was changed during Review. The complete standard gate on immutable
  implementation head `4e766af` therefore remains the latest full local gate.

## Next task boundary

Stop this Review task. The next fresh task must use Repair mode and address
only CONTOUR002-REV-001 through CONTOUR002-REV-005 in Draft PR #136. Reproduce
each finding with the independent regression specified in the review record,
implement the smallest complete repairs, rerun focused checks during
development, run the complete standard checks after the final code change,
update review evidence and this bounded handoff, commit, push, and stop for a
fresh independent re-review. Do not begin REQ-CONTOUR-003.

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
