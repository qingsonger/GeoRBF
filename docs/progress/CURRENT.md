# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Implement complete / REQ-CONVEX-001
- Requirement: REQ-CONVEX-001, Issue #81
- Implementation branch: `codex/req-convex-001-canonical-solver`
- Implementation pull request: #82 (Draft)
- Registry state: `implemented`; it is not `integrated`
- Implementation commit: `6796ccc` plus a following registry/handoff-only commit
- Direct dependencies REQ-SOFT-001, REQ-LINEQ-001, and REQ-SPIKE-004 are all
  `integrated`

## Implemented scope

- One private Clarabel 0.11.1 adapter maps canonical equality, bound, and
  ordered second-order-cone rows without exposing backend types.
- Soft L2 uses a diagonal PSD objective; L1 and Huber use exact explicit
  epigraph constructions. Hard relations never become objective terms.
- Options require strict tolerance, iteration, time, and memory policy.
  Serial QDLDL, equilibration, and iterative refinement are recorded;
  presolve and static/dynamic KKT regularization are disabled.
- Exact solved status is independently reviewed for original objective,
  primal/dual equations, cone membership, complementarity, duality gap, and
  original-unit hard residuals with complete provenance.
- Primal infeasibility is returned only with a normalized certificate that
  passes original-data stationarity, dual-cone, nonzero, and strict
  scale-aware separator review. Every other non-success status is rejected.
- Rust API, example, analytic/property/error tests, production benchmark,
  three-platform Ready/main benchmark-smoke routing, architecture policy,
  dependency audit, and change fragment are present.
- CLI is N/A until M8. C, C++, and Python are N/A until M9. Geological angular
  and thickness semantic compilation remains later work.

## Validation state

- Focused warning-denying all-target Clippy and all seven convex solver tests
  passed.
- The runnable Huber example passed with zero hard-bound violation.
- The release smoke benchmark passed at 8 and 16 variables with finite,
  deterministic checksums; the recorded first run was 0.4392 and 0.3477 ms.
- Production dependency review found 34 active Windows packages and 48
  all-target lock packages, only permissive declared licenses, a highest
  declared MSRV of Rust 1.77 with some omissions, no native-code path, and no
  finding in exact OSV or repository advisory queries.
- After the final production and registry changes, the stable tree passed the
  complete standard gate: workspace format, warning-denying all-target/all-
  feature Clippy, all-feature workspace tests, workspace Rustdoc, all 58
  requirement checks, and `git diff --check`. This following edit records only
  that validation evidence; no production, test, manifest, schema, CI, build,
  registry, API, normative, numerical, or dependency input changed afterward.

## Next task

Open a fresh Review task for Draft PR #82 and REQ-CONVEX-001 only. Supply the
project `math_reviewer` only the bounded requirement/dependency summaries,
SOLVER_POLICY, ADR-0011, PR diff, test/benchmark/dependency evidence, and CI
state. Review formulae, signs, epigraph equivalence, PSD objective, cone order,
hard constraints, original-unit KKT and certificate checks, settings,
allocations, provenance, interface dispositions, and registry truth. Record
P0-P3 findings but do not repair production code or begin REQ-INFEAS-001 in the
same task.

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable. Exact OSV and repository advisory API
queries are the performed dependency review; unavailable tools are not claimed.
