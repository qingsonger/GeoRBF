# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Implement / REQ-CENTER-001 complete
- Requirement: REQ-CENTER-001, open Issue #120
- Branch: `codex/req-center-001-rank-safe-centers`
- Pull request: #121 (Draft, awaiting fresh independent Review)
- Dependencies: REQ-SOLVE-001 and REQ-MODEL-001 are integrated
- Registry status: `planned` until independent Review, Ready CI, merge, and
  isolated integration-state evidence complete

## Implemented scope

- Added one GeoRBF-owned `CenterSelectionProblem<D>` for D=1, D=2, and D=3
  with finite locations, targets, and exact-symmetric row-major Gram storage.
- Implemented all-representer, ordered user-provided, seeded farthest-point,
  seeded residual-greedy, and seeded power-greedy strategies.
- Residual and power selection share checked Newton--Cholesky updates with the
  explicit scale threshold `n * epsilon * max_i(abs(K_ii))`.
- Every successful proposed selection passes the existing eight-pass RRQR,
  bounded-SVD, ambiguity-band, checked-Cholesky, and original-unit residual
  path under an explicit nonzero memory limit.
- Selection returns stable indices and diagnostics only. It never mutates a
  field problem, drops or softens a hard constraint, refits, regularizes,
  calls a pseudoinverse, or changes solver policy.
- Rust is implemented. CLI/schema work is N/A until M8; C, C++, and Python are
  N/A until M9.

## Validation state

- Nine focused center-selection integration tests pass.
- The center-selection rustdoc example passes.
- Warning-denying focused test and benchmark Clippy passes.
- The 48-candidate release benchmark smoke passes; the recorded 160-candidate
  baseline covers all five strategies.
- After the final production, test, manifest, and CI change, the complete
  standard workspace gate passed: format, warning-denying
  workspace/all-target/all-feature Clippy, all-feature workspace tests,
  workspace doctests, and the 58-requirement registry check.
- The later handoff and PR-number edits are documentation/registry evidence
  only; they change no production code, test, manifest, schema, CI, benchmark,
  build input, API, numerical behavior, or dependency.

## Next task boundary

After this implementation task commits, pushes, and opens its Draft PR, stop.
A fresh Review task must inspect only REQ-CENTER-001 and the PR diff, use the
isolated project `math_reviewer`, record findings, and must not repair
production code or begin REQ-TUNE-001.

## Durable evidence

- Acceptance criteria and exclusions: GitHub Issue #120
- Draft implementation: GitHub PR #121
- Requirement summary: `changes/REQ-CENTER-001.md`
- Architecture: `docs/architecture/ARCHITECTURE.md`
- Numerical policy: `docs/architecture/SOLVER_POLICY.md`
- Benchmark: `docs/benchmarks/REQ-CENTER-001.md`
- Production implementation: `crates/georbf/src/center_selection.rs`
- Independent tests: `crates/georbf/tests/center_selection.rs`

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, and API/ABI/schema snapshot checks are tracked by
later requirements and release gates. Local `actionlint` is unavailable. No
unavailable check is claimed as passed.
