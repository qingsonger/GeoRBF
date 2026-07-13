# REQ-KCALC-001 benchmark baseline

This dependency-free microbenchmark measures construction of a stable point
separation, a supplied `phi(r) = r^6` radial jet, and its complete Cartesian
value/gradient/Hessian/third-tensor expansion. Each case uses fixed inputs and
1,000,000 deterministic perturbations. It runs single-threaded and consumes a
checksum so the work remains observable.

## Environment

- Date: 2026-07-13
- Operating system: Microsoft Windows 11 Pro 10.0.26200, 64-bit
- Processor: 12th Gen Intel Core i7-1260P, 12 cores / 16 logical processors
- Rust: `rustc 1.96.1 (31fca3adb 2026-06-26)`, MSVC target, LLVM 22.1.2
- Cargo profile: optimized `bench`; no extra features or dependencies
- Threads: one benchmark thread
- Command: `cargo bench -p georbf --bench radial_kernel_calculus`

## Baseline

| Dimension | Iterations | Elapsed | Time per iteration | Checksum |
| --- | ---: | ---: | ---: | ---: |
| D=1 | 1,000,000 | 0.046789 s | 46.79 ns | `6.17211064944781885e7` |
| D=2 | 1,000,000 | 0.092890 s | 92.89 ns | `-2.02490778313203603e8` |
| D=3 | 1,000,000 | 0.170209 s | 170.21 ns | `1.69913329443976164e8` |

These values are an initial local comparison baseline, not a cross-machine
performance gate. Power state, scheduling, and background activity were not
controlled. CI runs the `--smoke` form for functional coverage only; a future
performance change should compare repeated full runs on equivalent hardware.
