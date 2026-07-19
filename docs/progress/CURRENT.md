# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair complete pending fresh re-review / REQ-CONVEX-001
- Requirement: REQ-CONVEX-001, Issue #81
- Implementation branch: `codex/req-convex-001-canonical-solver`
- Implementation pull request: #82 (Draft)
- Original reviewed implementation head:
  `29ca2a1682d93ba48a47605624bdba1453866f72`
- Repair code/test head: `55f339c5d80666b089d2e2bdfae03a8b2029ae12`
- Review record: `docs/reviews/PR-82-INDEPENDENT-REVIEW.md`
- Registry state: `implemented`; it is not `integrated`
- Direct dependencies REQ-SOFT-001, REQ-LINEQ-001, and REQ-SPIKE-004 are all
  `integrated`

## Repair disposition

- R82-001 implemented pending independent closure: certificate stationarity
  and separation use homogeneous scaled original-data products with
  representability guards. The feasible `1e-12 * x <= -1` bogus certificate
  is rejected and true certificates are invariant at scales `1e-12`, `1`, and
  `1e12`.
- R82-002 implemented pending independent closure: the hidden factor 64 and
  dimensioned raw floors are removed. Primal, dual, cone, hard-relation,
  complementarity, gap, and certificate decisions use the exact requested
  tolerance and homogeneous scales; equivalent equality-row decisions are
  invariant across `1e-12`, `1`, and `1e12`.
- R82-003 implemented pending independent closure: the semantic objective is
  evaluated directly from original relations/scales/losses, compiled and
  backend primal values are reviewed separately, and the dual is reconstructed
  as `-0.5 * x^T P x - b^T z`.
- R82-004 implemented pending independent closure: the effective memory limit
  is the smaller nonzero convex/execution limit, and diagnostics retain both
  requested limits plus the effective value.
- R82-005 implemented pending independent closure: nonallocating preflight
  precedes compiler/provenance cloning, accounts for owned metadata and
  auxiliaries, and bounds QDLDL fill by the dense full-KKT lower triangle.
  Later GeoRBF-owned vector and provenance allocation is fallible.
- R82-006 implemented pending independent closure: every material Clarabel
  0.11.1 setting is explicit and mirrored by an exact diagnostic snapshot,
  including QDLDL and the independent-review tolerance policy.
- R82-007 implemented pending independent closure: five private regressions
  and nine end-to-end tests cover certificates, scaling, semantic/dual
  objectives, memory, settings, status routing, nonzero L2/L1/both Huber
  branches, nonunit scale, violated soft bound/cone, and Lorentz rotation.
- No finding is marked closed by this Repair task. PR #82 stays Draft for a
  fresh independent re-review, and REQ-INFEAS-001 has not begun.

## Validation state

- Focused all-target/all-feature warning-denying Clippy, all five private
  repair tests, and all nine convex integration tests passed.
- The runnable Huber example passed with zero hard-bound violation.
- The 8/16 release smoke workload passed with unchanged finite deterministic
  checksums `4.00000000000000444` and `7.99999999999999911`; this run measured
  1.0105 and 0.8482 ms and is not a performance promise.
- After the final production/test change, exact code/test head
  `55f339c5d80666b089d2e2bdfae03a8b2029ae12` passed the complete standard gate:
  workspace format, warning-denying all-target/all-feature Clippy, all-feature
  workspace tests, workspace Rustdoc, all 58 requirement checks, and
  `git diff --check`.
- The subsequent review record, solver policy, requirement change fragment,
  and this bounded handoff change only documentation. They do not change
  production, test, manifest, schema, CI, build, registry, API, numerical, or
  dependency inputs, so the immutable code/test-head gate remains applicable.
- Draft CI run 29686949377 passed the configured Ubuntu Draft job on the prior
  review-evidence head `f18785d`. Repair-head Draft CI will be triggered by the
  Repair push; it is not claimed here before execution. Ready-only Windows,
  Ubuntu, macOS, and benchmark-smoke CI has not run for this Draft repair.

## Next task

Open a fresh independent re-review task for Draft PR #82 and REQ-CONVEX-001
only. Give a fresh read-only project `math_reviewer` the bounded requirement
and dependency summaries, Issue #81 criteria, M4 plan, solver policy, ADR-0011,
the complete exact PR diff, original findings, repair evidence, tests, example,
benchmark, registry, handoff, and exact-head CI state without inheriting this
Repair reasoning. Independently determine whether R82-001 through R82-007 are
closed and whether any new P0-P3 finding exists. If findings remain, record
them and stop without repair. If the re-review is clean, follow the mandatory
ready-head CI and integration sequence in `docs/CODEX_WORKFLOW.md`. Do not begin
REQ-INFEAS-001 in the re-review task.

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable. Exact OSV and repository advisory API
queries are the performed dependency review; unavailable tools are not claimed.
