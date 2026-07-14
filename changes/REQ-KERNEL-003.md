# REQ-KERNEL-003

Added Gaussian, inverse multiquadric, CPD-positive signed multiquadric, and
Matérn `1/2`, `3/2`, and `5/2` global-support kernels for D=1/D=2/D=3. Every
family has one positive coordinate-length `length_scale`, analytic radial
derivatives through third order, direct stable Cartesian expansion
coefficients, exact center capability, and structured construction and
evaluation diagnostics.

Independent 90-digit truth, finite differences, deterministic full-Gram SPD
and full projected-subspace CPD checks, center limits, exchange signs, tensor
symmetry, extreme exponential and rational tails, pathologies, compile-fail
dimensions, and thread-safety assertions cover the new runtime paths. A
runnable example and deterministic allocation-free benchmark accompany the
Rust API.

No compact-support kernel, arbitrary Matérn smoothness, amplitude parameter,
polynomial construction, anisotropy, functional, assembly, solver, schema,
binding, or compatibility behavior is included.
