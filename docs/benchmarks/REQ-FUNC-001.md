# REQ-FUNC-001 Atomic-Functional Baseline

Command:

```text
cargo bench -p georbf --bench atomic_functionals
```

Baseline environment: Microsoft Windows NT 10.0.26200.0, 12th Gen Intel Core
i7-1260P, `x86_64-pc-windows-msvc`, Rust 1.96.1, single benchmark process. Each
dimension uses a two-term observation expression, a two-term center expression,
a CPD-order-four polynomial space, and a fixed Gaussian Cartesian jet. The
kernel workload intentionally reuses the prebuilt jet so it measures only
functional pair dispatch, contraction, signs, weighting, and accumulation.

Four consecutive runs on 2026-07-14 produced these observed ranges:

| Dimension | Sample ns/iteration | Polynomial ns/iteration | Kernel ns/iteration |
| --- | ---: | ---: | ---: |
| D=1 | 3.44--11.37 | 330.50--395.78 | 14.38--18.71 |
| D=2 | 3.88--9.81 | 1118.65--1704.56 | 16.40--19.24 |
| D=3 | 3.44--4.48 | 2560.96--3808.82 | 37.91--45.58 |

Every run used 200,000 sample and kernel iterations and 20,000 polynomial
iterations. The `[sample, polynomial, kernel]` checksums were bit-identical
across all four runs:

- D=1: `[3.5e5, 1.953125e3, 1.67887952514609759e5]`
- D=2: `[3.41458980338835972e5, -2.56403338481308383e4, 1.06777021806923236e5]`
- D=3: `[3.54587585478012275e5, 6.18686217847877742e3, 7.15425299586308829e4]`

The polynomial action allocates its returned output and reusable scratch once
per call; it performs no per-basis or per-expression-term allocation. Sample
and kernel actions allocate nothing. Timings are a local regression baseline,
not a cross-machine performance promise. Ready PR and `main` CI run the shorter
`--smoke` workload on Windows, Ubuntu, and macOS.
