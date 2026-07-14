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
