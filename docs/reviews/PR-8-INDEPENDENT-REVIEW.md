# Independent Review of PR #8

- Requirement: REQ-COORD-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/7
- Pull request: https://github.com/qingsonger/GeoRBF/pull/8
- Branch: `codex/req-coord-001-coordinate-metadata-normalization`
- Reviewed pre-repair head: `634792b`
- Review date: 2026-07-13
- Mode: Independent review and repair

## Scope and evidence read

The review covered `AGENTS.md`, `V1_SCOPE.md`, all 58 entries in
`requirements/v1.yaml`, `docs/math/`, `docs/architecture/`, all accepted ADRs,
`docs/MASTER_PLAN.md`, `docs/progress/CURRENT.md`, the release checklist,
Issue #7, the complete PR diff and commit list, PR discussion and review state,
the three-platform CI logs, and the applicable test and benchmark obligations.
PR #8 had no submitted review, inline thread, or comment before this review.

The review remained within REQ-COORD-001. It did not begin geological
orientation conversion, anisotropy, kernel calculus, fitting, schemas, batch
APIs, or language bindings.

## Findings repaired

1. Unscaled Gauss-Jordan inversion could reject a finite matrix whose inverse
   was finite and representable because a normalized augmented row overflowed
   before the finite result was formed. The repair first attempts
   max-component row/column equilibration and partial-pivot LU solves. An
   explicit unscaled fallback preserves the original classification when the
   equilibrated representation cannot remain in the finite nonzero domain.
   Neither path adds a tolerance, jitter, regularization, pseudoinverse, or
   dependency.
2. The shared test tolerance used an absolute scale floor of one. Consequently,
   expected inverse entries such as `1 / f64::MAX` compared equal to zero and
   could not detect underflow regressions. Extreme inverse tests now use a pure
   relative tolerance and exact-zero checks. New analytic D=2 and D=3 cases
   cover spurious intermediate overflow, coupled extreme scales, and the
   unscaled fallback.
3. Metadata compatibility tests covered length unit and CRS but not axis order,
   vertical direction, handedness, or angle unit. A table-driven regression now
   verifies the structured mismatch field for every remaining convention.
4. The coordinate contract referred to a vertical axis without defining it
   for every supported dimension. Rustdoc and normative documents now define
   canonical X, X/Y, and X/Y/Z axes and make the final canonical axis vertical;
   `AxisOrder` identifies its stored component.

## Mathematical and numerical conclusion

- The normalization and inverse point formulae are consistent:
  `x_tilde = S^-1 (x - mu)` and `x = mu + S x_tilde`.
- The derivative chain rule is correct for a general, not necessarily
  symmetric, `S`: `g = S^-T g_tilde` and
  `H = S^-T H_tilde S^-1`. D=1, D=2 shear/rotation, and D=3 analytic cases
  provide independent truth. Translation correctly does not enter derivative
  transforms.
- The constructor retains exact zero-pivot decisions. Equilibration changes
  only the numerical representation and the documented fallback prevents an
  equilibration artifact from deciding the result. Near-singular but
  representable input remains accepted; singular and non-finite states remain
  structured errors.
- Axis permutations and the vertical-axis convention are dimension-gated for
  exactly D=1, D=2, and D=3. Compile-fail doctests reject unsupported
  dimensions.
- SPD/CPD classification, polynomial spaces, null spaces, kernel center limits,
  numerical rank, hard constraints, infeasibility, normals, tangents, polarity,
  cones, anisotropy, and solver conditioning are not introduced by
  REQ-COORD-001. The PR exposes no such capability.
- `hessian_to_original` is an algebraic coordinate conversion, not an
  unconditional fitted-model Hessian capability claim. Kernel and
  observation-order capability checks remain assigned to later requirements.

## Safety, API, allocation, and test conclusion

- Core production code remains `#![forbid(unsafe_code)]`, has no panic path on
  user input, global mutable state, adapter dependency, placeholder, or new
  dependency. Private fields preserve every validated invariant and errors
  identify invalid matrix or metadata state without printing.
- `LengthUnit` and optional WKT metadata own strings and therefore allocate only
  when such metadata is constructed or explicitly cloned. Axis metadata and
  affine transforms use fixed arrays. Construction and every point, gradient,
  and Hessian transform contain no heap allocation; this is established by
  direct path inspection without introducing an unsafe counting allocator.
- `AffineNormalization<D>` is immutable and `Copy`; coordinate values are
  `Send + Sync`. Construction remains O(D^3), operations O(D^2), and D is at
  most three. No batch hot path exists, so the benchmark obligation is
  correctly N/A rather than recorded as passed.
- Rust is the sole applicable interface. CLI, C, C++, and Python declarations
  are genuine N/A for this atomic requirement because project schemas and
  frozen binding surfaces do not yet exist. No adapter exposes a false
  coordinate capability.
- Tests use analytic inverses, analytic polynomial derivative transforms,
  round trips, invariants, exact metadata comparisons, compile failures, and
  observable error variants rather than another implementation or proprietary
  output.

No blocking mathematical, numerical, safety, API, allocation, performance, or
test finding remains after these repairs. REQ-COORD-001 must remain
`documented`, not `integrated`, until PR #8 is merged.

## Verification

Passed locally on Windows with Rust 1.96.1 after the repairs:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features` (35 passed, 0 failed)
- `cargo test --doc --workspace` (7 passed, including five compile-fail cases)
- `cargo test -p georbf --release --all-features` (21 integration tests and 7
  doctests passed)
- `cargo xtask requirements check` (58 requirements)
- `RUSTDOCFLAGS="-D warnings" cargo doc -p georbf --all-features --no-deps`
- `cargo metadata --format-version 1 --no-deps` (5 workspace members)
- `cargo tree --workspace --duplicates` (no duplicates)
- the coordinate-normalization example and actual CLI success/error paths
- forbidden-code scan and `git diff --check`

Pre-review GitHub Actions run 29248171268 passed for `634792b` on Windows,
Ubuntu, and macOS. Each job passed formatting, Clippy, workspace tests,
doctests, and all 58 requirement checks. The repair commit must pass the same
matrix before Draft PR #8 is marked ready.

## Checks not run

`cargo-nextest`, `cargo-deny`, `cargo-audit`, `cargo-semver-checks`, Miri,
sanitizers, fuzzing, mutation testing, and API/ABI/schema snapshot tooling are
not installed or not yet implemented. The pinned Rust toolchain does not offer
the Miri component. No benchmark was run because fixed-size D<=3 construction
and transforms are constant-bounded and no batch path exists; the requirement
records the obligation as N/A.
