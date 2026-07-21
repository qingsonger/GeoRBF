# PR #103 Independent Review

- Requirement: REQ-TREND-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/102
- Pull request: https://github.com/qingsonger/GeoRBF/pull/103
- Branch: `codex/req-trend-001-positive-definite-local-trends`
- Reviewed head: `48c9d516721928f98dd06242a2304b8d4c9f94e3`
- Repair code/test head: `643535f4ef181764baa6a5b45605711ee2a91f7d`
- F5-F6 repair code/test head: `147cc4f6a4cec226c752127f94076c0d954e2dfc`
- Latest re-reviewed head: `8396ec9957f9ea4ab6c6e252adbb218d5c18fbd4`
- Base head: `7487cfafd0739c1f63028d4b46d7505b4ca6c1b3`
- Review date: 2026-07-21
- Latest result: F1-F6 closed; two new P1 findings F7-F8 require repair

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

At Review completion, PR #103 remained Draft and REQ-TREND-001 remained
`implemented`, not `integrated`. The Review task stopped without changing
production code and required a fresh Repair task to address only F1-F4 before
another independent review. The following section records that later repair.

## Repair evidence pending fresh re-review

Repair code/test head `643535f4ef181764baa6a5b45605711ee2a91f7d`
addresses only F1-F4:

- F1: `SmoothSpatialWeight` is now an opaque public value over a private
  representation. Its public constructors are the only safe construction
  path, and an external Rustdoc compile-fail regression proves that callers
  cannot name or forge the former variants and cached Gaussian fields.
- F2: every nonzero amplitude whose represented square is zero now returns
  `NonRepresentableWeightAmplitudeSquare`; the `1e-200` regression cannot
  reach mixture construction or produce a zero strict-background diagonal.
- F3: Gaussian gradient and Hessian factors use direct normal arithmetic on
  the ordinary path and a combined logarithmic scale when the value or an
  intermediate product is zero, subnormal, or non-finite. The independent
  extreme-scale regression retains the representable
  `5.864931460100122e-45` Hessian at forty radii.
- F4: coverage and center weights request Value only, while query weights stop
  at the caller's demand. The `amplitude=1e150`, `radius=1e-80` regression
  proves Coverage and Value succeed although Second correctly returns a
  structured Hessian representability error.

The repaired tree passed all ten focused local-trend tests, all georbf
Rustdoc including both compile-fail contracts, the runnable example, and the
D=1/D=2/D=3 release benchmark smoke with unchanged deterministic checksums.
After the final production change, the complete stable-head standard gate
passed: workspace formatting, warning-denying all-target/all-feature Clippy,
all workspace tests with all features, all workspace Rustdoc, all 58
requirement checks, and diff whitespace validation. The subsequent review
record and bounded-handoff commit changes documentation only and does not
invalidate that gate.

This Repair task does not close its own findings. PR #103 remains Draft and
REQ-TREND-001 remains `implemented`; a fresh independent mathematical and
numerical re-review of the complete repaired PR diff is required next. No
other requirement work begins here.

## Fresh independent re-review

- Re-reviewed base: `7487cfafd0739c1f63028d4b46d7505b4ca6c1b3`
- Re-reviewed repair code/test head:
  `643535f4ef181764baa6a5b45605711ee2a91f7d`
- Re-reviewed final head: `1fcd80c998ae0b83a48aef7bae965d12f1a37889`
- Re-review date: 2026-07-21
- Result: F1-F4 closed; two new P1 findings require repair

An isolated read-only project `math_reviewer` received only the bounded
requirement summary and integrated dependency closure, Issue #102 acceptance
criteria, the M6 plan, ANISOTROPY and ADR-0005/ADR-0008 contracts, the complete
PR diff, directly relevant implementation and validation evidence, and the
original F1-F4 findings. It inherited no Implement or Repair reasoning and
made no repository or remote change. Final head `1fcd80c` differs from repair
head `643535f` only in this review record and the bounded handoff.

### Re-review disposition of F1-F4

- F1 is closed: the public weight is opaque over a private representation,
  and the external compile-fail construction barrier passes.
- F2 is closed: a nonzero amplitude whose represented square is zero returns
  `NonRepresentableWeightAmplitudeSquare`; the `1e-200` regression passes.
- F3 is closed for its required extreme derivative case: the independently
  derived `5.864931460100122e-45` Hessian at forty radii is retained. F5 below
  is a distinct Value-path evaluation-order defect.
