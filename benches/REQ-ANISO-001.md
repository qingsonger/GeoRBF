# REQ-ANISO-001 benchmark baseline

This dependency-free microbenchmark measures direct transformed-displacement
construction, stable radius evaluation, a supplied `phi(r)=r^6` radial jet,
complete transformed Cartesian expansion, and the global-anisotropy chain rule
through third order. Each D=1/D=2/D=3 case uses a fixed invertible transform and
1,000,000 deterministic query perturbations. It runs single-threaded and
consumes a checksum so the work remains observable.

## Environment

- Date: 2026-07-14
- Operating system: Microsoft Windows 11 Pro 10.0.26200, 64-bit
- Processor: 12th Gen Intel Core i7-1260P, 12 cores / 16 logical processors
- Rust: `rustc 1.96.1 (31fca3adb 2026-06-26)`, MSVC target, LLVM 22.1.2
- Cargo profile: optimized `bench`; no extra features or dependencies
- Threads: one benchmark thread
- Command: `cargo bench -p georbf --bench global_anisotropy`

## Initial implementation baseline

Four full measurements retained bit-identical checksums. The table reports the
median of those runs and the observed range.

| Dimension | Iterations | Median time/iteration | Observed range | Checksum |
| --- | ---: | ---: | ---: | ---: |
| D=1 | 1,000,000 | 61.44 ns | 48.65-82.93 ns | `9.64392288976221695e5` |
| D=2 | 1,000,000 | 180.81 ns | 136.44-241.46 ns | `-4.99447031386037767e7` |
| D=3 | 1,000,000 | 584.77 ns | 503.88-850.00 ns | `1.91948722928948164e8` |

These values are a local comparison baseline, not a cross-machine performance
gate. The spread reflects uncontrolled power state, scheduling, and background
activity. CI runs the `--smoke` form for functional coverage only. A future
performance change should compare repeated full runs on equivalent hardware.
