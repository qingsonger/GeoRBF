# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Implement / REQ-POLY-001 complete at Draft PR
- Requirement Issue: #34
- Branch: `codex/req-poly-001-polynomial-spaces`
- Draft pull request: #35
- Dependencies: REQ-DIM-001 is `integrated`; no blocker
- Registry state: `documented`; independent Review is still required

## Implemented scope

- Complete polynomial spaces for positive CPD order in exactly D=1, D=2, and
  D=3, with exact checked binomial term counts and deterministic graded
  descending-lexicographic multi-indices.
- Immutable Rust metadata plus allocation-free batch value and Cartesian
  gradient evaluation into caller-provided storage. Evaluation lowers
  exponents without coordinate division and tracks binary exponents so
  representable mixed monomials survive intermediate underflow.
- Structured zero-order, degree/term-count overflow, allocation, output-size,
  and non-finite-result errors; evaluation errors leave output unchanged.
- Independent combinatorial, analytic, origin, extreme-scale, reproduction,
  unsupported-dimension, and `Send + Sync` tests; synchronized rustdoc and math
  documentation; runnable example; changelog fragment; deterministic D=1/D=2/
  D=3 benchmark and Ready/main CI smoke coverage.
- CLI, C, C++, and Python are N/A because polynomial generation is internal.
  CPD rank diagnosis, null spaces, functionals, assembly, fitting, schemas,
  persistence, adapters, and solvers remain excluded.

## Next task

Open a fresh independent Review task for only REQ-POLY-001 and Draft PR #35.
Supply the compact
requirement summary and dependency closure, `docs/math/CPD_AND_POLYNOMIALS.md`,
ADR-0004, the PR diff, and validation and benchmark evidence to the project
`math_reviewer`. Do not repair findings or begin REQ-FUNC-001 in that task.

## Validation evidence

- Stable code/test head passed `cargo fmt --all -- --check`, warning-denying
  workspace Clippy, 129 workspace tests, 24 doctests/compile-fail tests, and all
  58 requirement checks.
- Focused Release polynomial tests passed all 10 cases. Strict warning-denying
  rustdoc, the runnable example, and polynomial benchmark smoke passed.
- Four full local polynomial benchmark runs had identical generation and
  evaluation checksums. Environment and timing ranges are recorded in
  `docs/benchmarks/REQ-POLY-001.md`.
- Only the registry PR link/status and this bounded handoff changed after the
  stable full gate; no production, test, manifest, schema, or build input
  changed.

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, allocation instrumentation, and API/ABI/schema
snapshot checks are tracked by later requirements and release gates. Local
`actionlint` is unavailable.
