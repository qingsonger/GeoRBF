# REQ-SPARSE-001 Compact Sparse Production Path

Command:

```text
cargo bench -p georbf --bench compact_sparse_field
```

The deterministic D=3 fixture uses an 8-by-8-by-8 unit grid, Wendland C4
support radius 1.01, 512 value representers, one process, one thread, and a
256 MiB explicit memory limit. The full symmetric matrix retains 3,200
nonzeros, at most seven per row, for density 0.01220703125.

Assembly timing starts with the validated immutable `FieldProblem` and includes
rstar bulk construction, every candidate query, exact strict-support
filtering, pair sorting/deduplication, kernel actions, canonical equality
compilation, exact symmetry review, and CSC materialization. Solve timing
starts with retained CSC and includes the checked backend CSC copy, AMD
symbolic and numeric LLT, solve, finite review, and exact original-unit
residual. Local evaluation uses one fitted model constructed before timing and
includes normalized-query support lookup, exact support filtering, selected
kernel actions, and original-coordinate gradient mapping.

Baseline environment: Microsoft Windows, 12th Gen Intel Core i7-1260P,
`x86_64-pc-windows-msvc`, Rust 1.96.1, one benchmark process and one thread.
Four consecutive five-iteration runs on 2026-07-23 produced:

| Phase | Time | Checksum |
| --- | ---: | ---: |
| Assembly | 1.2247--1.7348 ms/assembly | `1.60050000000000000e4` |
| Solve and review | 0.8663--1.27568 ms/solve | `3.12480267764224720e2` |
| Local value/gradient evaluation | 831.25--1129.69 ns/query | `2.74917062262429795e2` |

Stored nonzeros and all checksums were bit-identical across the four runs.
These are local regression observations, not cross-machine or final-library
performance promises. Ready PRs and `main` run the shorter 64-point
one-iteration `--smoke` workload on Windows, Ubuntu, and macOS.
