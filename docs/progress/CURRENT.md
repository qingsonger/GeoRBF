# Current GeoRBF Progress

- Current milestone: M1 / v0.1.0 — dimensions, geometry, coordinates,
  orientation, and kernel calculus
- Execution mode: Implement / implementation handoff
- Current requirement: REQ-KCALC-001
- Issue: #10
- Pull request: not yet opened
- Branch: `codex/req-kcalc-001-radial-kernel-calculus`

## Completed in this run

- Confirmed `main` at `da7634e` is clean, synchronized, and green on Windows,
  Ubuntu, and macOS; confirmed REQ-DIM-001 is integrated and no Repair or
  Review work is pending; created Issue #10 and the isolated REQ-KCALC-001
  branch.
- Added a geology-free radial calculus for exactly D=1/D=2/D=3. Stable
  max-component-scaled separation and caller-supplied finite radial jets expand
  to fixed-array value, gradient, Hessian, and third derivative tensors without
  allocation, dependencies, dynamic dispatch, unsafe code, or core output.
- Added the explicit smooth-center path `value`, zero gradient,
  `phi''(0) I`, and zero third tensor without radial quotients. Query/center
  derivatives use one exact minus sign per center argument, and non-finite or
  non-representable states return indexed structured errors.
- Added independent `r^6` polynomial truth, Gaussian finite differences,
  center-limit, exchange-sign, exact tensor-symmetry, extreme separation,
  structured-error, thread-safety, and unsupported-dimension tests.
- Added rustdoc, synchronized mathematical and architectural contracts, a
  runnable Rust example, a change record, and a deterministic dependency-free
  D=1/D=2/D=3 benchmark with a recorded single-thread baseline and CI smoke
  coverage.
- Created Issue #7 and the REQ-COORD-001 feature branch after confirming that
  REQ-DIM-001 is integrated, `main` CI is green, and no Repair or Review work
  is pending. Fixed the scope at metadata and affine coordinate transforms;
  reprojection, orientation, anisotropy, kernels, schemas, and bindings remain
  excluded.
- Added exact length-unit and opaque EPSG/WKT metadata, validated axis
  permutations, vertical direction, handedness, angle units, and deterministic
  compatibility errors that prevent silent coordinate mixing.
- Added finite, invertible affine normalization with exact zero-pivot partial
  pivoting and no tolerance, jitter, regularization, pseudoinverse, dependency,
  or hidden adjustment.
- Added point round trips plus analytic `S^-T` gradient and
  `S^-T H_tilde S^-1` Hessian transforms, including structured errors for
  singular matrices, non-representable inverses, invalid Hessians, and
  non-finite results.
- Added D=1/D=2/D=3, rotation, shear, near-singular, extreme-value, metadata,
  error-path, and unsupported-dimension coverage; synchronized the mathematical
  and model-format contracts, rustdoc, example, and change record.
- Completed the independent mathematical, numerical, safety, API, allocation,
  and test review of PR #8. Repaired scale-sensitive inversion, strengthened
  independent extreme-value truth and every-field metadata mismatch coverage,
  defined the vertical canonical axis, and recorded the evidence in
  `docs/reviews/PR-8-INDEPENDENT-REVIEW.md`.
- Confirmed the review-repair head passed GitHub Actions on Windows, Ubuntu,
  and macOS, updated the PR evidence, and marked PR #8 ready for maintainer
  review.
- Squash-merged PR #8 as commit `2292a54`; Issue #7 closed automatically, and
  post-merge `main` CI run 29250743578 passed on Windows, Ubuntu, and macOS.
  REQ-COORD-001 now satisfies every integration gate.
- Committed and pushed the implementation, opened Draft PR #8, and advanced
  REQ-COORD-001 to `documented`; integration remains forbidden until the
  independent review is recorded, CI is green, and the PR is merged.
- Created Issue #4 and the REQ-DIM-001 feature branch after confirming that
  bootstrap is integrated, `main` CI is green, and no Repair or Review work is
  pending.
- Fixed the scope at compile-time D=1/2/3 gating, finite geometry components,
  nonzero directions, overflow/underflow-safe unit directions, tests, rustdoc,
  and one Rust example; later coordinate, orientation, kernel, and binding work
  remains excluded.
- Added private-representation `Point`, `Vector`, `Direction`, and
  `UnitDirection` types. Fallible constructors report indexed non-finite values
  and reject zero directions without panicking.
- Added maximum-component-scaled normalization, including coverage at
  `f64::MAX` and the smallest positive subnormal value, plus deterministic
  scale-invariance, sign, conversion, and thread-safety tests.
