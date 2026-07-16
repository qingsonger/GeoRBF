# Examples

Examples are added with the requirement that implements each real capability.
Stage 0 contains no fitting example because no fitting API exists.

`REQ-DIM-001` adds `crates/georbf/examples/dimension_safe_geometry.rs`, which
constructs a finite point and an overflow-safe unit direction. Run it with:

```text
cargo run -p georbf --example dimension_safe_geometry
```

`REQ-COORD-001` adds `crates/georbf/examples/coordinate_normalization.rs`,
which records coordinate metadata, normalizes a point, and transforms a
gradient back to original coordinates. Run it with:

```text
cargo run -p georbf --example coordinate_normalization
```

`REQ-KCALC-001` adds `crates/georbf/examples/radial_kernel_calculus.rs`,
which expands caller-supplied radial derivatives through third order into
Cartesian kernel derivatives. Run it with:

```text
cargo run -p georbf --example radial_kernel_calculus
```

`REQ-KERNEL-001` adds `crates/georbf/examples/kernel_metadata.rs`, which
constructs an illustrative static family descriptor, validates a configured
parameter value, and queries a derivative capability without implementing a
concrete radial formula. Run it with:

```text
cargo run -p georbf --example kernel_metadata
```

`REQ-KERNEL-002` adds `crates/georbf/examples/polyharmonic_spline.rs`, which
constructs the D=2 order-two surface spline, evaluates its concrete radial
formula through the shared Cartesian calculus, and prints its value and query
gradient. Run it with:

```text
cargo run -p georbf --example polyharmonic_spline
```

`REQ-KERNEL-003` adds `crates/georbf/examples/smooth_global_kernels.rs`, which
constructs Gaussian and Matérn `5/2` kernels with physical length scales,
evaluates them through the shared D=3 Cartesian calculus, and prints values
and a query gradient. Run it with:

```text
cargo run -p georbf --example smooth_global_kernels
```

`REQ-KERNEL-004` adds `crates/georbf/examples/wendland_kernels.rs`, which
constructs a Wendland C4 kernel with a physical support radius, evaluates its
interior D=3 Cartesian jet, and demonstrates the exact zero value at the
support boundary. Run it with:

```text
cargo run -p georbf --example wendland_kernels
```

`REQ-ORIENT-001` adds `crates/georbf/examples/geological_orientation.rs`,
which converts right-hand-rule strike/dip and azimuth/plunge measurements to
validated D=3 directions while preserving polarity metadata. Run it with:

```text
cargo run -p georbf --example geological_orientation
```

`REQ-ANISO-001` adds `crates/georbf/examples/global_anisotropy.rs`, which
constructs a rotated D=2 ellipsoidal metric, evaluates a Gaussian in transformed
coordinates, and maps its gradient back to original coordinates. Run it with:

```text
cargo run -p georbf --example global_anisotropy
```

`REQ-FUNC-001` adds `crates/georbf/examples/atomic_functionals.rs`, which builds
a value/directional-derivative expression, applies it to a scalar-field sample
and the complete polynomial basis, and evaluates it against a distinct Gaussian
center representer. Run it with:

```text
cargo run -p georbf --example atomic_functionals
```

`REQ-IR-001` adds `crates/georbf/examples/problem_ir.rs`, which retains full
semantic source provenance and compiles equality and interval relations through
an explicit affine linearizer into solver-neutral canonical rows. Run it with:

```text
cargo run -p georbf --example problem_ir
```

`REQ-FIELD-001` adds `crates/georbf/examples/field_assembly.rs`, which keeps
mixed observation and center roles distinct while assembling a symmetric dense
Gaussian hard-equality system. Run it with:

```text
cargo run -p georbf --example field_assembly
```

`REQ-SOLVE-001` adds `crates/georbf/examples/dense_equality_solver.rs`, which
solves an analytic SPD system through explicit checked Cholesky, no
regularization, bounded refinement, condition estimation, and original-unit
residual review. Run it with:

```text
cargo run -p georbf --example dense_equality_solver
```

`REQ-MODEL-001` adds
`crates/georbf/examples/immutable_fitted_field.rs`, which consumes a normalized
hard-equality problem, fits an immutable Gaussian field, evaluates value,
gradient, and Hessian at an original-coordinate query, and prints retained
capabilities:

```text
cargo run -p georbf --example immutable_fitted_field
```
