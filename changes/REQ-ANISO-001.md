# REQ-ANISO-001

Added immutable fixed-size global anisotropy for D=1, D=2, and D=3 using the
documented `r_A(x,y)=||A(x-y)||` convention. Isotropic, spheroidal,
principal-axis ellipsoidal, arbitrary-transform, and exactly symmetric SPD
user-metric constructors expose the transform, inverse, metric, singular
values, and Euclidean condition number.

Spheroidal construction uses an orthonormal-frame factor rather than subtracting
large inverse-scale projectors, preserving representable high axis ratios.
Power-of-two congruence equilibration and exact-sign floating expansions certify
the leading principal minors of user and transform-derived metrics before a
metric is exposed; boundary-indefinite input and rounded singular `A^T A`
matrices are rejected explicitly. Exact expansion signs, rather than rounded
square-root bounds, decide determinant boundaries so valid near-singular SPD
inputs remain accepted.

Validation rejects non-finite or nonpositive lengths, unrepresentable
reciprocals and products, nonorthogonal ellipsoid axes beyond an explicit
caller tolerance, nonsymmetric or non-SPD metrics, singular transforms, and
explicitly policy-rejected condition numbers. Construction performs no jitter,
symmetrization, clipping, regularization, pseudoinverse, or hidden rank or
condition decision.

Point pairs are transformed as `A(x-y)` before stable radial evaluation. The
full original-coordinate chain rule through third order returns the existing
spatial jet type and preserves query/center signs, center limits, and exact
tensor permutation symmetry. Independent analytic, rotation, scaling, SPD,
extreme-value, compile-fail, and error-path tests accompany a runnable example
and deterministic allocation-free benchmark.

Local anisotropy, orientation-tensor estimation, SPD mixtures, observations,
assembly, fitted fields, solvers, schemas, persistence, and language adapters
remain outside this atomic requirement.
