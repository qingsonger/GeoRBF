# Independent Review of PR #11

- Requirement: REQ-KCALC-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/10
- Pull request: https://github.com/qingsonger/GeoRBF/pull/11
- Branch: `codex/req-kcalc-001-radial-kernel-calculus`
- Reviewed pre-repair head: `70a8339`
- Review-repair head: `462dca7`
- Review date: 2026-07-13
- Mode: Independent review and repair

## Scope and evidence read

The review covered `AGENTS.md`, `V1_SCOPE.md`, all 58 entries in
`requirements/v1.yaml`, `docs/math/`, `docs/architecture/`, every accepted
ADR, `docs/MASTER_PLAN.md`, `docs/progress/CURRENT.md`, the release checklist,
Issue #10, the complete PR diff and commit list, PR discussion and review
state, actual three-platform CI jobs, tests, examples, and the benchmark source
and recorded baseline. PR #11 had no submitted review, inline thread, issue
comment, or PR comment before this review.

The review remained within REQ-KCALC-001. It did not begin concrete kernel
families or metadata, SPD/CPD implementation, polynomial spaces, anisotropy,
orientation, functionals, assembly, solvers, persistence, or language
bindings.

## Findings repaired

1. The away-from-center third derivative reconstructed
   `(phi'' - phi'/r)/r` from independently rounded derivatives. For a Gaussian
   at `r = 1e-10`, this changes the mixed Cartesian third derivative from the
   analytic `1.96e-10` to about `2.22e-6`, a four-order-of-magnitude error;
   at other small radii it collapses the component to zero. D=2/D=3 away jets
   now carry finite, cancellation-resistant closed forms for `phi'/r` and
   `(phi'' - phi'/r)/r`. Hessian and third-tensor components use fused
   multiply-add expansion, and a near-center Gaussian analytic regression
   prevents reconstruction from returning.
2. A first version of that repair required the quotient coefficients in every
   dimension. D=1 needs neither coefficient: away derivatives are simply the
   signed radial first and third derivatives and the radial second derivative.
   Pure-derivative away jets therefore remain valid in D=1, while D=2/D=3
   return a structured `MissingRadialExpansionCoefficients` error. A smallest-
   subnormal-radius test with maximum finite derivatives proves that no
   irrelevant quotient is evaluated in D=1.
3. The original suite had exchange and tensor-permutation checks but no direct
   rotation-covariance test required by the repository review contract. A
   three-dimensional Gaussian case now independently rotates the point and
   transforms its gradient, Hessian, and third tensor through one, two, and
   three copies of the rotation matrix.
4. The benchmark had a single-run baseline. The reviewed implementation was
   measured in four full, single-thread runs; the record now reports medians,
   observed ranges, and identical checksums. The repair removes radial
   divisions from the tensor expansion and shows no performance regression.

## Mathematical and numerical conclusion

- With `d = x - y`, `r = ||d||`, and `u = d/r`, the implemented away formulas
  are `grad = phi' u`,
  `H = phi'' uu^T + (phi'/r)(I - uu^T)`, and
  `D3 = phi''' uuu + ((phi'' - phi'/r)/r)
  (delta-u correction)`. Independent expansion of `(d^T d)^3`, Gaussian
  finite differences, the near-center Gaussian Cartesian formula, exact tensor
  symmetries, and rotation covariance agree in D=1, D=2, and D=3.
- Each center-argument derivative contributes exactly one minus sign. Query,
  center, mixed, point-exchange, and third-order permutation cases agree. No
  adapter or later functional layer must insert another sign.
- Coincident points take a separate analytic path and evaluate no quotient:
  value `phi(0)`, zero gradient, `phi''(0) I`, and zero third tensor. Constructing
  a center jet is an explicit smooth-Euclidean-extension promise; it is not a
  universal kernel capability claim.
- Maximum-component-scaled separation keeps the intermediate norm in `[1,3]`
  after scaling, preserves the smallest positive subnormal in D=1, and rejects
  overflowing subtraction or a non-finite final radius. Every stored radial
  derivative, expansion coefficient, and output tensor component is finite or
  construction returns an indexed structured error.
