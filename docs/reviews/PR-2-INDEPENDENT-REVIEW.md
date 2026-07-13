# Independent Review of PR #2

- Requirement: REQ-BOOTSTRAP-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/1
- Pull request: https://github.com/qingsonger/GeoRBF/pull/2
- Branch: `bootstrap/specification`
- Review date: 2026-07-13
- Mode: Independent review and repair

## Scope and evidence read

The review covered `AGENTS.md`, `V1_SCOPE.md`, all 58 entries in
`requirements/v1.yaml`, `docs/math/`, `docs/architecture/`, all accepted ADRs,
`docs/MASTER_PLAN.md`, `docs/progress/CURRENT.md`, the PR diff and four commits,
Issue #1, PR metadata and discussion state, GitHub Actions run 29239841099, and
`benches/README.md`. The PR had no submitted review, inline thread, or comment
before this review.

## Findings repaired

1. The local-mixture contract did not prove strict positive definiteness when a
   nominal background weight could vanish. The repair requires a strictly PD
   background with finite, nowhere-zero weight, records an operational lower
   bound for conditioning, gives the Gram quadratic-form proof, and adds
   ADR-0008 plus future regression obligations.
2. D=1 made direction-only and axial normal complements empty and made an
   angular cone independent of its angle. The repair explicitly rejects those
   modes in D=1, preserves meaningful gradient-vector and polarity modes, and
   restricts angular cones to `0 <= theta < pi/2` with finite nonnegative
   `g_min`.
3. Derivative signs and center limits were only test intentions. The repair
   fixes the directional-functional convention, states `grad_x k = -grad_y k`,
   gives radial Hessian signs and analytic `r=0` limits, and requires exchange
   and center-limit tests.
4. CPD rank language did not define the invariants of scale-aware decisions.
   The repair requires recorded dimensionless equilibration, unit and row-scale
   invariance, RRQR/SVD ambiguity evidence, a numerically orthonormal null space,
   and positive definiteness of `Z^T K Z` without jitter.
5. The registry checker allowed an `integrated` requirement to retain
   `benchmark: planned`, accepted unknown fields and malformed inline lists,
   omitted `source_of_truth` enforcement, and over-reported nodes downstream of
   a dependency cycle as cycle members. The checker and ten unit tests now
   cover these failure modes.
6. The stage-0 CLI accepted trailing arguments after `--help` or `--version`
   and used Unicode-only argv iteration. It now rejects extra arguments, uses
   `args_os`, and returns a structured usage error for non-Unicode input.
7. The empty core crate was publishable even though all other Stage-0 packages
   disabled publication. The core now sets `publish = false`, and `xtask`
   rejects any prerelease workspace package that omits the safeguard.

## Checklist conclusion

- Mathematical definitions, signs, derivative limits, D=1/D=2/D=3 semantics,
  SPD/CPD classification prerequisites, polynomial completeness, null-space
  conditions, numerical rank scaling, conditioning, normals, polarity, cones,
  local anisotropy, and Hessian capabilities are internally consistent after
  the repairs above.
- Hard constraints remain hard. No implementation or contract silently adds
  jitter, regularization, a pseudoinverse fallback, constraint deletion, or a
  hard-to-soft conversion. Infeasibility and rank failure remain structured
  errors with provenance.
- Stage 0 contains no kernel, solver, fitted model, hot batch path, FFI symbol,
  C++ wrapper, or Python module. Runtime allocation, thread-safety, numerical
  interface parity, ABI/API snapshots, and mathematical benchmarks are
  therefore N/A for this requirement and remain assigned to explicit later
  requirements. Empty adapter crates expose no false capability.
- Rust production code remains safe and panic lints are enabled. New negative
  tests use independent malformed registry cases and observable CLI behavior;
  future mathematical requirements now name analytic, scale-invariance, SPD,
  center-limit, dimension-boundary, and capability evidence.
- Requirement, architecture, math, ADR, progress, and changelog records are
  updated. REQ-BOOTSTRAP-001 correctly remains `documented` until merge.

No blocking review finding remains in the Stage-0 scope after these repairs.
This conclusion is not a claim that later v1 mathematical capabilities are
implemented or that the open PR is integrated.

## Verification

Passed locally on Windows with Rust 1.96.1:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features` (14 passed, 0 failed)
- `cargo test --doc --workspace` (0 doctests, 0 failed)
- `cargo xtask requirements check` (58 requirements)
- `cargo metadata --format-version 1 --no-deps` (5 workspace members)
- `cargo tree --workspace --duplicates` (no duplicates)
- actual CLI success and usage-error exit paths
- `git diff --check`

The pre-review head `8d6b44e` passed GitHub Actions run 29239841099 on Windows,
Linux, and macOS. The pushed repair must also pass the three-platform matrix
before merge.

## Checks not run

`cargo-nextest`, `cargo-deny`, `cargo-audit`, `cargo-semver-checks`, Miri,
sanitizers, fuzzing, mutation testing, API/ABI/schema snapshot tooling, and
benchmark smoke tooling are not installed or not implemented at Stage 0. A
second general YAML parser was unavailable because PyYAML, Ruby/YAML, and
PowerShell `ConvertFrom-Yaml` are absent. No mathematical benchmark was run
because Stage 0 deliberately has no runtime mathematical path; the registry
records this obligation as N/A rather than as a passed benchmark.
