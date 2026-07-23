# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Integration state / REQ-ANISO-003 complete
- Requirement: REQ-ANISO-003, Issue #111
- Implementation pull request: #112, squash-merged as `07dd290`
- Integration-state branch: `codex/req-aniso-003-integration-state`
- Integration-state pull request: #113 (Draft until exact Ready CI is green)
- Initial reviewed head: `a698362`
- Repair code/test head: `4426a30`
- Clean re-reviewed head: `c6b2c26`
- Exact Ready head: `556b254`
- Stable full-gate head: `4426a30`
- Review state: ANISO003-REV-001 is independently closed; the complete PR has
  no remaining P0-P3 finding
- Dependencies: REQ-ANISO-002 and REQ-TREND-002 are integrated
- Registry state in this change: `integrated`

## Integration result

- A fresh isolated read-only project `math_reviewer` independently closed
  ANISO003-REV-001 and found no P0-P3 issue in the complete repaired PR.
- Independent orthogonal-metric truth gives condition two for the `(3, 1.5)`
  spheroid, four for the `[4, 1]` ellipsoid, one for the isotropic background,
  and four for the mixture summary. The regression preserves the spheroid axis
  `(1, 0)` and caller-ordered ellipsoid axes `(-1, 0)`, `(0, 1)`, their
  provenance, paired lengths, and explicit tolerance.
- Signed sample weights, squared coverage, axial direction jumps, deterministic
  order, structured errors, and fallible export allocations satisfy the
  reviewed contract.
- The fixed-SPD diagonal-congruence construction and strict background remain
  unchanged. No CPD path, jitter, clipping, regularization, pseudoinverse,
  refit, field mutation, or unconditional Hessian capability is introduced.
- Exact Ready head `556b254` passed complete Windows, Ubuntu, and macOS CI run
  29975641751, including every configured backend combination, benchmark
  smoke, and requirement validation.
- PR #112 squash-merged exactly once as `07dd290`; Issue #111 closed as
  completed. Post-merge `main` CI run 29976326673 passed the same complete
  three-platform gate on exact merge commit `07dd290`.
- This isolated integration-state change updates only the registry, review
  evidence, history index, and bounded handoff. It changes no production code,
  test, manifest, schema, CI, build input, API, numerical behavior, dependency,
  tag, or release.

## Validation state

- Exact stable production/test head `4426a30` passed workspace format,
  warning-denying workspace all-target/all-feature Clippy, all-feature
  workspace tests, all workspace Rustdoc tests, all 58 requirement checks, and
  complete diff whitespace validation.
- The isolated re-review passed all four public `anisotropy_diagnostics`
  integration tests, the D=4 compile-fail Rustdoc test, warning-denying focused
  Clippy, the runnable example, all 58 requirement checks, and complete diff
  whitespace validation.
- Exact Ready-head run 29975641751 and post-merge `main` run 29976326673 are
  both green on Windows, Ubuntu, and macOS, including every configured
  benchmark smoke.
- The isolated integration-state tree must pass the complete local standard
  gate and exact Ready-head CI before it merges.

## Next task boundary

Run the complete local standard gate on the final integration-state head. Mark
PR #113 Ready, wait for exact Ready-head Windows, Ubuntu, macOS, and
benchmark-smoke CI, merge only if green, and stop. Do not start another
requirement.

## Durable evidence

- Acceptance criteria and exclusions: closed GitHub Issue #111
- Merged implementation: GitHub PR #112
- Integration-state pull request: GitHub PR #113
- Independent review and Repair evidence:
  `docs/reviews/PR-112-INDEPENDENT-REVIEW.md`
- Requirement summary: `changes/REQ-ANISO-003.md`
- Public implementation and Rustdoc:
  `crates/georbf/src/anisotropy_diagnostics.rs`
- Independent tests: `crates/georbf/tests/anisotropy_diagnostics.rs`
- Runnable example: `crates/georbf/examples/anisotropy_diagnostics.rs`
- Mathematical contract: `docs/architecture/ANISOTROPY.md`, ADR-0005, ADR-0008

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, and API/ABI/schema snapshot checks are tracked by
later requirements and release gates. Local `actionlint` is unavailable. No
unavailable check is claimed as passed.
