# ADR-0012: Rstar and Faer for Compact Sparse Systems

- Status: Accepted
- Date: 2026-07-23
- Requirement: REQ-SPIKE-003

## Context

M7 requires support-neighbor assembly for compact kernels and solution without
densification. GeoRBF must select a deterministic spatial index, a sparse
storage direction, and a mature sparse factorization backend before production
implementation. The decision must preserve strict compact-support boundaries,
hard constraints, finite original-unit review, and explicit failure. It must
also cover correctness, scaling, maintenance, licensing, MSRV, unsafe and
native-code exposure, platforms, size, deterministic behavior, and
alternatives.

The excluded reproducible harness compares rstar 0.13.0 with kiddo 5.3.2 for
fixed-radius D=3 neighborhoods, and faer 0.24.4 with sprs 0.11.4 plus
sprs-ldl 0.10.0 for CSC factorization and solve. All candidates receive the
same strict-support Wendland C2 grid fixture and independent brute-force,
matrix-vector, analytic-solution, and original-unit residual truth.

## Decision

Select the rstar 0.13 release line for later immutable neighborhood indexing,
canonical CSC storage for sparse solver dispatch, and the faer 0.24 release
line for later checked sparse LLT. These exact versions are pinned only in the
excluded comparison crate. No production dependency, sparse adapter, fitting
path, or public API is added by this spike.

Later production adoption must:

1. Validate D=1, D=2, or D=3 points, finiteness, representable separations,
   positive support radii, expected memory, and cancellation before index or
   backend dispatch.
2. Bulk-load an immutable rstar index from stable center identifiers. Treat its
   query result only as a candidate set: independently recompute distance,
   apply the kernel's exact strict support rule, then sort and deduplicate by
   caller-stable row and column identity.
3. Accumulate GeoRBF-owned canonical triplets without a dense intermediate and
   materialize sorted CSC exactly once for the private factorization boundary.
   Record dimensions, stored nonzeros, density, support coverage, ordering,
   and memory evidence.
4. Validate exact symmetry and every finite stored value before checked sparse
   LLT. Reuse a symbolic factorization only when the complete sparsity pattern
   is proven identical. A non-positive pivot, factorization failure,
   nonrepresentable result, or failed residual review is a structured error.
5. Independently review finite solutions in scaled and original units. Any
   iterative refinement introduced later must reuse the unchanged matrix and
   factorization, be explicitly bounded, and accept only strictly improving
   original-unit residuals.
6. Keep rstar and faer types private. Public points, sparse diagnostics,
   policies, errors, systems, and fitted models remain GeoRBF-owned.
7. Use explicit serial execution initially. Do not configure a global thread
   pool or silently accept a caller thread count that the adapter cannot honor.
8. Never add jitter, diagonal substitution, regularization, a pseudoinverse,
   densification, constraint relaxation, or backend fallback to convert failure
   into success.
9. Re-audit the then-current exact patch releases, resolved features,
   advisories, memory behavior, and Windows/Ubuntu/macOS delivery before
   production lock-in.

This decision selects CSC because both evaluated factorization paths consume
column-compressed storage directly and faer's checked sparse LLT exposes a
fallible symbolic and numeric path. GeoRBF may retain row-oriented temporary
assembly state internally, but the solver boundary is canonical CSC and never
a public third-party matrix type.

## Correctness and deterministic evidence

Eight combined-feature tests pass. Both spatial indices reproduce the complete
brute-force pair set, including exact exclusion at the support boundary.
Candidate results are canonicalized before comparison, and repeated pair lists
are identical. The 512-point scaling case retains no more than 14 upper-triangle
pairs per point; the full symmetric fixture retains no more than 27 nonzeros
per point. Sparse matrix-vector truth is formed independently of both storage
libraries.

Both factorization candidates recover the known solution for every tested
finite SPD fixture. The largest recorded original-unit infinity residual is
`3.33066907387546962e-15`, with backward error below `1e-10`. Both reject the
same singular inconsistent system. Nonfinite points, radii, entries, vectors,
and malformed dimensions fail before backend dispatch. Per-candidate pair
lists, solutions, residuals, stored-nonzero counts, and checksums are
bit-identical across repeated runs.

Kiddo's public default `KdTree<f64, 3>` alias panics on the valid 1,000-point
axis-aligned fixture because more than 32 items share one coordinate. The
harness includes an expected-panic regression. Its timed comparison path uses
an explicit leaf bucket of 128 only to cover this fixed fixture. Arbitrary
future user data has no finite compile-time bound on repeated coordinates, so
increasing that constant is not a production safety policy. Rstar bulk-loading
and fixed-radius querying pass the same data without a data-dependent capacity
precondition or panic.

## Scaling and size evidence

