# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Integration state / REQ-ANISO-002 complete
- Requirement: REQ-ANISO-002, Issue #105
- Implementation pull request: #106, squash-merged as `bb099fe`
- Integration-state branch: `codex/req-aniso-002-integration-state`
- Integration-state pull request: #107 (Draft until exact Ready CI is green)
- Independently reviewed implementation head: `0b7f558`
- Clean re-review evidence / exact Ready head: `03d30a5`
- Final Repair code/test/normative-document head: `358199b`
- Dependencies: REQ-ORIENT-001 and REQ-ANISO-001 are integrated
- Registry state in this change: `integrated`

## Integration result

- A fresh isolated read-only `math_reviewer` closed ANISO002-REV-008 and found
  no P0-P3 issue in the complete repaired PR. ANISO002-REV-001 through
  ANISO002-REV-009 are closed.
- Independent exact-rational review rejects scales one and
  `1.0.to_bits() - 1`, accepts `1.0.to_bits() - 2`, and proves the returned
  correlation scale maximal. The decisive determinant interval bound is
  positive; 1,100 probes across all cubic signs and zero-factor cases found no
  upper-bound violation.
- Exact Ready head `03d30a5` passed complete Windows, Ubuntu, and macOS CI run
  29889050240, including every configured backend combination, benchmark smoke,
  and requirement validation.
- PR #106 squash-merged exactly once as `bb099fe`; Issue #105 closed as
  completed. Post-merge `main` CI run 29889557309 passed the same complete
  three-platform gate on exact merge commit `bb099fe`.
- This isolated integration-state change updates only the registry, review
  evidence, history index, and bounded handoff. It changes no production code,
  test, manifest, schema, CI, build input, API, numerical behavior, dependency,
  tag, or release.

## Validation state

- Exact reviewed implementation head `0b7f558` passed the complete local
  standard gate: workspace format, warning-denying workspace all-target/all-
  feature Clippy, all workspace tests with all features, workspace Rustdoc,
  all 58 requirement checks, and complete diff whitespace validation.
- Both exact Ready-head run 29889050240 and post-merge `main` run 29889557309
  are green on Windows, Ubuntu, and macOS, including every configured benchmark
  smoke.
- The isolated integration-state tree must pass the complete local standard
  gate and exact Ready-head CI before it merges.
- Local `actionlint` and the unavailable later tools listed below remain
  unexecuted and are not claimed as passed.

## Next task boundary

After the isolated integration-state pull request is green and merged, open a
fresh task and perform the mandatory preflight. Use
`cargo xtask requirements next`; do not start another requirement in this
task.

## Durable evidence

- Acceptance criteria and exclusions: closed GitHub Issue #105
- Merged implementation: GitHub PR #106
- Integration-state pull request: GitHub PR #107
- Independent findings and Repair evidence:
  `docs/reviews/PR-106-INDEPENDENT-REVIEW.md`
- Requirement summary and benchmark baseline: `changes/REQ-ANISO-002.md`
- Public implementation and Rustdoc: `crates/georbf/src/orientation_tensor.rs`
- Independent property/error tests: `crates/georbf/tests/orientation_tensor.rs`
- Actual allocation regression:
  `crates/georbf/tests/orientation_tensor_allocations.rs`
- Runnable example: `crates/georbf/examples/orientation_tensor.rs`
- Focused benchmark: `crates/georbf/benches/orientation_tensor.rs`
- Mathematical contract: `docs/architecture/ANISOTROPY.md`, ADR-0009, ADR-0010

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, and API/ABI/schema snapshot checks are tracked by
later requirements and release gates. Local `actionlint` is unavailable. No
unavailable check is claimed as passed.
