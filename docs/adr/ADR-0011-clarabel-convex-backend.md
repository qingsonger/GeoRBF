# ADR-0011: Clarabel for Canonical QP and SOCP

- Status: Accepted
- Date: 2026-07-19
- Requirement: REQ-SPIKE-004

## Context

M4 and M5 require convex quadratic objectives, equalities, linear bounds, and
second-order cones without weakening hard constraints or hiding infeasibility.
GeoRBF will not implement a production QP or SOCP interior-point solver. A
backend decision must cover canonical mapping, certificates and statuses,
determinism, maintenance, license, MSRV, unsafe exposure, delivery on Windows,
Ubuntu, and macOS, size, and performance before production dependency lock-in.

The reproducible excluded harness compared
[Clarabel.rs 0.11.1](https://github.com/oxfordcontrol/Clarabel.rs) and
[osqp 1.0.1](https://github.com/osqp/osqp.rs). Both solve the same analytic QP
and return reviewable primal-infeasibility certificates. Clarabel additionally
maps a quadratic objective and a product of zero, nonnegative, and Lorentz
cones directly into its native `A*x + s = b, s in K` form. OSQP natively
supports only linearly constrained QPs in `l <= A*x <= u` form and cannot
represent the required Lorentz cones.

## Decision

Select the Clarabel 0.11 release line as the preferred later production backend
for both canonical QP and SOCP dispatch. This spike pins 0.11.1 only in the
excluded comparison crate. It adds no production numerical dependency, solver,
or user-facing capability.

Later production adoption must:

1. Pin and re-audit the then-current exact patch release behind one private
   GeoRBF-owned convex adapter; no Clarabel matrix, cone, setting, solution, or
   status type may enter a public API.
2. Validate dimensions, finiteness, symmetry and positive-semidefiniteness of
   the quadratic objective, units, scaling, duplicates and conflicts, memory,
   and cone capability before dispatch.
3. Map equalities to zero cones, one-sided linear bounds to nonnegative cones,
   and angular or thickness constraints to explicitly ordered Lorentz cones.
   Preserve a row/cone provenance map for original-unit diagnostics.
4. Accept only a documented successful status after independent objective,
   equality, bound, cone, duality, and original-unit residual review. Reduced-
   accuracy, limit, numerical-error, and insufficient-progress statuses are
   structured failures unless a future explicit policy says otherwise.
5. Independently review every infeasibility certificate in original units,
   including stationarity, dual-cone membership, and a strict separating
   inequality. A backend status alone is not a GeoRBF certificate.
6. Keep presolve and backend KKT regularization disabled unless a later explicit
   policy proves semantic preservation and records every requested setting and
   relevant outcome. Never use solver regularization to change the objective or
   relax a hard constraint.
7. Record requested tolerances, equilibration, refinement, iteration and time
   limits, thread count, backend version, terminal status, iterations, and
   original-unit review. The production adapter must not permit an unrecorded
   fallback or accept hidden automatic changes.
8. Repeat exact-head correctness, certificate, deterministic-repeat, benchmark,
   dependency, advisory, and Windows/Ubuntu/macOS delivery checks before
   production lock-in.

Clarabel 0.11.1 does not compile for this harness with every feature disabled
because an unconditional error variant refers to `serde_json`. The spike
therefore disables defaults and enables only `serde`; production adoption must
recheck whether that upstream limitation still exists. No optional BLAS,
LAPACK, SDP, Python, Julia, or Pardiso path is selected.

## Evidence and tradeoffs

Seven combined-feature tests pass. Both candidates recover the independent QP
solution `(0.5, 1.5)`. Clarabel recovers the independent SOCP solution
`(5, 3, 4)`. Exact primal-infeasible cases cover linearly constrained QP for
both candidates and a Lorentz-cone contradiction for Clarabel. Certificate
reviews check `A^T z`, a strict separator, and applicable dual-cone membership.
Repeated solve reports are bit-identical per backend. Approximate statuses are
rejected, as are nonfinite inputs before dispatch.

The Windows release probe used fixed input, one process, and three iterations.
Across sizes 16, 32, and 64, Clarabel QP totals were 0.3950--0.8912 ms and OSQP
totals were 0.3436--1.1584 ms. The ranges overlap at 16 and 32; Clarabel was
faster in every 64-variable repeat. Clarabel SOCP totals were
0.2874--1.0695 ms. Checksums were bit-identical across repeats for each backend
and size. These small dependency-selection probes are not production
performance promises.

On x86_64 Windows MSVC, the Clarabel-only release harness was 499,200 bytes and
the OSQP-only harness was 349,696 bytes. The exact reachable graphs contained
34 and 7 external Rust packages; their cached crate archives totaled 1,996,202
and 3,480,139 bytes. Every declared license is permissive. The highest declared
MSRV in the selected graphs was Rust 1.71 for Clarabel and 1.65 for OSQP; both
graphs build on GeoRBF's pinned Rust 1.96.1. Some transitive packages omit an
MSRV, so successful pinned-toolchain builds remain the operative evidence.

A conservative scan counted Rust source lines containing the word `unsafe`,
not unsafe blocks or proven defects: 813 lines across 19 packages in the
Clarabel graph and 124 across 6 packages in the OSQP graph. OSQP additionally
builds 126 vendored C/header files containing 39,779 lines through `osqp-sys`;
Clarabel's selected QDLDL path is Rust-only. GeoRBF's own core remains
unsafe-free.

On 2026-07-19 both candidate crates were current, non-yanked crates.io
releases under Apache-2.0. Clarabel's repository had been pushed on 2026-04-13;
the osqp.rs repository on 2025-04-21; neither was archived or disabled. An OSV
batch query of all 41 distinct exact registry packages in both reachable graphs
found no advisory, and both repositories reported no GitHub security advisory.
`cargo-audit` and `cargo-deny` were unavailable locally, so these API queries
are the performed review rather than a claim that either tool ran.

## Rejected alternatives

- **OSQP 1.0.1:** retained as a QP-only fallback if later production evidence
  demonstrates a material need. It has a strong typed certificate API and a
  smaller executable, but it cannot express SOCP constraints and adds a CMake,
  C compiler, FFI, and vendored-C delivery path. Splitting QP and SOCP across
  backends would also duplicate scaling, status, and diagnostic policy without
  evidence of a current benefit.
- **ecos-rs 0.1.0 and scs-rs 0.1.0:** the available Rust bindings declare no
  MSRV, use native solver interfaces, and their repositories were last pushed
  in March and February 2020. They do not offer a stronger current Rust
  delivery or maintenance case than Clarabel.
- **A modeling-layer wrapper:** rejected because GeoRBF already owns the
  canonical IR and requires precise control of cone ordering, provenance,
  statuses, certificates, and diagnostics.
- **An in-repository QP or SOCP solver:** forbidden by the architecture and
  numerical policy.

## Consequences

REQ-CONVEX-001 may use this selection only after the production re-audit and
behind the private adapter described above. It remains responsible for
GeoRBF-owned scaling, provenance, certificate review, memory policy,
diagnostics, and independent truth tests. This ADR does not make QP or SOCP a
user-visible capability, authorize hidden regularization or constraint
relaxation, or claim three-platform verification before the exact ready PR head
passes CI.