Three consecutive optimized Windows runs used one process and three iterations
at 216, 512, and 1,000 points. Candidate pair and nonzero counts were identical.
Rstar index totals were `0.9615--1.0887 ms`, `2.2309--2.7713 ms`, and
`6.5732--8.0464 ms`; kiddo totals were `1.0149--1.3044 ms`,
`2.2709--2.9989 ms`, and `4.8760--6.0248 ms`. Kiddo was faster at 1,000 points,
but that advantage does not offset its valid-input panic and fixed-bucket
precondition.

Faer factor-and-solve totals were `1.6899--2.0256 ms`,
`5.4346--6.6258 ms`, and `9.3249--13.1647 ms`. Sprs/sprs-ldl totals were
`1.3485--1.6228 ms`, `5.2117--6.4961 ms`, and `16.9863--21.7024 ms`.
The small cases overlap or favor sprs; faer is materially faster at the largest
measured case. These measurements select a direction and are not a production
performance promise.

The minimal faer+rstar graph contains 47 external packages, 3,518,941 bytes of
cached crate archives, and a 2,808,832-byte x86_64 Windows release harness. The
sprs+rstar graph contains 25 packages, 1,399,464 archive bytes, and a
262,144-byte harness. Replacing rstar with kiddo yields 55 packages and
3,739,064 archive bytes for faer, or 39 packages and 2,111,761 archive bytes
for sprs. The corresponding binaries are 2,807,808 and 261,632 bytes. These
are workload-binary observations, not final library or CLI size promises.

## Maintenance, license, MSRV, and safety review

On 2026-07-23, faer 0.24.4, rstar 0.13.0, sprs 0.11.4,
sprs-ldl 0.10.0, and kiddo 5.3.2 were non-yanked; the first four were their
crates' current stable releases, and kiddo 5.3.2 was the current stable release
beside a 6.0 alpha. Faer and rstar were released in June and May 2026. Their
repositories were active and not archived. Sprs' repository was active, but
sprs-ldl's current crate was last published in June 2022.

Faer declares MIT and Rust 1.84. Rstar declares MIT OR Apache-2.0 and Rust
1.85. Their selected graph is pure Rust and all declared licenses are
permissive. The highest declared graph MSRV is Rust 1.85, below GeoRBF's pinned
Rust 1.96.1, although 30 transitive packages omit an MSRV. The exact pinned
build is therefore retained as operative evidence.

Sprs declares MIT OR Apache-2.0 and Rust 1.85, but sprs-ldl carries
LGPL-2.1 code derived from Timothy Davis' LDL implementation, including
redistribution and documentation obligations. That license is not selected for
GeoRBF's permissive production core. Kiddo declares MIT OR Apache-2.0 and Rust
1.85, but its default-capacity panic remains a safety blocker independent of
license.

A conservative scan counted Rust source lines containing the word `unsafe`,
not unsafe blocks or proven defects. The faer+rstar graph contains 4,804 such
lines across the complete package sources; the sprs+rstar graph contains
1,548. Neither graph contains C, C++, header, assembly, or native build source
files. The GeoRBF core and the spike both deny unsafe code.

An OSV exact-version batch query covered all 76 external packages reachable in
the combined normal graph. It found only RUSTSEC-2024-0436 for faer's
transitive `paste 1.0.15`; this is an unmaintained-package advisory without a
reported severity, not a memory-safety vulnerability. The rstar, sprs, and
kiddo GitHub repositories reported no repository security advisory. Local
`cargo-audit` and `cargo-deny` are unavailable, so these exact API queries are
the performed advisory review rather than claims that either tool ran.

## Rejected alternatives

- **Kiddo 5.3.2:** numerically reproduces the pair truth and is faster at the
  largest measured index case, but its public default tree panics on valid
  axis-aligned data. A larger fixed bucket only moves the failure boundary and
  cannot establish the user-input safety contract.
- **Sprs 0.11.4 plus sprs-ldl 0.10.0:** correct on the tested SPD and singular
  cases, smaller, and competitive at small sizes. It is rejected because the
  factorization is LGPL-2.1, its current crate dates to 2022, its symmetry path
  contains assertion/panic preconditions, and its largest measured solve is
  slower than faer.
- **Nalgebra-sparse 0.12.0:** offers sparse matrix construction and operations
  but no mature direct sparse factorization satisfying this requirement. It
  would still require a second numerical backend and duplicate storage policy.
- **SuiteSparse bindings:** mature but introduce native compilation, FFI,
  platform packaging, and separate licensing obligations without evidence that
  they are needed for the initial M7 scope.
- **A handwritten index or sparse factorization:** rejected. GeoRBF will not
  implement a production sparse direct solver, and the spike gives no evidence
  that a custom spatial index is justified.

## Consequences

REQ-SPARSE-001 may use this selection only through private GeoRBF-owned
adapters and only after the production re-audit. It remains responsible for
D=1/D=2/D=3 coverage, memory and cancellation policy, canonical sparse
assembly, support diagnostics, dense-sparse parity, solver integration, fitted
model evaluation, and public error design. This ADR does not mark any platform
verified before exact Ready-head CI, add a production dependency, authorize
hidden regularization, or create a public sparse API.
