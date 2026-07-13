# GeoRBF Master Plan

## Delivery policy

Work proceeds through one atomic requirement or one PR repair at a time. A
milestone starts only after all requirements in earlier milestones and every
declared dependency are integrated. Requirements are selected by the current
milestone, satisfied dependencies, priority, and smallest independently
verifiable scope.

| Milestone | Version | Exit theme |
| --- | --- | --- |
| M0 | v0.0.1 | Specification, workspace, CI, persistent controls |
| M1 | v0.1.0 | Dimensions, geometry, coordinates, orientation, kernel calculus |
| M2 | v0.2.0 | Polynomial spaces, CPD, atomic functionals |
| M3 | v0.3.0 | Unified hard-equality scalar field and dense solvers |
| M4 | v0.4.0 | Levels, soft losses, linear bounds, QP/SOCP integration |
| M5 | v0.5.0 | Normals, tangents, cones, thickness, multi-field projects |
| M6 | v0.6.0 | Global orientation estimation and SPD local trends |
| M7 | v0.7.0 | Sparse path, center selection, tuning, performance |
| M8 | v0.8.0 | Level sets, schemas, persistence, complete CLI |
| M9 | v0.9.0 | C/C++/Python, parity, API and artifact freeze |
| M10 | v1.0.0 | Full verification, documentation, supply chain, RC, release |

## Stage gates

M0 deliberately implements no RBF mathematics. M1 through M3 establish the
mathematical representation before inequalities or geological semantics. QP
and SOCP work waits for the backend spike. Local trends wait for a verified SPD
kernel family. Sparse work waits for compact kernels and a backend spike.
Bindings wait for the Rust API and schemas to stabilize.

Every numerical backend selection requires a spike that records maintenance,
license, MSRV, unsafe use, platforms, binary size, alternatives, correctness
cases, and performance. Accepted results become ADRs before production
dependency lock-in.

## Completion evidence

Each PR records:

- requirement and issue identifiers;
- acceptance criteria and exclusions;
- mathematical or architecture changes;
- independent truth and error-path tests;
- relevant benchmarks and baseline comparisons;
- interface and schema impacts;
- exact checks run and skipped checks;
- a changelog fragment and the next task in `docs/progress/CURRENT.md`.

The release gates are listed in `docs/release/RELEASE_CHECKLIST.md`. Milestone
labels do not override the machine-readable status in `requirements/v1.yaml`.
