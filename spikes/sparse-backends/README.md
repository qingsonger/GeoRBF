# REQ-SPIKE-003 compact-sparse harness

This excluded, non-production crate compares the exact spatial-index and sparse
factorization candidate versions used by the REQ-SPIKE-003 decision. It does
not expose a GeoRBF API or participate in the production workspace dependency
graph.

Run the independent neighborhood, assembly, analytic-solution, failure, and
repeatability cases with:

```text
cargo test --manifest-path spikes/sparse-backends/Cargo.toml --all-features
```

Run the reproducible scaling workload with:

```text
cargo run --manifest-path spikes/sparse-backends/Cargo.toml --release --all-features
```

`--smoke` uses the shorter 64- and 216-point CI workload. The spatial-index
features are `kiddo-index` and `rstar-index`; the sparse-factorization features
are `faer-backend` and `sprs-backend`. A minimal build selects exactly one from
each group, for example:

```text
cargo test --manifest-path spikes/sparse-backends/Cargo.toml \
  --no-default-features --features faer-backend,rstar-index
```

Selecting no spatial index or no sparse backend is rejected at compile time.
CI verifies all four minimal cross-products, the combined configuration, both
negative configurations, warning-denying Clippy, formatting, and the release
smoke workload. Draft PRs run that gate on Ubuntu; Ready PRs and `main` run it
on Windows, Ubuntu, and macOS.

The fixture is a three-dimensional unit grid with Wendland C2 support radius
1.75. Independent brute force enumerates every strict-interior pair. Candidate
queries are independently remeasured, strict-filtered, sorted by row and
column, and deduplicated before assembly. Exact support-boundary pairs are not
stored. The resulting full symmetric matrix has at most 27 nonzeros per point
and is constructed directly from sparse triplets without a dense intermediate.

Both factorization paths receive the same finite row-major triplets and known
truth solution. The harness constructs CSC storage, performs sparse
Cholesky-family factorization, solves once, and independently reviews the
original-unit infinity residual, backward error, and analytic solution. A
singular inconsistent system must fail. No jitter, diagonal adjustment,
regularization, pseudoinverse, dense fallback, or backend switch exists.

A hand-derived three-point case separately fixes
`phi(0) = 1`, `phi(1/2) = 3/16`, and `phi(1) = 0`. It checks each candidate's
actual CSC shape, column pointers, sorted unique row indices, stored values and
symmetry, storage-level matrix-vector product, and recovered solution without
using the harness kernel, assembly, or row-major matrix-vector helpers to
create the expected values.

Kiddo's public default `KdTree<f64, 3>` alias has a leaf bucket of 32 and
panics on the valid 10-by-10-by-10 axis-aligned fixture when too many points
share one coordinate. A dedicated expected-panic regression preserves that
evidence. The comparison path uses an explicit bucket of 128 only so the fixed
benchmark can finish; this bounded workaround is not an accepted production
policy. Rstar requires no equivalent data-dependent capacity assumption.

The scaling workload reports explicit end-to-end phases. Index rows time query,
strict filtering, canonicalization, and checksum work. Solver rows time triplet
allocation, CSC construction, factorization, solve, original-unit and analytic
review, and checksum work. It also reports strict pair count, stored nonzeros,
deterministic checksum, and original-unit residual. Its fixed data and one
process make repeat comparisons reproducible; timings remain machine-local
selection evidence, not a stable performance API or isolated factorization
measurement.
