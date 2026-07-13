# CPD Kernels and Polynomial Side Conditions

For a CPD kernel of order `m`, GeoRBF generates the complete polynomial space
of total degree at most `m-1` in D dimensions. Its term count is

```text
binomial(D + m - 1, D).
```

Multi-indices are generated deterministically; no fixed point count or
cubic-only shortcut is permitted.

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
