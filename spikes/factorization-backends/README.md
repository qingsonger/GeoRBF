# REQ-SPIKE-001 dense-factorization harness

This excluded, non-production crate compares the exact candidate versions used
by the REQ-SPIKE-001 decision. It does not expose a GeoRBF API or participate in
the production workspace dependency graph.

Run the independent analytic-truth and failure-path cases with:

```text
cargo test --manifest-path spikes/factorization-backends/Cargo.toml --all-features
```

Run the reproducible comparison workload with:

```text
cargo run --manifest-path spikes/factorization-backends/Cargo.toml --release
```

`--smoke` runs the shorter CI workload. The `faer-backend` and
`nalgebra-backend` features can be selected individually with
`--no-default-features --features <feature>` for dependency and binary-size
inspection. Selecting neither backend is rejected at compile time. CI verifies
both single-backend configurations, the combined configuration, the negative
empty configuration, and the smoke workload on Windows, Ubuntu, and macOS once
the PR is ready.

The SPD cases use each candidate's checked Cholesky entry point. The symmetric
indefinite cases use Bunch--Kaufman LBLT with symmetric pivoting, including a
leading-zero-diagonal case that requires a 2-by-2 pivot block. Every returned
solution must be finite and pass an original-unit normwise backward-error
review. Singular and wrong-factorization-path cases must fail explicitly.

The iterative-refinement experiment is bounded to three corrections. It
refactors the unchanged matrix with the requested backend, accepts a correction
only when the original-unit residual strictly decreases, and records both the
initial and final residual. It never adds jitter, substitutes a diagonal,
switches factorization, calls a pseudoinverse, or relaxes a hard equation.
