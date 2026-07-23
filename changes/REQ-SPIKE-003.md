# REQ-SPIKE-003

Evaluated compact-support spatial indices, canonical CSC assembly, and sparse
factorization candidates and accepted ADR-0012. The decision selects the rstar
0.13 release line for later immutable neighborhood indexing and the faer 0.24
release line for later private checked sparse LLT adoption.

Added an excluded reproducible comparison crate pinned to rstar 0.13.0,
kiddo 5.3.2, faer 0.24.4, sprs 0.11.4, and sprs-ldl 0.10.0. Eight
combined-feature regressions originally covered brute-force strict-radius
truth, exact support-boundary exclusion, deterministic pair canonicalization,
sparse Wendland C2 symmetry and locality, analytic solution truth,
original-unit residual review, singular rejection, malformed and nonfinite
input, repeated reports, scaling, and kiddo's default-bucket valid-input panic.
The SPIKE003-REV-001/002 Repair raises that total to ten: a hand-derived
three-point Wendland case inspects both candidates' actual CSC storage,
storage-level matrix-vector product, and recovered solution, while a benchmark
schema regression requires explicit end-to-end phase labels. CI covers all four
minimal index/backend combinations, the combined configuration, both
missing-capability compile failures, and a release smoke workload.

The repaired solver timings are explicitly end-to-end
triplet/CSC-construction, factorization, solve, residual and analytic review,
and checksum measurements. Index timings are explicitly end-to-end candidate
construction, query, strict filtering, canonicalization, and checksum
measurements. They are dependency-selection evidence and do not claim isolated
factorization speed or query-only performance.

The audit records current releases, maintenance, permissive and LGPL license
differences, declared and missing MSRVs, unsafe and native-code exposure, one
unmaintained `paste` advisory, dependency and binary size, deterministic
Windows scaling, and the complete three-platform Ready-CI strategy.

The production workspace gains no spatial-index or sparse-solver dependency,
sparse adapter, fitted-field path, public matrix type, schema, user API,
regularization, jitter, pseudoinverse, densification, or backend fallback.
Rust, CLI, C, C++, and Python interface dispositions remain N/A for this
dependency spike. Production sparse fitting and evaluation remain in
REQ-SPARSE-001.
