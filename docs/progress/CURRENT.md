# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Review / fresh independent re-review of PR #49 repair
- Requirement: REQ-SPIKE-001, Issue #48
- Branch: `codex/req-spike-001-dense-factorization`
- Draft pull request: #49
- Original reviewed head: `b194061163e3e15add68c044a9ed040b23f3bdd8`
- Review record: `docs/reviews/PR-49-INDEPENDENT-REVIEW.md`
- Registry state: `documented` (not integrated)
- Dependency: REQ-BOOTSTRAP-001 is `integrated`

## Repair result

- P1-1: original-unit residual review now rejects a nonfinite residual entry,
  residual norm, matrix/vector norm, denominator, or backward error. The
  finite `A = [f64::MAX]`, `b = [0]`, `x = [2]` overflow counterexample is a
  regression.
- P1-2: backend factor construction is separated from right-hand-side solves.
  One factorization is reused for the initial solution and all zero-to-three
  refinement corrections. An instrumented regression observes exactly one
  factorization and multiple solves during an accepted correction.
- P1-3: the indefinite truth case is `[[0, 2], [2, 0]]`, whose zero diagonal
  under every symmetric permutation forces a 2-by-2 first pivot. Both backend
  tests inspect and require the exposed 2-by-2 block, verify the analytic
  solution, and require checked-Cholesky rejection.
- The performance record was regenerated with factorization reuse. Nalgebra
  retained the lower median in all six Windows measurements, with overlapping
  64-square LBLT ranges.

## Next task

Open a fresh Review task for only the repaired PR #49. Supply the requirement
summary, dependency closure, normative documents, complete PR diff, original
P1 findings, and validation evidence to a read-only independent reviewer. Do
not inherit this repair reasoning, do not repair production or test code in the
same task, keep the PR Draft, and do not begin REQ-IR-001.

If the fresh review is clean, stop with its findings recorded so a later fresh
Review task can perform the ready-head CI and integration sequence required by
`AGENTS.md` and `docs/CODEX_WORKFLOW.md`.

## Validation evidence

- Combined, faer-only, and nalgebra-only focused configurations each passed all
  8 tests, including the three repair regressions; warning-denying all-target
  all-feature spike Clippy passed.
- The no-backend configuration failed with the required compile error, and the
  repaired optimized smoke workload passed.
- Three consecutive optimized complete 32/64/128 workloads passed with stable
  per-backend checksums, residuals, and accepted refinement counts. The
  repaired ranges are recorded in `docs/benchmarks/REQ-SPIKE-001.md`.
- The stable repair code/test head passed workspace formatting, warning-denying
  workspace Clippy, all-feature workspace tests, workspace doc tests, all 58
  requirement checks, and `git diff --check`. This final evidence-only handoff
  update changes no code, tests, manifests, schemas, or build inputs.
- No three-platform ready-head CI is claimed while the PR remains Draft.

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, allocation instrumentation, and API/ABI/schema
snapshot checks are tracked by later requirements and release gates. Local
`actionlint` is unavailable. The performed advisory review was an OSV batch API
query of every exact selected package; it is not a claim that unavailable
audit tools ran.
