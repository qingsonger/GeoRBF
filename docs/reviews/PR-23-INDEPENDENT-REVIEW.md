# Independent Review of PR #23

- Requirement: REQ-KERNEL-004
- Issue: https://github.com/qingsonger/GeoRBF/issues/22
- Pull request: https://github.com/qingsonger/GeoRBF/pull/23
- Branch: `codex/req-kernel-004-wendland-compact-support`
- Reviewed pre-repair head: `b50c036`
- Review-repair commit: `05b74c0`
- Review date: 2026-07-14
- Mode: Independent mathematical and numerical review and repair

## Scope and evidence read

An independent read-only subagent reviewed the complete PR diff, kernel source,
rustdoc, exact-rational and property tests, example, benchmark and baseline,
requirements registry, current progress, Issue #22 acceptance criteria, and
three-platform CI. The review covered formulae, signs, physical dimensions,
SPD classification, center limits, support-boundary smoothness, extreme
floating-point range, Cartesian expansion, allocation, safety, interface
parity, documentation, and scope containment.

The strict-SPD classification and normalized formulas were cross-checked
against Holger Wendland's
[1995 construction](https://doi.org/10.1007/BF02123482). Independent symbolic
expansion checked every first-through-third radial derivative and the direct
`a=phi'/r` and `b=(phi''-a)/r` coefficients. Independent exact-rational and
200-digit decimal calculations checked the embedded ordinary and subnormal
reference values.

The review remained within REQ-KERNEL-004. It did not begin sparse indexing or
assembly, orientation, anisotropy, polynomial construction, functionals,
fields, solvers, schemas, or language bindings.

## Finding repaired

The first review found one merge-blocking test-evidence gap, not a production
formula defect. The existing support test proved exact positive zero at and
beyond `rho`, but did not prove that value and derivatives through order three
approach that branch from the interior for every smoothness member. The
extreme-input test also did not force stored reciprocal powers to underflow
before a complete scaled product returned a representable nonzero subnormal.
Those paths were claimed by Issue #22, the registry, and the progress record.

Repair commit `05b74c0` adds two independent regressions:

1. Exact binary distances `t=2^-4, 2^-8, 2^-12` verify monotone left-limit
   convergence for value and radial derivatives one through three. The tested
   boundary powers are `[4,3,2,1]`, `[6,5,4,3]`, and `[8,7,6,5]` for C2, C4,
   and C6. Dividing by those powers converges to independently derived signed
   constants `[5,-20,60,-120]`, `[56/3,-112,560,-2240]`, and
   `[66,-528,3696,-22176]`.
2. At `rho=1e162`, the test first proves the stored reciprocal square is zero,
   then verifies all three center Hessian coefficients recover the independently
   rounded negative subnormal bit pattern `0x8000000000000004`. At
   `rho=1e108` and `q=1/4`, it proves the stored reciprocal cube is zero, then
   checks independently rounded nonzero bit patterns for `phi'''` and `b`:
   `(0x9,0x7)`, `(0xf,0x9)`, and `(0x13,0xc)` for C2, C4, and C6.

The same independent reviewer re-read the repair, reproduced its leading
constants and subnormal rounding, and reported no remaining or new finding.
The repaired exact-bit tests then passed on Windows, Ubuntu, and macOS, closing
the residual platform-libm risk identified during re-review.

## Mathematical and numerical conclusion

- With `q=r/rho` and `t=(1-q)_+`, the implemented normalized catalog is
  `t^4(1+4q)`, `t^6(1+6q+35q^2/3)`, and
  `t^8(1+8q+25q^2+32q^3)`. Dividing the conventional C4 member by three gives
  center value one and preserves strict positive definiteness.
- Wendland's dimension-three functions are strictly positive definite in
  D=3. Restricting distinct points to embedded one- or two-dimensional
  subspaces preserves strict positive definiteness, supporting the D=1/D=2
  declarations. Deterministic Cholesky checks of sampled Gram matrices are
  regression evidence, not the proof replacing the cited theorem.
- Independent differentiation confirms every supplied radial derivative and
  direct expansion coefficient. D=1 needs no quotient; D=2 and D=3 receive
  direct cancellation-resistant `a` and `b` values.
- The boundary powers above make value and derivatives through third order
  converge to the exact zero branch. At positive separations this includes
  `r=rho` and the exterior, exactly matching the declared away capability.
- C2 expands as `1-10q^2+20q^3-15q^4+4q^5`; its Euclidean center supports
  derivatives through second order with Hessian `-20 rho^-2 I`, while its
  nonzero one-sided radial third derivative is not a spatial third tensor. C4
  and C6 have zero cubic terms and complete third-order center jets with
  Hessians `-(56/3) rho^-2 I` and `-22 rho^-2 I`.
- A center/boundary split forms `t` as `1-q` on the center half and
  `(rho-r)/rho` near the boundary. This avoids sacrificing either extreme
  center-scale or support-edge information. The ordinary direct path is used
  unless the complete factored product is zero or non-finite; the combined-log
  retry preserves representable final subnormals. Genuinely non-finite final
  values return structured errors.
- Construction rejects non-positive or non-finite support radii and reciprocal
  derivative scales that overflow. There is no jitter, tolerance-based branch,
  pseudoinverse, regularization, or user-data adjustment.
- Query derivatives act on `x-y`, and every center derivative contributes one
  minus sign. D=3 tests confirm gradient exchange, Hessian symmetry and mixed
  sign, and third-tensor permutation symmetry.
- Sealed compile-time dimension bounds and compile-fail doctests reject D=0
  and D=4. Tests, examples, Gram matrices, and benchmarks exercise the
  applicable D=1/D=2/D=3 paths.

## Safety, API, allocation, performance, and interfaces

- The core retains `#![forbid(unsafe_code)]`. Scoped scans found no unsafe
  block, panic, placeholder, core output, global mutable state, or dependency
  addition. Invalid input returns structured construction, evaluation, or
  shared-calculus errors.
- A configured `Wendland` contains only immutable validated scalar state, is
  `Copy` and `Send + Sync`, and has no builder lifetime. Value, derivative,
  metadata, radial-jet, and Cartesian-jet paths allocate no collection and use
  no clone or trait-object dispatch. The two `dyn Error` occurrences are the
  standard `Error::source` signatures outside numerical evaluation.
- The benchmark exercises C2/C4/C6 in D=1/D=2/D=3 with fixed arrays and no
  per-iteration allocation. Four full runs retain bit-identical checksums.
  Unpinned timing variation is reported in `benches/REQ-KERNEL-004.md`; no
  cross-machine latency promise is made.
- Rust is the sole applicable surface. CLI selection and project schemas do
  not yet exist; C/C++/Python wait for the reviewed Rust surface and later
  frozen ABI/schema requirements. Their explicit N/A reasons are accurate,
  and no adapter duplicates a formula.
- API, ABI, and schema snapshot tooling is not implemented yet. The registry
  correctly remains `documented`, not `integrated`, until merge and post-merge
  gates pass.

## Verification

Passed locally on Windows with Rust 1.96.1 after the repair:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features` (90 tests)
- `cargo test --doc --workspace` (17 doctests, including fifteen compile-fail
  dimension cases)
- `cargo test -p georbf --release --all-features` (76 integration tests and 17
  doctests)
- strict rustdoc with `RUSTDOCFLAGS=-D warnings`
- `cargo xtask requirements check` (58 requirements)
- `cargo metadata --format-version 1 --no-deps`
- `cargo tree --workspace --duplicates` (no duplicate dependency tree)
- all four benchmark smoke workloads
- four full 1,000,000-iteration Wendland catalog runs with medians of 170.95,
  330.13, and 474.78 ns/iteration in D=1/D=2/D=3 and bit-identical checksums
- all five exercised kernel examples
- scoped forbidden-code and allocation/dynamic-dispatch scans
- `git diff --check`

Pre-review GitHub Actions runs
[29303117570](https://github.com/qingsonger/GeoRBF/actions/runs/29303117570)
and
[29303155305](https://github.com/qingsonger/GeoRBF/actions/runs/29303155305)
passed on Windows, Ubuntu, and macOS. Review-repair run
[29303657839](https://github.com/qingsonger/GeoRBF/actions/runs/29303657839)
passed the same complete three-platform matrix at `05b74c0`, including both new
exact-bit subnormal tests and the four benchmark smoke workloads.

## Checks not run

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. The pinned Rust 1.96.1 toolchain does not provide its Miri
component. Sanitizers, executable fuzz targets, mutation testing, allocation
instrumentation, and API/ABI/schema snapshot tooling are not implemented.
Allocation was established by direct fixed-value path inspection rather than
an unsafe counting allocator. These later gates remain tracked by the
requirement registry and release checklist.
