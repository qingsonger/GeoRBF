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

GeoRBF tests `Q` with scale-aware column-pivoted rank-revealing QR and a bounded
SVD review on every v1 CPD assembly. RRQR is the risk screen; the SVD is the
independent review, and disagreement is itself an ambiguous decision.
Coordinates are never altered to fabricate unisolvency. A deficient system
returns a structured error with the effective rank, scale, and exactly zero
center rows or polynomial columns when present.

Rank decisions are made on a recorded dimensionless equilibration of `Q`. The
implemented numerical adapter applies eight deterministic alternating
infinity-norm row and column equilibration passes and records cumulative
multipliers satisfying `Q_scaled = D_row Q D_column`. Exactly zero rows and
columns remain zero for diagnosis; a nonzero cumulative multiplier that
overflows or underflows is an error, as is any scaling operation that would
round a nonzero matrix entry to zero before the opposite-axis pass can act.
The policy records original and scaled
matrix norms, both thresholds and ranks, the RRQR diagonal, all SVD singular
values, the scaled condition estimate, and the ambiguity band. Changing
coordinate units or multiplying a functional by a nonzero scale must not
change the rank decision.

If bounded SVD review does not converge, no effective rank is reported. The
structured error retains the matrix shapes and norms, equilibration passes and
scales, original zero indices, RRQR diagonal, threshold, and rank; all
SVD-derived fields and the final decision are explicitly unavailable.

The RRQR and SVD thresholds are respectively

```text
tau_qr  = max(row_count, column_count) * eps * max_i(abs(R_ii)),
tau_svd = max(row_count, column_count) * eps * sigma_max.
```

Values strictly greater than their threshold contribute to effective rank.
The closed SVD ambiguity guard band is `[tau_svd / 16, 16 tau_svd]`. A singular
value in that band, or RRQR/SVD rank disagreement, returns a structured
ambiguous-rank error even when one strict comparison says full rank. SVD review
never authorizes a minimum-norm solution of a deficient or marginal hard
system.

Weights obey

```text
Q^T w = 0.
```

A clearly full-rank system constructs a basis `U` for the safely equilibrated
null space using backend QR column-space vectors followed by deterministic
twice-reorthogonalized completion. Because
`Q_scaled = D_row Q D_column`, the basis is mapped back as `z = D_row u` with
product-wise binary-exponent normalization and twice reorthogonalized again to
produce a numerically orthonormal basis `Z` for the original `null(Q^T)`.
Binding verification checks the true matrix infinity
norms (maximum absolute row sums) of column-scaled `Q^T Z` and `Z^T Z - I`
against `64 * row_count * eps`. Original-unit residuals are accumulated from
product-wise binary mantissas and exponents rather than from a column-max
normalization, so neither overflowing products nor a dynamic range wider than
the `f64` exponent range can erase a finite representable residual. An
unrepresentable result is an explicit error rather than a fabricated finite
value. The public API retains center and atomic-functional provenance, and
expanded weights record that they were formed as `w = Z y`; each expansion
rechecks `Q^T w = 0` at the same scaled tolerance. Projected energy is

```text
0.5 y^T (Z^T K Z) y.
```

For a valid CPD kernel and full-rank `Q`, `Z^T K Z` must be positive definite
to the documented scaled tolerance. REQ-CPD-001 validates finite symmetric
input and forms the projection without regularization; positive-definiteness
classification belongs to later field/solver assembly. Failure is never
repaired with jitter.

An equivalent KKT formulation is acceptable when its side condition and
rank behavior match. Tests compare both paths on independent cases and verify
polynomial reproduction in D=1, D=2, and D=3.
