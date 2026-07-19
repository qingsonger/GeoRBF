# PR #79 Independent Review

- Requirement: REQ-SPIKE-004
- Issue: https://github.com/qingsonger/GeoRBF/issues/78
- Pull request: https://github.com/qingsonger/GeoRBF/pull/79
- Branch: `codex/req-spike-004-qp-socp-backends`
- Reviewed head: `10e0266fb83fb432f668cc4dfe1edd99dd176da8`
- Base head: `5b5db20f5133dddaf1088c3952a1e241478b312f`
- Review date: 2026-07-19
- Result: changes requested; R79-001 (P1) and R79-002 (P2) remain

## Scope and independence

A fresh read-only project `math_reviewer` received only the bounded
REQ-SPIKE-004 summary and integrated dependency closure, Issue #78 acceptance
criteria, the M4 plan, ADR-0007, ADR-0011, the solver policy, benchmark and
spike contracts, the complete PR diff, and recorded validation evidence. It
did not inherit implementation reasoning and made no repository or remote
changes.

The reviewer checked the QP and SOCP formulae, signs, dimensions, objective
conventions, cone ordering, analytic truth, exact statuses, infeasibility
certificates, hard constraints, solver settings, hidden regularization,
determinism, feature isolation, dependency and platform evidence, benchmark
claims, interface dispositions, registry state, and bounded handoff.

## Findings

### P1 R79-001: certificate reviews are not scale-aware

`spikes/convex-backends/src/main.rs:321-344` and
`spikes/convex-backends/src/main.rs:381-394` compare absolute certificate
stationarity with `2e-7` while requiring only an absolute separator below
`-1e-8`. For the Clarabel QP, the nonnegative vector
`z = (2e-8, 0)` passes both thresholds even though `A^T z = -2e-8` is nonzero
and its relative stationarity error is one. The OSQP review analogously accepts
`delta_y = (-2e-8, 0)`. The oracle can therefore approve a vector that is not
a Farkas certificate, weakening the requirement's infeasibility evidence.

Required repair: normalize every nonzero certificate before review, or use
dimensionless residuals scaled by `||A|| ||z||` with a scale-aware strict
separation margin. Add synthetic-certificate regressions proving that the
near-zero nonstationary examples above are rejected while positive rescalings
of a valid certificate remain valid.

### P2 R79-002: QP performance comparison includes asymmetric construction

`spikes/convex-backends/src/main.rs:420-458` constructs Clarabel's diagonal
matrices from O(n) CSC entries, while
`spikes/convex-backends/src/main.rs:461-489` emits n-squared values twice for
OSQP's two identity matrices. `spikes/convex-backends/src/main.rs:623-629`
times construction and solving together. Consequently the statement at
`docs/adr/ADR-0011-clarabel-convex-backend.md:78` and the corresponding table
in `docs/benchmarks/REQ-SPIKE-004.md:18` that Clarabel was faster in every
64-variable repeat is not a backend comparison with equivalent fixture-
construction work. This does not overturn the SOCP-capability rationale, but
it makes the reported QP performance evidence misleading.

Required repair: use sparse O(n) fixtures for both backends, or measure
equivalent prebuilt setup and solve phases separately, then regenerate the
table and affected prose. Add a fixture regression that checks dimensions,
semantics, and O(n) stored nonzeros for both QP paths.

## Independent mathematical review

With `y = 2 - x`, the shared QP objective becomes `x^2 - x - 2`, so the
unique feasible optimum is `(0.5, 1.5)` with objective `-2.25`. Clarabel's
zero and nonnegative cone rows and OSQP's two-sided bounds encode the equality
and box constraints with the stated signs.

Fixing the SOCP components to `(3, 4)` and requiring `(t, 3, 4)` to lie in the
Lorentz cone gives `t >= 5`; minimizing `t` yields `(5, 3, 4)`. The Clarabel
rows produce that slack in its `A*x + s = b` convention.

A conic primal-infeasibility certificate requires `z` in the dual cone,
`A^T z = 0`, and `b^T z < 0`. The OSQP bound form requires
`A^T delta_y = 0` and
`u^T (delta_y)_+ + l^T (delta_y)_- < 0`. The implemented signs and dual-cone
membership checks are correct; R79-001 concerns the non-homogeneous numerical
acceptance thresholds.

## Validation evidence

- The local branch, remote branch, and Draft PR matched exact reviewed head
  `10e0266fb83fb432f668cc4dfe1edd99dd176da8`; the worktree was clean.
- Draft Ubuntu CI run 29674034129 passed on that exact head. The Ready-only
  Windows, Ubuntu, macOS, and benchmark-smoke matrix correctly did not run.
- Exact implementation commit `682c9a632a1103f7799daa34c5d8aaac042ea9cf`
  passed the recorded complete standard workspace gate and focused spike
  checks. The later reviewed head changed only the PR link in requirement and
  handoff evidence.
- The production workspace graph and public Rust, CLI, C, C++, and Python
  surfaces are unchanged. SPD/CPD classification, center limits, polynomial
  spaces, rank decisions, rotation invariance, and Hessian support are not
  applicable to this excluded dependency spike.
- The reviewer confirmed the analytic QP and SOCP solutions, canonical row and
  cone signs, strict exact solver-status routing, and dual-cone conventions,
  subject to the two findings above.
- This Review task changes only this review record and the bounded handoff.
  Workspace formatting, all 58 requirement checks, and staged whitespace
  checks passed on the resulting review evidence tree.

No other P0, P1, P2, or P3 finding was reported.

## Disposition

PR #79 remains Draft and REQ-SPIKE-004 remains `implemented`. A fresh Repair
task must address only R79-001 and R79-002, add the specified regressions,
regenerate affected benchmark evidence, run focused and final standard checks,
record repair evidence here and in the bounded handoff, commit, push, and stop
for a fresh independent re-review. Do not begin REQ-CONVEX-001.
