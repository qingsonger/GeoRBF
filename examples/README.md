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
