# Independent Review of PR #17

- Requirement: REQ-KERNEL-002
- Issue: https://github.com/qingsonger/GeoRBF/issues/16
- Pull request: https://github.com/qingsonger/GeoRBF/pull/17
- Branch: `codex/req-kernel-002-polyharmonic-surface-splines`
- Reviewed pre-repair head: `a2cab4b`
- Review-repair commit: `bc320c5`
- Review date: 2026-07-14
- Mode: Independent mathematical and numerical review and repair

## Scope and evidence read

The review covered `AGENTS.md`, `V1_SCOPE.md`, all 58 entries in
`requirements/v1.yaml`, `docs/math/`, `docs/architecture/`, every accepted
ADR, `docs/MASTER_PLAN.md`, `docs/progress/CURRENT.md`, the release checklist,
Issue #16, the complete PR diff and commit list, PR discussion and review
state, source, rustdoc, tests, example, benchmark source and baseline, and the
actual three-platform CI jobs. PR #17 had no submitted review, inline thread,
issue comment, or PR comment before this review.

The sign and CPD-order audit was cross-checked against the generalized
Fourier/conditional-positivity theory in Micchelli's
[1986 paper](https://doi.org/10.1007/BF01893414) and the dimension-dependent
surface-spline construction in Duchon's
[rotation-invariant seminorm paper](https://raw.githubusercontent.com/dill/duchon-typeset/master/Duchon-1977-Splines_minimizing_rotation-invariant_semi-norms.pdf).

The review remained within REQ-KERNEL-002. It did not begin smooth global or
compact kernel families, polynomial construction, numerical rank decisions,
CPD side-condition enforcement, anisotropy, functionals, assembly, solvers,
constraints, schemas, or language bindings.

## Findings repaired

1. The derivative and Cartesian expansion formulas evaluated a bare
   `r.powf(q)` before multiplying by the analytic coefficient. For valid
   high powers this silently erased representable results: at `p=219` and
   `r=2^-5`, the correctly rounded odd-power second derivative is 23 units of
   `2^-1074` and the third derivative is `0x27853` units, while the original
   implementation returned zero because the intermediate radial power had
   already underflowed. The same defect affected the even logarithmic branch
   and the direct `a`/`b` coefficients. The implementation now treats the
   integer power and coefficient as one scaled product for range
   classification: ordinary finite values use `powi`, while only a zero or
   non-finite extreme product is re-evaluated in the log domain. Exact
   subnormal bit-pattern regressions cover odd `p=219` and even `p=1090`.
2. The fixed-reference `log(r)` contract was documented but lacked an
   independent projection test, and an initial review note described the
   added pairwise radial polynomial too loosely as a member of the single-
   point side space. For even `p=2k`, coordinate scaling by `c` adds
   `s_p c^p log(c) ||x-y||^p`. Expansion in `x` and `y` gives total degree
   `2k`, so every monomial has degree at most `k` in at least one argument;
   the degree-`k` CPD moment conditions therefore annihilate its projected
   Gram energy. Independent second- and third-difference stencils now verify
   the expected `c^2` and `c^4` energy scaling for `p=2` and `p=4`, and the
   documentation uses the precise projection statement.

## Mathematical and numerical conclusion

- For integer `p >= 1`, the implemented sign
  `s_p=(-1)^(floor(p/2)+1)` gives the CPD-positive convention: `-r`,
  `r^2 log(r)`, `r^3`, `-r^4 log(r)`, and so on. The declared minimal generic
  CPD order is `floor(p/2)+1`, hence the later side space is the complete
  polynomials through total degree `floor(p/2)` in each supported dimension.
  Deterministic projected Gram tests are positive for powers one through six
  in D=1, D=2, and D=3.
- `SurfaceSpline<D>(m)` accepts exactly `2m>D`, derives `p=2m-D`, supports
  only its compile-time D, and declares the Sobolev side-space order `m`.
  This deliberately can exceed the generic formula's minimal CPD order, as
  for `SurfaceSpline<3>(2)` with `p=1`; positivity is preserved on the smaller
  null space and the surface-spline polynomial null space remains degree
  `m-1`.
- Direct differentiation confirms every odd and even value-through-third
  formula. The expansion coefficients are exactly `a=phi'/r` and
  `b=(phi''-a)/r`, evaluated from closed forms rather than cancellation-prone
  subtraction. Embedded 80-digit values, independent finite differences,
  tensor symmetry, and query/center exchange signs agree.
- The Euclidean center extension exists exactly through order `p-1`, capped
  at the implemented third order. Metadata therefore reports value only for
  `p=1`, first order for `p=2`, second order for `p=3`, and third order for
  `p>=4`. Unsupported complete center jets fail explicitly; a finite one-sided
  radial derivative is not misreported as a spatial center derivative.
- D=1 uses only the radial derivatives and never demands `a` or `b`. D=2 and
  D=3 consume the stable expansion coefficients through the shared Cartesian
  calculus. Both surface-spline compile-time rejection tests and runtime truth
  cases cover the exact supported dimensions.
- Negative and non-finite radii, surface-order overflow, non-finite analytic
  results, non-finite expansion coefficients, and unavailable center jets
  produce structured errors. There is no jitter, regularization, rank
  tolerance, pseudoinverse, hard-constraint softening, or fallback success
  path in this kernel-only change.
- Complete polynomial generation, scale-aware RRQR/SVD rank review,
  condition estimates, null-space enforcement, hard constraints,
  infeasibility, normals, tangents, polarity, angle cones, local anisotropy,
  and fitted-model Hessian capability are outside this PR and are not claimed.
  The kernel metadata is only one future input to a combined Hessian
  capability decision.

## Safety, API, allocation, performance, and tests

- The core retains `#![forbid(unsafe_code)]`. Direct scoped scans found no
  unsafe block, panic, unwrap/expect, placeholder, core output, global mutable
  state, synchronization primitive, or new dependency in the reviewed path.
  Public construction and evaluation handle user input through `Result`.
- Kernel values are immutable fixed scalar state, `Copy`, and `Send + Sync`.
  The value, derivative, metadata, and jet success paths contain no `Vec`,
  `Box`, `String`, collection, clone, trait-object dispatch, or heap
  allocation. Test-only projected Gram construction uses owned matrices and a
  small independent elimination routine; that is not production code.
- The benchmark hot loop uses fixed arrays and exercises value, gradient,
  Hessian, and third derivatives in D=1/D=2/D=3. Four post-repair full runs
  retained bit-identical checksums. Medians of 135.28, 215.09, and 245.43
  ns/iteration remain inside the original unpinned observed ranges; the
  extreme log-domain fallback is not entered by the ordinary power-five
  workload.
- Rust is the sole applicable surface. CLI selection, C ABI, C++ wrapper, and
  Python kernel objects correctly remain N/A until their versioned contracts
  exist. The public Rust API has rustdoc and a runnable example. No API, ABI,
  or schema snapshot tool exists yet, so no snapshot result is claimed.
- Tests use embedded high-precision values, finite differences, exact
  subnormal results derived independently of the evaluator, analytic moment
  stencils, projected CPD properties, center limits, dimension compile
  failures, exchange signs, tensor symmetry, pathologies, and trait
  assertions. They do not use Surfe or another implementation as an oracle.

No blocking mathematical, numerical, safety, API, allocation, performance, or
test finding remains after these repairs. REQ-KERNEL-002 must remain
`documented`, not `integrated`, until PR #17 is merged and post-merge gates
pass.

## Verification

Passed locally on Windows with Rust 1.96.1 after the repairs:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features` (68 passed, 0 failed)
- `cargo test --doc --workspace` (13 passed, including eleven compile-fail
  cases)
- `cargo test -p georbf --release --all-features` (54 integration tests and 13
  doctests passed)
- `cargo xtask requirements check` (58 requirements)
- `RUSTDOCFLAGS="-D warnings" cargo doc -p georbf --all-features --no-deps`
- `cargo metadata --format-version 1 --no-deps` (five workspace members)
- `cargo tree --workspace --duplicates` (no duplicates)
- both benchmark smoke workloads and four full post-repair polyharmonic runs
- the polyharmonic/surface-spline example and actual CLI success paths
- scoped forbidden-code and allocation/dynamic-dispatch scans
- `git diff --check`

Pre-review GitHub Actions run 29291325410 passed for `a2cab4b` on Windows,
Ubuntu, and macOS. Review-repair run
[29292921933](https://github.com/qingsonger/GeoRBF/actions/runs/29292921933)
then passed for `bc320c5` on all three platforms. Every job passed formatting,
Clippy, 68 workspace tests, 13 doctests, both benchmark smoke workloads, and
all 58 requirement checks.

## Checks not run

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. The pinned Rust 1.96.1 toolchain does not provide its Miri
component. Sanitizers, executable fuzz targets, mutation testing, allocation
instrumentation, and API/ABI/schema snapshot tooling are not implemented.
Allocation was therefore established by direct fixed-value path inspection
rather than an unsafe counting allocator.
