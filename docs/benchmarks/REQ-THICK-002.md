# REQ-THICK-002 Sampled Geometric Thickness Baseline

Command:

```text
cargo bench -p georbf --bench sampled_thickness_validation
```

Baseline environment: Microsoft Windows NT 10.0.26200.0, 12th Gen Intel Core
i7-1260P, `x86_64-pc-windows-msvc`, Rust 1.96.1, single benchmark process. The
immutable D=1 field exactly reproduces `f(x)=x` through a complete quadratic
CPD polynomial space. Each validation processes 32 prebuilt selected locations
between the adjacent values -1 and +1, with 16 bracketing steps per side, at
most 48 bisection iterations, and four requested quantiles.

The 500-iteration optimized run on 2026-07-20 measured 2588.35 microseconds per
complete validation and produced measurement checksum `16000` plus distance
checksum `1000.0`. The one-iteration `--smoke` run measured 1951.10
microseconds with measurement checksum `32` and distance checksum `2.0`.
Timings are a local regression baseline, not a cross-machine performance
promise. Ready PR and `main` CI run the smoke workload on Windows, Ubuntu, and
macOS.

The THICK002-REV-001 through THICK002-REV-004 repair adds checked progress and
cancellation boundaries to each fitted-field evaluation and measures the
returned intersections directly. On the same machine, the repaired optimized
run measured 2299.12 microseconds per validation with unchanged checksums
`16000` and `1000.0`; the repaired smoke measured 1793.80 microseconds with
unchanged checksums `32` and `2.0`.

The THICK002-REV-005 repair passes caller execution metadata through the
controlled entry point without changing the default convenience benchmark
path or sampled-thickness mathematics. Its one-iteration smoke run measured
2471.60 microseconds with unchanged checksums `32` and `2.0`.
