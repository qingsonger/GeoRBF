# ADR-0009: Nalgebra for RRQR Screening and SVD Rank Review

- Status: Accepted
- Date: 2026-07-14
- Requirement: REQ-SPIKE-002

## Context

CPD polynomial actions and later dense assembly require scale-aware rank
diagnosis. The solver policy requires RRQR for rank risk and SVD for rank
review, with no pseudoinverse or hidden regularization. A production numerical
dependency cannot be selected until correctness, scaling, maintenance,
licensing, MSRV, unsafe use, platforms, size, alternatives, and deterministic
behavior have been evaluated.

The reproducible harness compared [faer 0.24.4](https://crates.io/crates/faer/0.24.4)
and [nalgebra 0.35.0](https://crates.io/crates/nalgebra/0.35.0) with default
features disabled. Both provide pure-Rust column-pivoted QR and SVD and both
passed the same analytic-rank, exact-deficiency, near-threshold, diagonal-scale,
finite-input, and repeatability cases on the pinned Rust 1.96.1 Windows build.

## Decision

Select the nalgebra 0.35 release line as the preferred later production
backend for RRQR screening and SVD rank review. This spike pins 0.35.0 only in
the excluded comparison crate; the production `georbf` crate does not yet gain
a numerical dependency. The requirement that implements CPD rank enforcement
must add nalgebra behind an internal solver adapter, repeat the dependency
audit for the then-current patch release, and keep all nalgebra matrix types
out of public geometry and model APIs.

The rank policy evaluated here is:

1. Reject empty, nonfinite, or unrepresentable matrices before backend
   dispatch. Preserve exact zero rows and columns for explicit diagnosis.
2. Apply eight deterministic alternating infinity-norm row and column
   equilibration passes. Record cumulative row and column scales when this
   policy moves into production. Reject both unrepresentable cumulative
   multipliers and any scaling step that rounds a nonzero entry to zero.
3. Form the dimensionless RRQR screen threshold
   `tau_qr = max(m,n) * eps * max_i(abs(R_ii))`.
4. Form the SVD review threshold
   `tau_svd = max(m,n) * eps * sigma_max`. Singular values strictly greater
   than `tau_svd` contribute to effective rank.
5. Treat RRQR/SVD disagreement or a threshold-adjacent singular value as
   review evidence, never as permission to perturb the matrix. The production
   diagnostic must retain both ranks, both thresholds, the equilibration
   scales, matrix norms, condition estimate, and scaled and original-unit
   residuals.
6. Use `SVD::try_new` with a finite, recorded iteration limit and convert
   non-convergence into a structured error. Preserve all completed
   equilibration and RRQR evidence, and mark SVD-derived evidence and the final
   decision unavailable. Do not call nalgebra's pseudoinverse or minimum-norm
   solve as a rank-deficiency fallback.

The exact fixed pass count and threshold multiplier are an initial recorded
policy, not an authorization to hide ambiguous cases. The CPD implementation
must add a threshold-adjacent guard-band diagnostic and independent
high-precision review cases before treating this as production rank logic.

## Evidence and tradeoffs

Metadata and source were audited on 2026-07-14. Nalgebra 0.35.0 declares Rust
1.89 and Apache-2.0; faer 0.24.4 declares Rust 1.84 and MIT. Both are compatible
with GeoRBF's Rust 1.96.1 and MIT distribution. Their repositories and current
releases were active in the preceding two months.

With the minimal features used by the harness, `cargo tree` resolved 14
dependencies for nalgebra and 47 for faer, excluding the spike root. Every
resolved package declared a permissive license. The crates.io archives were
396,463 bytes and 1,897,499 bytes respectively. On x86_64 Windows MSVC, the
same release harness was 232,448 bytes with nalgebra alone and 2,754,048 bytes
with faer alone. These are comparison-harness sizes, not a final GeoRBF binary
promise.

Neither graph is unsafe-free. A conservative source scan for unsafe blocks,
functions, impls, and traits found 2,608 matches across 9 packages in the
nalgebra graph and 4,291 across 35 packages in the faer graph; the top-level
crates accounted for 549 and 1,203 matches. These counts include generated and
test code and are exposure indicators rather than proofs of unsoundness.
GeoRBF's own core remains `unsafe_code = "deny"`; later production adoption
requires version pinning, vulnerability review, the three-platform gate, and
dynamic-analysis coverage when that tooling becomes available.

The three repeated 32, 64, and 128 square-matrix runs showed nalgebra faster at
32 and 64 and overlapping faer at 128. Both candidates produced bit-identical
per-backend checksums across repeats. Nalgebra's materially smaller dependency
and binary footprint, mature maintenance history, complete RRQR/SVD API, and
explicit fallible bounded SVD entrypoint outweigh faer's larger performance-
oriented implementation for the current dense rank-review scope.

## Rejected alternatives

- **faer 0.24.4:** numerically capable and retained as the first fallback if a
  later production benchmark demonstrates a material need. Its minimal graph
  and binary were substantially larger in this spike, and its additional
  performance machinery expands the unsafe audit surface.
- **ndarray-linalg 0.18.1:** offers LAPACK-backed factorizations, but its
  [documented backend selection](https://github.com/rust-ndarray/ndarray-linalg#backend-features)
  introduces native compiler/linker and redistribution choices. Its upstream
  tested-environment table is backend- and architecture-dependent, which is
  unnecessary for the current portable dense scope.
- **linfa-linalg 0.2.1:** pure Rust and small, but its released QR is not column
  pivoted. Its truncated and compact SVD support does not replace the required
  RRQR screen.
- **an in-repository implementation:** rejected. GeoRBF will not implement a
  production QR or SVD backend internally.

## Consequences

REQ-CPD-001 may rely on this selection but must still implement and test the
GeoRBF-owned scaling, thresholds, diagnostics, null-space policy, and error
mapping. This ADR does not integrate a production solver, authorize a
pseudoinverse, or mark the backend as platform-verified before the exact PR
head completes the repository's Windows, Ubuntu, and macOS gate.

## REQ-CPD-001 production re-audit

Re-audited on 2026-07-15 before production adoption. Crates.io still listed
0.35.0 as the only non-yanked 0.35 patch release, with Apache-2.0 licensing and
Rust 1.89 MSRV. The upstream repository was active on 2026-06-30 and was not
archived or disabled. The selected minimal `std` feature remains compatible
with GeoRBF's Rust 1.96.1 and pure-Rust Windows, Ubuntu, and macOS CI strategy.

The production graph resolves 13 unique external packages including nalgebra.
Every declared license is MIT, Apache-2.0, Zlib, or an OR-expression of those
permissive licenses; the highest declared transitive MSRV is 1.89. An OSV batch
query for every exact resolved package/version and GitHub's global and
repository security-advisory APIs returned no known advisory for the graph or
nalgebra 0.35.0. `cargo-audit` and `cargo-deny` remain unavailable locally, so
the exact API queries are recorded as the performed vulnerability review, not
as claims that those tools ran.

A conservative source-line scan found the word `unsafe` on 2,661 Rust source
lines across 9 of the 13 external packages. This is an exposure indicator, not
an unsafe-block count or soundness proof. GeoRBF continues to forbid unsafe in
its own core. The optimized 64-center CPD benchmark executable is 283,648 bytes
on x86_64 Windows MSVC; this workload binary is a size observation, not a
library or final CLI promise.

The earlier alternatives remain unchanged: faer carries a materially larger
graph for this dense scope, native-LAPACK options add platform and
redistribution complexity, linfa-linalg lacks the required released
column-pivoted QR path, and an in-repository QR/SVD implementation remains
forbidden. The re-audit therefore confirms nalgebra 0.35.0 with exact pinning,
default features disabled, and only `std` enabled.
