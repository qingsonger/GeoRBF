# CPD Kernels and Polynomial Side Conditions

For a CPD kernel of order `m`, GeoRBF generates the complete polynomial space
of total degree at most `m-1` in D dimensions. Its term count is

```text
binomial(D + m - 1, D).
```

Multi-indices are generated deterministically; no fixed point count or
cubic-only shortcut is permitted.

The Rust polynomial-space layer accepts the positive CPD order directly and
generates monomials `p_alpha(x)=product_i x_i^alpha_i`. It orders terms by
increasing total degree and then lexicographically from the first Cartesian
axis with larger exponents first. Thus the D=2 terms through degree two are

```text
(0,0), (1,0), (0,1), (2,0), (1,1), (0,2).
```

Values and Cartesian first derivatives are evaluated into caller-provided
storage. The derivative on axis `j` is formed by lowering `alpha_j` directly,

```text
partial_j p_alpha(x) =
    alpha_j x_j^(alpha_j-1) product_(i != j) x_i^alpha_i,
```

so evaluation never divides by a coordinate and remains well-defined on axes
and at the origin. Products track a binary exponent internally so premature
intermediate underflow does not erase a representable mixed monomial. This is
an evaluation method, not a change of basis or hidden polynomial scaling.
Input points already carry the core finite-coordinate invariant. Term-count
overflow, allocation failure, output-length mismatch, and non-finite results
are structured errors; an error leaves caller output unchanged. This layer
applies no coordinate normalization. Later CPD assembly owns the documented
dimensionless equilibration and rank policy.

For center functionals `M_j` and polynomial basis members `p_alpha`, define

```text
Q[j, alpha] = M_j p_alpha.
```

GeoRBF tests `Q` with scale-aware rank-revealing QR and, near a decision
boundary, an SVD review. Coordinates are never altered to fabricate
unisolvency. A deficient system returns a structured error with the effective
rank, scale, and implicated centers.

Rank decisions are made on a recorded dimensionless equilibration of `Q`.
Row scales come from center-functional units and norms; polynomial columns are
scaled after coordinate normalization. Zero scales are errors. The numerical
policy records the equilibration, matrix norm, threshold, RRQR diagonal, SVD
singular values when reviewed, effective rank, and the ambiguity band. Changing
coordinate units or multiplying a functional by a nonzero scale must not
change the rank decision. SVD review diagnoses a marginal decision; it does not
authorize a minimum-norm solution of a deficient hard system.

Weights obey

```text
Q^T w = 0.
```

A numerically orthonormal null-space basis `Z` permits `w = Z y` and projected
energy

```text
0.5 y^T (Z^T K Z) y.
```

For a valid CPD kernel and full-rank `Q`, `Z^T K Z` must be positive definite
to the documented scaled tolerance. Failure is reported as a classification,
assembly, or rank error; it is not repaired with jitter.

An equivalent KKT formulation is acceptable when its side condition and
rank behavior match. Tests compare both paths on independent cases and verify
polynomial reproduction in D=1, D=2, and D=3.
