# PR #100 Independent Review

- Requirement: REQ-PROJECT-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/99
- Pull request: https://github.com/qingsonger/GeoRBF/pull/100
- Branch: `codex/req-project-001-independent-multi-field-projects`
- Reviewed head: `16c8001089f400df7d0220acb9217176bcc394ac`
- Base head: `dcaf2a3df30cb35babe35225053d109102f238e0`
- Review date: 2026-07-21
- Result: clean first independent review; no P0-P3 finding

## Scope and independence

A fresh read-only project `math_reviewer` received only the bounded
REQ-PROJECT-001 summary and integrated dependency closure, Issue #99 acceptance
criteria and exclusions, the M5 plan, the architecture and ADR-0001 contracts,
the exact seven-file PR diff, directly called fitted-model APIs, tests,
registry, handoff, CI state, and validation evidence. It inherited no Implement
reasoning and made no repository or remote change.

The reviewer independently checked dimension bounds, deterministic identity
and ordering, immutable field ownership, reference lookup and delegation,
original-coordinate derivative behavior, capability and center-limit
propagation, allocation and partial-success behavior, `Send + Sync`, builder
independence, hidden coupling or regularization, interface dispositions,
benchmark applicability, and requirement truthfulness.

## Findings

No P0, P1, P2, or P3 finding was identified. No repair or regression test is
required.

## Independent truth review

- `GeoProject<D>` privately owns `FittedField<D>` entries in insertion order,
  rejects empty input and duplicate identifiers, uses checked project-owned
  allocation, and returns no partial project on failure.
- Identifier lookup linearly selects the unique stored identifier and returns
  immutable borrows only. It cannot select, mutate, refit, or couple another
  field.
- Every `ReferenceFieldInput` output method is an exact pass-through to the
  resolved `&FittedField<D>`. The project layer therefore adds no formula,
  sign, unit conversion, allocation policy, regularization, or alternate
  evaluation path.
- Delegated fitted-model evaluation retains its existing capability and center-
  limit checks. Original-coordinate gradients and Hessians continue to use the
  fitted model's reviewed affine chain rules, `S^-T grad(f)` and
  `S^-T H S^-1`.
- No SPD/CPD classification, polynomial space, rank decision, hard constraint,
  infeasibility behavior, or solver behavior is introduced or changed.
- Private owned state plus borrow-only access preserves immutability and
  builder independence. Compile-time bounds restrict all project types to
  D=1, D=2, and D=3, and public compile checks establish `Send + Sync`.
- Rust implementation status, adapter deferrals, benchmark N/A, and
  `implemented` rather than `integrated` are truthful.

The reviewer noted three non-actionable coverage gaps. The reference test uses
identity normalization and smoke-checks gradient and Hessian success rather
than comparing derivative components under a non-identity transform or an
unsupported Hessian error. Checked allocation failure has no deterministic
allocator-failure fixture. Tests do not combine fields with different
coordinate conventions. Direct delegation and ownership make these gaps
non-blocking for the stated requirement.

## Validation and disposition

- Local and remote branch heads matched exact reviewed head `16c8001`, base
  head `dcaf2a3`, and the worktree was clean before this evidence-only change.
- Draft CI run 29796378377 passed its configured Ubuntu correctness gate on
  exact reviewed head `16c8001`. The Ready-only Windows, Ubuntu, macOS, and
  benchmark-smoke matrix was skipped as designed and is not claimed as passed.
- The independent reviewer passed all six project integration tests, the D=4
  compile-fail Rustdoc test, and the complete diff whitespace check.
- The parent Review task independently passed the same six project tests and
  D=4 compile-fail Rustdoc test. Exact implementation head `16c8001` retains
  the complete standard local gate recorded by Implement: workspace format,
  warning-denying all-target/all-feature Clippy, all-feature workspace tests,
  workspace Rustdoc, all 58 requirement checks, and `git diff --check`.
- `cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are
  not installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers,
  executable fuzzing, mutation testing, general allocation instrumentation,
  API/ABI/schema snapshots, and local `actionlint` remain unavailable or
  deferred. No unexecuted check is claimed as passed.
- This Review change updates only this review record and the bounded handoff.
  It changes no production, test, manifest, schema, CI, build, API, numerical,
  registry, or dependency input and therefore does not invalidate the exact-
  implementation-head standard gate.

PR #100 remains Draft and REQ-PROJECT-001 remains `implemented`, not
`integrated`. A fresh Review/re-review task must independently inspect the final
evidence head. If it remains free of P0-P3 findings, it may synchronize PR
evidence, mark the PR ready, wait for the complete Windows, Ubuntu, macOS, and
benchmark-smoke CI on that exact ready head, merge exactly once only after all
required CI is green, and record truthful integration state. Do not begin
another requirement in that task.
