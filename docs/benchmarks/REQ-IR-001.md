# REQ-IR-001 Semantic-to-Canonical IR Baseline

Command:

```text
cargo bench -p georbf --bench problem_ir
```

Baseline environment: Microsoft Windows NT 10.0.26200.0, 12th Gen Intel Core
i7-1260P, `x86_64-pc-windows-msvc`, Rust 1.96.1, single benchmark process. Each
iteration compiles 96 immutable semantic constraints: 32 equalities, 32 closed
linear bounds, and 32 two-component second-order cones. The explicit
linearizer produces one sparse coefficient per expression. Compilation clones
complete provenance, forms identity scaling, validates indices and constants,
and computes the checked numeric-storage estimate.

Four consecutive 1,000-iteration runs on 2026-07-15 produced these observed
ranges:

| Dimension | Microseconds per complete compilation |
| --- | ---: |
| D=1 | 44.98--65.35 |
| D=2 | 31.21--58.75 |
| D=3 | 28.28--45.96 |

Every run produced checksums `5384000`, `5392000`, and `5400000` for D=1, D=2,
and D=3 respectively. They correspond to bit-identical checked numeric
estimates of 5,384, 5,392, and 5,400 bytes per iteration; the dimension delta is
the explicit identity variable-scaling vector. The workload is
allocation-sensitive by design because each canonical problem owns its rows,
provenance, and scaling independently of the semantic builder. Timings are a
local regression baseline, not a cross-machine performance promise. Ready PR
and `main` CI run the shorter ten-iteration `--smoke` workload on Windows,
Ubuntu, and macOS.
