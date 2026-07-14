# Independent Review of PR #20

- Requirement: REQ-KERNEL-003
- Issue: https://github.com/qingsonger/GeoRBF/issues/19
- Pull request: https://github.com/qingsonger/GeoRBF/pull/20
- Branch: `codex/req-kernel-003-smooth-global-kernels`
- Reviewed pre-repair head: `624139f`
- Review-repair commit: `0e56498`
- Review date: 2026-07-14
- Mode: Independent mathematical and numerical review and repair

## Scope and evidence read

The review covered `AGENTS.md`, `V1_SCOPE.md`, all 58 entries in
`requirements/v1.yaml`, every file in `docs/math/`, `docs/architecture/`, and
`docs/adr/`, `docs/MASTER_PLAN.md`, `docs/progress/CURRENT.md`, the release
checklist, Issue #19, the complete PR diff and commit list, PR discussion and
review state, source, rustdoc, tests, example, benchmark source and baseline,
and the actual three-platform CI logs. PR #20 had no submitted review or inline
review thread before this review.

The definiteness and scaling audit was cross-checked against Micchelli's
[conditional-positive-definiteness paper](https://pages.stat.wisc.edu/~wahba/stat860public/pdf1/micchelli.interpolation.86.pdf)
and Rasmussen and Williams'
[Matérn definition and half-integer formulas](https://gaussianprocess.org/gpml/chapters/RW.pdf).
An independent 100-digit Python `decimal` calculation reproduced all 36
embedded value, radial-derivative, and expansion-coefficient reference values.

The review remained within REQ-KERNEL-003. It did not begin compact-support
kernels, orientation, anisotropy, polynomial generation, numerical rank
policy, CPD side-condition enforcement, functionals, assembly, fitting,
solvers, constraints, schemas, or language bindings.

## Findings repaired

1. The sampled SPD/CPD test evaluated only one arbitrary quadratic form for
   each random Gram matrix. One positive energy cannot establish that a sampled
   matrix is positive definite on its complete admissible space. The repaired
   test now verifies symmetry and performs an independent scale-aware Cholesky
   review of every 6-by-6 SPD Gram matrix. For the signed multiquadric it uses
   the explicit basis `Z=[I; -1]` of the full constant-zero subspace and factors
   the complete 5-by-5 matrix `Z^T K Z`. Five deterministic point sets in each
   of D=1, D=2, and D=3 cover 75 SPD and 15 projected-CPD matrices. The original
   random-energy check remains as an additional property. The same repair adds
   Matérn `1/2` and `3/2` to the exchange-sign/tensor test and proves that an
   extreme signed-multiquadric first derivative remains representable even
   when its value is not.
2. Crate-level rustdoc still said other kernel families were deferred after
   this PR publicly exported the smooth-global catalog. It now lists the new
   catalog and identifies compact-support kernels, rather than all other
   kernels, as deferred.

No production formula defect was found. No blocking finding remains after the
test and documentation repairs.

## Mathematical and numerical conclusion

- The Gaussian convention is exactly `exp(-r^2/(2 ell^2))`. Direct
  differentiation confirms its three radial derivatives, `a=phi'/r`, and
  `b=(phi''-a)/r`; the center Hessian is `-ell^-2 I` and the odd center tensors
  through order three vanish.
- Micchelli's completely-monotone examples support strict positive
  definiteness of inverse multiquadrics in every dimension. The implementation
  uses the `-1/2` member and its derivatives and direct expansion coefficients
  are algebraically correct.
- Micchelli proves the positive square-root multiquadric is strictly negative
  on the constant-zero subspace. Negating it therefore gives GeoRBF's positive
  projected-energy convention. The declared CPD order is exactly one, so the
  later complete polynomial side space is degree zero, namely constants. The
  repaired `Z^T K Z` test covers the whole sampled side-condition null space,
  not one selected weight vector.
- Rasmussen and Williams define the Matérn argument as
  `sqrt(2 nu) r/ell`, give the `3/2` and `5/2` closed forms, identify `1/2` as
  the exponential member, and show the squared-exponential limit under this
  scaling. Their spectral density is positive throughout frequency space;
  strict positive definiteness at distinct points follows from that positive
  spectrum. This last strictness statement is an inference from the displayed
  spectral density.
- Direct differentiation confirms all three Matérn derivative sets and both
  expansion coefficients. The center expansions give value-only support for
  `1/2`, support through order two with Hessian `-3 ell^-2 I` for `3/2`, and
  support through order three with Hessian `-(5/(3 ell^2)) I` for `5/2`. A
  finite one-sided radial derivative is never promoted to a nonexistent
  Euclidean center derivative.
- D=1 uses no radial quotient. D=2 and D=3 receive direct finite `a` and `b`
  coefficients. Sealed `SupportedDimension` bounds and compile-fail doctests
  reject D=0 and D=4, while truth, Gram, benchmark, and calculus paths exercise
  D=1, D=2, and D=3.
- Query derivatives act on `x-y`; every center derivative contributes exactly
  one minus sign. The repaired exchange test now covers all six family members
  and confirms symmetric Hessians and third tensors.
- Construction rejects invalid physical scales and any reciprocal scale whose
  promised third-order derivative scale is non-finite. Coordinate rescaling
  preserves values and applies the expected inverse power to derivatives and
  expansion coefficients.
- Exponential fallback thresholds are conservative. At Gaussian `q=64` and
  Matérn `t=2048`, even a maximum finite third-order scale and the largest
  supplied polynomial factor leave the combined log magnitude below the
  smallest `f64` subnormal. Returning zero beyond those points cannot erase a
  representable result. Below them, combined-log evaluation rescues
  representable derivatives after a direct exponential underflow. Rational
  paths use `hypot`, bounded `q/h`, and combined logarithms; the repaired MQ
  extreme test distinguishes a non-representable value from its finite first
  derivative.
- This PR makes no rank, condition-number, solver, hard-constraint, or
  infeasibility decision. The Cholesky threshold above is test-only evidence
  for sampled definiteness, not a production rank policy. Complete polynomial
  generation, scale-aware RRQR/SVD review, and null-space enforcement remain
  assigned to later requirements.
- Normal, tangent, polarity, angle-cone, local anisotropy, and fitted-model
  Hessian semantics are also outside this PR. The catalog contains no such
  vocabulary. Its exact SPD/CPD and center metadata lets later compilation
  reject CPD local mixtures and intersect, rather than widen, Hessian
  capability.

## Safety, API, allocation, performance, and interfaces

- The core retains `#![forbid(unsafe_code)]`. Scoped scans found no unsafe
  block, panic, unwrap/expect, placeholder, core output, global mutable state,
  synchronization primitive, or new dependency in the numerical path. User
  input failures return structured construction, evaluation, or shared
  calculus errors.
- Kernel instances contain only validated scalar state, are immutable and
  `Copy`, and compile as `Send + Sync`. Value, derivative, metadata, radial-jet,
  and Cartesian-jet success paths use no collection, clone, trait-object
  dispatch, or heap allocation. The two `dyn Error` occurrences are standard
  `Error::source` signatures outside numerical evaluation. The repaired test's
  owned matrices are test-only.
- The benchmark exercises all six members in D=1/D=2/D=3 with fixed arrays and
  no per-iteration allocation. Four review runs retained bit-identical
  checksums. Unpinned times varied substantially, but this review changed no
  runtime code; the observed timing spread is therefore reported rather than
  presented as a controlled before/after performance claim.
- Rust is the sole applicable surface. CLI kernel selection, C ABI, C++
  wrapper, and Python kernel objects correctly remain N/A until their project
  schemas and frozen adapter contracts exist. No adapter contains a duplicate
  formula. Public Rust types have rustdoc and a runnable example.
- API, ABI, and schema snapshot tooling does not exist yet, so no snapshot
  result is claimed. `requirements/v1.yaml` remains `documented`, not
  `integrated`, until merge and post-merge gates pass.

## Verification

Passed locally on Windows with Rust 1.96.1 after the repairs:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features` (78 passed, 0 failed)
- `cargo test --doc --workspace` (15 passed, including thirteen compile-fail
  cases)
- `cargo test -p georbf --release --all-features` (64 integration tests and 15
  doctests passed)
- `cargo xtask requirements check` (58 requirements)
- strict rustdoc with `RUSTDOCFLAGS=-D warnings`
- `cargo metadata --format-version 1 --no-deps` (five workspace members)
- `cargo tree --workspace --duplicates` (no duplicate dependency tree)
- all three benchmark smoke workloads
- four full 1,000,000-iteration smooth-global runs with medians of 521.29,
  734.47, and 1083.47 ns/iteration in D=1/D=2/D=3 and observed ranges of
  349.52-591.62, 520.28-993.10, and 742.10-1199.52 ns/iteration; checksums
  matched the recorded baseline bit-for-bit
- all four examples and actual CLI success and expected-error paths
- scoped forbidden-code and allocation/dynamic-dispatch scans
- `git diff --check`

Pre-review GitHub Actions
[run 29299576028](https://github.com/qingsonger/GeoRBF/actions/runs/29299576028)
passed for `624139f` on Windows, Ubuntu, and macOS. Review-repair
[run 29300983870](https://github.com/qingsonger/GeoRBF/actions/runs/29300983870)
passed for `0e56498` on all three platforms. Every job passed formatting,
Clippy, 78 workspace tests, 15 doctests, all three benchmark smoke workloads,
and all 58 requirement checks.

## Checks not run

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. The pinned Rust 1.96.1 toolchain does not provide its Miri
component. Sanitizers, executable fuzz targets, mutation testing, allocation
instrumentation, and API/ABI/schema snapshot tooling are not implemented.
Allocation was therefore established by direct fixed-value path inspection
rather than an unsafe counting allocator. These later gates remain tracked by
the requirement registry and release checklist.
