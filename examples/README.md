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
