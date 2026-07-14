# Core Mathematical Specification

## Domain and dimensions

The sole mathematical product is a scalar field

```text
f: R^D -> R, D in {1, 2, 3}.
```

Public APIs admit exactly those three dimensions through a sealed compile-time
dimension bound. D=0 and arbitrary unverified dimensions are invalid. A project
may contain several independent fields; this does not create several core
algorithms.

## Geometry primitives

`Point<D>`, `Vector<D>`, `Direction<D>`, and `UnitDirection<D>` exist only for
`D` in `{1, 2, 3}`. Every stored component is a finite `f64`; constructors
reject NaN and either infinity with the zero-based component index. A zero
`Vector` is valid, while `Direction` and `UnitDirection` reject a vector whose
components are all positive or negative zero. Their representation is private,
so safe callers cannot bypass these invariants.

`Direction` preserves the finite, nonzero magnitude supplied by the caller.
For its components `v_i`, `UnitDirection` computes

```text
s   = max_i |v_i|
q_i = v_i / s
n   = sqrt(sum_i q_i^2)
u_i = q_i / n.
```

The direction invariant guarantees `s > 0`. At least one `|q_i|` is one and
every `|q_i| <= 1`, so forming the norm cannot overflow or collapse to zero in
the supported dimensions. The stored `u` has Euclidean norm one up to floating
point roundoff. This requirement assigns no physical units, coordinate axes,
CRS, or orientation semantics; those are introduced by later requirements.

## Field representation

For center representers `M_j` at centers `c_j`, weights `w_j`, polynomial basis
`p`, and coefficients `beta`, a fitted field is

```text
f(x) = sum_j w_j M_j^(y) k(x, c_j) + p(x)^T beta.
```

An observation functional `L_i` produces the matrix action

```text
A_ij = L_i^(x) M_j^(y) k(x_i, c_j).
```

`DirectionalDerivative(x, u)` means `u^T grad_x` for a validated unit
direction `u`; the same definition is applied in the center variable for
`M_j^(y)`. No adapter or assembly path inserts an additional sign. For a
stationary symmetric kernel, this convention gives

```text
grad_x k(x, y) = -grad_y k(x, y).
```

Kernel calculus must use analytic center limits rather than evaluate formulas
containing `1/r` at `r=0`. Exchange-identity tests cover value/derivative and
derivative/derivative actions, and an all-representer matrix must be symmetric
to its scale-derived numerical tolerance.

For a stationary radial kernel, calculus consumes an explicit finite radial
jet rather than selecting a kernel family. Away from the center it expands
`phi`, `phi'`, `phi''`, and `phi'''` into fixed-size Cartesian value, gradient,
Hessian, and third-derivative tensors. In D=2 and D=3 the away jet also carries
stable closed forms for `phi'/r` and `(phi'' - phi'/r)/r`; the calculus does not
reconstruct the second coefficient from nearly equal rounded derivatives near
the center. D=1 needs neither radial quotient. A center jet instead promises a
smooth Euclidean extension and expands directly to value `phi(0)`, zero
gradient, `phi''(0) I`, and a zero third tensor. A center-argument derivative
contributes exactly one minus sign, independent of derivative-axis ordering.

Observations and center representers are separate collections. This distinction
is retained even when an all-representer strategy initially gives them the same
points and functionals.

The only v1 atomic functionals are value at a point and directional derivative
at a point. A finite linear expression of atoms represents differences,
coordinate derivatives, directional gradients, tangents, normal complements,
monotonicity, and polarity.

The implemented atom stores a validated finite `Point<D>`, a validated
`UnitDirection<D>` for a directional derivative, and an opaque stable caller
provenance identifier. A `FunctionalExpr` is a nonempty insertion-ordered list
of finite coefficient/atom pairs. Applying an expression to caller-supplied
field samples requires one explicitly aligned finite value/gradient sample per
term. Polynomial action follows the deterministic complete-basis order. Kernel
action requests the exact combined derivative order (zero, one, or two) for
each observation-term/center-term point pair and contracts a Cartesian jet
prefix with zero or one unit direction on each side. A complete jet supplies
the same prefix through second order; coincident kernels may instead provide
only the analytic center prefix their declared capability supports. No missing
higher derivative is fabricated. Every coefficient product and sum must remain
finite; failure reports the originating term provenance rather than returning
a partial value.

