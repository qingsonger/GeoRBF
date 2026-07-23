# REQ-CENTER-001 Rank-Safe Center Selection

Command:

```text
cargo bench -p georbf --bench center_selection
```

The deterministic D=1 fixture uses 160 ordered candidate locations, the
strictly positive-definite Gram matrix
`K_ij = exp(-0.5 * (x_i - x_j)^2)`, finite trigonometric target residuals, a
fixed `0x5eed` seed, one process, one thread, and a 512 MiB explicit numerical
review limit. All-representer reviews 160 centers; the other four strategies
review 48. Timings include strategy work, selected-principal-matrix
materialization, eight-pass equilibration, RRQR, bounded SVD, checked
Cholesky, and zero-right-hand-side residual review.

Baseline environment: Microsoft Windows, 12th Gen Intel Core i7-1260P,
`x86_64-pc-windows-msvc`, Rust 1.96.1, one benchmark process and one thread.
One three-iteration run on 2026-07-23 produced:

| Strategy | Time per selection | Checksum |
| --- | ---: | ---: |
| All representers | 11.7152 ms | `3.83681738713080995e4` |
| User provided | 726.8 us | `3.58829270378251977e3` |
| Farthest point | 851.4 us | `1.18934228323300140e4` |
| Residual greedy | 1.3064 ms | `1.20199230075995365e4` |
| Power greedy | 1.4527 ms | `1.13829427882236050e4` |

These are local regression observations, not cross-machine or final-library
performance promises. Ready PRs and `main` run the shorter 48-candidate,
12-selected, one-iteration `--smoke` workload on Windows, Ubuntu, and macOS.
