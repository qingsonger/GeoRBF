# Current GeoRBF Progress

- Current milestone: M1 / v0.1.0 — dimensions, geometry, coordinates,
  orientation, and kernel calculus
- Execution mode: Implement / next atomic requirement
- Current requirement: REQ-ORIENT-001
- Issue: not yet created
- Pull request: not yet opened
- Branch: not yet created

## Completed in this run

- Squash-merged PR #23 as commit `7fdb56f`; Issue #22 closed automatically,
  and post-merge `main` CI run 29303883398 passed on Windows, Ubuntu, and
  macOS with the complete job set. REQ-KERNEL-004 now satisfies every
  integration gate.
- Review-evidence commit `b1a989f` passed GitHub Actions run 29303789259 on
  Windows, Ubuntu, and macOS with the complete job set. PR #23 was then marked
  ready for maintainer review and its title/body were synchronized with the
  final evidence before merge.
- Completed the independent mathematical, numerical, safety, API, allocation,
  performance, interface, documentation, and test review of PR #23. The first
  pass found one merge-blocking test-evidence gap but no production formula
  defect; the complete evidence is recorded in
  `docs/reviews/PR-23-INDEPENDENT-REVIEW.md`.
- Repaired the evidence gap with all-family left-limit checks of the exact
  boundary powers and leading constants through third order, plus independent
  exact-bit second-, third-, and expansion-coefficient regressions after
  stored reciprocal powers underflow. Independent re-review reported no
  remaining or new finding.
- Review-repair commit `05b74c0` passed GitHub Actions run 29303657839 on
  Windows, Ubuntu, and macOS, including 90 workspace tests, 17 doctests, all
  four benchmark smoke workloads, and all 58 requirement checks. This closes
  the re-review's residual cross-platform subnormal-rounding risk.
- Committed the isolated implementation as `122abcd`, pushed
  `codex/req-kernel-004-wendland-compact-support`, and opened Draft PR #23.
  The requirement registry links the PR and records the truthful pre-review
  `documented` state.
- Confirmed clean synchronized `main` at `2d57174`, the correct origin and
  worktree, no tags, no open Issue, PR, review, or failed CI, and green
  three-platform `main` run 29301969855; confirmed REQ-KERNEL-001 is
  integrated, then created Issue #22 and the isolated REQ-KERNEL-004 branch.
- Added normalized Wendland C2, C4, and C6 kernels for D=1/D=2/D=3 with one
  positive coordinate-length support radius, exact positive-zero boundary and
  exterior branches, strict-SPD metadata, and exact center capabilities.
- Added analytic interior value-through-third derivatives and direct stable
  D=2/D=3 expansion coefficients. A center/boundary split forms the support
  coordinate without avoidable cancellation, and complete factored-product
  range classification preserves representable extreme results without
  hidden regularization.
- Added exact-rational truth, independent finite differences, deterministic
  full-Gram Cholesky SPD checks in every dimension, boundary smoothness,
  center limits, coordinate-scale covariance, exchange signs, tensor
  symmetry, pathologies, D=0/D=4 compile failures, and `Send + Sync`
  assertions.
- Added synchronized math and architecture contracts, the original Wendland
  paper citation, rustdoc, a runnable D=3 example, change record, and a
  deterministic allocation-free three-member benchmark with CI smoke
  coverage. Sparse indexing and assembly remain explicitly excluded.
- Four full 1,000,000-iteration benchmark runs retained bit-identical
  checksums. D=1/D=2/D=3 median catalog-workload times were 170.95, 330.13,
  and 474.78 ns/iteration; environment and observed ranges are recorded in
  `benches/REQ-KERNEL-004.md`.
- The final local gate passed formatting, warning-denying workspace Clippy,
  90 workspace tests, 17 doctests, 76 release integration tests, rustdoc with
  warnings denied, all four benchmark smoke workloads, the five exercised
  kernel examples, and all 58 requirement checks. Clippy's initial test-only
  iterator suggestion was repaired before the final pass.
- Squash-merged PR #20 as commit `c887cda`; Issue #19 closed automatically,
  and post-merge `main` CI run 29301786246 passed on Windows, Ubuntu, and
  macOS with the complete job set. REQ-KERNEL-003 now satisfies every
  integration gate.