`ObservationFunctional<D>` and `CenterRepresenter<D>` are distinct types. They
may contain equal expressions, but no API silently converts one role into the
other. These wrappers carry no relation, enforcement, geological meaning,
canonical row, center-selection policy, or solver behavior.

## Coordinates

Coordinate metadata records an exact length-unit identifier, opaque optional
EPSG and WKT values, a permutation from component positions to canonical axes,
the positive vertical direction, handedness, and the external angle unit.
Canonical axes are X in D=1, X/Y in D=2, and X/Y/Z in D=3; the last canonical
axis is vertical, and the axis permutation identifies its stored component.
Metadata is preserved and compared exactly. The core performs no unit aliasing,
automatic unit conversion, CRS lookup, or reprojection; data with different
metadata must be converted explicitly before it is combined. Internal angular
mathematics uses radians.

Geological orientation conversion exists only in D=2 and D=3 and keeps planar
unit normals distinct from linear unit directions. Each value preserves
positive, negative, or unknown/axial polarity metadata. Degree or radian input
is validated before conversion. In the canonical right-handed D=3 local frame,
X is easting, Y is northing, Z is up, compass azimuth `a` is clockwise from +Y,
dip `d` is nonnegative from horizontal, and signed plunge `p` is positive
downward. The positive reference vectors are

```text
n(a,d) = [sin(d) sin(a), sin(d) cos(a),  cos(d)]
l(a,p) = [cos(p) sin(a), cos(p) cos(a), -sin(p)].
```

Right-hand-rule strike maps to `a = strike + pi/2 (mod 2pi)`. The D=2
vertical-section analogues are `[sin(d), cos(d)]` for a plane normal and
`[cos(p), -sin(p)]` for a lineation. Orientation construction does not create
an observation, constraint, or nonzero-gradient claim. Noncanonical axes or
vertical signs require an explicit input conversion.

Fitting uses normalized coordinates

```text
x_tilde = S^-1 (x - mu),
```

where `S` is invertible and carries the coordinate scaling policy. The model
stores finite `mu` and finite `S`; `S^-1` must also be representable with finite
components. `S` is a general matrix and may contain scaling, rotation,
permutation, or shear. Construction first attempts max-component row/column
equilibration and partial-pivot LU solves. This scaling changes only the
numerical representation. If equilibration or its solve would leave the finite
nonzero `f64` domain, construction retries unscaled partial-pivot elimination
so an equilibration artifact alone cannot reject the original matrix. Both
paths use exact zero-pivot decisions. Neither adds a singularity tolerance,
jitter, regularization, or pseudoinverse. Singular matrices and inverses that
cannot be produced with finite components are explicit errors.

The inverse point transform is

```text
x = mu + S x_tilde.
```

With `g_tilde` and `H_tilde` evaluated in normalized coordinates,
original-coordinate derivatives are

```text
g = S^-T g_tilde
H = S^-T H_tilde S^-1.
```

Finite inputs that overflow or otherwise produce a non-finite point, gradient,
or Hessian are rejected rather than stored. These transforms introduce no
orientation semantics or anisotropy metric; later requirements build those
concepts on this coordinate contract.

## Global anisotropy

A fixed global anisotropy acts on the original point-pair displacement as

```text
z = A(x-y),
r_A(x,y) = ||z||,
B = A^T A.
```

Only D=1, D=2, and D=3 exist. `A` is finite with a finitely representable
inverse, and the stored finite `B` is SPD. Isotropic, spheroidal, and
ellipsoidal constructors parameterize rows of `A` with positive axis lengths;
arbitrary transforms and exactly symmetric SPD user metrics are also accepted.
The metric path uses unregularized Cholesky and rejects every nonpositive
computed pivot. Exact leading-principal-minor signs are certified first with
power-of-two congruence scaling and fixed-size floating expansions, for both a
user metric and the rounded `A^T A` derived from a transform. Thus an accepted
stored matrix is SPD and remains consistent with its distance factor. There is
also a strict necessary `|B_ij| > max(B_ii, B_jj)` rejection before product
formation; it cannot reject an SPD matrix and prevents invalid finite entries
from overflowing the exact expansion. There is no hidden symmetry tolerance,
jitter, clipping, regularization, or pseudoinverse. Ellipsoidal orthogonality
and maximum condition-number tolerances exist only as explicit caller inputs.

