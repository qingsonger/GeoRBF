# PR #103 Independent Review

- Requirement: REQ-TREND-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/102
- Pull request: https://github.com/qingsonger/GeoRBF/pull/103
- Branch: `codex/req-trend-001-positive-definite-local-trends`
- Reviewed head: `48c9d516721928f98dd06242a2304b8d4c9f94e3`
- Base head: `7487cfafd0739c1f63028d4b46d7505b4ca6c1b3`
- Review date: 2026-07-21
- Result: three P1 findings and one P2 finding; repair required

## Scope and independence

A fresh read-only project `math_reviewer` received only the bounded
REQ-TREND-001 summary and integrated dependency closure, Issue #102 acceptance
criteria and exclusions, the M6 plan, ANISOTROPY and ADR-0005/ADR-0008
contracts, the exact PR diff, tests, example, benchmark, registry, handoff, CI
state, and validation evidence. It inherited no Implement reasoning and made no
repository or remote change.

The reviewer independently checked formulae, signs, dimensions, units, SPD and
CPD classification, represented floating-point limits, center capability,
rotation behavior, product-rule completeness, independent truth, allocation
and hot-path behavior, hidden regularization, interface dispositions,
diagnostics, benchmark obligations, and requirement truthfulness.

## Findings

### F1 - P1: public weight variants bypass validation invariants

`SmoothSpatialWeight` publicly exposes both variants and every invariant-
bearing field at `crates/georbf/src/local_trend.rs:96-117`. External crates can
therefore bypass `try_constant` and `try_gaussian`, including the cached
Gaussian reciprocal-radius values consumed at lines 192-212. An external-
crate compile probe constructed a Gaussian declaring radius one while setting
both cached reciprocals to zero; it compiled and evaluates as a different
function with false derivatives. A direct `Constant { value: 1.0e308 }` also
bypasses the helper's square check.

`LocalTrendMixture::try_new` checks definiteness and the selected background at
lines 463-500 but does not revalidate non-background weights or cached
Gaussian fields. The public API therefore does not enforce the validated-
analytic-weight premise required by the SPD and product-rule contracts.

Required regression: add an external compile-fail test proving callers cannot
construct the variants or cached fields directly. If the representation stays
publicly constructible, mixture construction must instead reject forged and
noncanonical weights in an integration regression.

### F2 - P1: an accepted strict background can underflow to zero

`validate_amplitude` at `crates/georbf/src/local_trend.rs:1051-1063` rejects an
overflowed square but accepts an underflowed zero square. The finite nonzero
constant `1.0e-200` therefore passes `try_constant`; a matching policy minimum
also passes background validation at lines 484-499. Evaluation at lines
653-657 then forms a zero diagonal contribution, while coverage at lines
586-592 reports `background_squared_weight == 0.0` despite the documented
strict positivity at lines 343-346.

The exact-real square is `1e-400 > 0`, but the returned represented kernel has
`K(x,x) == 0.0` and is not strictly positive definite. The implementation can
therefore silently lose its core positive-definiteness guarantee on accepted
input.

Required regression: `try_constant(1.0e-200)`, or mixture construction with it
as background, must return a structured square-underflow/nonrepresentability
error. It must never accept the mixture and return zero diagonal or background
coverage.

### F3 - P1: Gaussian value underflow erases representable derivatives

`gaussian_weight_jet` forms the gradient and Hessian by multiplying the already
rounded value at `crates/georbf/src/local_trend.rs:1104-1126`. With radius
`1e-150`, amplitude one, and displacement `40 * radius`, `exp(-800)` rounds to
zero, so the implementation returns zero gradient and Hessian. Independent
80-digit evaluation gives a representable gradient
`-1.467149833671075e-196` and Hessian `5.864931460100122e-45`.

Valid input therefore receives analytically incorrect weight jets and local-
mixture derivatives even though the demanded results fit in `f64`.

Required regression: add a public-mixture Hessian test with that Gaussian
scale, a representable strict constant background, query at `40 * radius`, and
kernel center at the weight center. Compare against a log-scaled analytic
oracle and require the local Hessian contribution to be approximately
`5.864931460100122e-45`, not zero.

### F4 - P2: value and coverage paths compute unused Hessians

