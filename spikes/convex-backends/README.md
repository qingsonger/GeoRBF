# REQ-SPIKE-004 convex-backend harness

This excluded, non-production crate compares the exact candidate versions used
by the REQ-SPIKE-004 decision. It does not expose a GeoRBF API or participate in
the production workspace dependency graph.

Run the analytic-truth, original-unit residual, infeasibility-certificate, and
repeatability cases with:

```text
cargo test --manifest-path spikes/convex-backends/Cargo.toml --all-features
```

Run the reproducible comparison workload with:

```text
cargo run --manifest-path spikes/convex-backends/Cargo.toml --release
```

`--smoke` runs the shorter CI workload. The `clarabel-backend` and
`osqp-backend` features can be selected individually with
`--no-default-features --features <feature>` for dependency and binary-size
inspection. Selecting neither backend is rejected at compile time. CI verifies
both single-backend paths, the combined path, the negative empty configuration,
and the smoke workload on Windows, Ubuntu, and macOS once the PR is ready.

The shared QP minimizes `0.5*x^T*x + q^T*x` subject to one equality and finite
box bounds. Its independent solution is `(0.5, 1.5)`. Clarabel receives the
canonical form `A*x + s = b`, with one zero cone and one nonnegative cone;
OSQP receives the equivalent `l <= A*x <= u` form. The SOCP case minimizes
`t` subject to fixed components `(3, 4)` and `(t, 3, 4)` in the Lorentz cone,
whose independent optimum is `t = 5`.

Infeasible cases require the exact primal-infeasible status and independently
review the returned certificate. The review checks stationarity, a strictly
negative separating value, and the applicable nonnegative or Lorentz dual-cone
membership. Approximate solved or infeasible statuses are not accepted as
success. Every accepted finite solution is checked against the objective,
equalities, bounds, and cone in original units.

The harness fixes tolerances, iteration limits, OSQP's adaptive-rho interval,
and single-threaded Clarabel dispatch. Clarabel presolve and static and dynamic
KKT regularization are disabled. Its deterministic equilibration and internal
linear-solve refinement remain explicitly enabled. OSQP polishing and warm
starts are disabled. The harness never adds objective regularization, changes a
hard constraint, calls a pseudoinverse, or switches backends after failure.

Clarabel 0.11.1 requires its `serde` feature in this build: disabling all
features leaves an unconditional `serde_json` error type unresolved upstream.
No BLAS, LAPACK, SDP, Python, Julia, or Pardiso feature is enabled.