- Review-evidence commit `4ff343c` passed GitHub Actions run 29301164389 on
  Windows, Ubuntu, and macOS with the complete job set. PR #20 was then marked
  ready for maintainer review.
- Completed the independent mathematical, numerical, safety, API, allocation,
  performance, interface, and test review of PR #20. The complete evidence is
  recorded in `docs/reviews/PR-20-INDEPENDENT-REVIEW.md`.
- Repaired the SPD/CPD evidence gap by replacing single-vector positivity with
  symmetry and independent Cholesky checks of every sampled SPD Gram matrix and
  the full signed-multiquadric `Z^T K Z` constant-zero projection. Added all six
  families to exchange-sign coverage and an extreme MQ derivative regression.
- Corrected stale crate-level rustdoc that still described smooth-global
  kernels as deferred. No production formula defect was found.
- Review-repair commit `0e56498` passed GitHub Actions run 29300983870 on
  Windows, Ubuntu, and macOS, including formatting, Clippy, 78 workspace tests,
  15 doctests, all three benchmark smoke workloads, and all 58 requirement
  checks.
- Committed the isolated implementation as `f829757`, pushed
  `codex/req-kernel-003-smooth-global-kernels`, and opened Draft PR #20. The
  requirement registry now links the PR and records the truthful pre-review
  `documented` state.
- Confirmed implementation-and-registry head `57930eb` passed PR #20 GitHub
  Actions run 29299417381 on Windows, Ubuntu, and macOS with the complete job
  set, including all three benchmark smoke workloads.
- Confirmed handoff-state head `bd57bee` passed PR #20 GitHub Actions run
  29299484831 on Windows, Ubuntu, and macOS with the same complete job set.
- Created Issue #19 with explicit Gaussian, inverse-multiquadric, signed
  multiquadric, and Matérn `1/2`, `3/2`, and `5/2` acceptance criteria after
  confirming clean synchronized `main`, no open Repair or Review work, and an
  integrated REQ-KERNEL-001 dependency.
- Added one-length-scale, global-support Gaussian, inverse multiquadric,
  CPD-order-one signed multiquadric, and three explicit Matérn members for
  D=1/D=2/D=3, with exact SPD/CPD metadata and center capabilities.
- Added analytic positive-radius derivatives and direct stable expansion
  coefficients through third order. Exponential log-domain and rational
  `hypot`/scaled-product fallbacks preserve representable extreme tails rather
  than accepting intermediate zero or infinity as the final result.
- Added 90-digit truth, independent finite differences, deterministic random
  SPD and projected-CPD checks, exact center limits, coordinate-scale
  covariance, exponential and rational extreme cases, exchange signs, tensor
  symmetry, pathologies, D=0/D=4 compile failures, and `Send + Sync`
  assertions.
- Added synchronized math and architecture contracts, source citations,
  rustdoc, a runnable D=3 example, change record, and deterministic
  allocation-free six-member benchmark with CI smoke coverage.
- Four full 1,000,000-iteration benchmark runs retained bit-identical
  checksums. D=1/D=2/D=3 median catalog-workload times were 489.39, 790.00,
  and 913.75 ns/iteration; environment and observed ranges are recorded in
  `benches/REQ-KERNEL-003.md`.
- Squash-merged integration-state PR #18 as commit `4ffdb0d`; final `main` CI
  run 29298112320 passed on Windows, Ubuntu, and macOS, and REQ-KERNEL-002 is
  recorded as integrated.
- Squash-merged PR #17 as commit `68ad3e9`; Issue #16 closed automatically,
  and post-merge `main` CI run 29297902909 passed on Windows, Ubuntu, and
  macOS. REQ-KERNEL-002 now satisfies every integration gate.
- Completed the independent mathematical, numerical, safety, API, allocation,
  performance, and test review of PR #17. The complete evidence is recorded
  in `docs/reviews/PR-17-INDEPENDENT-REVIEW.md`.
- Repaired premature intermediate underflow that silently zeroed
  representable high-power derivatives and D=2/D=3 expansion coefficients.
  Exact odd and logarithmic subnormal bit patterns now prevent regression.
- Added independent `p=2` and `p=4` CPD projection stencils for the fixed
  logarithmic reference-length contract. Four full post-repair benchmark runs
  retained identical checksums with no performance regression.
