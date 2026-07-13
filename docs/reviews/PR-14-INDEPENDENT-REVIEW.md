# Independent Review of PR #14

- Requirement: REQ-KERNEL-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/13
- Pull request: https://github.com/qingsonger/GeoRBF/pull/14
- Branch: `codex/req-kernel-001-kernel-metadata-capabilities`
- Reviewed pre-repair head: `8ffd00c`
- Review-repair commit: `178bf1c`
- Review-evidence commit: `fb085fa`
- Review date: 2026-07-13
- Mode: Independent review and repair

## Scope and evidence read

The review covered `AGENTS.md`, `V1_SCOPE.md`, all 58 entries in
`requirements/v1.yaml`, `docs/math/`, `docs/architecture/`, every accepted
ADR, `docs/MASTER_PLAN.md`, `docs/progress/CURRENT.md`, the release checklist,
Issue #13, the complete PR diff and commit list, PR discussion and review
state, actual three-platform CI jobs, source, tests, rustdoc, the example, and
the existing benchmark smoke path. PR #14 had no submitted review, inline
thread, issue comment, or PR comment before this review.

The review remained within REQ-KERNEL-001. It did not begin a concrete kernel
family or formula, SPD/CPD proof, polynomial construction, numerical rank,
orientation, anisotropy, functional, assembly, solver, constraint, schema,
adapter, or compatibility requirement.

## Findings repaired

1. `CpdOrder` rejected zero but its public API did not bind order `m` directly
   to GeoRBF's normative complete polynomial side space of total degree at
   most `m - 1`. That ambiguity could make order one mean either no polynomial
   or constants to a later consumer. Rustdoc and the mathematical contract now
   state `Pi_(m-1)^D`, and `maximum_polynomial_degree()` exposes the mapping
   without an underflow-prone caller subtraction. Regressions cover orders one
   and three and the largest representable order.
2. The derivative model described away support as positive separation, but the
   compact-support contract did not explicitly say whether that included the
   support boundary. `KernelSupport::Compact` now promises the exact zero
   extension for `r >= rho`; the interior one-sided derivatives at `rho` must
   match that extension through `away_through`. Thus sparse assembly cannot
   infer boundary smoothness from compactness alone, and a concrete kernel
   must lower its capability or fail validation if the boundary contract is
   not met.
3. Combined matrix/query demand tests sampled four cases and could miss an
   off-by-one or asymmetric table entry. An independent explicit 4-by-4 truth
   table now checks all 16 order pairs for both query and matrix classification.
   A dedicated `KernelDimensions::supports::<0>()` compile-fail case complements
   D=4, and parameter regressions now cover leading digits/underscores,
   whitespace, non-ASCII names, valid digits, signed zero, the smallest
   positive subnormal, finite extrema, NaN, and both infinities.
4. The requirement registry called all metadata queries constant-time, while
   duplicate validation is `O(P^2)` and name/radius lookup is `O(P)` for `P`
   borrowed definitions. The API and registry now state those costs honestly.
   This does not create a benchmark obligation: family metadata is a short
   configuration-time descriptor and introduces no runtime numerical path.

## Mathematical and API conclusion

- Strict positive definiteness and CPD remain disjoint variants. A CPD order
  is positive and applies in every declared supported dimension. Order `m`
  requires the complete polynomial space of total degree at most `m - 1`;
  construction, term-count overflow and size diagnostics, rank checks, and
  null-space enforcement remain assigned to REQ-POLY-001 and REQ-CPD-001.
- Dimension metadata is a nonempty subset of D=1, D=2, and D=3. The sealed
  typed query admits exactly those dimensions; D=0 and D=4 fail compilation.
- Derivative capability is hierarchical. Center order cannot exceed away
  order. Matrix order is observation plus center-functional order; query order
  is output plus center-functional order. The maximum input sum is six, so the
  `u8` addition cannot overflow, and every sum above third order is classified
  unsupported. The exhaustive independent table agrees with the implementation.
- `SupportedEverywhere` requires the declared derivative at `r=0` and every
  positive separation. `SupportedAwayFromCenters` excludes only `r=0`; for a
  compact kernel it includes `r=rho` and the zero exterior. This metadata does
  not manufacture an origin or support-boundary limit and does not promise a
  fitted-model Hessian; later capabilities must intersect kernel, functional,
  mixture, and transform requirements.
