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

Weights obey

```text
Q^T w = 0.
```

A null-space basis `Z` permits `w = Z y` and projected energy

```text
0.5 y^T (Z^T K Z) y.
```

An equivalent KKT formulation is acceptable when its side condition and
rank behavior match. Tests compare both paths on independent cases and verify
polynomial reproduction in D=1, D=2, and D=3.
