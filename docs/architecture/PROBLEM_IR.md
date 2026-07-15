# Problem Intermediate Representations

## SemanticProblemIR

The semantic form retains stable observation identifiers, source file and
one-based line, original units, field path, optional constraint group, compiled
functional expressions, relation, enforcement, loss, and execution options.
The Rust type is idiomatically named `SemanticProblemIr<D>` and is available
only for D=1, D=2, and D=3. It rejects empty problems, duplicate observation
identifiers, empty metadata, malformed intervals and cones, invalid soft-loss
parameters, count overflow, and allocation failure without partial success.

The implemented functional layer precedes this IR and preserves only an opaque
stable provenance identifier per expression term. It does not claim that the
identifier is a complete semantic source location. `SemanticProblemIr` maps
each complete relation to richer observation and input provenance without
changing the mathematical atom or inserting a kernel-derivative sign.

Semantic expressions contain an `ObservationFunctional<D>` and a finite
constant. Relations are equality, one- or two-sided linear bound, or
second-order cone. Enforcement is always explicit: hard constraints compile in
this requirement, while soft enforcement retains a positive scale and
SquaredL2, AbsoluteL1, or positive-delta Huber metadata. Soft objective and
epigraph construction is rejected as an unsupported path until its dedicated
requirements and approved backend exist.

## CanonicalProblem

The complete planned numerical form is

```text
minimize 0.5 z^T H z + g^T z
subject to
    A_eq z = b_eq
    lower <= A_lin z <= upper
    ||F_j z + f_j||_2 <= c_j^T z + d_j.
```

`REQ-IR-001` implements the constraint portion of this form. The compiler
accepts an explicit caller linearizer that maps each compiled functional to a
finite sparse affine expression over declared variable blocks. This keeps
kernel, basis, polynomial, and center assembly in later layers. The IR owns all
relation mapping: for `a^T z + q = t` it stores `a^T z = t-q`; linear bounds are
shifted by the same constant; cone expressions retain their affine constants.
The linearizer is never asked to insert relation signs or constants.

`CanonicalProblem` records deterministic variable-block offsets, equality
rows, two-sided linear rows, cones, complete row provenance, explicit identity
scaling, required solver capabilities, and a checked numeric-storage estimate.
It contains no level, horizon, normal, tangent, stratigraphy, lithology, kernel,
or third-party linear-algebra type. Canonicalization validates sparse ordering,
finite nonzero coefficients, variable indices, shifted-scalar overflow,
allocation, and estimate arithmetic before returning an immutable result. It
does not scale, regularize, add jitter or hidden variables, relax constraints,
or select a solver.

Centers and observations remain separate through both forms. Later semantic
compilers and assembly requirements add their own finite-value, unit,
normalization, duplicate/conflict, polynomial-rank, derivative-capability,
anisotropy, gauge, contrast, and operational memory-limit checks before
allocation or solution; this IR does not claim those later validations.

At the pre-IR functional boundary, `ObservationFunctional<D>` and
`CenterRepresenter<D>` remain different Rust types. Semantic IR accepts only
the observation-side wrapper. A caller-provided affine linearizer is the sole
bridge to canonical numeric rows; neither IR chooses centers or assembles a
kernel matrix.
