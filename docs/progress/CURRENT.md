# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review complete with findings / REQ-CONVEX-001
- Requirement: REQ-CONVEX-001, Issue #81
- Implementation branch: `codex/req-convex-001-canonical-solver`
- Implementation pull request: #82 (Draft)
- Reviewed implementation head: `29ca2a1682d93ba48a47605624bdba1453866f72`
- Review record: `docs/reviews/PR-82-INDEPENDENT-REVIEW.md`
- Registry state: `implemented`; it is not `integrated`
- Direct dependencies REQ-SOFT-001, REQ-LINEQ-001, and REQ-SPIKE-004 are all
  `integrated`

## Independent review result

- A fresh read-only project `math_reviewer` reviewed the bounded requirement
  and dependency summaries, Issue criteria, M4 plan, solver policy, ADR-0011,
  complete exact PR diff, tests, example, benchmark, dependency evidence,
  registry, handoff, and exact-head CI. It inherited no implementation
  reasoning and made no repository or remote change.
- R82-001 (P1): the absolute `A^T z` threshold accepts a bogus normalized
  certificate for the feasible scaled problem `1e-12 * x <= -1`.
- R82-002 (P1): the hidden factor-64 and raw unit floors make solved/hard-row
  acceptance depend on equivalent nonzero row scaling and exceed the requested
  tolerance without recorded policy.
- R82-003 (P1): original objective review reuses compiled auxiliaries, `P`, and
  `q`, while the duality gap reuses both backend objectives; neither is the
  independently reconstructed semantic or dual objective claimed by policy.
- R82-004 (P1): `ExecutionOptions::memory_limit_bytes` is ignored instead of
  being combined with the convex solver limit.
- R82-005 (P1): peak-memory checking occurs after compiler allocations, omits
  owned provenance and unbounded sparse factor fill, and therefore does not
  justify the documented conservative peak-working-set claim.
- R82-006 (P2): material Clarabel defaults and the selected direct solver are
  not completely explicit or recorded in GeoRBF diagnostics.
- R82-007 (P2): existing regressions do not independently exercise nonzero L2,
  L1, both Huber branches, scale, violated soft-bound/cone objectives, negative
  certificate review, status routing, row scaling, or Lorentz rotation.
- No P0 or other P1-P3 finding was reported. Algebraic hard/soft row signs,
  ordered cone mapping, L2/L1/Huber formulae, PSD objective, dual-cone
  convention, disabled presolve/KKT regularization, backend isolation, and
  interface and registry dispositions were independently confirmed.

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
- Draft CI run 29683566407 passed the configured Ubuntu Draft job on exact
  reviewed head `29ca2a1`. The Ready-only Windows, Ubuntu, and macOS workspace
  and benchmark-smoke matrix correctly did not run.
- This Review task changes only the review record and bounded handoff. It does
  not change production, test, manifest, schema, CI, build, registry, API,
  normative, numerical, or dependency inputs.

## Next task

Open a fresh Repair task for Draft PR #82 and REQ-CONVEX-001 only. Repair
R82-001 through R82-007 without expanding the requirement or beginning
REQ-INFEAS-001. Add the independent certificate, row-scaling, semantic
objective, dual reconstruction, effective-memory, pre-allocation/fill,
settings-snapshot, status-routing, and Lorentz-rotation regressions specified
in `docs/reviews/PR-82-INDEPENDENT-REVIEW.md`. Run focused checks during repair
and the complete standard gate once after the final production change. Update
review evidence and this bounded handoff, commit, push, and stop for a fresh
independent re-review.

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, general allocation instrumentation, and API/ABI/
schema snapshot checks are tracked by later requirements and release gates.
Local `actionlint` is unavailable. Exact OSV and repository advisory API
queries are the performed dependency review; unavailable tools are not claimed.