- F4 is closed: Coverage and center factors request Value only, query weights
  stop at caller demand, and the extreme amplitude/radius regression passes.

### F5 - P1: representable Gaussian values are erased by exponential underflow

`gaussian_weight_jet` forms `amplitude * exponent.exp()` at
`crates/georbf/src/local_trend.rs:1134`. The Value path returns that result at
line 1144 before the log-scaled derivative recovery can apply. With accepted
`amplitude=1e150`, `radius=1e-150`, and displacement `40 * radius`, binary64
`exp(-800)` is zero, but independent 100-digit evaluation gives the
representable full weight `3.667874584177687e-198`. With the center weight at
its amplitude, the local component value is approximately
`3.667874584177687e-48`, also representable. The implementation instead
silently discards that positive-semidefinite component and returns only the
background contribution.

Required regression: construct a public one-dimensional mixture with those
parameters, the Gaussian-weight center as the kernel center, a query at forty
radii, a unit-scale Gaussian kernel, and a small strict background. Value must
retain approximately `3.667874584177687e-48` rather than returning the
background-only result.

### F6 - P1: large Gaussian radii cache a false zero reciprocal square

`try_gaussian` accepts `inverse_radius_squared` whenever it is finite at
`crates/georbf/src/local_trend.rs:176-178`. For `radius=1e200` that cached value
rounds to zero. Hessian evaluation passes the zero at line 1173, and the
scaled helper explicitly returns zero at line 1197. With accepted
`amplitude=1e154`, the true center weight Hessian is the representable
`-1e-246`. A public mixture using a unit-scale Gaussian kernel at its
one-dimensional inflection separation consequently loses a representable
component Hessian of approximately `-6.065306597126334e-93`.

Required regression: `try_gaussian(center, 1e154, 1e200)` must return
`NonRepresentableWeightRadius`, consistent with its documented inverse-
derivative contract. If the radius remains accepted, a public inflection-point
mixture regression must instead retain the component Hessian above.

### Re-review mathematical and validation evidence

- In exact arithmetic the SPD proof remains sound: each diagonal congruence is
  positive semidefinite and the finite nonzero constant-background congruence
  is strictly positive definite. CPD rejection remains exhaustive.
- Product-rule terms, signs, Hessian symmetry, units, rotation behavior,
  fixed-anisotropy transformation, and center-capability intersection are
  correct apart from F5-F6. No hidden jitter, regularization, clipping,
  pseudoinverse, unsafe code, or pointwise heap allocation was found.
- Polynomial spaces, rank policy, hard constraints, and infeasibility are not
  applicable to this no-solve path. Interface N/A dispositions, diagnostics,
  benchmark wiring, and the registry's `implemented` state remain truthful.
- The reviewer passed all ten focused local-trend tests, all georbf Rustdoc,
  the runnable example, D=1/D=2/D=3 release benchmark smoke, workspace format,
  warning-denying georbf all-target/all-feature Clippy, all 58 requirement
  checks, complete diff whitespace validation, and independent 100-digit
  calculations for F5-F6.
- Draft CI run 29806055584 passed the exact final head's configured Ubuntu
  correctness gate. The Ready-only Windows/Ubuntu/macOS and benchmark-smoke
  matrix did not run and is not claimed as passed.
- The complete stable-head standard gate remains the exact repair-head result
  recorded above; subsequent commits through this re-review evidence change
  only review and handoff documentation. The reviewer did not rerun the full
  workspace all-feature test/Clippy/Rustdoc gate and does not claim otherwise.
- The unavailable nextest, deny, audit, semver, Miri, sanitizer, fuzzing,
  mutation, allocation-instrumentation, API/ABI/schema, and actionlint checks
  remain unexecuted and are not claimed as passed.

PR #103 remains Draft and REQ-TREND-001 remains `implemented`, not
`integrated`. A fresh Repair task must address only F5-F6, add the required
regressions, rerun the stable-head standard gate, update this evidence and the
bounded handoff, push, and stop for another independent re-review. This Review
task does not repair production code, mark the PR ready, merge, or begin
another requirement.

## Repair evidence pending fresh re-review: F5-F6

Repair code/test head `147cc4f6a4cec226c752127f94076c0d954e2dfc`
addresses only F5-F6:

