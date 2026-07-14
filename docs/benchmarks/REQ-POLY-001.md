# REQ-POLY-001 Polynomial-Space Baseline

Command:

```text
cargo bench -p georbf --bench polynomial_spaces
```

Baseline environment: Windows NT 10.0.26200.0, 12th Gen Intel Core i7-1260P,
`x86_64-pc-windows-msvc`, Rust 1.96.1, single benchmark process. The workload
uses CPD order 8, giving 8, 36, and 120 terms in D=1, D=2, and D=3. Four
consecutive runs on 2026-07-14 produced these observed ranges:

| Dimension | Generation ns/iteration | Value + gradient ns/iteration |
| --- | ---: | ---: |
| D=1 | 96.75--218.05 | 775.43--1285.90 |
| D=2 | 162.21--232.42 | 8982.59--11869.16 |
| D=3 | 1360.72--1612.82 | 29384.22--53714.75 |

Every run used 10,000 generation iterations and 100,000 evaluation iterations.
Generation checksums were respectively 80,000, 360,000, and 1,200,000. The
evaluation checksums were bit-identical across all four runs: approximately
`1.37934246587414003e5`, `1.015625e4`, and `1.77001953125e2`. Timings are a
local regression baseline, not a cross-machine performance promise; the broad
range reflects ordinary laptop scheduling noise.

The evaluation loop reuses caller-owned value and gradient buffers and performs
no per-iteration heap allocation. Ready PR and `main` CI run the shorter
`--smoke` workload on Windows, Ubuntu, and macOS.
