# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Implement complete; Draft PR publication in progress
- Requirement: REQ-CONTOUR-002, two-dimensional isolines
- Issue: #135
- Branch: `codex/req-contour-002-isolines`
- Draft pull request: pending publication
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

## Next task boundary

Commit and push the stable implementation, open a Draft PR, link its exact head
and number in the registry and handoff, and stop. The next fresh task must
perform an isolated mathematical/numerical Review of only REQ-CONTOUR-002 and
its Draft PR. Do not begin REQ-CONTOUR-003.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #135
- Core implementation: `crates/georbf/src/contour/isoline.rs`
- Independent tests: `crates/georbf/tests/isoline.rs`
- User guide: `docs/user-guide/ISOLINES.md`
- Requirement summary: `changes/REQ-CONTOUR-002.md`
- Benchmark evidence: `docs/benchmarks/REQ-CONTOUR-002.md`
- Release benchmark: `crates/georbf/benches/isolines.rs`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, and API/ABI/schema snapshot checks are tracked by
later requirements and release gates. Local `actionlint` is unavailable. No
unavailable check is claimed as passed.
