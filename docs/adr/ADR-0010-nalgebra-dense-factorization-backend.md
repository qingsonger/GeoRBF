# ADR-0010: Nalgebra for Dense Cholesky and Pivoted LBLT

- Status: Accepted
- Date: 2026-07-15
- Requirement: REQ-SPIKE-001

## Context

M3 requires dense solves for SPD equality systems and symmetric-indefinite KKT
systems. The solver contract requires checked Cholesky, stable symmetric
pivoting, explicit failure, bounded iterative refinement, and residual review
in original units. It forbids diagonal substitution, hidden jitter,
pseudoinverse fallback, and handwritten production numerical solvers. A
production backend cannot be selected before correctness, maintenance,
licensing, MSRV, unsafe exposure, platforms, size, deterministic behavior,
performance, and alternatives have been recorded.

The reproducible excluded harness compared nalgebra 0.35.0 and faer 0.24.4
with default features disabled. Both are pure Rust and provide checked LLT plus
Bunch--Kaufman LBLT with symmetric 1-by-1 and 2-by-2 pivots. Both passed the
same analytic SPD solution, mandatory 2-by-2-pivot indefinite solution and
factor inspection, wrong-Cholesky-path, singular-system, ill-conditioned
scaling, bounded one-factorization refinement, overflowed-residual rejection,
finite-input, and deterministic-repeat cases on pinned Rust 1.96.1 for x86_64
Windows MSVC.

## Decision

Select the nalgebra 0.35 release line as the preferred later production backend
for checked dense Cholesky and symmetric-pivoted Bunch--Kaufman LBLT. This
spike pins 0.35.0 only in the excluded comparison crate. It does not expand the
production numerical adapter or add a user-facing solver capability.

Later production adoption must follow this policy:

1. Validate shape, finiteness, symmetry, scaling, units, memory, and rank or
   definiteness expectations before dispatch.
2. Use nalgebra's checked Cholesky entry point for SPD systems. Never call the
   substitute-diagonal or unchecked entry points.
3. Use nalgebra's Bunch--Kaufman LBLT for symmetric-indefinite systems. A zero
   pivot or failed solve is a structured error; do not switch factorization or
   regularize implicitly.
4. Review finite solutions with scaled and original-unit residuals. Reject a
   nonfinite solution or failed residual policy even when factorization returns.
5. If refinement is requested by an explicit later policy, keep it bounded,
   solve corrections with the unchanged matrix and factorization, record every
   accepted step and both residuals, and stop rather than accept a correction
   that does not strictly reduce the original-unit residual.
6. Keep nalgebra matrices and decomposition types private. GeoRBF-owned matrix,
   error, and diagnostic types remain the only public contract.
7. Re-audit the then-current exact patch release, resolved graph, advisories,
   three-platform behavior, and optimized workload before production lock-in.

## Evidence and tradeoffs

All eight independent harness tests passed for the combined and both single-
backend configurations. The indefinite truth case has a zero leading diagonal
and requires a 2-by-2 pivot; checked Cholesky rejects it for both candidates.
The singular inconsistent case is not reported as success. Nalgebra exposes a
fallible LBLT solve and reports a zero pivot directly. Faer's high-level LBLT
constructor and solve are infallible at the type level, so the harness must
reject its singular output through finite and original-unit residual review.

Three consecutive optimized Windows runs covered 32, 64, and 128 square
systems, three iterations each, with one factorization reused for bounded
refinement. Nalgebra had the lower median in every measured Cholesky and LBLT
cell, although the 64-square LBLT ranges overlap. At size 128 its three-
iteration ranges were 0.7985--1.0767 ms for Cholesky and 1.0141--1.3643 ms for
LBLT, compared with faer's 1.4935--1.7552 ms and 1.3904--1.7739 ms. Per-backend
checksums, residuals, and accepted refinement counts were bit-identical across
the three repeats. These small and medium dense measurements are a selection
probe, not a final solver performance promise.