- F5: Gaussian Value evaluation now falls back from a non-normal direct
  `amplitude * exp(exponent)` result to the combined logarithmic scale. The
  public one-dimensional regression retains the independently calculated
  `3.667874584177687e-48` local mixture contribution when `exp(-800)` alone
  rounds to zero.
- F6: Gaussian construction now rejects a radius if either its reciprocal or
  reciprocal square rounds to zero. The public `radius=1e200`,
  `amplitude=1e154` regression returns
  `NonRepresentableWeightRadius` instead of caching a false zero derivative
  scale.

Both regressions failed against the pre-repair implementation and passed after
the bounded fix. All 12 focused local-trend tests, all georbf Rustdoc, the
runnable example, and D=1/D=2/D=3 release benchmark smoke passed. The smoke
reported approximately 242 ns, 424 ns, and 1.21 us per Hessian evaluation,
respectively, with the established deterministic checksums.

After the final production change, the exact repair head passed the complete
standard gate: workspace format, warning-denying workspace all-target/all-
feature Clippy, all workspace tests with all features, workspace Rustdoc, all
58 requirement checks, and diff whitespace validation. The unavailable-check
list recorded above remains unchanged and no unavailable check is claimed as
passed.

This section records Repair evidence only and does not independently close
F5-F6. PR #103 remains Draft and REQ-TREND-001 remains `implemented`; a fresh
independent mathematical and numerical re-review of the complete repaired diff
is required next. This Repair does not mark the PR ready, merge it, or begin
another requirement.

## Fresh independent re-review after F5-F6 repair

- Re-reviewed base: `7487cfafd0739c1f63028d4b46d7505b4ca6c1b3`
- Re-reviewed F5-F6 repair code/test head:
  `147cc4f6a4cec226c752127f94076c0d954e2dfc`
- Re-reviewed final head: `8396ec9957f9ea4ab6c6e252adbb218d5c18fbd4`
- Re-review date: 2026-07-21
- Result: F1-F6 closed; two new P1 findings F7-F8 require repair

An isolated read-only project `math_reviewer` received only the bounded
requirement summary and integrated dependency closure, Issue #102 acceptance
criteria, the M6 plan, ANISOTROPY and ADR-0005/ADR-0008 contracts, the complete
PR diff, directly relevant implementation and validation evidence, and the
preceding findings and repairs. It inherited no Implement or Repair reasoning
and made no repository or remote change. Final head `8396ec9` differs from
repair code/test head `147cc4f` only in this review record and the bounded
handoff.

### Re-review disposition of F1-F6

- F1-F4 remain closed for their required regressions.
- F5 is closed: combined-logarithm Value evaluation retains the repaired
  representable result, and its public regression passes.
- F6 is closed: construction rejects zero reciprocal or reciprocal-square
  scales, and its public regression passes.

### F7 - P1: scaled-displacement underflow erases a representable gradient

`gaussian_weight_jet` computes `displacement * inverse_radius` at
`crates/georbf/src/local_trend.rs:1120`. A nonzero result may round to zero;
the zero is then used as an exact derivative factor at lines 1154-1160 and is
short-circuited at lines 1223-1225.

For `b(x) = a exp(-x^2/(2r^2))`, a unit Gaussian kernel evaluated at its
center has zero query gradient, so independent 120-digit arithmetic gives

```text
d/dx [b(x)b(y)k(x,y)] at x=y=delta
  = -a^2 exp(-delta^2/r^2) delta/r^2.
```

With `a=1e154`, `r=3`, and `delta=f64::from_bits(1)`, the scaled displacement
rounds to zero but the complete mixture-gradient contribution is the normal,
representable `-5.489618287124962e-17`. The implementation returns zero. This
violates the complete-gradient contract and the documented promise that an
intermediate rounded zero does not erase a representable derivative.

Required regression: construct a public D=1 mixture with those parameters,
`query == kernel_center == [f64::from_bits(1)]`, a unit Gaussian kernel,
isotropic anisotropy, and a constant strict background. Demand `First` and
require the gradient to approximate `-5.489618287124962e-17`, not zero.

### F8 - P1: mixed scaled-coordinate underflow erases a Hessian entry

At `crates/georbf/src/local_trend.rs:1171`, the implementation forms
`scaled[row] * scaled[column]` before entering the logarithmically stable
helper. Two nonzero scaled coordinates can therefore produce a zero
coefficient, which lines 1223-1225 return as exact zero.

