# PR #61 Independent Review

- Requirement: REQ-MODEL-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/60
- Pull request: https://github.com/qingsonger/GeoRBF/pull/61
- Branch: `codex/req-model-001-immutable-fitted-field`
- Reviewed head: `14d21d1240432b308e45ebc0fd00fc1a96e37b35`
- Stable implementation head: `8652bb4`
- Base head: `a569eac3af0bb8da3c02cd717829f93f04250189`
- Review date: 2026-07-16
- Result: one P2 and one P3 finding; repair required

## Scope and independence

A fresh read-only `xhigh` mathematical reviewer received only the bounded
REQ-MODEL-001 requirement and integrated dependency summaries, Issue #60
acceptance criteria and exclusions, the M3 plan, scoped mathematical,
architecture, and model-format contracts, the complete PR diff, tests,
examples, benchmark evidence, and exact-head CI state. It did not inherit the
implementation task's reasoning and made no repository or remote changes.

The reviewer independently checked formulae, query and center signs,
dimensions, coordinate and anisotropy chain rules, SPD and CPD behavior,
center limits, polynomial Hessians, output capabilities, exact coincidences,
hard constraints, hidden regularization or pseudoinverse behavior,
allocations, immutable `Send + Sync` use, deterministic model-record order,
structured errors and diagnostics, interface dispositions, benchmark routing,
tests, and registry evidence.

An initial reviewer interpretation treated the global-anisotropy `x` and `y`
in `docs/math/MATH_SPEC.md` as external coordinates and proposed changing the
implemented normalized-coordinate kernel semantics. A separate fresh
read-only adjudication rejected that production finding: Issue #60 explicitly
requires normalizing the query before applying the retained center
representers, and the coordinate contract evaluates `g_tilde` and `H_tilde`
in normalized model coordinates before mapping them back through `S`.
The accepted implementation and its `exp(-0.25)` regression are therefore
consistent. The ambiguity that enabled the misreading remains the P3 finding
below.

## Findings

### P2-1: fitted CPD models discard rank, null-space, and projected-energy evidence

`crates/georbf/src/field.rs:491-493` consumes a `DenseFieldSystem<D>` into only
its general `FieldAssemblyDiagnostics` and optional `PolynomialSpace<D>`.
This extracts the polynomial space from `CpdFieldAssembly<D>` but drops the
accepted `CpdNullSpace`, including its RRQR/SVD rank diagnostics and verified
basis quality, together with the retained `Z^T K Z` projected-energy matrix.

`crates/georbf/src/model.rs:422-425` consequently retains only the general
assembly summary and dense-solve diagnostics, and
`crates/georbf/src/model.rs:568` cannot transfer the discarded CPD evidence
into `FittedFieldDiagnostics` or `FittedFieldRecord`.

This does not change the already solved coefficients, but it violates Issue
#60's requirement that the immutable fitted field own all assembly and solve
diagnostics needed after fitting and the `docs/architecture/MODEL_FORMAT.md`
contract for complete assembly and solve evidence. The lost rank and
projected-energy evidence cannot be reconstructed from the fitted
coefficients.

Required repair: retain the complete CPD assembly evidence in the fitted
model's immutable diagnostics and expose it through the deterministic record
view without introducing a persistence schema. Extend the CPD model regression
with more centers than polynomial terms and assert the retained polynomial
space, RRQR/SVD rank decision, verified null-space dimensions and quality, and
nonempty projected-energy evidence in deterministic order.

### P3-1: anisotropy terminology does not identify the caller coordinate system

`docs/math/MATH_SPEC.md:220` says global anisotropy acts on the "original
point-pair displacement", while `crates/georbf/src/anisotropy.rs:597` calls
the chain-rule result "original-query derivatives". In the fitted-model
composition those words mean the input coordinate system of the anisotropy
layer, which is the normalized model coordinate system, not the external
coordinate convention retained by `FittedField`.

