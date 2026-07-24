# REQ-CONTOUR-001 One-Dimensional Level-Point Benchmark

Command:

```text
cargo bench -p georbf --bench level_points
```

The benchmark fits one exact CPD quadratic field before timing. Each timed
iteration scans 64 requested intervals (128 examined midpoint-split segments)
over `[-4, 4]`, extracts the two analytic roots and one non-level stationary
point, and folds all returned coordinates, residuals, derivatives, values, and
the fitted-field evaluation count into a deterministic checksum. Fitting,
field assembly, rank review, factorization, request construction, and settings
validation are outside the timed loop.

The normal workload runs 2,000 extractions. `--smoke` runs two extractions and
is routed through Ready/main Windows, Ubuntu, and macOS CI. Timings are local
regression evidence, not cross-machine performance promises.

Baseline environment: Microsoft Windows NT 10.0.26200.0, 12th Gen Intel Core
i7-1260P (12 cores, 16 logical processors), `x86_64-pc-windows-msvc`, Rust
1.96.1, one benchmark process and one thread. The 2026-07-24 release run
measured 125,356.75 ns per extraction and produced the deterministic
2,000-iteration checksum `2.50500000000000000e5`. The two-iteration smoke run
produced checksum `2.50500000000000000e2`.