The minimal x86_64 Windows dependency graphs contained 13 external packages
for nalgebra and 41 for faer. Crates.io archives were 396,463 and 1,897,499
bytes; the repaired single-backend release harnesses were 216,064 and 2,692,608
bytes.
All resolved licenses are permissive. The highest declared MSRV was Rust 1.89
for nalgebra's graph and 1.85 for faer's, both below GeoRBF's pinned 1.96.1.

A conservative scan counted Rust source lines containing the word `unsafe`,
not unsafe blocks or proven defects: 2,661 lines across 9 nalgebra-graph
packages and 4,361 lines across 32 faer-graph packages. GeoRBF's core and the
spike both continue to deny unsafe code.

On 2026-07-15, nalgebra 0.35.0 and faer 0.24.4 were current non-yanked releases;
their upstream repositories were active and not archived. An OSV batch query
of every exact selected package found no advisory in the 13-package nalgebra
graph. The 41-package faer graph carried RUSTSEC-2024-0436 for the unmaintained
`paste` transitive dependency; the advisory has no severity and reports
maintenance status rather than a memory-safety vulnerability. `cargo-audit`
and `cargo-deny` were unavailable locally, so this is recorded as an OSV API
query, not as a claim that either tool ran.

## Rejected alternatives

- **faer 0.24.4:** numerically capable and retained as a fallback if later
  large-matrix evidence shows a material advantage. It was slower for every
  measured workload here, had a substantially larger graph and binary, exposed
  no fallible high-level LBLT solve, and included the unmaintained `paste`
  advisory in its selected graph.
- **ndarray-linalg 0.18.1 with LAPACK:** exposes checked Cholesky and
  Bunch--Kaufman-backed symmetric solves, but requires an application-level
  choice among OpenBLAS, Netlib, or Intel MKL. Its documented platform table
  lists only Intel MKL across Linux, Windows, and macOS; OpenBLAS and Netlib add
  compiler, linker, redistribution, and deployment variation unnecessary for
  this portable dense scope. The MKL option also adds its separate license.
- **A handwritten in-repository solver:** rejected. GeoRBF will not implement
  a production Cholesky or symmetric-indefinite factorization internally.
- **Diagonal LDLT without pivoting or general LU:** rejected for the symmetric-
  indefinite KKT contract because they do not provide the required stable
  symmetric 1-by-1/2-by-2 pivot path.

## Consequences

Later M3 dense-solver requirements may use this selection only behind the
private numerical adapter and only after the production re-audit. This ADR does
not integrate a new solver, promise platform verification before the exact PR
head completes ready CI, authorize hidden regularization, or change any Rust,
CLI, C, C++, or Python interface.

## REQ-SOLVE-001 production re-audit

Re-audited on 2026-07-16 before dense-solver adoption. `REQ-CPD-001` had
already pinned nalgebra exactly at 0.35.0 with default features disabled and
only `std` enabled, so this requirement adds no dependency, feature, lockfile,
license, MSRV, or unsafe-audit surface. Crates.io still reported 0.35.0 as the
current release. The package declares Apache-2.0 and Rust 1.89, below the
workspace's pinned Rust 1.96.1. The upstream repository was active on
2026-06-30 and was neither archived nor disabled.

The `georbf` normal graph remains 13 unique external packages including
nalgebra. Every declared license is MIT, Apache-2.0, Zlib, or a permissive
OR-expression, and the highest declared MSRV remains 1.89. An OSV batch query
covered all 19 exact registry packages in the complete workspace resolution
and returned no advisory; GitHub reported no repository security advisory.
`cargo-audit` and `cargo-deny` remain unavailable locally, so those exact API
queries are the performed vulnerability review rather than a claim that the
unavailable tools ran.

The selected backend remains private to `solver.rs`; public matrices,
policies, solutions, diagnostics, and errors are GeoRBF-owned, and the core
continues to deny unsafe code. The optimized Windows 64-by-64 solver benchmark
executable is 301,568 bytes. This is a workload-binary observation, not a
library or final CLI promise. The previously rejected alternatives remain
unchanged: faer expands this dense scope materially, native LAPACK adds
platform and redistribution choices, and handwritten or non-pivoted production
factorizations violate the accepted numerical contract.
