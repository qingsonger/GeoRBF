# REQ-SPIKE-001 Dense Factorization Backend Spike

Command:

```text
cargo run --manifest-path spikes/factorization-backends/Cargo.toml --release
```

Baseline environment: Microsoft Windows NT 10.0.26200.0, 12th Gen Intel Core
i7-1260P, `x86_64-pc-windows-msvc`, Rust 1.96.1, one benchmark process. Both
candidates used default features disabled. Each timed iteration used a
deterministic matrix and RHS, performed the requested checked Cholesky or
pivoted LBLT solve, reviewed the original-unit residual, and attempted at most
three explicit refinement corrections. A correction was accepted only if the
original-unit infinity-norm residual strictly decreased.

Three consecutive runs on 2026-07-15 produced these total times for three
iterations:

| Factorization | Size | faer 0.24.4 | nalgebra 0.35.0 |
| --- | ---: | ---: | ---: |
| Cholesky | 32 x 32 | 0.1654--0.2684 ms | 0.0785--0.1049 ms |
| Pivoted LBLT | 32 x 32 | 0.0829--0.1166 ms | 0.0578--0.0795 ms |
| Cholesky | 64 x 64 | 0.8509--1.1879 ms | 0.3025--0.4242 ms |
| Pivoted LBLT | 64 x 64 | 0.6630--1.0005 ms | 0.3426--0.5620 ms |
| Cholesky | 128 x 128 | 2.2664--3.3045 ms | 1.2601--2.0680 ms |
| Pivoted LBLT | 128 x 128 | 2.4950--3.4727 ms | 1.9469--2.5055 ms |

Per-backend checksums, initial and final residuals, and accepted refinement
counts were bit-identical across the three repeats. Checksums are not compared
between candidates because valid rounding paths differ. Independent tests
compare both solutions with analytic truth and require original-unit backward
error no larger than `1e-8`.

Minimal-feature release binaries of the same harness were 207,872 bytes for
nalgebra alone and 2,683,904 bytes for faer alone. The exact x86_64 Windows
graphs contained 13 and 41 external packages, and the candidate crate archives
were 396,463 and 1,897,499 bytes. These are comparison-harness observations,
not promises for a future GeoRBF library or adapter.

The benchmark is a dependency-selection probe, not a stable performance API.
Draft CI runs the short combined workload on Ubuntu. Ready PR and `main` CI run
the same 16 and 32 `--smoke` workload, both single-backend test configurations,
and the combined configuration on Windows, Ubuntu, and macOS.
