# REQ-KERNEL-002 benchmark

The benchmark exercises a power-five `PolyharmonicSpline` away from centers in
D=1, D=2, and D=3. Each iteration validates a perturbed finite point,
constructs the stable separation, evaluates the concrete radial derivatives
and direct expansion coefficients, expands the complete Cartesian jet, and
accumulates value, gradient, Hessian, and third-derivative components. The hot
loop uses fixed arrays and performs no heap allocation or dynamic dispatch.

Run the deterministic single-thread workload with:

```text
cargo bench -p georbf --bench polyharmonic_spline
```

Use `-- --smoke` for the 20,000-iteration CI workload. The normal workload
runs 1,000,000 iterations per dimension.

## Local baseline

Recorded on 2026-07-14 using:

- Intel Core i7-1260P, 12 cores / 16 logical processors;
- 16 GB RAM;
- Windows 11 Professional 10.0.26200, x86_64-pc-windows-msvc;
- Rust 1.96.1, LLVM 22.1.2;
- release benchmark profile, one benchmark process, no explicit CPU pinning.

Four consecutive full runs produced identical checksums. Timing variability is
reported rather than hidden:

| Dimension | Median ns/iteration | Observed range ns/iteration | Checksum |
| --- | ---: | ---: | ---: |
| D=1 | 148.70 | 97.63–197.70 | `-4.40079060999637544e7` |
| D=2 | 209.10 | 154.00–286.63 | `6.52190543411652818e7` |
| D=3 | 237.61 | 175.04–354.24 | `-7.78147375743490160e7` |

These measurements establish a reproducible initial baseline, not a
cross-machine performance promise. Regression decisions must compare identical
inputs, build settings, hardware policy, and checksum.
