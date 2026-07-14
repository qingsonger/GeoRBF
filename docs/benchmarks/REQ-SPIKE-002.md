# REQ-SPIKE-002 RRQR and SVD Backend Spike

Command:

```text
cargo run --manifest-path spikes/rank-backends/Cargo.toml --release
```

Baseline environment: Microsoft Windows NT 10.0.26200.0, 12th Gen Intel Core
i7-1260P, `x86_64-pc-windows-msvc`, Rust 1.96.1, one benchmark process. Both
candidates used default features disabled. Every timed iteration generated no
random input and ran the same eight-pass equilibration, column-pivoted QR, R
diagonal extraction, SVD, thresholds, and checksum accumulation.

Three consecutive runs on 2026-07-14 produced these total times for three
iterations:

| Size | faer 0.24.4 | nalgebra 0.35.0 |
| ---: | ---: | ---: |
| 32 x 32 | 0.5544--0.7032 ms | 0.3138--0.3732 ms |
| 64 x 64 | 1.6848--2.2996 ms | 1.4213--1.9242 ms |
| 128 x 128 | 8.2705--11.1731 ms | 8.1772--12.9324 ms |

Checksums were bit-identical across the three repeats for each backend and
size. Checksums are intentionally not compared between backends because they
include RRQR diagonal sums, and valid pivot choices can differ. The independent
tests compare rank properties and SVD review results instead.

Minimal-feature release binaries of the same harness were 2,754,048 bytes for
faer alone and 232,448 bytes for nalgebra alone. The resolved dependency counts
excluding the spike package were 47 and 14. These measurements include only
the comparison executable and are not promises for a future GeoRBF library or
adapter.

The benchmark is a dependency-selection probe, not a stable performance API.
Ready PR and `main` CI run the 16 and 32 `--smoke` workloads on Windows, Ubuntu,
and macOS; the complete local runs provide the larger scaling baseline.