- Review-repair commit `bc320c5` passed GitHub Actions run 29292921933 on
  Windows, Ubuntu, and macOS, including formatting, Clippy, 68 workspace
  tests, 13 doctests, both benchmark smoke workloads, and all 58 requirement
  checks.
- Review-evidence commit `d217fcd` passed the same complete three-platform
  matrix in GitHub Actions run 29293285706. Issue #16 and PR #17 now record the
  review evidence, and PR #17 is ready for maintainer review.
- Confirmed PR #17 implementation and registry head `8c3c38d` passed GitHub
  Actions run 29291234362 on Windows, Ubuntu, and macOS, including formatting,
  Clippy, 65 workspace tests, 13 doctests, both benchmark smoke workloads, and
  all 58 requirement checks.
- Committed and pushed the isolated implementation as `ed311a4`, opened Draft
  PR #17, and advanced REQ-KERNEL-002 to `documented`. Integration remains
  forbidden until independent review, green CI, and merge are complete.
- Confirmed clean synchronized `main` at `0d0aede`, no open Issue or PR, no
  tags, and green three-platform CI; confirmed REQ-KERNEL-001 is integrated,
  then created Issue #16 and the isolated REQ-KERNEL-002 branch.
- Added CPD-positive integer-power polyharmonic splines for D=1/D=2/D=3 and
  dimension-specific surface splines with `2m>D`, derived power `2m-D`, and
  explicit complete-polynomial CPD order.
- Added analytic value-through-third radial derivatives, direct stable D=2/D=3
  expansion coefficients, exact `min(p-1,3)` center capability, and structured
  construction, radius, center-jet, and non-representable-result errors.
- Added embedded 80-digit truth, independent finite differences, deterministic
  random projected-CPD checks in every dimension, center, exchange-sign,
  tensor-symmetry, pathological-input, compile-fail, and thread-safety tests.
- Added synchronized mathematical and architecture contracts, rustdoc, a
  runnable surface-spline example, change record, deterministic allocation-free
  benchmark, four-run local baseline, and CI benchmark smoke coverage.
- Squash-merged PR #14 as commit `d83e2d2`; Issue #13 closed automatically,
  and post-merge `main` CI run 29289145369 passed on Windows, Ubuntu, and
  macOS. REQ-KERNEL-001 now satisfies every integration gate.
- Completed the independent API, definiteness/CPD-order, dimension,
  capability, parameter, compact-support, allocation, safety, and test review
  of PR #14. The complete evidence is recorded in
  `docs/reviews/PR-14-INDEPENDENT-REVIEW.md`.
- Repaired the ambiguous CPD-order contract by binding order `m` to complete
  polynomial degree `m-1`, made compact support an exact zero extension whose
  boundary smoothness is part of away capability, and corrected the metadata
  complexity declaration.
- Added an independent exhaustive combined-order table, D=0 compile failure,
  and parameter-name and floating-point boundary regressions. Repair commit
  `178bf1c` passed GitHub Actions run 29264468028 on Windows, Ubuntu, and
  macOS, including formatting, Clippy, workspace tests, doctests, benchmark
  smoke, and all 58 requirement checks.
- Review-evidence commit `fb085fa` passed the same complete three-platform
  matrix in GitHub Actions run 29264718581. PR #14 was then marked ready for
  maintainer review; REQ-KERNEL-001 correctly remains `documented` until merge.
- Committed and pushed the isolated implementation as `18b9e6f`, opened Draft
  PR #14, and advanced REQ-KERNEL-001 to `documented`. Integration remains
  forbidden until independent review, green CI, and merge are complete.
- Confirmed PR #14 implementation and registry head `34b84e0` passed GitHub
  Actions run 29262474106 on Windows, Ubuntu, and macOS, including formatting,
  Clippy, workspace tests, doctests, benchmark smoke, and all 58 requirement
  checks.
- Confirmed clean synchronized `main` at `332deeb`, no open Issue or PR, no
  tags, and green three-platform CI; confirmed REQ-DIM-001 and REQ-KCALC-001
  are integrated; created Issue #13 and the isolated REQ-KERNEL-001 branch.
