# REQ-CPD-001

Added GeoRBF-owned CPD polynomial rank enforcement for exactly D=1, D=2, and
D=3. Center representers assemble deterministic row-major actions
`Q[j, alpha] = M_j p_alpha` with reusable polynomial scratch and preserved
center/atomic provenance. The internal nalgebra 0.35.0 adapter performs the
accepted eight-pass dimensionless equilibration, column-pivoted RRQR screen,
bounded SVD review, explicit factor-16 ambiguity guard, condition diagnostics,
and clear rank-deficiency failure without coordinate mutation, jitter,
pseudoinverse, or minimum-norm fallback.

Clearly full-rank inputs construct and verify an orthonormal basis for
`null(Q^T)`. Reduced coordinates expand only as provenance-bearing `w = Z y`
weights with a rechecked polynomial side condition. A finite symmetric-energy
helper forms `Z^T K Z` without regularization or premature solver policy.
Independent tests cover polynomial reproduction, null-space residual and
orthogonality, value and directional-derivative action assembly, exact
degeneracy in D=1/D=2/D=3, coordinate-unit and nonzero row-scale invariance,
analytic threshold adjacency, deterministic repeatability, error paths, and
projected/KKT equivalence.

The production dependency re-audit confirms the exact minimal-feature
nalgebra release, permissive 13-package external graph, Rust 1.89 maximum
declared MSRV, no advisories returned by exact OSV/GitHub queries, recorded
unsafe exposure, and a 283,648-byte optimized workload binary. Rust API,
rustdoc, example, diagnostics, benchmark, CI smoke routing, and normative docs
are updated. CLI, C, C++, and Python are N/A because no field/schema/binding
surface exists yet; this internal mathematical layer does not introduce one.