`try_coverage` requests a full weight jet at
`crates/georbf/src/local_trend.rs:578-582`. `try_evaluate` does likewise for
both query and center weights at lines 636-644, even though the center needs
only a value and `Value`/`First` query demand does not need a Hessian. The jet
unconditionally builds the Hessian at lines 1121-1131.

For amplitude `1e150` and radius `1e-80`, the value, amplitude square, and
inverse-radius square are finite, but the center Hessian scale overflows.
Construction succeeds; coverage and value-only evaluation then fail solely
because an unused Hessian was formed. This contradicts the demand-bounded API,
makes coverage unavailable for an accepted weight, and performs unnecessary
full-matrix work on value and gradient hot paths.

Required regression: with those parameters, prove `try_coverage` and
`try_evaluate(..., Value)` succeed without Hessian arithmetic. `Second` may
return a structured representability error.

## Independent mathematical review

- In exact arithmetic with valid finite weights, each `D_r K_r D_r` is PSD
  and the constant nonzero background term `c^2 K_bg` is SPD. Fixed invertible
  anisotropy preserves distinctness and strict PD. F2 shows that the accepted
  represented `f64` domain does not preserve the background premise.
- CPD rejection is exhaustive for the current definiteness enum and records
  component and order. Polynomial spaces, rank decisions, side conditions,
  hard constraints, and infeasibility are not applicable to this no-solve SPD
  path.
- The query gradient and Hessian product-rule formulae have the correct terms,
  signs, dimensions, and symmetry. The Gaussian formula is algebraically
  correct; F3 is an unstable evaluation-order defect.
- Center capability intersection and explicit third-order rejection are
  correct. Compile-time bounds admit only D=1, D=2, and D=3.
- Under an orthogonal coordinate change, the fixed-anisotropy and product-rule
  algebra gives the expected invariant value and rotated gradient/Hessian,
  apart from ordinary rounding and the extreme-scale defects above.
- Radius has coordinate-length units, amplitude has scalar-weight units, the
  background policy ratio is dimensionless, and anisotropy/kernel scale
  composition matches the architecture contract.
- No jitter, regularization, clipping, pseudoinverse, solver adjustment,
  unsafe code, global mutable state, or user-input panic path was found.
- Evaluation and coverage allocate no heap memory, but F4 violates the stated
  demand-bounded arithmetic and hot-path obligation.
- Diagnostics and interface dispositions are otherwise deterministic and
  truthful. Registry state remains `implemented`; integration is not claimed.

## Validation and disposition

- Local and remote heads matched exact reviewed head `48c9d51`; the worktree
  was clean before this evidence-only change.
- Draft CI run 29803650524 passed its configured Ubuntu correctness gate on the
  exact reviewed head. The Ready-only Windows, Ubuntu, macOS, and benchmark-
  smoke matrix was skipped as designed and is not claimed as passed.
- The independent reviewer passed all eight local-trend tests, the D=4 compile-
  fail Rustdoc test, the runnable example, release benchmark smoke in all three
  dimensions, workspace formatting, warning-denying georbf all-target/all-
  feature Clippy, all 58 requirement checks, and diff whitespace validation.
  Its external compile probe and high-precision calculations reproduced F1-F3.
- The parent Review task passed the same eight focused tests, all georbf
  Rustdoc including D=4 rejection, all 58 requirement checks, and complete PR
  diff whitespace validation.
- Exact implementation head `48c9d51` retains the complete stable-head standard
  gate recorded by Implement. This Review change updates only this review
  record and the bounded handoff, so it changes no production, test, manifest,
  schema, CI, build, API, numerical, registry, or dependency input.
- `cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
  installed. Miri is unavailable for pinned Rust 1.96.1. Sanitizers,
  executable fuzzing, mutation testing, general allocation instrumentation,
  API/ABI/schema snapshots, and local `actionlint` remain unavailable or
  deferred. No unexecuted check is claimed as passed.

PR #103 remains Draft and REQ-TREND-001 remains `implemented`, not
`integrated`. A fresh Repair task must address only F1-F4, add the specified
regressions, run focused checks while iterating, run the complete standard gate
after the final code change, update this evidence and the bounded handoff,
commit, push, and stop for a fresh independent re-review. This Review task must
not repair production code or begin another requirement.
