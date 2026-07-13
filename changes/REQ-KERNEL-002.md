# REQ-KERNEL-002

Added CPD-positive integer-power polyharmonic splines for D=1/D=2/D=3 and
dimension-specific surface splines with the exact `2m>D` validity rule. Both
entry points have global support and no redundant scalar shape parameter.

Analytic radial values and derivatives through third order feed the existing
Cartesian calculus. D=2/D=3 expansion coefficients use direct closed forms;
the implementation does not reconstruct a cancellation-sensitive coefficient
from separately rounded derivatives. Center capability is exact through
`min(p-1,3)`, and incomplete center smoothness is reported rather than filled
with invented derivatives.

Structured diagnostics cover invalid powers, invalid or overflowing surface
orders, negative or non-finite radii, unavailable complete center jets, and
non-representable derivatives or coefficients. Independent embedded
high-precision truth, finite differences, deterministic random projected-CPD
checks, center and exchange properties, compile-fail dimensions, pathologies,
an example, and a dependency-free benchmark cover the new runtime path.

No smooth global-support kernel, compact kernel, polynomial construction, CPD
rank enforcement, anisotropy, functional, assembly, solver, schema, binding,
or compatibility behavior is included.