- Added formula-free metadata for strict positive definiteness versus positive
  CPD order, nonempty D=1/D=2/D=3 support sets, hierarchical away/center
  derivative orders, and explicit everywhere/away/unsupported classification.
- Added matrix and query capability calculations that include center-functional
  derivative demand and reject sums beyond third order without widening
  Hessian support.
- Added borrowed, allocation-free parameter definitions with deterministic
  names, unit dimensions, descriptions, finite value constraints, uniqueness,
  and compact-radius reference consistency. The generic `shape_parameter` name
  and zero/non-length compact radii are rejected structurally.
- Added metadata, CPD, dimension, capability, parameter, compact-support,
  compile-fail, and thread-safety coverage; synchronized rustdoc, mathematical
  and architectural contracts, a runnable example, and the change record.
- Squash-merged PR #11 as commit `10f5a4d`; Issue #10 closed automatically,
  and post-merge `main` CI run 29260593600 passed on Windows, Ubuntu, and
  macOS. REQ-KCALC-001 now satisfies every integration gate.
- Confirmed review-repair head `462dca7` passed GitHub Actions run 29259163954
  on Windows, Ubuntu, and macOS, including the benchmark smoke and all 58
  requirement checks. Updated the PR evidence and marked PR #11 ready only
  after the repaired head was green.
- Completed the independent mathematical, numerical, safety, API, allocation,
  benchmark, and test review of PR #11. Repaired catastrophic near-center
  cancellation in mixed third derivatives by requiring stable D=2/D=3 radial
  expansion coefficients, preserved quotient-free D=1 behavior, added
  near-center analytic and rotation-covariance regressions, and recorded the
  evidence in `docs/reviews/PR-11-INDEPENDENT-REVIEW.md`.
- Repeated the full benchmark four times after the repair. D=1/D=2/D=3 median
  times were 36.54, 51.86, and 106.40 ns/iteration with identical checksums;
  the observed ranges and environment are recorded in
  `benches/REQ-KCALC-001.md`.
- Committed and pushed the complete isolated implementation as `c690c73`,
  opened Draft PR #11, and advanced REQ-KCALC-001 to `documented`. Integration
  remains forbidden until the independent review is recorded and the PR is
  merged.
- Confirmed PR #11 head `bcdc736` passed GitHub Actions run 29253691069 on
  Windows, Ubuntu, and macOS, including the new benchmark smoke step. The first
  macOS attempt failed before checkout in runner setup; rerunning the failed job
  completed every repository step successfully.
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

None. REQ-KERNEL-004 is integrated. REQ-ORIENT-001 has no unfinished
dependency and is the smallest remaining P1-ready M1 requirement, ahead of the
larger global-anisotropy implementation.

## Next atomic task

Create the REQ-ORIENT-001 Issue with explicit analytic conversion, convention,
polarity, rotation-invariance, invalid-input, and D=2/D=3 acceptance criteria,
then create its isolated feature branch. Implement only validated geological
orientation conversions to the existing dimension-safe direction types and
synchronize applicable docs, interfaces, diagnostics, examples, and tests. Do
not begin normal/tangent constraints, global anisotropy, fields, assembly, or
solver work in the same requirement.

## Latest full test result

Completed locally on Windows with Rust 1.96.1 on 2026-07-14:

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`:
  passed.
- `cargo test --workspace --all-features`: passed; 90 tests, 0 failures on
  Windows. The Unix matrix additionally runs the non-Unicode argv regression.
- `cargo test --doc --workspace`: passed; 17 doctests, including fifteen
  unsupported-dimension compile-fail cases, 0 failures.
- `cargo test -p georbf --release --all-features`: passed; 76 integration tests
  and 17 doctests, 0 failures.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p georbf --all-features --no-deps`:
  passed.
- `cargo xtask requirements check`: passed; 58 requirements.
- `cargo metadata --format-version 1 --no-deps`: passed.
- `cargo tree --workspace --duplicates`: passed; no duplicates.
- `cargo bench -p georbf --bench radial_kernel_calculus -- --smoke`: passed
  for deterministic D=1/D=2/D=3 workloads. Four full 1,000,000-iteration local
  review runs had medians of 36.54, 51.86, and 106.40 ns/iteration
  respectively with identical checksums; see `benches/REQ-KCALC-001.md` for
  the environment and observed ranges.