Issue #60 and the fitted-model architecture establish the implemented order:
normalize the external query, evaluate the retained kernel and optional
anisotropy in model coordinates, then map derivatives back through `S`.
The production mathematics is correct, but the overloaded word "original"
caused a fresh independent reviewer to derive the wrong physical contract.

Required repair: clarify the global-anisotropy mathematical and Rustdoc
contracts so `x` and `y` are explicitly points in the caller's current
coordinate system, and distinguish derivatives before the anisotropy transform
from external original-coordinate fitted-model derivatives. Add no behavior
change; retain the existing combined normalization/anisotropy truth test.

## Independently verified evidence

- Local HEAD, the remote branch, and Draft PR #61 matched exact reviewed head
  `14d21d1240432b308e45ebc0fd00fc1a96e37b35`; the worktree was clean.
- Draft Ubuntu CI run 29480459334 passed formatting, warning-denying workspace
  Clippy, all-feature workspace tests, workspace Rustdoc, spike gates, and all
  58 requirement checks on the exact reviewed head. The Ready-only Windows,
  Ubuntu, macOS, and benchmark-smoke matrix correctly did not run.
- Exact implementation head `8652bb4` passed the complete local standard gate
  recorded in the requirement evidence. The later reviewed head changes only
  registry and bounded-handoff PR-link evidence.
- Query/center derivative actions preserve exactly one center-argument minus
  sign through third order. The anisotropy and normalization chain rules,
  complete polynomial Hessians, conservative capability aggregation, and
  exact-coincidence rejection are otherwise internally consistent.
- The same concrete kernel definition drives assembly and evaluation. Model
  state is privately owned and immutable, and no finite differences, hidden
  coefficient repair, pseudoinverse, hidden regularization, or solver fallback
  were introduced.
- CLI, C, C++, and Python N/A dispositions match the M8 schema/CLI and M9
  binding boundaries. The fitted-field benchmark excludes fitting from the
  timed loop and is routed into the Ready three-platform benchmark-smoke gate.

Residual non-findings are limited independent coverage of noncommuting
normalization and anisotropy transforms, D=3 analytic evaluation truth, and
mixed-axis polynomial Hessians. The complete Ready-head platform and benchmark
gate remains intentionally pending.

## Bounded repair evidence

A subsequent Repair task addressed only P2-1 and P3-1. This section records
implementer evidence and is not a fresh independent re-review.

- `DenseFieldSystem::into_model_parts` now transfers the complete immutable
  `CpdFieldAssembly<D>` instead of extracting and discarding only its
  polynomial space.
- `FittedFieldDiagnostics<D>` retains that evidence and exposes it through
  both fitted-model diagnostics and `FittedFieldRecord::cpd_assembly`.
  Polynomial evaluation borrows the same retained assembly object's complete
  polynomial space, avoiding a duplicate or reconstructable approximation.
- The CPD model regression now uses four centers with three polynomial terms,
  repeats the fit, and checks deterministic polynomial-action order, matching
  complete evidence, full RRQR/SVD rank, a verified 4-by-1 null space and
  quality bounds, and a finite positive nonempty 1-by-1 projected energy.
- The mathematical specification, architecture, model-format contract, and
  anisotropy Rustdoc now state that global anisotropy consumes points and
  returns pre-transform derivatives in its caller's current coordinate system.
  They separately identify the fitted-model normalization chain rule that
  produces external original-coordinate outputs. No numerical behavior or
  existing combined anisotropy/normalization truth test changed.
- Focused checks passed: all six model tests, all five field-assembly tests,
  and all thirteen global-anisotropy tests.
- The complete local standard gate passed on the final repair tree: format,
  warning-denying workspace Clippy, all-feature workspace tests, workspace
  Rustdoc including compile-fail dimension boundaries, all 58 requirement
  checks, and `git diff --check`.

## Disposition

The bounded repair evidence indicates that P2-1 and P3-1 were addressed, but
this independent review result is not self-updating. PR #61 must remain Draft
and REQ-MODEL-001 remains `implemented` until a fresh read-only independent
re-review examines the exact repair head. Do not begin REQ-EXEC-001.
