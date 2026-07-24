# REQ-TUNE-001 Deterministic Parameter Tuning

Command:

```text
cargo bench -p georbf --bench parameter_tuning
```

The deterministic D=2 fixture uses an ordered 8-by-8 location grid and 128
explicit candidates spanning bounded kernel length, compact-support radius,
regularization, axis ratio, and influence radius. All searches use seed
`0x5eed`. Cross-validation uses five deterministic nonempty folds. Its analytic
evaluator has a known length optimum of two and returns finite weighted held-
out errors. The generalized-cross-validation evaluator supplies 64
observations, 12 effective degrees of freedom, and an analytic residual. The
power evaluator reviews 128 samples and supplies an analytic worst-case squared
power. The benchmark measures only the GeoRBF-owned validation, fold
construction, scoring, diagnostics, and deterministic selection layer; actual
field fits remain caller-owned and are not represented by the analytic
evaluator timing.

Baseline environment: Microsoft Windows, 12th Gen Intel Core i7-1260P,
`x86_64-pc-windows-msvc`, Rust 1.96.1, one benchmark process and one thread.
One 100-iteration run on 2026-07-24 produced:

| Strategy | Time per selection | Checksum |
| --- | ---: | ---: |
| Fixed | 418 ns | `6.40000000000000000e3` |
| Distance heuristic | 49.245 us | `1.96184571271557033e4` |
| Cross-validation | 12.437 us | `1.94724409448819279e4` |
| Generalized cross-validation | 824 ns | `5.40520430508316167e3` |
| Power function | 909 ns | `1.94724409448819279e4` |

These are local regression observations, not cross-machine or final-library
performance promises. Ready and `main` CI run a 16-candidate, one-iteration
`--smoke` workload on Windows, Ubuntu, and macOS.
