# REQ-SPIKE-003

Evaluated compact-support spatial indices, canonical CSC assembly, and sparse
factorization candidates and accepted ADR-0012. The decision selects the rstar
0.13 release line for later immutable neighborhood indexing and the faer 0.24
release line for later private checked sparse LLT adoption.

Added an excluded reproducible comparison crate pinned to rstar 0.13.0,
kiddo 5.3.2, faer 0.24.4, sprs 0.11.4, and sprs-ldl 0.10.0. Eight
combined-feature regressions cover brute-force strict-radius truth, exact
support-boundary exclusion, deterministic pair canonicalization, sparse
Wendland C2 symmetry and locality, independent matrix-vector and analytic
solution truth, original-unit residual review, singular rejection, malformed
and nonfinite input, repeated reports, scaling, and kiddo's default-bucket
valid-input panic. CI covers all four minimal index/backend combinations, the
combined configuration, both missing-capability compile failures, and a release
smoke workload.

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
