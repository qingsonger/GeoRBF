# REQ-SPIKE-002 rank-backend harness

This excluded, non-production crate compares the exact candidate versions used
by the REQ-SPIKE-002 decision. It does not expose a GeoRBF API or participate in
the production workspace dependency graph.

Run the independent truth and property cases with:

```text
cargo test --manifest-path spikes/rank-backends/Cargo.toml --all-features
```

Run the reproducible comparison workload with:

```text
cargo run --manifest-path spikes/rank-backends/Cargo.toml --release
```

`--smoke` runs the shorter CI workload. The `faer-backend` and
`nalgebra-backend` features can be selected individually with
`--no-default-features --features <feature>` for dependency and binary-size
inspection.

Every case validates a nonempty finite matrix, applies eight alternating
infinity-norm row and column equilibration passes, and records separate RRQR
and SVD thresholds. Each threshold is
`max(row_count, column_count) * f64::EPSILON * leading_value`, where the
leading value is the largest absolute R diagonal for RRQR or the largest
singular value for SVD. RRQR is only a rank-risk screen. The SVD result is the
review classification, and any disagreement remains visible in the report.

The harness never calls either candidate's pseudoinverse or solve API. Its
purpose is dependency evaluation and regression evidence, not a production
solver path.