- The dimension seal admits exactly D=1, D=2, and D=3. Compile-fail doctests
  reject D=0 and D=4; focused truth tests cover every supported dimension and
  the nontrivial rotation check covers D=3.
- REQ-KCALC-001 selects no kernel family and exposes no definiteness metadata.
  SPD/CPD classification, complete CPD polynomial spaces, null spaces,
  numerical rank, scaling and condition estimates, hard constraints,
  infeasibility, normals, tangents, polarity, cones, and local anisotropy are
  genuinely outside this PR. No implementation or documentation claims those
  later capabilities.
- `SpatialKernelJet` represents derivatives only after the caller supplies the
  required finite jet data. It does not claim unconditional fitted-model
  Hessian capability; kernel smoothness and observation/center derivative
  demand remain assigned to REQ-KERNEL-001 and later model requirements.

## Safety, API, allocation, and test conclusion

- The core retains `#![forbid(unsafe_code)]`. Direct source scans found no
  panic, unchecked unwrap/expect, placeholder, global mutable state, adapter
  dependency, core output, or new dependency in the reviewed path. Private
  fields prevent safe callers from constructing non-finite geometry, jets,
  coefficients, or tensors.
- Separation, radial coefficients, jets, and D<=3 derivative tensors use only
  fixed arrays and scalar values. Construction and all derivative accessors
  contain no `Vec`, `Box`, trait object, clone, or heap-allocation operation.
  `Box<dyn Error>` and printing occur only in tests, examples, and the benchmark
  executable, outside the core and outside the timed success-path operations.
- Public values are immutable, `Copy` where appropriate, and `Send + Sync`.
  There is no shared mutable state, so concurrent read-only construction and
  access have no thread-safety dependency. Exact symmetry is created by one
  computed representative per tensor orbit.
- Rust is the sole applicable surface. CLI, C, C++, and Python declarations are
  correctly N/A because this internal calculus is not a command, ABI, or
  binding surface. The public Rust additions have rustdoc and a runnable
  example; no API/ABI/schema snapshot tool exists yet, so none is claimed.
- Tests use direct Cartesian polynomial derivatives, independent finite
  differences, analytic Gaussian center behavior, rotations, exchange
  identities, exact tensor permutations, extreme floating-point inputs,
  compile failures, structured errors, and trait assertions. They do not use
  Surfe or another implementation as an oracle.

No blocking mathematical, numerical, safety, API, allocation, benchmark, or
test finding remains after these repairs. REQ-KCALC-001 must remain
`documented`, not `integrated`, until PR #11 is merged.

## Verification

Passed locally on Windows with Rust 1.96.1 after the repairs:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features` (46 passed, 0 failed)
- `cargo test --doc --workspace` (9 passed, including seven compile-fail cases)
- `cargo test -p georbf --release --all-features` (32 integration tests and 9
  doctests passed)
- `cargo xtask requirements check` (58 requirements)
- `RUSTDOCFLAGS="-D warnings" cargo doc -p georbf --all-features --no-deps`
- `cargo metadata --format-version 1 --no-deps` (5 workspace members)
- `cargo tree --workspace --duplicates` (no duplicates)
- `cargo bench -p georbf --bench radial_kernel_calculus -- --smoke`
- four full benchmark runs for D=1/D=2/D=3 with identical checksums
- the radial-kernel-calculus example and actual CLI success/error paths
- scoped forbidden-code/allocation scans and `git diff --check`

Pre-review GitHub Actions run 29254116500 passed for `70a8339` on Windows,
Ubuntu, and macOS. Review-repair run 29259163954 then passed for `462dca7` on
all three platforms. Every job passed formatting, Clippy, workspace tests,
doctests, benchmark smoke, and all 58 requirement checks. PR #11 was marked
ready for maintainer review only after that repaired-head run completed.

## Checks not run

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. The `cargo-miri` launcher exists, but the pinned Rust 1.96.1
toolchain does not provide its Miri component. Sanitizers, executable fuzz
targets, mutation testing, allocation instrumentation, and API/ABI/schema
snapshot tooling are not yet implemented. Allocation was therefore established
by direct fixed-array path inspection rather than an unsafe counting allocator.
