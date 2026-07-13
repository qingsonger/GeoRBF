# REQ-COORD-001

Added exact coordinate metadata for D=1, D=2, and D=3: validated length-unit
identifiers, opaque EPSG/WKT metadata, component-to-canonical axis order,
positive vertical direction, handedness, and external angle units. Compatibility
checks reject silent mixing of any differing metadata field; the core performs
no unit conversion or CRS reprojection.

Added private-invariant affine normalization for
`x_tilde = S^-1 (x - mu)`. Finite scale matrices are inverted with partial
pivoting and exact zero-pivot decisions, without a tolerance, jitter,
regularization, pseudoinverse, or numerical dependency. Singular matrices,
non-representable inverses, invalid Hessians, and non-finite operation results
return structured errors.

Point normalization and denormalization support general finite scale matrices.
Gradient and Hessian results transform back through `S^-T` and
`S^-T H_tilde S^-1`. Tests cover all supported dimensions, metadata errors,
unit/CRS mismatch, translation, scaling, rotation, shear, analytic derivative
truth, near-singular matrices, representable numeric extremes, overflow, and
unsupported dimensions. A runnable Rust example documents the public path.

Orientation conversion, anisotropy, kernel calculus, reprojection, persistence
schemas, batch APIs, language bindings, and Surfe compatibility remain out of
scope.
