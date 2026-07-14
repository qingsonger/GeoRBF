# Independent Review of PR #38

- Requirement: REQ-FUNC-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/37
- Pull request: https://github.com/qingsonger/GeoRBF/pull/38
- Branch: `codex/req-func-001-atomic-functionals`
- Reviewed head: `6dcbb9fa8d874cd5de4217e6f5f1deeac9927e0b`
- Base head: `d5848d143134f009712be4a4286b6e371eae6f2a`
- Review date: 2026-07-14
- Mode: Independent mathematical and numerical review

## Scope and verdict

A fresh read-only `math_reviewer` received only the bounded requirement and
dependency summaries, relevant normative documents and ADRs, the complete PR
diff, tests, benchmark evidence, and PR validation state. It established the
functional and kernel-capability contract independently from the implementation
task's reasoning.

The review found one P1 defect and no P0, P2, or P3 finding. PR #38 must remain
Draft and REQ-FUNC-001 must remain `in_progress` until a fresh Repair task adds
the required independent regression, implements the smallest complete repair,
runs the final stable-head checks, and pushes. A fresh re-review is required
after that repair.

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

P1-1 requires Repair. This review task records evidence only and makes no
production or test change. The next task must repair only P1-1 and stop after
its focused checks, final standard gate, handoff update, commit, and push. It
must not start another requirement or mark the PR ready.