For a transformed-coordinate kernel jet, the constant-map chain rule is

```text
g_x[i]       = sum_a A[a,i] g_z[a],
H_x[i,j]     = sum_ab A[a,i] H_z[a,b] A[b,j],
T_x[i,j,k]   = sum_abc A[a,i] A[b,j] A[c,k] T_z[a,b,c].
```

The transform is applied before query/center argument signs. Thus a center
argument still contributes exactly one minus sign at every derivative order,
and a smooth radial center Hessian `phi''(0) I` becomes
`phi''(0) B`. The implementation computes `A(x-y)` directly, uses the existing
stable radius and analytic center rules, and rejects every non-finite
displacement, radius, or derivative result.

## Derivative capability

The matrix derivative demand is observation order plus center-functional order.
The query demand is requested output order plus center-functional order. A
model reports value, gradient, and Hessian capability as supported everywhere,
supported only away from centers, or unsupported. Kernel smoothness and center
limits decide the result; Hessian support is never unconditional. A local
mixture additionally requires the corresponding derivatives of every spatial
weight, including all product-rule terms.

Kernel metadata records hierarchical maximum orders through third order:
`away_through` and an optional `center_through`, with the center order no
greater than the away order. A combined demand greater than third order is
unsupported. If a demanded order is at most the center maximum it is supported
everywhere; otherwise, if it is at most the away maximum it is supported only
away from centers; all remaining demands are unsupported. Later fitted-model
capabilities must intersect this kernel result with functional, mixture, and
transform requirements rather than widening it. The away maximum applies at
every positive separation, including a compact kernel's support boundary and
zero exterior branch; boundary smoothness is therefore part of the declared
capability rather than an independent assumption by sparse assembly.

The first concrete CPD catalog entries use the signed polyharmonic family
`s_p r^p` for odd integer `p` and `s_p r^p log(r)` for even integer `p`, where
`s_p = (-1)^(floor(p/2)+1)`. Generic members support all three dimensions and
have CPD order `floor(p/2)+1`. A dimension-specific surface spline of Sobolev
order `m` requires `2m>D`, uses `p=2m-D`, and retains CPD order `m`. Both
entries have global support and no scalar shape parameter. Their center
capability is limited to `min(p-1,3)` and the complete radial jet path rejects
requests that would imply a nonexistent higher center derivative.

The smooth global catalog uses one positive physical `length_scale`. Gaussian
`exp(-(r/ell)^2/2)`, inverse multiquadric, and Matérn `1/2`, `3/2`, and `5/2`
are strictly positive definite in D=1/D=2/D=3. Multiquadric uses the negative
square-root sign required for positive projected energy and declares CPD order
one. All supply positive-radius derivatives and direct expansion coefficients
through third order. Gaussian, both multiquadrics, and Matérn `5/2` have full
third-order center jets; Matérn `3/2` stops at second order and Matérn `1/2`
at value. Extreme exponential and rational products are range-classified as a
whole so an intermediate zero or infinity does not erase a representable
final tail.

The compact catalog uses the normalized dimension-three Wendland C2, C4, and
C6 polynomials with `q=r/rho` and `t=max(1-q,0)`:

```text
t^4 (1+4q),
t^6 (1+6q+35q^2/3),
t^8 (1+8q+25q^2+32q^3).
```

Each has one positive coordinate-length `support_radius=rho`, center value
one, strict positive definiteness in D=1/D=2/D=3, and exact zero extension for
`r>=rho`. Analytic value-through-third derivatives and direct `a` and `b`
coefficients carry enough powers of `t` to match the zero branch through the
declared away order. C2 stops at second spatial order at the center; C4 and C6
have complete third-order center jets. Forming `t` as `1-q` on the center half
and `(rho-r)/rho` on the boundary half, then range-classifying the complete
factored product, protects both near-center and near-boundary extreme-scale
evaluation. This numerical catalog does not imply a sparse index, sparse
matrix format, or sparse solver.

## Correctness policy

Truth comes from analytic fields, high-precision evaluation, independent finite
differences, invariance properties, and documented SPD or CPD properties. No
external geological implementation is a correctness oracle. Numerical
tolerances are scale-derived and recorded by the future `NumericalPolicy`.
