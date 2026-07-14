# REQ-KERNEL-004 benchmark

The benchmark evaluates Wendland C2, C4, and C6 kernels at interior support
points in D=1, D=2, and D=3. Each iteration validates one perturbed point and
stable separation, evaluates all three radial jets and direct expansion
coefficients, expands their complete Cartesian jets, and accumulates value,
gradient, Hessian, and third-derivative components. The hot loop uses fixed
arrays and performs no heap allocation or dynamic dispatch.

Run the deterministic single-thread workload with:

```text
cargo bench -p georbf --bench wendland_kernels
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
| D=1 | 170.95 | 135.33–196.28 | `8.39156446419097856e6` |
| D=2 | 330.13 | 230.95–408.87 | `3.74400229259619676e6` |
| D=3 | 474.78 | 446.83–579.44 | `3.62162010189632303e5` |

Each iteration evaluates all three smoothness members, so these are complete
catalog-workload times rather than single-kernel latency. They establish a
reproducible initial baseline, not a cross-machine performance promise.
Regression decisions must compare identical inputs, build settings, hardware
policy, and checksum.
