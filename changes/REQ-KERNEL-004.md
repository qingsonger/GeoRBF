# REQ-KERNEL-004

Added normalized Wendland C2, C4, and C6 compact-support kernels for
D=1/D=2/D=3. Every member has one positive coordinate-length
`support_radius`, exact positive-zero value and derivative branches for
`r >= support_radius`, analytic interior radial derivatives through third
order, direct stable Cartesian expansion coefficients, exact center
capabilities, and structured construction and evaluation diagnostics.

Independent exact-rational truth, finite differences, deterministic full-Gram
SPD checks, support-boundary continuity, center limits, coordinate-scale
covariance, exchange signs, tensor symmetry, extreme and pathological inputs,
compile-fail dimensions, and thread-safety assertions cover the new runtime
paths. A runnable example and deterministic allocation-free benchmark
accompany the Rust API.

No sparse assembly, neighborhood index, anisotropy, amplitude parameter,
polynomial construction, functional, field, solver, schema, binding, or
compatibility behavior is included.