- `cargo bench -p georbf --bench polyharmonic_spline -- --smoke`: passed for
  deterministic power-five D=1/D=2/D=3 workloads. Four full post-review
  1,000,000-iteration runs had medians of 135.28, 215.09, and 245.43
  ns/iteration respectively with bit-identical checksums; see
  `benches/REQ-KERNEL-002.md` for the environment and observed ranges.
- `cargo bench -p georbf --bench smooth_global_kernels -- --smoke`: passed for
  deterministic six-family D=1/D=2/D=3 workloads. Four full 1,000,000-iteration
  runs had medians of 489.39, 790.00, and 913.75 ns/iteration respectively
  with bit-identical checksums; see `benches/REQ-KERNEL-003.md`.
- `cargo bench -p georbf --bench wendland_kernels -- --smoke`: passed for
  deterministic C2/C4/C6 D=1/D=2/D=3 workloads. Four full 1,000,000-iteration
  runs had medians of 170.95, 330.13, and 474.78 ns/iteration respectively
  with bit-identical checksums; see `benches/REQ-KERNEL-004.md`.
- `cargo run -p georbf --example radial_kernel_calculus`: passed.
- `cargo run -p georbf --example kernel_metadata`: passed.
- `cargo run -p georbf --example polyharmonic_spline`: passed.
- `cargo run -p georbf --example smooth_global_kernels`: passed.
- `cargo run -p georbf --example wendland_kernels`: passed; the support
  boundary value was exact positive zero.
- Scoped forbidden-pattern and hot-path allocation/dynamic-dispatch scans:
  passed for the Wendland implementation. The only `dyn` occurrences are the
  two standard `Error::source` return types outside numerical evaluation.
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
- REQ-KCALC-001 PR #11 GitHub Actions run 29253691069 for implementation and
  registry head `bcdc736`: passed on `windows-latest`, `ubuntu-latest`, and
  `macos-latest`, including formatting, Clippy, workspace tests, doctests,
  benchmark smoke, and all 58 requirement checks. The initial macOS runner
  setup failure passed on the failed-job retry without a code change.
- Pre-review PR #11 GitHub Actions run 29254116500 for head `70a8339` passed on
  Windows, Ubuntu, and macOS, including formatting, Clippy, workspace tests,
  doctests, benchmark smoke, and all 58 requirement checks.
- Review-repair PR #11 GitHub Actions run 29259163954 for head `462dca7` passed
  on Windows, Ubuntu, and macOS with the same complete job set. PR #11 was then
  marked ready for maintainer review.
- REQ-KCALC-001 post-merge `main` GitHub Actions run 29260593600 for merge
  commit `10f5a4d`: passed on Windows, Ubuntu, and macOS, including formatting,
  Clippy, workspace tests, doctests, benchmark smoke, and all 58 requirement
  checks.
- REQ-KCALC-001 final integration-state `main` GitHub Actions run 29261006123
  for commit `332deeb`: passed on Windows, Ubuntu, and macOS with the complete
  job set.
- REQ-KERNEL-001 Draft PR #14 GitHub Actions run 29262474106 for head
  `34b84e0`: passed on Windows, Ubuntu, and macOS with the complete job set.
- REQ-KERNEL-001 pre-review GitHub Actions run 29262615142 for head `8ffd00c`
  passed on Windows, Ubuntu, and macOS with the complete job set.
- REQ-KERNEL-001 review-repair GitHub Actions run 29264468028 for commit
  `178bf1c` passed on Windows, Ubuntu, and macOS with the complete job set.
- REQ-KERNEL-001 review-evidence GitHub Actions run 29264718581 for commit
  `fb085fa` passed on Windows, Ubuntu, and macOS with the complete job set; PR
  #14 was then marked ready for maintainer review.
- REQ-KERNEL-001 post-merge `main` GitHub Actions run 29289145369 for merge
  commit `d83e2d2` passed on Windows, Ubuntu, and macOS with the complete job
  set.
- REQ-KERNEL-001 final integration-state `main` GitHub Actions run 29289556802
  for commit `0d0aede` passed on Windows, Ubuntu, and macOS with the complete
  job set.
