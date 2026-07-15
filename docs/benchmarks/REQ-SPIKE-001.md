# REQ-SPIKE-001 Dense Factorization Backend Spike

Command:

```text
cargo run --manifest-path spikes/factorization-backends/Cargo.toml --release
```

Baseline environment: Microsoft Windows NT 10.0.26200.0, 12th Gen Intel Core
i7-1260P, `x86_64-pc-windows-msvc`, Rust 1.96.1, one benchmark process. Both
candidates used default features disabled. Each timed iteration used a
deterministic matrix and RHS, performed the requested checked Cholesky or
pivoted LBLT factorization exactly once, reused those factors for the initial
solution and at most three explicit refinement corrections, and reviewed the
original-unit residual. A correction was accepted only if the original-unit
infinity-norm residual strictly decreased.

Three consecutive runs on 2026-07-15 produced these total times for three
iterations:

| Factorization | Size | faer 0.24.4 | nalgebra 0.35.0 |
| --- | ---: | ---: | ---: |
| Cholesky | 32 x 32 | 0.1373--0.2164 ms | 0.0573--0.0939 ms |
| Pivoted LBLT | 32 x 32 | 0.0670--0.1796 ms | 0.0491--0.0613 ms |
| Cholesky | 64 x 64 | 0.4163--0.5373 ms | 0.2330--0.2596 ms |
| Pivoted LBLT | 64 x 64 | 0.2936--0.3815 ms | 0.2733--0.3632 ms |
| Cholesky | 128 x 128 | 1.4935--1.7552 ms | 0.7985--1.0767 ms |
| Pivoted LBLT | 128 x 128 | 1.3904--1.7739 ms | 1.0141--1.3643 ms |

These ranges replace the pre-review measurements that reconstructed a
factorization for every accepted correction. Nalgebra had the lower median in
all six repaired measurements; the 64-square LBLT ranges overlap. Per-backend
checksums, initial and final residuals, and accepted refinement counts were
bit-identical across the three repeats. Checksums are not compared between
candidates because valid rounding paths differ. Independent tests compare both
solutions with analytic truth, inspect the mandatory 2-by-2 pivot block, and
require finite original-unit evidence with backward error no larger than
`1e-8`.

Minimal-feature release binaries of the repaired harness were 216,064 bytes for
nalgebra alone and 2,692,608 bytes for faer alone. The exact x86_64 Windows
graphs contained 13 and 41 external packages, and the candidate crate archives
were 396,463 and 1,897,499 bytes. These are comparison-harness observations,
not promises for a future GeoRBF library or adapter.

The benchmark is a dependency-selection probe, not a stable performance API.
Draft CI runs the short combined workload on Ubuntu. Ready PR and `main` CI run
the same 16 and 32 `--smoke` workload, both single-backend test configurations,
and the combined configuration on Windows, Ubuntu, and macOS.
