# REQ-NORMAL-001

Added immutable D=1/D=2/D=3 Rust semantics for `GradientVector`,
`DirectionOnly`, `DirectionWithPolarity`, `AngularCone`, and
`AxialDirection`. Vector observations expand into caller-provenanced scalar
relations in a public deterministic role order. Gradient vectors become
Cartesian equalities; direction-only and axial values become exact
orthogonal-complement equalities; oriented direction adds
`n^T grad f >= g_min`; angular tolerance adds the ordered Lorentz relation
`||T^T grad f|| <= tan(theta) n^T grad f` plus the explicit projection bound.
The shared solver IR and Clarabel adapter receive only equality, linear-bound,
and cone forms, never geological mode names.

The D=2/D=3 complement is built deterministically by projecting the Cartesian
axis least aligned with the unit normal and completing the frame in D=3. Basis
signs are canonicalized, so direction-only and axial scalar rows are
binary-exact under `n -> -n`; orthonormal and projector identities hold to
floating-point roundoff. `DirectionOnly`, `AxialDirection`, and `AngularCone`
are structurally rejected in D=1 because they would be vacuous or angle
insensitive. `GradientVector` and oriented projection remain available in D=1.

Angular inputs explicitly select degrees or radians and must lie in the convex
domain from zero inclusive to a right angle exclusive. Minimum oriented
gradient projections are finite, nonnegative, and always caller supplied;
zero is accepted only explicitly. Invalid or non-finite directions are
prevented by `UnitDirection`. Invalid angles, minimums, provenance counts,
duplicate scalar-row identifiers, soft-loss metadata, allocation, and shared
IR failures return structured errors. Nothing clips an angle, weakens a hard
constraint, adds jitter or regularization, or invents the non-convex claim
`|n^T grad f| >= g_min`.

Near-zero fitted-gradient review is diagnostic-only. The caller supplies a
positive finite reference scale in gradient units and a finite nonnegative
dimensionless threshold. The checked absolute threshold, overflow-safe
Euclidean magnitude, decision, complete source path, and observation ID remain
observable. There is no hidden default scale or constraint insertion.

Eight independent tests cover analytic gradient components, polarity signs,
orthonormal complements, binary-exact axial sign reversal, rotated angular-
cone margins, degree/radian equivalence, invalid angles and directions,
provenance counts, explicit hard/soft lowering, D=1/D=2/D=3 boundaries,
subnormal near-zero magnitudes, diagnostic sources, determinism, and
`Send + Sync`. A runnable angular-cone example and deterministic mixed-mode
D=3 compilation benchmark accompany Rustdoc and the normative mathematics.

Rust is implemented. CLI is N/A because the stage-0 command exposes only help
and version and complete project/schema commands belong to M8. C, C++, and
Python are N/A because their M9 requirements follow Rust API and schema freeze;
no adapter may reimplement these signs or angle domains. The focused benchmark
is implemented. No tangent, thickness, multi-field, persistence, global
orientation estimation, local anisotropy, adapter, release, or subsequent
requirement work was added.