- Parameter names are deterministic ASCII lower snake case, unique per
  family, and cannot use `shape_parameter`. Definitions carry an explicit
  physical dimension, nonempty description, and finite/nonnegative/positive
  scalar domain. Value validation handles signed zero, subnormal values,
  finite extrema, NaN, and infinities without a panic.
- Compact support references an existing strictly positive coordinate-length
  parameter. Missing references, incorrect units, and non-positive domains are
  structured construction errors. This is a static family promise; a future
  configured kernel must still validate its actual radius and formula.

## Safety, allocation, performance, interfaces, and tests

- The core retains `#![forbid(unsafe_code)]`. Direct source scans found no
  unsafe block, panic, unwrap/expect, placeholder, core output, global mutable
  state, synchronization primitive, dynamic dispatch, or new dependency in
  the reviewed module. User-controlled invalid metadata returns structured
  errors.
- Metadata uses scalar values, a three-flag dimension array, borrowed strings,
  and borrowed parameter slices. Construction and access contain no `Vec`,
  `Box`, `String`, clone, collection, or allocation operation. `O(P^2)`
  duplicate checking is explicit; support and name lookup are linear; fixed
  definiteness, derivative, dimension, and support accessors are constant-time.
- Public metadata values are immutable, `Copy` where appropriate, and
  `Send + Sync`. There is no shared mutable state, background work, or
  thread-affine resource.
- Rust is the sole applicable surface. CLI, C, C++, and Python are correctly
  N/A until a concrete kernel object and frozen adapter contract exist. The
  public Rust API has rustdoc and a runnable example. API/ABI/schema snapshot
  tooling is not implemented yet, so no snapshot result is claimed.
- Tests are contract and independent-table tests rather than comparisons to
  another implementation. No concrete radial value exists in this PR, so a
  Gram-matrix SPD/CPD test, support-boundary formula truth, origin limit,
  finite-difference derivative, or performance baseline would be fabricated;
  those remain mandatory for REQ-KERNEL-002 through REQ-KERNEL-004.
- The requirement's benchmark disposition is N/A because this change has no
  runtime numerical path. The existing radial-calculus benchmark smoke still
  passed to prove that workspace benchmark coverage was not disrupted.

No blocking mathematical, safety, API, allocation, performance, interface, or
test finding remains after these repairs. REQ-KERNEL-001 must remain
`documented`, not `integrated`, until PR #14 is merged.

## Verification

Passed locally on Windows with Rust 1.96.1 after the repairs:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features` (56 passed, 0 failed)
- `cargo test --doc --workspace` (11 passed, including nine compile-fail cases)
- `cargo test -p georbf --release --all-features` (42 integration tests and 11
  doctests passed)
- `cargo xtask requirements check` (58 requirements)
- `RUSTDOCFLAGS="-D warnings" cargo doc -p georbf --all-features --no-deps`
- `cargo metadata --format-version 1 --no-deps` (five workspace members)
- `cargo tree --workspace --duplicates` (no duplicates)
- `cargo bench -p georbf --bench radial_kernel_calculus -- --smoke`
- the kernel-metadata example and actual CLI success/error paths
- scoped forbidden-code and allocation/dynamic-dispatch scans
- `git diff --check`

Pre-review GitHub Actions run 29262615142 passed for `8ffd00c` on Windows,
Ubuntu, and macOS. Review-repair run 29264468028 then passed for `178bf1c` on
all three platforms. Every job passed formatting, Clippy, workspace tests,
doctests, benchmark smoke, and all 58 requirement checks. Review-evidence run
29264718581 passed the same complete matrix for `fb085fa`, after which PR #14
was marked ready for maintainer review.

## Checks not run

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. The `cargo-miri` launcher exists, but the pinned Rust 1.96.1
toolchain does not provide its Miri component. Sanitizers, executable fuzz
targets, mutation testing, allocation instrumentation, and API/ABI/schema
snapshot tooling are not implemented. Allocation was therefore established by
direct borrowed/fixed-value path inspection rather than an unsafe counting
allocator.
