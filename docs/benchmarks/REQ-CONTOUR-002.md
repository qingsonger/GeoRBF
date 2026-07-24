# REQ-CONTOUR-002 Two-Dimensional Isoline Benchmark

Command:

```text
cargo bench -p georbf --bench isolines
```

The benchmark fits one exact CPD quadratic circle field before timing. Each
timed iteration runs disambiguated marching squares on a fixed 64 by 64 grid
over `[-1.5, 1.5]^2`, refines the radius-0.9 level set, verifies one closed
component, and folds every returned coordinate, fitted value, residual,
evaluation count, and topology length into a deterministic checksum. Fitting,
field assembly, rank review, factorization, request construction, and settings
validation are outside the timed loop.

The normal workload runs 500 extractions. `--smoke` runs two extractions and is
routed through Ready/main Windows, Ubuntu, and macOS CI. Timings are local
regression evidence, not cross-machine performance promises.

Baseline environment: Microsoft Windows NT 10.0.26200.0, 12th Gen Intel Core
i7-1260P (12 cores, 16 logical processors), `x86_64-pc-windows-msvc`, Rust
1.96.1, one benchmark process and one thread. The 2026-07-24 release run
measured 19,386,121.40 ns per extraction and produced the deterministic
500-iteration checksum `4.58249999999998696e6`. The two-iteration smoke run
produced checksum `1.83299999999997817e4`.