- REQ-KERNEL-002 Draft PR #17 GitHub Actions run 29291234362 for implementation
  and registry head `8c3c38d` passed on Windows, Ubuntu, and macOS with the
  complete job set, including both benchmark smoke workloads.
- REQ-KERNEL-002 review-repair GitHub Actions run 29292921933 for commit
  `bc320c5` passed on Windows, Ubuntu, and macOS with the complete job set.
- REQ-KERNEL-002 review-evidence GitHub Actions run 29293285706 for commit
  `d217fcd` passed on Windows, Ubuntu, and macOS with the complete job set; PR
  #17 was then marked ready for maintainer review.
- REQ-KERNEL-002 final ready-head GitHub Actions run 29293436289 for commit
  `bf58228` passed on Windows, Ubuntu, and macOS with the complete job set.
- REQ-KERNEL-002 post-merge `main` GitHub Actions run 29297902909 for merge
  commit `68ad3e9` passed on Windows, Ubuntu, and macOS with the complete job
  set.
- REQ-KERNEL-002 final integration-state `main` GitHub Actions run 29298112320
  for commit `4ffdb0d` passed on Windows, Ubuntu, and macOS with the complete
  job set.
- REQ-KERNEL-003 Draft PR #20 GitHub Actions run 29299417381 for
  implementation-and-registry head `57930eb` passed on Windows, Ubuntu, and
  macOS with the complete job set, including all three benchmark smoke
  workloads.
- REQ-KERNEL-003 handoff-state GitHub Actions run 29299484831 for commit
  `bd57bee` passed on Windows, Ubuntu, and macOS with the complete job set.
- REQ-KERNEL-003 final pre-review GitHub Actions run 29299576028 for commit
  `624139f` passed on Windows, Ubuntu, and macOS with the complete job set.
- REQ-KERNEL-003 review-repair GitHub Actions run 29300983870 for commit
  `0e56498` passed on Windows, Ubuntu, and macOS with the complete job set.
- REQ-KERNEL-003 review-evidence GitHub Actions run 29301164389 for commit
  `4ff343c` passed on Windows, Ubuntu, and macOS with the complete job set; PR
  #20 was then marked ready for maintainer review.
- REQ-KERNEL-003 post-merge `main` GitHub Actions run 29301786246 for merge
  commit `c887cda` passed on Windows, Ubuntu, and macOS with the complete job
  set.
- REQ-KERNEL-004 Draft PR #23 GitHub Actions runs 29303117570 and 29303155305
  passed on Windows, Ubuntu, and macOS with the complete job set, including all
  four benchmark smoke workloads.
- REQ-KERNEL-004 review-repair GitHub Actions run 29303657839 for commit
  `05b74c0` passed on Windows, Ubuntu, and macOS with the complete job set,
  including the repaired boundary and subnormal-range tests.
- REQ-KERNEL-004 review-evidence GitHub Actions run 29303789259 for commit
  `b1a989f` passed on Windows, Ubuntu, and macOS with the complete job set; PR
  #23 was then marked ready for maintainer review.
- REQ-KERNEL-004 post-merge `main` GitHub Actions run 29303883398 for merge
  commit `7fdb56f` passed on Windows, Ubuntu, and macOS with the complete job
  set.

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. The `cargo-miri` launcher is installed, but the pinned Rust 1.96.1
toolchain does not provide its Miri component. Sanitizers, executable fuzz
targets, mutation testing, allocation instrumentation, and API/ABI/schema
snapshot checks are not yet implemented. A second
full-YAML-parser check was not run because PyYAML, Ruby/YAML, and PowerShell
`ConvertFrom-Yaml` are unavailable; the dependency-free strict registry checker
did run. Stage 0 has no runtime mathematical path, so its benchmark obligation
is explicitly N/A. REQ-DIM-001 fixed-size validation and normalization are
constant-time and add no dependency, so its benchmark obligation is also N/A.
REQ-COORD-001 construction and transforms are also constant-bounded for D at
most three and introduce no batch path, so its benchmark obligation is N/A.
REQ-KERNEL-001 stores borrowed descriptions and performs configuration-time
`O(P^2)` duplicate checks or `O(P)` lookup over short parameter slices, while
fixed derivative, dimension, and support access is constant-time. It adds no
runtime numerical path, so its benchmark obligation is N/A.
These later checks are tracked by requirements and the release checklist.
