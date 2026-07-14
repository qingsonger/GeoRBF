# REQ-POLY-001

Added complete dimension-generic polynomial spaces for exactly D=1, D=2, and
D=3. Positive CPD order `m` generates every multi-index of total degree at most
`m-1` exactly once in deterministic graded descending-lexicographic order, with
the exact binomial term count and checked count and allocation arithmetic.

The Rust API exposes immutable multi-indices and allocation-free batch value
and Cartesian-gradient evaluation into caller-provided buffers. Derivatives
lower exponents directly without coordinate division, while internal binary
exponent tracking preserves representable mixed monomials across extreme
coordinate scales. Invalid order, degree or term-count overflow, allocation
failure, output mismatch, and non-finite results return structured errors
without partial output, hidden basis scaling, regularization, or coordinate
mutation.

Independent combinatorial, analytic, origin, reproduction, pathological-input,
unsupported-dimension, and thread-safety tests accompany synchronized rustdoc,
a runnable example, and a deterministic D=1/D=2/D=3 generation/evaluation
benchmark. CPD rank diagnosis, null-space construction, functionals, assembly,
fitting, solvers, schemas, persistence, and adapters remain outside this atomic
requirement.
