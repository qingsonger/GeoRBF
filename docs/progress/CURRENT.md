# Current GeoRBF Progress

This file is a bounded handoff for the next task. Completed history belongs in
`docs/progress/HISTORY.md`, requirement change fragments, independent review
records, benchmark reports, Git, and GitHub.

## Active repository work

- Mode: Repair / PR #49 review findings P1-1 through P1-3
- Requirement: REQ-SPIKE-001, Issue #48
- Branch: `codex/req-spike-001-dense-factorization`
- Draft pull request: #49
- Reviewed head: `b194061163e3e15add68c044a9ed040b23f3bdd8`
- Review record: `docs/reviews/PR-49-INDEPENDENT-REVIEW.md`
- Registry state: `documented` (not integrated)
- Dependency: REQ-BOOTSTRAP-001 is `integrated`

## Independent review result

- P1-1: nonfinite original-unit residual evidence can produce a NaN backward
  error that bypasses the current greater-than rejection. Every residual
  entry, norm, denominator, and backward error must be finite.
- P1-2: every refinement correction calls the top-level solve and constructs a
  new factorization. The initial solution and all corrections must reuse one
  factorization of the unchanged matrix.
- P1-3: the current three-by-three indefinite case permits an all-1-by-1 LDLT
  after symmetric permutation, so it does not prove the claimed mandatory
  2-by-2 pivot. Replace it with independent truth that forces a 2-by-2 block.
- No additional P0, P2, or P3 issue was identified. Review mode made no
  production or test repair and did not begin REQ-IR-001.

## Next task

Open a fresh Repair task for only PR #49 findings P1-1, P1-2, and P1-3. Add the
specified independent regressions, implement the smallest complete repairs,
run focused checks during development, then run the complete standard workspace
checks once on the stable repair head. Update the review record and this bounded
handoff, commit, push, and stop for a fresh independent re-review. Keep the PR
Draft and do not begin REQ-IR-001.

## Validation evidence

- Before review, combined and both single-backend focused configurations passed
  6/6 tests; the no-backend configuration failed with the required compile
  error; optimized smoke and complete 32/64/128 workloads passed.
- The implementation code/test head passed formatting, warning-denying
  workspace Clippy, all-feature workspace tests, workspace doc tests, all 58
  requirement checks, and `git diff --check`.
- Draft CI run 29400346664 passed the Ubuntu gate on exact reviewed head
  `b194061163e3e15add68c044a9ed040b23f3bdd8`; the ready-only three-platform
  matrix was correctly skipped.
- This Review task changes only independent-review evidence, the registry
  document link, and this bounded handoff. It does not change production,
  tests, manifests, schemas, build inputs, APIs, dependencies, or numerical
  behavior.

## Checks not yet available

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers, executable
fuzzing, mutation testing, allocation instrumentation, and API/ABI/schema
snapshot checks are tracked by later requirements and release gates. Local
`actionlint` is unavailable. The performed advisory review was an OSV batch API
query of every exact selected package; it is not a claim that unavailable audit
tools ran.
