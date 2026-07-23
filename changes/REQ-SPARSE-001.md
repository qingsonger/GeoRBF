# REQ-SPARSE-001

Issue #117 defines the acceptance criteria and exclusions for the first
production compact-support sparse path; Draft PR #118 carries the
implementation for independent Review. The Rust core now assembles the same
hard-equality `FieldProblem<D>` directly into GeoRBF-owned sorted-unique CSC for
exactly D=1, D=2, and D=3, solves it through private checked faer sparse LLT,
and retains one immutable support index inside the existing `FittedField<D>`
for local-center evaluation.

The rstar index stores stable center and functional-term identities. D=1 and
D=2 coordinates are zero-padded into three private index coordinates because
rstar rejects a one-coordinate tree; exact support mathematics remains in the
original compile-time dimension. Candidate hits are independently recomputed
with the stable isotropic or global-anisotropy separation and accepted only for
`radius < support_radius`. Representer pairs are sorted and deduplicated before
kernel action. Assembly reflects one upper value exactly, preserves the shared
canonical hard equalities, validates finite exact symmetry, and materializes
full symmetric CSC without a dense intermediate.

`SparseFitOptions` requires an explicit nonzero memory limit and selected
`FaerLlt` factorization. Diagnostics retain indexed terms, raw candidate hits,
exact supported pairs, exact-support row coverage independent of numeric kernel
zeros, stored nonzeros, density, effective limit, faer 0.24.4, AMD ordering,
and exact original-unit residual evidence. Assembly memory evidence separately
sums the retained index, every reserved canonical relation-vector capacity,
equality row and provenance-string capacities, scaling, CSC, right-hand side,
accepted-pair, reflected-entry, row-buffer, and bulk-load payloads at each
checked stage. Solve evidence adds the complete borrowed retained system,
backend CSC copy, conservative dense lower-triangle factor fill, vectors, and
exact accumulators. The fixed acceptance tolerance is
`128 * dimension * epsilon`. Singular systems, nonrepresentable neighborhoods,
allocation and memory limits, cancellation, unsupported thread counts,
nonfinite solutions, and residual failures are structured errors. No jitter,
regularization, equilibration, refinement, diagonal substitution,
pseudoinverse, densification, hard-to-soft conversion, relaxation, or backend
fallback exists.

The existing fitted-field type gained `try_fit_sparse` and controlled variants;
there is no second scalar-field or model hierarchy. Sparse fitted evaluations
query only local support centers and return visited-versus-total center
evidence. Dense fitted fields continue to evaluate all centers. The retained
index and coefficients remain immutable, cloneable, deterministic, and
`Send + Sync`; third-party point, tree, matrix, and factorization types are not
public.

Nine independent integration tests cover the hand-derived three-point
Wendland C2 CSC and analytic solution; the exact support-boundary structural
zero; dense-sparse coefficient/value/gradient/Hessian parity in D=1/D=2/D=3;
mixed and co-located Value and DirectionalDerivative representers, including
an exact-zero action retained by the support-coverage graph; local-center
evaluation counts; conservative anisotropic candidates with exact support
filtering; repeated complete CSC and diagnostic determinism; 64-point versus
512-point bounded-neighbor growth; exact assembly and solve memory sums with
limits between retained, assembly-peak, and solve-peak payloads; hard canonical
conflict; assembly and solve cancellation; nonrepresentable candidate radius;
and singular-factorization rejection. Existing field, model, and execution
suites remain green. An internal canonical-IR regression independently sums
all relation-vector, equality-term, provenance-string, variable-block, and
scaling capacities, including the three logically empty but reserved relation
buffers, and requires the retained equality-only payload to match.

The production re-audit retains the exact versions and default-disabled
features selected by ADR-0012: rstar 0.13.0 and faer 0.24.4 with `std` and
`sparse-linalg`. Both remained the current stable docs.rs releases on
2026-07-23. The minimal selected graph remains the spike's 47 permissively
licensed pure-Rust external packages, Rust 1.85 maximum declared MSRV, no
native source, and 4,804 conservative source-line matches for the word
`unsafe`; GeoRBF itself continues to forbid unsafe code. The production core's
complete reachable normal graph contains 117 external packages, a maximum
declared MSRV of Rust 1.89, and 55 packages without a declared MSRV; it builds
under pinned Rust 1.96.1. An exact-version OSV batch recheck found only
RUSTSEC-2024-0436 for transitive `paste 1.0.15`, the same unmaintained-package
advisory recorded by ADR-0012, with no reported severity or memory-safety
vulnerability.

The reproducible production benchmark separately times support-index through
CSC assembly, CSC-copy/factor/solve/original-unit review, and immutable local
evaluation. Ready and `main` CI run its release smoke workload on Windows,
Ubuntu, and macOS.

The final stable implementation state passed the complete standard workspace
gate on 2026-07-23: format, warning-denying all-target/all-feature Clippy,
all-feature workspace tests, workspace doctests, and the 58-requirement
registry check. The 64-point release benchmark smoke also passed after the
last production-code change.

Rust is implemented. CLI and versioned schemas are N/A until M8 complete-CLI
and persistence requirements. C, C++, and Python are N/A until the M9 adapter,
parity, and API-freeze requirements; the C++ wrapper depends on the future C
ABI. CPD sparse solving, center selection, parameter tuning, topology,
persistence, and general performance work remain excluded. Independent Review,
Ready-head CI, merge, and isolated integration-state evidence remain required
before the registry status can become `integrated`.