- Added compile-fail doctests for D=0 and D=4, a runnable construction example,
  the precise normalization contract, and the REQ-DIM-001 change record.
- Completed the independent mathematical, numerical, safety, API, performance,
  and test review of PR #5. No runtime-code defect was found. Repaired the stale
  CI reference in this handoff and recorded the evidence in
  `docs/reviews/PR-5-INDEPENDENT-REVIEW.md`.
- Confirmed the review-record head passed GitHub Actions on Windows, Ubuntu, and
  macOS, then marked PR #5 ready for maintainer review.
- Squash-merged PR #5 as commit `7dfdb18`; Issue #4 closed automatically, and
  post-merge `main` CI run 29246177488 passed on Windows, Ubuntu, and macOS.
  REQ-DIM-001 now satisfies every integration gate.
- Committed and pushed the implementation, opened Draft PR #5, and advanced
  the registry status to `documented`; integration remains forbidden until an
  independent review is complete and the PR is merged.
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
- Marked PR #2 ready and squash-merged it as commit `36ad660`; Issue #1 closed
  automatically. REQ-BOOTSTRAP-001 now satisfies the integration gate.

## Current blockers

Integration remains blocked by the mandatory independent mathematical and
numerical review, green pull-request CI, and merge. The local implementation,
tests, documentation, example, and benchmark obligations are complete.

## Next atomic task

Commit and push the isolated REQ-KCALC-001 implementation, open its Draft PR,
record the PR identifier in the requirement registry, and let a separate
session perform the mandatory independent mathematical, numerical, safety,
API, allocation, benchmark, and test review. Do not begin kernel families,
metadata, orientation, polynomial, or solver work before REQ-KCALC-001 is
reviewed and integrated.

## Latest full test result

Completed locally on Windows with Rust 1.96.1 on 2026-07-13:

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`:
  passed.
- `cargo test --workspace --all-features`: passed; 43 tests, 0 failures on
  Windows. The Unix matrix additionally runs the non-Unicode argv regression.
- `cargo test --doc --workspace`: passed; 9 doctests, including seven
  unsupported-dimension compile-fail cases, 0 failures.
- `cargo test -p georbf --release --all-features`: passed; 29 integration tests
  and 9 doctests, 0 failures.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p georbf --all-features --no-deps`:
  passed.
- `cargo xtask requirements check`: passed; 58 requirements.
- `cargo metadata --format-version 1 --no-deps`: passed.
- `cargo tree --workspace --duplicates`: passed; no duplicates.
- `cargo bench -p georbf --bench radial_kernel_calculus -- --smoke`: passed
  for deterministic D=1/D=2/D=3 workloads. The full 1,000,000-iteration local
  baseline measured 46.79, 92.89, and 170.21 ns/iteration respectively; see
  `benches/REQ-KCALC-001.md` for environment and checksum details.
- `cargo run -p georbf --example radial_kernel_calculus`: passed.
- Scoped forbidden-pattern and core allocation/dynamic-dispatch scans: passed.
- Actual CLI checks: `--version` returned success and `--version fit` returned
  the documented usage error with exit code 2.
- `git diff --check`: passed.
- Baseline `main` GitHub Actions run 29246462335 for commit `34468a3`: passed on
  `windows-latest`, `ubuntu-latest`, and `macos-latest`; formatting, Clippy,
  workspace tests, doctests, and all 58 requirement checks passed in every job.
- REQ-COORD-001 post-merge `main` GitHub Actions run 29250743578 for merge
  commit `2292a54`: passed on `windows-latest`, `ubuntu-latest`, and
  `macos-latest`; formatting, Clippy, workspace tests, doctests, and all 58
  requirement checks passed in every job.
- Latest `main` GitHub Actions run 29251067778 for commit `da7634e`: passed on
  `windows-latest`, `ubuntu-latest`, and `macos-latest`.

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, `cargo-semver-checks`, Miri,
sanitizers, fuzzing, mutation testing, and API/ABI/schema snapshot checks are
not installed or not yet implemented. A second
full-YAML-parser check was not run because PyYAML, Ruby/YAML, and PowerShell
`ConvertFrom-Yaml` are unavailable; the dependency-free strict registry checker
did run. Stage 0 has no runtime mathematical path, so its benchmark obligation
is explicitly N/A. REQ-DIM-001 fixed-size validation and normalization are
constant-time and add no dependency, so its benchmark obligation is also N/A.
REQ-COORD-001 construction and transforms are also constant-bounded for D at
most three and introduce no batch path, so its benchmark obligation is N/A.
These later checks are tracked by requirements and the release checklist.
