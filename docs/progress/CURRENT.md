# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Implement
- Requirement: REQ-FUNC-001, Issue #37
- Branch: `codex/req-func-001-atomic-functionals`
- Draft pull request: pending initial implementation push
- Registry state in this change: `in_progress`
- Dependencies: REQ-DIM-001, REQ-KCALC-001, and REQ-POLY-001 are `integrated`

## Implementation result

- Added the two atomic functionals and nonempty finite linear expressions for
  D=1, D=2, and D=3 with deterministic term order and opaque stable provenance.
- Added analytic sample, complete-polynomial, and observation/center kernel-jet
  action. Query/center signs come only from the shared kernel calculus.
- Added distinct observation-functional and center-representer wrapper types;
  no semantic observation, constraint, assembly, fit, or solver was introduced.
- Added structured coefficient, sample, allocation, shape, polynomial, kernel,
  and non-finite accumulation diagnostics without partial result values.
- Added independent truth and failure tests, synchronized rustdoc and normative
  detail, a runnable example, a D=1/D=2/D=3 benchmark and baseline, and CI smoke
  coverage.
- Rust: implemented. CLI/C/C++/Python: N/A because problem schemas, fitted
  models, and stable binding surfaces do not exist yet.

## Validation state

- Focused functional integration tests pass: 9/9.
- Strict `georbf` all-target Clippy passes.
- Normal benchmark ran four consecutive times with stable checksums; the smoke
  workload also passes.
- The final stable-code standard gate passed: format, full workspace/all-target/
  all-feature Clippy with warnings denied, full workspace tests, documentation
  tests, and requirement-registry validation.
- After that gate, only this handoff and Draft PR linkage metadata may change;
  no production, test, manifest, schema, or build input may change without a
  new final gate.

## Next task

Finish only this Implement task: run the standard workspace gate once on the
stable head, commit and push, open the Draft PR, record its number in the
registry and this handoff, push that metadata-only follow-up, and stop. The
independent mathematical and numerical Review must start in a fresh task.

## Durable evidence

- Requirement summary: `changes/REQ-FUNC-001.md`
- Benchmark baseline: `docs/benchmarks/REQ-FUNC-001.md`
- Acceptance criteria and exclusions: GitHub Issue #37

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, allocation instrumentation, and API/ABI/schema
snapshot checks are tracked by later requirements and release gates. Local
`actionlint` is unavailable.
