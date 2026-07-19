# REQ-LINEQ-001 Linear Semantic Compilation Baseline

Command:

```text
cargo bench -p georbf --bench linear_constraint_compilation
```

Baseline environment: Microsoft Windows NT 10.0.26200.0, 12th Gen Intel Core
i7-1260P, `x86_64-pc-windows-msvc`, Rust 1.96.1, single benchmark process. Each
iteration validates and lowers 96 immutable D=1 semantic constraints, evenly
covering lower, upper, closed interval, explicitly oriented inside, scalar-gap,
and directional-monotonicity forms. It then canonicalizes them over 192 field
variables, clones complete provenance, checks exact hard-row conflicts, forms
identity scaling, and computes the checked numeric-storage estimate.

The 2,000-iteration run on 2026-07-19 measured 184.93 microseconds per complete
build and compilation and produced checksum `12800000`, corresponding to a
bit-identical checked numeric estimate of 6,400 bytes per iteration. The
two-iteration `--smoke` run completed at 174.45 microseconds per iteration with
checksum `12800`. Timings are a local regression baseline, not a cross-machine
performance promise. CI uses the shorter smoke workload.
