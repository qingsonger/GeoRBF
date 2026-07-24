# REQ-PERF-001 Dense and Sparse Performance Baseline

Command:

```text
cargo bench -p georbf --bench performance_baseline
```

The report begins with a fixed CSV header and every data row starts with schema
identifier `georbf.performance.v1`. Columns are workload, retained centers,
queries, explicit worker count, iterations, exact center visits per batch,
checked logical peak bytes, nanoseconds per query, and checksum. Workload
names, column order, fixed data order, and checksum coverage are stable
regression inputs; timing values are observations, not API promises.

The dense fixture is a 4-by-4-by-4 D=3 grid with 64 Gaussian value
representers and 512 fixed queries. Every query visits all 64 centers, for
32,768 center evaluations per batch. Its checked value/gradient output payload
is 24,576 bytes; no polynomial or compact-index scratch is required. The
compact fixture is a 6-by-6-by-6 unit grid with 216 Wendland C4 value
representers, support radius 1.01, and the same number of fixed queries. Exact
support filtering visits 2,175 centers per batch. Its logical peaks are 26,304,
28,032, and 31,488 bytes for one, two, and four workers.

Baseline environment: Microsoft Windows NT 10.0.26200.0, Intel Family 6 Model
154 (12th Gen Intel Core i7-1260P), `x86_64-pc-windows-msvc`, Rust 1.96.1,
one benchmark process. Four consecutive 20-iteration release runs produced:

| Workload | Workers | Nanoseconds/query | Peak bytes | Center visits | 20-iteration checksum |
| --- | ---: | ---: | ---: | ---: | ---: |
| Dense value/gradient | 1 | 7245.78--7915.35 | 24,576 | 32,768 | `3.11388211017631875e3` |
| Dense value/gradient | 2 | 4197.59--5083.99 | 24,576 | 32,768 | `3.11388211017631875e3` |
| Dense value/gradient | 4 | 3339.87--3935.15 | 24,576 | 32,768 | `3.11388211017631875e3` |
| Sparse value/gradient | 1 | 743.20--1184.58 | 26,304 | 2,175 | `-2.83389253435951275e2` |
| Sparse value/gradient | 2 | 924.72--1347.07 | 28,032 | 2,175 | `-2.83389253435951275e2` |
| Sparse value/gradient | 4 | 690.50--1640.38 | 31,488 | 2,175 | `-2.83389253435951275e2` |

Dense median time improved from 7597.04 ns/query at one worker to 4429.73 at
two and 3374.33 at four, directional speedups of 1.72x and 2.25x. The much
smaller sparse per-query workload is dominated by scoped-thread startup at
this batch size: its medians were 798.29, 947.11, and 749.49 ns/query.
Threading is therefore an explicit caller choice, not an automatic policy.
Checksums, center visits, and returned values were bit-identical across all
runs and worker counts.

The `--smoke` workload uses 27 dense centers, 64 sparse centers, 32 queries,
and one iteration at each worker count. Ready PRs and `main` run that shorter
workload on Windows, Ubuntu, and macOS. Existing
`field_assembly`, `dense_equality_solver`, and `compact_sparse_field`
benchmarks remain the phase-isolated assembly and solver baselines; this
requirement's report measures the new ordered batch path and thread scaling.