In D=2 with amplitude `1`, radius `1e-154`, weight center `[0,0]`, and
`x=y=[delta,delta]` for `delta=f64::from_bits(1)`, each scaled coordinate is
`4.9406564584124656e-170`, but their binary64 product is zero. At a unit
Gaussian kernel center, independent 120-digit arithmetic gives the analytic
mixed mixture Hessian

```text
b(x)^2 delta^2/r^4 = 2.4410086240052807e-31,
```

while the implementation returns zero.

Required regression: construct that public D=2 mixture, demand `Second`, and
require both symmetric off-diagonal entries to approximate
`2.4410086240052807e-31`.

### Re-review validation and disposition

- The reviewer passed all 12 focused local-trend tests and diff whitespace
  validation. Those tests do not cover F7-F8. Independent 120-digit
  calculations reproduced both findings.
- The parent Review task passed the same 12 focused tests, all georbf Rustdoc,
  the runnable example, all 58 requirement checks, complete diff whitespace
  validation, and the complete exact-head standard gate: workspace format,
  warning-denying workspace all-target/all-feature Clippy, all workspace tests
  with all features, workspace Rustdoc, and requirement validation.
- Draft CI run 29807655190 passed its configured Ubuntu correctness gate on
  exact re-reviewed head `8396ec9`. Ready-only Windows, Ubuntu, macOS, and
  benchmark-smoke CI did not run and is not claimed as passed.
- The SPD proof, CPD rejection, product-rule signs and dimensions, capability
  intersection, center handling, ordinary-scale rotation behavior,
  allocation-free point evaluation, interface dispositions, diagnostics,
  benchmark wiring, and lack of hidden regularization are otherwise sound.
  Polynomial spaces, rank decisions, hard constraints, and infeasibility are
  not applicable to this no-solve primitive.
- No P0, P2, or P3 finding was identified. The unavailable nextest, deny,
  audit, semver, Miri, sanitizer, fuzzing, mutation, allocation-
  instrumentation, API/ABI/schema, and actionlint checks remain unexecuted and
  are not claimed as passed.

PR #103 remains Draft and REQ-TREND-001 remains `implemented`, not
`integrated`. A fresh Repair task must address only F7-F8, add both public
regressions, rerun focused checks and the complete stable-head standard gate,
update this evidence and the bounded handoff, push, and stop for another fresh
independent re-review. This Review task does not repair production code, mark
the PR ready, merge it, or begin another requirement.

## Repair evidence pending fresh re-review: F7-F8

Repair code/test head `2b5189d624045c16f2ca7a55b73ee6f24960e999`
addresses only F7-F8:

- F7: Gaussian gradients now combine the unscaled displacement directly with
  the cached inverse-radius square and the amplitude/exponent scale. The public
  D=1 regression retains the independent 120-digit truth
  `-5.489618287124962e-17` when `delta * inverse_radius` would round to zero.
- F8: mixed Gaussian Hessian entries now combine both inverse-radius-square
  factors and both unscaled displacements in one stable product. Canonical axis
  order preserves bitwise symmetry. The public D=2 regression retains both
  symmetric entries at the independent truth `2.4410086240052807e-31` when the
  product of scaled coordinates would round to zero.

Both regressions failed with exact zero against the pre-repair implementation
based on reviewed branch head `2dd51227ffc8b908835df07d779d450480a4d137`,
then passed after the bounded repair. All 14 focused local-trend tests, all
georbf Rustdoc, the runnable example, and the D=1/D=2/D=3 release benchmark
smoke passed. The smoke reported approximately 225 ns, 465 ns, and 1.11 us per
Hessian evaluation, respectively, with the established deterministic
checksums.

After the final code change, exact repair head `2b5189d` passed the complete
standard gate: workspace format, warning-denying workspace all-target/all-
feature Clippy, all workspace tests with all features, workspace Rustdoc, all
58 requirement checks, and diff whitespace validation. The unavailable
nextest, deny, audit, semver, Miri, sanitizer, fuzzing, mutation, allocation-
instrumentation, API/ABI/schema, and actionlint checks remain unexecuted and
are not claimed as passed.

This section records Repair evidence only and does not independently close
F7-F8. PR #103 remains Draft and REQ-TREND-001 remains `implemented`; a fresh
independent mathematical and numerical re-review of the complete repaired diff
is required next. This Repair does not mark the PR ready, merge it, or begin
another requirement.
