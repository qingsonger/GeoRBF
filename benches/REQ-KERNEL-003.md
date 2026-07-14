# REQ-KERNEL-003 benchmark

The benchmark evaluates Gaussian, inverse multiquadric, CPD-positive signed
multiquadric, and Matérn `1/2`, `3/2`, and `5/2` kernels away from centers in
D=1, D=2, and D=3. Each iteration validates one perturbed point and stable
separation, evaluates all six radial jets and direct expansion coefficients,
expands their complete Cartesian jets, and accumulates value, gradient,
Hessian, and third-derivative components. The hot loop uses fixed arrays and
performs no heap allocation or dynamic dispatch.

Run the deterministic single-thread workload with:

```text
cargo bench -p georbf --bench smooth_global_kernels
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

Four consecutive full runs produced bit-identical checksums. Timing
variability is reported rather than hidden:

| Dimension | Median ns/iteration | Observed range ns/iteration | Checksum |
| --- | ---: | ---: | ---: |
| D=1 | 489.39 | 430.15–666.93 | `2.96442513794696936e6` |
| D=2 | 790.00 | 593.10–909.64 | `1.37211915742105083e6` |
| D=3 | 913.75 | 742.94–1303.35 | `-9.52149703426045453e4` |

Each iteration evaluates six family members, so these are complete catalog
workload times rather than single-kernel latency. They establish a
reproducible initial baseline, not a cross-machine performance promise.
Regression decisions must compare identical inputs, build settings, hardware
policy, and checksum.
