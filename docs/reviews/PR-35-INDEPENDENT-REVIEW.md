# Independent Review of PR #35

- Requirement: REQ-POLY-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/34
- Pull request: https://github.com/qingsonger/GeoRBF/pull/35
- Branch: `codex/req-poly-001-polynomial-spaces`
- Reviewed head: `acc65c667932c14d461e2bedd028eea5f1d2bfd8`
- Base head: `feeb608b8b2046731d62f4c49f31ee9737524517`
- Review date: 2026-07-14
- Mode: Independent mathematical and numerical review

## Scope and verdict

A fresh read-only `math_reviewer` received only the compact requirement and
dependency summaries, `docs/math/CPD_AND_POLYNOMIALS.md`, ADR-0004, the M2
milestone context, the complete PR diff, and the recorded validation and
benchmark evidence. It independently reviewed polynomial completeness and
ordering, values and Cartesian derivatives, origin and axis limits, extreme
floating-point products, error atomicity, allocations, dimension gating,
interfaces, tests, documentation, benchmarks, and the requirement state.

Verdict: one P3 evidence finding blocks marking the Draft PR ready. No P0, P1,
or P2 finding was identified. The current production implementation satisfies
the reviewed mathematical and error-atomicity contracts; the finding is a
missing regression for one public failure path.

## Finding

### P3-1 -- Joint-output error atomicity lacks independent regression evidence

`crates/georbf/src/polynomial.rs:287` promises that both outputs passed to
`PolynomialSpace::try_evaluate` remain unchanged on any error. The failure
tests at `crates/georbf/tests/polynomial.rs:191` exercise the value-only and
gradient-only methods separately, but the joint method has only a success
case. A regression that writes the values before validating the gradient
length would therefore pass the current suite while violating the explicit
no-partial-success acceptance criterion.

Required repair: add one joint-call regression using a valid three-entry value
buffer and an invalid two-entry gradient buffer, both initialized with distinct
sentinels. Assert a `Gradients` `OutputLengthMismatch` with expected length
three and actual length two, then assert that both buffers remain exactly
unchanged. No production-code change is indicated unless that regression
exposes a defect.

## Independent mathematical and numerical conclusion

- The generated set is exactly every multi-index with total degree at most
  `m-1`; its count is `binomial(D+m-1,D)`. The recursive enumeration is unique,
  complete, and ordered by increasing total degree then descending
  lexicographic exponent order.
- Cartesian derivatives have the correct sign, dimension, and direct
  exponent-lowering formula. They do not divide by coordinates, and their
  origin and coordinate-axis limits are correct.
- Binary-exponent product tracking prevents a representable mixed monomial
  from disappearing through intermediate underflow. It changes neither the
  basis nor user coordinates and introduces no hidden scaling or
  regularization.
- Degree, term-count, allocation, output-length, and non-finite-result failures
  are structured. The current implementation validates all joint outputs
  before writing and is atomic; P3-1 concerns missing independent regression
  evidence for that behavior.
- Complete total-degree polynomial spaces are closed under rotations. No
  per-basis-term rotation invariance is required.
- The PR does not implement or claim center-action matrices, rank decisions,
  null spaces, KKT systems, hard-constraint handling, SPD/CPD solves, or
  Hessian capabilities.

## Safety, interfaces, performance, and evidence

- The public implementation is safe Rust, immutable after construction,
  restricted by the existing compile-time dimension gate to D=1, D=2, and
  D=3, and performs no heap allocation during evaluation.
- Rust is the only applicable interface. CLI, C, C++, Python, schemas, and
  persistence are correctly N/A for this internal polynomial-basis increment.
- The deterministic benchmark counts agree with the complete-space formula,
  covers generation plus allocation-free value and gradient evaluation, and
  is wired into Ready/main three-platform benchmark smoke.
- Stable code/test head `8369aac` passed formatting, warning-denying workspace
  Clippy, 129 workspace tests, 24 doctests and compile-fail tests, all 58
  requirement checks, ten focused Release polynomial tests, strict rustdoc,
  the runnable example, and benchmark smoke. Four full local benchmark runs
  retained identical checksums.
- Reviewed head `acc65c6` differs from that tested code head only in the
  requirement PR link/status and bounded handoff. Draft CI run 29329182602
  passed the Ubuntu correctness gate on exact reviewed head `acc65c6`; the
  Ready-only matrix was correctly skipped.

## Disposition and residual risk

Keep PR #35 in Draft. A fresh Repair task must address only P3-1, run the
focused regression and the final standard checks on the stable repair head,
update this review record and the bounded handoff, push, and stop for a fresh
independent re-review. Do not begin REQ-FUNC-001.

Allocation instrumentation, Miri, sanitizers, fuzzing, mutation testing, and
API snapshots are not yet available. The handoff records those as later gates;
they are residual program risks, not additional findings in REQ-POLY-001.
