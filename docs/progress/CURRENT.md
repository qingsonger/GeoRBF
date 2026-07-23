# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Clean independent re-review; Ready CI and integration required
- Requirement: REQ-ANISO-003, Issue #111
- Branch: `codex/req-aniso-003-export-diagnostics`
- Draft pull request: #112
- Base: `main` at `37cb91d` after REQ-TREND-002 integration
- Reviewed head: `a698362`
- Repair head: `4426a30`
- Clean re-reviewed head: `c6b2c26`
- Stable full-gate head: `4426a30`
- Dependencies: REQ-ANISO-002 and REQ-TREND-002 are integrated
- Registry state in this change: `in_progress`

## Scope and implementation

- Adds deterministic owned, renderer-neutral anisotropy diagnostic exports for
  exactly D=1, D=2, and D=3 without changing or refitting the compiled local
  SPD mixture.
- Exports caller-ordered control positions, honest spheroidal or ellipsoidal
  resolved axes and lengths, source confidence, signed strengths, influence
  radii, optional compact regions, condition numbers, and sign-invariant
  adjacent direction jumps.
- Exports strict-background policy/condition evidence and caller-ordered sample
  positions with every signed mixture weight, aggregate squared coverage,
  background contribution, active-component count, and domain membership.
- Exports source-aware low-confidence reference directions as control/axis
  region records. Explicit directions are excluded and an absent compact region
  remains explicit.
- No GUI, VTK, encoder, versioned schema, CLI, binding, solver, or numerical
  dependency is added. Schema/CLI work remains M8 and adapters remain M9.

## Independent review state

- A fresh isolated read-only project `math_reviewer` re-reviewed exact head
  `c6b2c26` from the bounded requirement and dependency summaries, Issue #111,
  M6 plan, ANISOTROPY and ADR-0005/ADR-0008 contracts, complete repaired PR
  diff, original finding, tests, and validation evidence.
- Repair head `4426a30` addresses ANISO003-REV-001 by extending the independent
  diagnostic-schema test with exact exported axis components and provenance,
  axis-length pairing, ellipsoid tolerance, per-control condition numbers, and
  the summary maximum condition number.
- Independent truth for the existing orthogonal test fixtures is condition two
  for the `(3, 1.5)` spheroid, four for the `[4, 1]` ellipsoid, one for the
  isotropic background, and four for the mixture summary. The expected axes are
  spheroid `(1, 0)` and caller-ordered ellipsoid `(-1, 0)`, `(0, 1)`.
- No other P0-P3 finding was identified. Durable evidence is in
  `docs/reviews/PR-112-INDEPENDENT-REVIEW.md`.
- The fresh reviewer independently closed ANISO003-REV-001 and found no new
  P0-P3 issue in the complete PR.

## Validation state

- Focused `anisotropy_diagnostics` integration tests pass all four schema,
  low-confidence, direction-jump, coverage, dimension, and error-path cases.
- The module's compile-fail D=4 Rustdoc test passes.
- Focused warning-denying Clippy and the runnable example pass.
- Exact repair head `4426a30` passed workspace format, warning-denying workspace
  all-target/all-feature Clippy, all-feature workspace tests, all workspace
  Rustdoc tests, all 58 requirement checks, and complete diff whitespace
  validation.
- The Repair passed all four focused integration tests, the D=4 compile-fail
  Rustdoc test, the runnable example, and warning-denying focused Clippy.
- The isolated re-review passed those same focused tests, Clippy, example, all
  58 requirement checks, and complete diff whitespace validation.
- Draft Ubuntu CI run 29975187579 passed on exact re-reviewed head `c6b2c26`;
  the Ready-only three-platform and benchmark-smoke matrix has not yet run.
- The repair changes only the independent test; production code, API,
  numerical behavior, manifests, registry state, schema, CI, and dependencies
  are unchanged.

## Next task boundary

Commit and push this evidence-only clean re-review conclusion, synchronize the
PR evidence, and mark PR #112 Ready. Wait for the complete Windows, Ubuntu, and
macOS matrix with every benchmark smoke on that exact Ready head. Merge exactly
once only if it is green, wait for the exact merge commit's complete `main` CI,
then record truthful integration through an isolated integration-state change.
Do not start another requirement.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #111
- Requirement summary: `changes/REQ-ANISO-003.md`
- Public implementation and Rustdoc:
  `crates/georbf/src/anisotropy_diagnostics.rs`
- Independent tests: `crates/georbf/tests/anisotropy_diagnostics.rs`
- Runnable example: `crates/georbf/examples/anisotropy_diagnostics.rs`
- Mathematical contract: `docs/architecture/ANISOTROPY.md`, ADR-0005, ADR-0008
- Independent review: `docs/reviews/PR-112-INDEPENDENT-REVIEW.md`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, and API/ABI/schema snapshot checks are tracked by
later requirements and release gates. Local `actionlint` is unavailable. No
unavailable check is claimed as passed.
