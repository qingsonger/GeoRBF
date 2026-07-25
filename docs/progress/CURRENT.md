# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Implement / ready to publish Draft PR for fresh independent review
- Requirement: REQ-CONTOUR-003, three-dimensional isosurfaces
- Issue: #138
- Branch: `codex/req-contour-003-isosurfaces`
- Draft pull request: pending first push
- Dependency: REQ-MODEL-001 is integrated
- Registry status: `in_progress`
- Base `main`: `54d7f6e`; post-merge CI run `30107248318` passed Windows,
  Ubuntu, macOS, and every benchmark smoke

## Implemented scope

- `FittedField<3>` accepts a finite target level, finite
  original-coordinate box, explicit X/Y/Z grid and refinement limits,
  positive finite tolerances, and an explicit marching method.
- `MarchingSimplices` uses a globally conforming six-tetrahedron Freudenthal
  split. `TopologyAwareMarchingCubes` constructs cube-boundary loops and uses
  one scale-normalized bilinear asymptotic decision per alternating-sign
  shared face with a deterministic exact-zero tie.
- Exact endpoints and true sign brackets produce intersections; brackets use
  bounded fitted-value bisection. Canonical global grid-vertex and grid-edge
  identities deduplicate crossings without a hidden spatial radius.
- Analytic original-coordinate fitted gradients provide unit vertex normals.
  Triangle winding faces the positive-gradient side.
- Mesh topology validates edge incidence, opposing shared-edge traversal,
  requested-box boundary location, and deterministic connected components.
  Multiple cube-boundary loops with unproved interior connectivity are
  conservatively rejected.
- Exact target-level sampled edges, unsupported exact-vertex patterns,
  refinement exhaustion, zero-gradient normals, collapsed triangles,
  non-manifold edges, inconsistent winding, and interior boundaries are
  structured failures with no partial report.
- The controlled serial path rejects unsupported thread counts and checks
  cancellation around every analytic fitted-field query.
- Rust and benchmark surfaces are implemented. CLI awaits the versioned
  schema/persistence/complete-CLI requirements; C, C++, and Python remain M9.

## Validation state

- Eight independent exact-CPD-polynomial integration tests cover a transformed
  plane, a closed sphere, the conforming simplex reference, a saddle and
  shared-face decision, multiple-loop interior underdetermination, deterministic
  output, triangle/normal orientation, exact-edge degeneracy, invalid inputs,
  refinement exhaustion, work overflow, cancellation, and serial policy.
- Focused all-target/all-feature warning-denying `georbf` Clippy, focused
  isosurface tests, and the isosurface Rustdoc pass.
- Release benchmark smoke passes with one closed sphere, one iteration, and
  checksum `1.78575000000023807e5`.
- The normal 50-iteration release benchmark records 907,046,074.00
  ns/extraction and checksum `8.92875000000006333e6` on the documented local
  environment.
- The final stable-head complete workspace standard gate has not yet run.

## Next task boundary

Commit and push the implementation, create the Draft PR, record its number,
then run the complete standard workspace gate on the stable head. Commit and
push final evidence and stop. A fresh task must perform independent
`math_reviewer` Review; do not review or integrate this PR in the Implement
task and do not begin REQ-SCHEMA-001.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #138
- Core implementation: `crates/georbf/src/contour/isosurface.rs`
- Independent tests: `crates/georbf/tests/isosurface.rs`
- User guide: `docs/user-guide/ISOSURFACES.md`
- Requirement summary: `changes/REQ-CONTOUR-003.md`
- Benchmark evidence: `docs/benchmarks/REQ-CONTOUR-003.md`
- Release benchmark: `crates/georbf/benches/isosurfaces.rs`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, and API/ABI/schema snapshot checks are tracked by
later requirements and release gates. Local `actionlint` is unavailable. No
unavailable check is claimed as passed.
