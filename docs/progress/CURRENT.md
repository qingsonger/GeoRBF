# Current GeoRBF Progress

- Current milestone: M0 / v0.0.1 — specification and engineering baseline
- Execution mode: Review / repair of PR #2
- Current requirement: REQ-BOOTSTRAP-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/1
- Pull request: https://github.com/qingsonger/GeoRBF/pull/2 (Draft)
- Branch: `bootstrap/specification`

## Completed in this run

- Confirmed that remote `main` contained only the MIT license and no open
  issues, pull requests, CI runs, or tags.
- Created the stage-0 branch and Rust 2024 workspace skeleton with four adapter
  boundaries and `xtask`.
- Added 58 machine-readable v1 requirements with dependency, priority,
  interface, test, document, benchmark, issue, PR, and status fields.
- Added scope, master plan, six mathematical contracts, six architecture
  contracts, eight accepted ADRs, release gates, repository instructions,
  changelog, Issue/PR templates, and three-platform CI.
- Implemented requirement validation for schema headers, required fields,
  status transitions, interface declarations, issue/PR identifiers,
  dependency existence and integration, dependency cycles, forbidden v1
  completion markers, and production-source placeholders.
- Committed and pushed the complete bootstrap baseline and opened Draft PR #2.
- Updated the pinned checkout action from v4 to v7.0.0 after CI reported the
  retired Node.js 20 runtime; the replacement run passed on all three platforms.
- Completed an independent review of PR #2 covering the mathematical,
  numerical, safety, interface, documentation, test, and benchmark checklist.
- Repaired derivative-sign and center-limit contracts, CPD scaling and
  null-space diagnostics, D=1 normal semantics, angular-cone validation,
  strict-SPD local-mixture prerequisites, and orientation-weight validation.
- Strengthened the requirement checker to reject unknown or malformed schema
  content, report only true dependency-cycle members, and forbid an
  `integrated` requirement with an unfinished benchmark obligation.
- Made the stage-0 CLI reject extra and non-Unicode arguments without panicking
  and added regression tests. Disabled accidental publication for every
  prerelease workspace package and made `xtask` enforce that policy. The
  complete review evidence is in
  `docs/reviews/PR-2-INDEPENDENT-REVIEW.md`.

## Current blockers

No known local blocker. The bootstrap requirement remains `documented`, not
`integrated`, until the review repair has green three-platform CI and PR #2 is
merged.

## Next atomic task

Confirm the review-repair CI result, then let the repository owner decide when
to mark PR #2 ready and merge under branch rules. After merge, update
REQ-BOOTSTRAP-001 to `integrated`; do not begin REQ-DIM-001 before that state.

## Latest full test result

Completed locally on Windows with Rust 1.96.1 on 2026-07-13:

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`:
  passed.
- `cargo test --workspace --all-features`: passed; 14 tests, 0 failures on
  Windows. The Unix matrix additionally runs the non-Unicode argv regression.
- `cargo test --doc --workspace`: passed; 0 doctests, 0 failures.
- `cargo xtask requirements check`: passed; 58 requirements.
- `cargo metadata --format-version 1 --no-deps`: passed.
- `cargo tree --workspace --duplicates`: passed; no duplicates.
- Actual CLI checks: `--version` returned success and `--version fit` returned
  the documented usage error with exit code 2.
- `git diff --check`: passed.
- Pre-review GitHub Actions run 29239841099 for commit `8d6b44e`: passed on
  `windows-latest`, `ubuntu-latest`, and `macos-latest`.

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, `cargo-semver-checks`, Miri,
sanitizers, fuzzing, mutation testing, API/ABI/schema snapshot checks, and
benchmark smoke tooling are not installed or not yet implemented. A second
full-YAML-parser check was not run because PyYAML, Ruby/YAML, and PowerShell
`ConvertFrom-Yaml` are unavailable; the dependency-free strict registry checker
did run. Stage 0 has no runtime mathematical path, so its benchmark obligation
is explicitly N/A. These later checks are tracked by requirements and the
release checklist.
