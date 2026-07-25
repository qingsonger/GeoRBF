# REQ-CONTOUR-003 Three-Dimensional Isosurface Benchmark

Command:

```text
cargo bench -p georbf --bench isosurfaces
```

The benchmark fits one exact CPD quadratic sphere field before timing. Each
timed iteration runs topology-aware marching cubes on a fixed 24 by 24 by 24
grid over `[-1, 1]^3`, refines the radius-0.73 level set, verifies one closed
component, and folds every returned coordinate, normal, fitted value,
residual, evaluation count, and triangle count into a deterministic checksum.
Fitting, assembly, rank review, factorization, request construction, and
settings validation are outside the timed loop.

The normal workload runs 50 extractions. `--smoke` runs one extraction and is
routed through Ready/main Windows, Ubuntu, and macOS CI. Timings are local
regression evidence, not cross-machine performance promises.

Baseline environment: Microsoft Windows NT 10.0.26200.0, 12th Gen Intel Core
i7-1260P (12 cores, 16 logical processors), `x86_64-pc-windows-msvc`, Rust
1.96.1, one benchmark process and one thread. The 2026-07-25 release run
measured 907,046,074.00 ns per extraction and produced the deterministic
50-iteration checksum `8.92875000000006333e6`. The one-iteration smoke run
measured 1,000,777,300 ns and produced checksum `1.78575000000023807e5`.
