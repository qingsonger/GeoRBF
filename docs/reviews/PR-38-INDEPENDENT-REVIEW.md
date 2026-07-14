# Independent Review of PR #38

- Requirement: REQ-FUNC-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/37
- Pull request: https://github.com/qingsonger/GeoRBF/pull/38
- Branch: `codex/req-func-001-atomic-functionals`
- Original reviewed head: `6dcbb9fa8d874cd5de4217e6f5f1deeac9927e0b`
- Repair and fresh re-reviewed head: `264c46a31908a85eb76289ae43e1bad8b5c2ea00`
- Base head: `d5848d143134f009712be4a4286b6e371eae6f2a`
- Review date: 2026-07-14
- Mode: Independent mathematical and numerical review

## Scope and verdict

A fresh read-only `math_reviewer` received only the bounded requirement and
dependency summaries, relevant normative documents and ADRs, the complete PR
diff, tests, benchmark evidence, and PR validation state. It established the
functional and kernel-capability contract independently from the implementation
task's reasoning.

The original review found one P1 defect and no P0, P2, or P3 finding. A fresh
read-only `math_reviewer` subsequently reviewed the complete repaired diff at
exact head `264c46a` without inheriting the Repair reasoning, independently
verified P1-1 closed, and found no P0, P1, P2, or P3 issue. The reviewed head is
eligible for the ready-head CI and integration sequence; it is not integrated
until that complete CI is green and the PR is merged.

## Finding

### P1-1 -- Kernel action over-demands a complete third-order jet

`crates/georbf/src/functional.rs:585` requires every evaluator invocation to
return a complete `SpatialKernelJet<D>` before the atom-pair dispatch at
`crates/georbf/src/functional.rs:600` determines the derivative order actually
needed.

This conflicts with the demand-based capability contract. Matrix demand is the
observation derivative order plus the center-functional derivative order
(`docs/math/MATH_SPEC.md:216`), and support is determined against that demand
(`docs/math/MATH_SPEC.md:224`). Every REQ-FUNC-001 atom pair therefore demands
only order zero, one, or two. Matérn 1/2 supports value at the center, while
Matérn 3/2 and Wendland C2 support through second order without promising a
complete third-order center jet (`docs/math/MATH_SPEC.md:245` and
`docs/math/KERNEL_REQUIREMENTS.md:233`).

The current callback type nevertheless forces callers through the complete-jet
construction path. Valid coincident actions such as Matérn 1/2 value/value and
Matérn 3/2 derivative/derivative therefore cannot be evaluated unless the
caller fabricates unsupported higher derivatives. The Gaussian-only kernel
tests do not expose this because Gaussian has a complete third-order center
jet.

Required regression: at `query == center`, apply one Value observation to one
Value center representer using Matérn 1/2 and assert the exact action
`k(x, x) = 1` succeeds using only the zero-order demand. A companion coincident
Matérn 3/2 directional-derivative/directional-derivative case should establish
that an exact second-order demand does not require a third-order jet.

The Repair task must preserve the shared kernel-calculus sign convention and
must not fabricate derivatives, widen kernel center capability, or introduce a
kernel-family special case in the functional layer.

## Reviewed evidence

- Issue #37 acceptance criteria and exclusions; REQ-FUNC-001 plus the
  integrated REQ-DIM-001, REQ-KCALC-001, and REQ-POLY-001 dependency summaries.
- `V1_SCOPE.md`, the M2 plan, `docs/math/MATH_SPEC.md`,
  `docs/math/KERNEL_REQUIREMENTS.md`, `docs/math/CPD_AND_POLYNOMIALS.md`,
  `docs/architecture/ARCHITECTURE.md`, `docs/architecture/PROBLEM_IR.md`, and
  ADR-0001, ADR-0004, ADR-0006, and ADR-0007.
- The complete `origin/main...6dcbb9f` diff, implementation tests, runnable
  example, deterministic benchmark and baseline, change fragment, requirement
  state, bounded handoff, PR metadata, and Draft CI state.
- Dependency review evidence for the shared kernel-calculus and complete
  polynomial-space contracts.

At the reviewed head, the focused functional tests passed 9/9, the functional
compile-fail doctest passed, benchmark smoke passed for D=1/D=2/D=3, and
`git diff --check` passed. Draft CI run 29334259493 passed the Ubuntu
correctness gate on the exact reviewed head. The final full local standard gate
remains the stable implementation-code gate recorded for `3203360`; subsequent
changes through the reviewed head contain only requirement linkage metadata and
the bounded handoff.

## Disposition

P1-1 was repaired in the subsequent bounded Repair task. The evaluator now
receives the exact atom-pair demand and returns a `SpatialKernelJetPrefix` that
can stop at value, first, or second order. Complete jets convert through the
maximum order any v1 atom pair needs. Coincident prefixes use analytic radial
center limits and do not expose an unsupported higher derivative. Returning a
prefix below the stated demand is a structured `InsufficientDerivativeOrder`
error with both term provenances.

The required coincident Matérn 1/2 value/value regression passes with zero-order
demand and exact action one. The Matérn 3/2 derivative/derivative companion
passes with second-order demand and the analytic mixed action `3/4` for length
scale two. Existing Gaussian sign, exchange, center-limit, bilinearity, error,
example, and benchmark-smoke paths remain green.

Focused functional tests pass 10/10, focused Clippy passes with warnings denied,
the runnable example passes, benchmark smoke passes in D=1/D=2/D=3 with the
recorded checksums, and `git diff --check` passes. After the final production
and test change, the complete standard workspace gate passed: formatting,
workspace Clippy with all targets/features, workspace tests with all features,
workspace rustdoc, and all 58 requirement-registry checks.

## Fresh re-review result

The independent reviewer confirmed that the evaluator receives the exact
zero-, first-, or second-order demand before prefix construction. Value-only
and value-through-Hessian center prefixes expose only their promised analytic
limits, complete jets convert through every order a v1 atom pair can demand,
and an insufficient prefix retains both term indices and provenances.

Independent contraction of the four atom pairs confirms `k`,
`u^T grad_x k`, `v^T grad_y k = -v^T grad_x k`, and
`u^T H_xy v = -u^T H_xx v`. For coincident Matérn 3/2 with length scale two,
`beta^2 = 3/4`, so the same-direction mixed action is exactly the tested
`3/4`. The Matérn 1/2 value action is exactly one and requests no derivative.
No third derivative is fabricated in either case.

The fresh reviewer also checked finite arithmetic and error atomicity,
allocation behavior, D=1/D=2/D=3 gating, type separation, safety, interface
dispositions, documentation, benchmark wiring, and the absence of hidden
regularization or out-of-scope semantics. No P0-P3 finding remains.

The reviewer passed formatting; focused warning-denying Clippy; all 10
functional tests; the functional compile-fail doctest; the runnable example;
atomic-functional benchmark smoke; all 58 requirement checks;
`git diff --check`; the duplicate-dependency check; and scoped forbidden-code
and allocation scans. Draft Ubuntu CI run 29339066111 is green on the exact
re-reviewed head `264c46a`.

After synchronizing this review evidence and moving the registry only from
`in_progress` to `documented`, the complete local standard gate passed:
formatting, warning-denying workspace Clippy with all targets and features,
workspace tests with all features, workspace rustdoc, and all 58 requirement
checks. These evidence changes alter no production code, tests, manifest,
schema, API, build input, or numerical behavior.
