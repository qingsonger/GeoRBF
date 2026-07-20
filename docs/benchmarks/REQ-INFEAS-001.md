# REQ-INFEAS-001 Constraint Review Baseline

Command:

```text
cargo bench -p georbf --bench constraint_diagnostics
```

Baseline environment: Microsoft Windows NT 10.0.26200.0, 12th Gen Intel Core
i7-1260P, `x86_64-pc-windows-msvc`, Rust 1.96.1, single benchmark process. The
immutable canonical input contains 96 hard two-coefficient affine equalities in
32 independent groups. Each group contains one exact row, one positively scaled
exact duplicate, and one sign-reversed near duplicate. One review performs the
deterministic equality-then-bound pair traversal, scale-aware comparison, and
fallible cloning of complete provenance for 96 reported pairs. It does not
delete, merge, rescale, or otherwise rewrite a constraint.

The 5,000-iteration run on 2026-07-20 measured 145.75 microseconds per review
and produced checksum `480000`, exactly 96 pair diagnostics per iteration. The
eight-iteration `--smoke` run measured 178.09 microseconds per review and
produced checksum `768`. Timings are a local regression baseline, not a
cross-machine performance promise.
