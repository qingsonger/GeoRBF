# REQ-FIELD-001 Dense Field-Assembly Baseline

Command:

```text
cargo bench -p georbf --bench field_assembly
```

Baseline environment: Microsoft Windows NT 10.0.26200.0, 12th Gen Intel Core
i7-1260P, `x86_64-pc-windows-msvc`, Rust 1.96.1, one benchmark process and one
thread. Each iteration assembles one fixed 32-center Gaussian all-representer
system. Every row contains a value plus a directional derivative. Assembly
validates exact observation/center alignment and derivative capabilities,
evaluates 528 upper-triangle entries, reflects them, compiles canonical rows,
and reviews symmetry. The workload performs no solve.

Four consecutive 100-iteration runs on 2026-07-15 produced these observed
ranges:

| Dimension | Microseconds per complete assembly | 100-iteration checksum |
| --- | ---: | ---: |
| D=1 | 181.36--229.45 | `8.05703833724105498e4` |
| D=2 | 330.42--430.40 | `6.76463046055061568e4` |
| D=3 | 382.64--569.63 | `5.96108373270946468e4` |

Checksums were bit-identical across all four runs for each dimension. The
timings are a local allocation-and-assembly regression baseline, not a
cross-machine performance promise. Ready PR and `main` CI run the shorter
two-iteration `--smoke` workload on Windows, Ubuntu, and macOS.
