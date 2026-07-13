# REQ-KERNEL-001

Added a formula-free kernel metadata and derivative-capability model. Kernel
families can declare strict positive definiteness or a positive CPD order, a
nonempty subset of D=1/D=2/D=3, maximum away and smooth-center derivative
orders, and global or parameterized compact support.

Matrix and query capability queries include center-functional derivative demand
and classify results as supported everywhere, supported only away from centers,
or unsupported. Demands above the third-order calculus range are unsupported;
metadata never fabricates center behavior or promises an unconditional fitted-
model Hessian.

Parameter definitions use unique explicit lower-snake-case names, documented
physical dimensions, and finite/nonnegative/positive value constraints. The
generic name `shape_parameter` is rejected. Compact support must reference an
existing strictly positive coordinate-length radius parameter. All validation
uses structured errors and the model performs no allocation or dynamic
dispatch.

No concrete radial formula, SPD/CPD proof, polynomial space, anisotropy,
orientation, functional, assembly, solver, schema, language binding, or Surfe
compatibility is included.
