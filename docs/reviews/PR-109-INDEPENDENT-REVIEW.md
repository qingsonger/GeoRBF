# PR #109 Independent Review

- Requirement: REQ-TREND-002
- Issue: https://github.com/qingsonger/GeoRBF/issues/108
- Pull request: https://github.com/qingsonger/GeoRBF/pull/109
- Branch: `codex/req-trend-002-region-controls`
- Reviewed head: `9781e8fd6ac140b444a3858c3e1ce38565f42f85`
- Base head: `8535880c2d9cf2d580ac97bddf0610f9f6a68f61`
- Review date: 2026-07-22
- Result: P1 TREND002-REV-001 and P2 TREND002-REV-002/003 require Repair

## Scope and independence

A fresh read-only project `math_reviewer` received only the bounded
REQ-TREND-002 summary and integrated dependency closure, Issue #108 acceptance
criteria and exclusions, the M6 plan, ANISOTROPY and ADR-0005/ADR-0008
contracts, the exact PR diff, tests, benchmark and CI wiring, registry,
handoff, and validation evidence. It inherited no Implement reasoning and made
no repository or remote change.

The reviewer independently checked formulae, signs, dimensions, units,
SPD/CPD classification, center and boundary limits, rotation invariance,
positive definiteness, Hessian and C2 capabilities, finite and extreme
arithmetic, independent truth, allocations, hidden numerical adjustments,
public interfaces, diagnostics, tests, documentation, CI, and requirement
truthfulness. Polynomial rank, hard-constraint, infeasibility, and solver paths
do not apply to this compiler requirement.

## Findings

### TREND002-REV-001 - P1: regional derivatives lose representable values

Affected code:

- `crates/georbf/src/local_trend.rs:1415-1423`
- `crates/georbf/src/local_trend.rs:1445-1450`

`smootherstep_jet` first forms the dimensionless `t^2` and the cancellation-
prone expanded second derivative `60 t (2 t^2 - 3 t + 1)`. Only afterward does
`axis_region_gate` multiply by `1 / width` or `1 / width^2`. Accepted narrow
transition widths can therefore erase a physical derivative that remains
finite and representable.

For valid `width = 1e-153`, `x = 1e-323`, and
`t = x / width ~= 9.88131291682493e-171`, represented `t^2` underflows and the
implementation returns a zero first derivative. Independent evaluation of

```text
S'(t) / width = 30 t^2 (1 - t)^2 / width
```

gives approximately `2.929210348806549e-186`, which is finite and
representable. At `x = next_down(width)`, the represented parameter is
`next_down(1)`. The expanded second-derivative factor rounds to zero, while the
equivalent factored expression

```text
60 t (t - 1) (2 t - 1) / width^2
```

is approximately `-6.661338147750937e291`, also finite and representable.

Impact: accepted input silently loses required regional gradient and Hessian
terms, violating the C2/product-rule derivative and explicit representability
contracts. A fresh Repair must add private one-dimensional `axis_region_gate`
regressions at both points, compare against independently scaled and factored
analytic truth, and require finite nonzero derivatives before implementing the
smallest scale-safe evaluation.

### TREND002-REV-002 - P2: exact compact support can error outside the region

Affected code:

- `crates/georbf/src/local_trend.rs:1509-1518`
- `crates/georbf/src/local_trend.rs:1666-1672`

The region jet is known before the Gaussian displacement, but the evaluator
always forms `point - center` before using an exactly zero gate. For region
`[-1, 1]`, center `-f64::MAX`, and finite query `f64::MAX`, the gate and all
demanded derivatives are exactly zero, so the regional basis jet is
identically zero. The subtraction nevertheless overflows and returns
`NonFiniteWeightDisplacement`.

Impact: a compactly supported weight does not provide its documented exact
zero behavior outside the region. A fresh Repair must add a coverage or
private regional-weight regression through Hessian order for that configuration
and require a zero regional contribution without error before short-circuiting
the provably zero gate.

### TREND002-REV-003 - P2: required independent truth evidence is incomplete

Affected evidence:

- `crates/georbf/tests/trend_controls.rs:99-150`
- `crates/georbf/tests/trend_controls.rs:210-250`
- `crates/georbf/tests/trend_controls.rs:378-441`
- `crates/georbf/tests/trend_controls.rs:445-515`
- `changes/REQ-TREND-002.md:41-45`

The spheroidal/ellipsoidal test checks ordering, diagnostics, retained lengths,
and coverage, but not the compiled transform, metric, or manual kernel truth.
The finite-difference test checks only diagonal Hessian entries. No compiler-
level excessive-condition rejection is tested. Reference-gradient failures
cover only missing-project and below-minimum paths, not unknown fields,
unavailable evaluation, zero gradients, or unrepresentable norms. The change
fragment therefore overstates satisfaction of Issue #108's explicit
independent-test acceptance criterion.

Impact: required mathematical and structured-error behavior can regress while
the stated evidence remains green. A fresh Repair must add:

1. rotated spheroidal and ellipsoidal comparisons against hand-formed `B`
   matrices or manual kernel evaluations;
2. compiler rejection of a known excessive length ratio under an explicit
   maximum-condition policy;
3. an independent finite-difference check of a mixed regional Hessian entry;
4. a compact reference-gradient error table for unknown, unavailable, zero,
   and unrepresentable cases.

No additional P0, P1, P2, or P3 finding was identified.

## Validation and disposition

- The reviewer verified the exact base/head, merge base, twelve-file PR diff,
  and clean scoped worktree. The tail from stable evidence head `22760ef` to
  reviewed head `9781e8f` changes only `docs/progress/CURRENT.md`.
- The reviewer and parent Review task each passed all six public
  `trend_controls` tests. The parent also ran the public example successfully.
- Draft CI passed its configured Ubuntu correctness gate on exact reviewed
  head `9781e8f`. The Ready-only Windows, Ubuntu, macOS, and benchmark-smoke
  matrix was skipped as designed and is not claimed as passed.
- Exact stable evidence head `22760ef` retains the complete standard local gate
  recorded by Implement: workspace format, warning-denying all-target/all-
  feature Clippy, all-feature workspace tests, workspace Rustdoc, all 58
  requirement checks, and complete diff whitespace validation.
- The full workspace gate and benchmark were not rerun in this Review task.
  `actionlint`, nextest, deny, audit, semver, Miri, sanitizers, fuzzing,
  mutation testing, allocation instrumentation, and API/ABI/schema snapshots
  remain unavailable or deferred. No unexecuted check is claimed as passed.
- The fixed-kernel diagonal-congruence construction and constant strict-PD
  background otherwise preserve SPD; CPD kernels are explicitly rejected;
  direction-jump signs, dimensions, and reference normalization are otherwise
  correct; Hessian capability remains kernel-gated; and no hidden
  regularization or new dependency was found.

PR #109 remains Draft and REQ-TREND-002 remains `implemented`, not
`integrated`. A fresh Repair task must address only TREND002-REV-001,
TREND002-REV-002, and TREND002-REV-003, add the required regressions, run
focused checks and one complete stable-head standard gate after the final code
change, update this record and the bounded handoff, push, and stop for a fresh
independent re-review. Do not begin another requirement.

## Repair evidence pending fresh independent re-review

Repair code/test/evidence head `5f357891de88f9d3f64ed671e2769d8f6ce84c66`
addresses only the three findings above and remains pending a fresh independent
re-review:

- TREND002-REV-001: physical smootherstep derivatives are now formed with the
  width scale already present and the second derivative uses the factored
  `60 t (t - 1) (2 t - 1) / width^2` expression. Private D=1 regressions retain
  finite nonzero first and second derivatives at both reviewed extreme points.
- TREND002-REV-002: an exactly zero regional jet returns before Gaussian
  displacement formation. A private Hessian-order regression uses region
  `[-1, 1]`, center `-f64::MAX`, and query `f64::MAX` and requires an exact
  zero jet without error.
- TREND002-REV-003: public independent evidence now compares rotated
  spheroidal and ellipsoidal metrics with hand-formed `B`, rejects a 100:1
  length ratio under maximum condition 10, checks a mixed regional Hessian by
  four-point finite differences, and tables structured unknown-field,
  unavailable-at-center, zero-gradient, and unrepresentable-norm failures.

Focused validation passed all nine public `trend_controls` tests, all three
private repair regressions, the runnable example, and the release-mode focused
benchmark smoke (approximately 11.4 us for four controls and 43.0 us for
sixteen controls on this development machine). Exact repair head `5f35789`
passed workspace format, warning-denying workspace all-target/all-feature
Clippy, all-feature workspace tests, workspace Rustdoc, all 58 requirement
checks, and complete diff whitespace validation. The first full-gate attempt
stopped at Clippy on test-only lint violations; those tests were corrected and
the complete gate was rerun from the beginning to green.

This evidence does not close the independent findings by itself. PR #109
remains Draft and REQ-TREND-002 remains `implemented`, not `integrated`, until
a fresh read-only re-review confirms the repairs and checks for new findings.

## Fresh independent re-review after Repair

- Exact reviewed head: `e8596df9172fad00c8049f8a8b92a30fe47da0b5`
- Repair code/test/evidence head:
  `5f357891de88f9d3f64ed671e2769d8f6ce84c66`
- Base head: `8535880c2d9cf2d580ac97bddf0610f9f6a68f61`
- Re-review date: 2026-07-22
- Result: TREND002-REV-001 through TREND002-REV-003 are closed; P1
  TREND002-REV-004 and P2 TREND002-REV-005/006 require Repair

A fresh isolated read-only project `math_reviewer` received only the bounded
REQ-TREND-002 summary and integrated dependency closure, Issue #108 acceptance
criteria and exclusions, the M6 plan, ANISOTROPY and ADR-0005/ADR-0008
contracts, the complete PR and Repair diffs, directly relevant source, tests,
example and benchmark, and the recorded validation evidence. It inherited no
Implement or Repair reasoning transcript and made no repository, Git, or
GitHub change.

### Closure of TREND002-REV-001 through TREND002-REV-003

- TREND002-REV-001 is closed. `scaled_smootherstep_jet` evaluates the physical
  first derivative as `30 (t / width) t (t - 1)^2` and the physical second
  derivative as `60 t (t - 1) (2 t - 1) / width^2`, with the correct units and
  signs. The two reviewed extreme-value regressions retain finite nonzero
  derivatives.
- TREND002-REV-002 is closed as scoped. An all-zero regional jet returns before
  Gaussian displacement formation, and the reviewed `f64::MAX/-f64::MAX`
  Hessian regression passes without error.
- TREND002-REV-003 is closed. Public evidence now includes hand-formed rotated
  spheroidal and ellipsoidal metrics, a mixed regional Hessian finite
  difference, explicit condition-policy rejection, and the complete required
  reference-gradient error table.

### TREND002-REV-004 - P1: gate underflow erases representable weight terms

Affected code:

- `crates/georbf/src/local_trend.rs:1455`
- `crates/georbf/src/local_trend.rs:1518-1538`
- `crates/georbf/src/local_trend.rs:1883-1891`

For D=1, region `[0, 2]`, width one, control location, query and kernel center
`t = 1e-110`, strength `1e154`, and radius one, every constructor invariant is
satisfied, including the represented strength square `1e308`. The gate first
forms

```text
S(t) = t^3 (6 t^2 - 15 t + 10),
```

which underflows to zero before the strength is applied. Independently ordered
physical products remain finite and nonzero:

```text
strength S(t)   ~= 1e-175,
strength S'(t)  ~= 3e-65,
strength S''(t) ~= 6e45.
```

At equal query and center, the local Hessian term `b(y) b''(x)` is about
`6e-130` and is also representable. Value-only center evaluation nevertheless
short-circuits the underflowed gate to zero, erasing that Hessian contribution.
This violates the represented value and complete product-rule contract.

A Repair must first add a private D=1 `regional_gaussian_weight_jet` regression
with those exact inputs and second-derivative demand. Amplitude-first analytic
truth must require finite nonzero value, first derivative, and second
derivative before the smallest scale-safe evaluation change is implemented.

### TREND002-REV-005 - P2: compact support still evaluates the fixed kernel

Affected code:

- `crates/georbf/src/local_trend.rs:916-932`

Use a D=1 regional control with region `[-1, 1]`, transition width `0.25`,
location zero, unit strength and radius, and spheroidal lengths `0.5`, so the
fixed transform is `A = 2`. Evaluate at query `f64::MAX` and kernel center zero
through Hessian order. The query regional jet is identically zero, hence the
local component and all its query derivatives are mathematically zero. The
mixture evaluator nevertheless evaluates the component kernel; forming the
transformed separation overflows and returns a kernel/anisotropy error. The
unit-isotropic Gaussian background remains evaluable and is exactly zero at
that separation.

A Repair must add a public compiled-mixture regression for this configuration
and require `Ok` with exact zero value, gradient, and Hessian contribution from
the compact local component before skipping a provably zero query factor ahead
of fixed-kernel evaluation.

### TREND002-REV-006 - P2: a loose derivative bound rejects valid C2 widths

Affected code:

- `crates/georbf/src/local_trend.rs:113-119`

For region `[0, 1]` and width `w = 5e-154`, `1 / w^2 = 4e306` is finite. The
exact smootherstep maxima are

```text
max |S''(t)| / w^2 = (10 / sqrt(3)) / w^2 ~= 2.3094e307,
max S'(t) / w      = (15 / 8) / w        ~= 3.75e153,
```

so both demanded derivatives are representable. Construction nevertheless
rejects the width because its unattained loose bound `60 / w^2` overflows.

A Repair must add a D=1 construction regression requiring
`SmoothRegion::try_new([0], [1], 5e-154)` to succeed and verify that the second
derivative at `t = (3 - sqrt(3)) / 6` is finite and approximately `2.3094e307`
before replacing the nonphysical validation bound.

### Re-review validation and disposition

- All nine public `trend_controls` tests and all three private Repair
  regressions pass. Those regressions close TREND002-REV-001 through
  TREND002-REV-003 but do not cover the new findings.
- Exact Repair head `5f35789` retains the recorded complete local standard
  gate. The tail to reviewed head `e8596df` changes only the review record and
  bounded handoff, so no production, test, manifest, schema, CI, or build input
  changed.
- Draft Ubuntu CI run 29895932230 passed its configured correctness gate on
  exact reviewed head `e8596df`. The Ready-only Windows, Ubuntu, macOS, and
  benchmark-smoke matrix was skipped as designed and is not claimed as passed.
- The fixed-SPD diagonal-congruence construction and strict constant background
  otherwise preserve SPD. CPD rejection, rotation truth, reference-gradient
  behavior, Hessian capability checks, deterministic ordering, interfaces, and
  the absence of hidden regularization otherwise satisfy the reviewed scope.
- Polynomial rank, hard constraints, infeasibility, and solver behavior are
  not applicable to this compiler requirement. The recorded unavailable and
  deferred checks remain unexecuted and are not claimed as passed.

PR #109 remains Draft and REQ-TREND-002 remains `implemented`, not
`integrated`. A fresh Repair task must address only TREND002-REV-004,
TREND002-REV-005, and TREND002-REV-006, add the specified regressions, run
focused checks and one complete stable-head standard gate after the last code
change, update this record and the bounded handoff, push, and stop for another
fresh independent re-review. Do not begin another requirement.

## Second Repair evidence pending fresh independent re-review

Second Repair code/test/evidence head:
`00c9b3dae63b754c3e1bb89e29cdd5df0aeaaaa2`.

This Repair addresses only TREND002-REV-004, TREND002-REV-005, and
TREND002-REV-006 and remains pending a fresh independent re-review:

- TREND002-REV-004: regional smootherstep value and derivative factors now
  retain signed logarithmic scale through their combination with the control
  strength and Gaussian exponent. The reviewed D=1 inputs (`t = 1e-110`,
  strength `1e154`, unit radius, equal query and control center) retain finite
  nonzero value, first derivative, and second derivative against independently
  amplitude-first analytic truth.
- TREND002-REV-005: the mixture skips a component before center-weight and
  fixed-kernel evaluation when its complete demanded query weight jet is
  exactly zero. A public compiled-mixture Hessian regression uses the reviewed
  compact D=1 control, `A = 2`, and query `f64::MAX`; evaluation succeeds with
  exact zero value, gradient, and Hessian rather than reporting transformed-
  separation overflow.
- TREND002-REV-006: region construction now checks the attained smootherstep
  curvature maximum `10 / sqrt(3) / width^2` instead of the unattained loose
  `60 / width^2` bound. A D=1 regression accepts `width = 5e-154` and verifies
  the finite approximately `2.3094e307` second derivative at the analytic
  maximizer.

Focused validation passes all ten public `trend_controls` tests, all five
private local-trend regressions, warning-denying georbf all-target/all-feature
Clippy, the runnable example, and the release-mode focused benchmark smoke
(approximately 12.0 us for four controls and 49.7 us for sixteen controls on
this development machine). After the final production and Rustdoc change, the
complete stable-head standard gate passed: workspace format, warning-denying
workspace all-target/all-feature Clippy, all-feature workspace tests, workspace
Rustdoc, all 58 requirement checks, and complete diff whitespace validation.

This Repair evidence does not close its own findings. PR #109 remains Draft
and REQ-TREND-002 remains `implemented`, not `integrated`, until a fresh
isolated read-only `math_reviewer` verifies the exact Repair head and checks for
new P0-P3 findings.

## Fresh independent re-review after second Repair

- Exact reviewed head: `482002164a3af66e8d85e88ddfed0f1d559165a8`
- Second Repair code/test/evidence head:
  `00c9b3dae63b754c3e1bb89e29cdd5df0aeaaaa2`
- Base and merge-base head:
  `8535880c2d9cf2d580ac97bddf0610f9f6a68f61`
- Re-review date: 2026-07-22
- Result: TREND002-REV-004 through TREND002-REV-006 are closed; P2
  TREND002-REV-007 requires Repair

A fresh isolated read-only project `math_reviewer` received only the bounded
REQ-TREND-002 summary and integrated dependency closure, Issue #108 acceptance
criteria and exclusions, the M6 plan, ANISOTROPY and ADR-0005/ADR-0008
contracts, the complete PR and second-Repair diffs, directly relevant source,
tests, example and benchmark, and the recorded validation evidence. It
inherited no Implement or Repair reasoning transcript and made no repository,
Git, or GitHub change.

### Closure of TREND002-REV-004 through TREND002-REV-006

- TREND002-REV-004 is closed. Signed logarithmic gate factors are combined
  with strength and the Gaussian exponent before conversion back to represented
  values. The reviewed D=1 value, first derivative, and second derivative stay
  finite and nonzero when the unscaled smootherstep value underflows.
- TREND002-REV-005 is closed as scoped. A complete demanded query weight jet
  that is exactly zero skips center-weight and fixed-kernel evaluation, and the
  reviewed public Hessian regression returns exact zeros.
- TREND002-REV-006 is closed. Transition validation uses the attained
  smootherstep curvature maximum `10 / sqrt(3) / width^2`; width `5e-154` and
  its finite analytic maximum curvature are retained by the regression.

### TREND002-REV-007 - P2: zero center factor does not skip the fixed kernel

Affected code and missing symmetric regression:

- `crates/georbf/src/local_trend.rs:962-973`
- `crates/georbf/tests/trend_controls.rs:782`

For one component

```text
K_r(x, y) = b_r(x) b_r(y) k_r(x, y),
```

an exactly zero center factor `b_r(y)` makes the value and every query
derivative through Hessian order exactly zero. Every product-rule term retains
that center factor, so the fixed kernel is irrelevant. The evaluator reads the
zero center value but then calls the fixed kernel unconditionally.

The reviewer independently reversed the existing compact-query regression:
D=1, region `[-1, 1]`, transition width `0.25`, control and query at zero,
spheroidal length `0.5` so `A = 2`, center `f64::MAX`, and second-derivative
demand. The center gate is exactly zero, but the irrelevant transformed
separation overflows and evaluation returns
`NonFiniteTransformedDisplacementComponent { axis: 0 }` for component one.
The local component is algebraically zero, so compact support is incorrectly
argument-asymmetric at finite extreme inputs.

A Repair must first extend
`compact_control_skips_overflowing_fixed_kernel_when_query_factor_is_zero`
with the reversed query and center. It must require successful evaluation with
the same background truth and exact-zero value, gradient, and Hessian from the
compact local component. The smallest implementation change must skip the
fixed kernel once the center weight is exactly zero.

### Re-review validation and disposition

- Exact reviewed head and merge base were verified, and the tail from
  `00c9b3d` to `4820021` changes only the review record and bounded handoff.
- The reviewer passed all ten public `trend_controls` tests, all five private
  local-trend regressions, the runnable example, workspace format, all 58
  requirement checks, and complete PR diff whitespace validation. The parent
  Review task independently passed the same public and private focused tests.
- The reviewer independently reproduced TREND002-REV-007 with the exact public
  construction above. Existing tests do not cover the reversed center-zero
  case.
- Draft Ubuntu CI run 29898025166 passed its configured correctness gate on
  exact reviewed head `4820021`. The Ready-only Windows, Ubuntu, macOS, and
  benchmark-smoke matrix remains intentionally unexecuted.
- The fixed-SPD diagonal-congruence construction, strict constant background,
  CPD rejection, product-rule signs and units, rotation behavior, Hessian
  capability checks, deterministic diagnostics, allocation behavior,
  interfaces, and absence of hidden regularization otherwise satisfy the
  reviewed scope. Polynomial rank, solver hard constraints, and infeasibility
  are not applicable.

PR #109 remains Draft and REQ-TREND-002 remains `implemented`, not
`integrated`. A fresh Repair task must address only TREND002-REV-007, add the
specified symmetric regression, run focused checks and one complete stable-head
standard gate after the final code change, update this record and the bounded
handoff, push, and stop for another fresh independent re-review. Do not begin
another requirement.

## Third Repair evidence pending fresh independent re-review

Third Repair code/test/evidence head:
`8203876292982f6ca6765b5fc8963358373bba79`.

This Repair addresses only TREND002-REV-007 and remains pending a fresh
independent re-review:

- The public compact-control regression now evaluates the reviewed D=1 case in
  both argument orders through Hessian demand. Query zero with center
  `f64::MAX` succeeds with the same exact-zero background truth as the existing
  reversed case, so the compact local contribution is exact zero for value,
  gradient, and Hessian.
- After the center value-only weight evaluation, an exactly zero center factor
  skips the component before fixed-kernel evaluation. This is algebraically
  valid because every value and query-derivative product-rule term contains
  that center factor, and it restores argument-symmetric compact support at
  finite extreme inputs.
- Rustdoc, the anisotropy architecture contract, and the requirement change
  fragment now state the symmetric query-jet/center-factor short-circuit.

The new regression reproduced the reviewed
`NonFiniteTransformedDisplacementComponent` failure before the production
change. Focused validation then passed the regression, all ten public
`trend_controls` tests, all five private local-trend regressions,
warning-denying georbf all-target/all-feature Clippy, the runnable example, and
the release-mode compilation benchmark smoke (approximately 16.6 us for four
controls and 49.3 us for sixteen controls on this development machine).

After the final Rust change and one rustfmt-only correction found by the first
format check, the complete standard gate was rerun from the beginning and
passed on the stable code/test/evidence tree: workspace format,
warning-denying workspace all-target/all-feature Clippy, all-feature workspace
tests, workspace Rustdoc, all 58 requirement checks, and complete diff
whitespace validation. This documentation-only evidence tail changes no
production, test, manifest, schema, CI, or build input.

This Repair evidence does not close its own finding. PR #109 remains Draft and
REQ-TREND-002 remains `implemented`, not `integrated`, until a fresh isolated
read-only `math_reviewer` verifies exact Repair head `8203876` and checks for
new P0-P3 findings.

## Fresh independent re-review after third Repair

- Exact reviewed head: `3a8ba8f8ad9e83b29ebf8bb8ef0f8cd35ae87de2`
- Third Repair code/test/evidence head:
  `8203876292982f6ca6765b5fc8963358373bba79`
- Base and merge-base head:
  `8535880c2d9cf2d580ac97bddf0610f9f6a68f61`
- Re-review date: 2026-07-22
- Result: TREND002-REV-007 is closed; P1 TREND002-REV-008 requires Repair

A fresh isolated read-only project `math_reviewer` received only the bounded
REQ-TREND-002 summary and integrated dependency closure, Issue #108 acceptance
criteria and exclusions, the M6 plan, ANISOTROPY and ADR-0005/ADR-0008
contracts, the complete PR and third-Repair diffs, directly relevant source,
tests, example and benchmark, and the recorded validation evidence. It
inherited no Implement or Repair reasoning transcript and made no repository,
Git, or GitHub change.

### Closure of TREND002-REV-007

TREND002-REV-007 is closed as scoped. Exact Repair head `8203876` skips an
exactly zero compact center factor before fixed-kernel evaluation, and the
public regression covers both argument orders through Hessian demand. Every
query derivative of `b(x) b(y) k(x, y)` retains `b(y)`, so this short-circuit
is algebraically valid for a mathematically exact compact-support zero.

### TREND002-REV-008 - P1: rounded-zero Gaussian factors erase a representable mixture value

Affected code and overstated contract:

- `crates/georbf/src/local_trend.rs:957-970`
- `docs/architecture/ANISOTROPY.md:360`

`WeightJet` retains only represented `f64` values, so the repaired
short-circuits cannot distinguish a mathematically exact compact-support zero
from a Gaussian value that merely underflowed to represented zero. Consider a
valid D=1 non-regional control at zero with strength `1e154`, influence radius
one, a Gaussian fixed kernel, isotropic fixed length `100`, query zero, and
kernel center `47`. The center-weight logarithm is

```text
log(b(47)) = log(1e154) - 47^2 / 2 ~= -749.9019,
```

so the individual represented center weight is zero. The independently
combined value is nevertheless

```text
b(0) b(47) k(0, 47)
  = exp(2 log(1e154) - 47^2 / 2)
    exp(-(47 / 100)^2 / 2)
  ~= 1.878351700364362e-172,
```

which is finite and representable. With center `47`, the center-factor branch
skips the component; after reversing the arguments, the query-jet branch skips
it. Both orders therefore return zero and silently erase the valid local
kernel contribution. This violates the scale-aware evaluation contract and
shows that represented zero alone does not prove a mathematical zero.

A fresh Repair must first add a public compiled D=1 regression with those
exact inputs. It must evaluate both argument orders with value demand, isolate
the local term using a background whose value is zero at separation `47`, and
compare with the independent logarithmic-domain value above. The smallest
production repair must retain zero provenance or logarithmic scale far enough
to skip only a mathematically exact compact-support zero; it must not weaken
the valid TREND002-REV-007 compact short-circuit.

### Re-review validation and disposition

- The reviewer verified the complete PR diff, exact Repair and documentation
  tail, and independently derived the counterexample above. It passed all ten
  public trend-control tests, all five private local-trend regressions, the
  exact TREND002-REV-007 regression, and complete diff whitespace validation.
- The parent Review task independently passed the same ten public and five
  private focused tests, workspace format, all 58 requirement checks, and
  complete diff whitespace validation on exact reviewed head `3a8ba8f`.
- Exact stable Repair head `8203876` retains the recorded complete local
  standard gate. The tail to `3a8ba8f` changes only this review record and the
  bounded Markdown handoff.
- Draft Ubuntu CI run 29902996233 passed its configured correctness gate on
  exact reviewed head `3a8ba8f`. The Ready-only Windows, Ubuntu, macOS, and
  benchmark-smoke matrix remains intentionally unexecuted.
- The fixed-SPD diagonal-congruence proof, strict background, CPD rejection,
  smootherstep signs and units, C2/Hessian product rules, rotated metric truth,
  reference-gradient behavior, condition policy, diagnostics, allocation
  behavior, interfaces, and absence of hidden regularization otherwise satisfy
  the reviewed scope. Polynomial rank, hard constraints, solver infeasibility,
  and polynomial spaces are not applicable.

No additional P0, P2, or P3 finding was identified. PR #109 remains Draft and
REQ-TREND-002 remains `implemented`, not `integrated`. A fresh Repair task must
address only TREND002-REV-008, add the specified regression, run focused checks
and one complete stable-head standard gate after the final code change, update
this record and the bounded handoff, push, and stop for another fresh
independent re-review. Do not begin another requirement.

## Fourth Repair evidence pending fresh independent re-review

Fourth Repair code/test/contract and stable full-gate head:
`accad99a9f84145d75199dc6e9fc1bf5996e3a77`.

This Repair addresses only TREND002-REV-008 and remains pending a fresh
independent re-review:

- A public compiled D=1 regression uses strength `1e154`, unit influence
  radius, fixed Gaussian length `100`, query/control points zero and `47`, and
  both argument orders. Its strict background kernel underflows at that
  separation, isolating the local term, and its expected value is formed
  independently in the logarithmic domain.
- Internal weight jets now retain represented value, signed logarithmic scale,
  and mathematical exact-zero provenance. Mixture value, gradient, and Hessian
  products combine the query and center weight scales before final conversion
  to `f64`, so an individually underflowed Gaussian factor cannot erase a
  representable contribution.
- Only a mathematically exact-zero complete query jet or center factor skips
  fixed-kernel evaluation. The exact compact regional short-circuit from
  TREND002-REV-007 remains argument-symmetric through Hessian demand.
- The direct represented path preserves the established ordered factor
  multiplication, so existing narrow/subnormal derivative truth is unchanged;
  logarithmic evaluation is the fallback when the direct product underflows or
  overflows.

Before the production change, the new regression reproduced the reviewed
failure with expected `1.87835170036433494e-172` and actual zero. Focused
validation then passed all eleven public `trend_controls` tests, all fifteen
`local_trend` integration tests, the five private local-trend unit regressions,
warning-denying georbf all-target/all-feature Clippy, the runnable example, and
the release-mode compilation benchmark smoke (approximately 12.4 us for four
controls and 43.9 us for sixteen controls on this development machine).

After the final production, test, Rustdoc, architecture, and change-fragment
edit, the complete standard gate passed on exact stable head `accad99`:
workspace format, warning-denying workspace all-target/all-feature Clippy,
all-feature workspace tests, workspace Rustdoc, all 58 requirement checks, and
complete diff whitespace validation. This evidence and bounded-handoff tail
changes no production, test, manifest, schema, CI, or build input.

This Repair evidence does not close its own finding. PR #109 remains Draft and
REQ-TREND-002 remains `implemented`, not `integrated`, until a fresh isolated
read-only `math_reviewer` verifies exact Repair head `accad99` and checks for
new P0-P3 findings.

## Fresh independent re-review after fourth Repair

- Exact reviewed fourth Repair code/test/contract head:
  `accad99a9f84145d75199dc6e9fc1bf5996e3a77`
- Documentation-only evidence head:
  `d8c5a9eefe9cee74b073f6a6a70170b96c0cfc75`
- Base and merge-base head:
  `8535880c2d9cf2d580ac97bddf0610f9f6a68f61`
- Re-review date: 2026-07-22
- Result: TREND002-REV-008 is closed and TREND002-REV-007 remains closed; P1
  TREND002-REV-009, TREND002-REV-010, and TREND002-REV-011 require Repair

A fresh isolated read-only project `math_reviewer` received only the bounded
REQ-TREND-002 summary and integrated dependency closure, Issue #108 acceptance
criteria and exclusions, the M6 plan, ANISOTROPY and ADR-0005/ADR-0008
contracts, the complete PR and fourth-Repair diffs, directly relevant source,
tests, example and benchmark, and recorded validation evidence. It inherited
no Implement or Repair reasoning transcript and made no repository, Git, or
GitHub change.

### Closure of TREND002-REV-008 and retained TREND002-REV-007

- TREND002-REV-008 is closed for its exact published strength, influence
  radius, fixed-kernel length, separation, and both argument orders. The
  regression retains the independently derived finite local value when one
  Gaussian weight individually underflows.
- TREND002-REV-007 remains closed. Mathematically exact compact query jets and
  center factors still skip the irrelevant fixed kernel symmetrically through
  Hessian demand.

### TREND002-REV-009 - P1: rounded log equality is treated as exact cancellation

Affected code and contract:

- `crates/georbf/src/local_trend.rs:1611-1631`
- `crates/georbf/src/local_trend.rs:1842-1849`
- `docs/architecture/ANISOTROPY.md:359-362`

`StableFactor::sum` returns the mathematical exact-zero sentinel whenever two
opposite-sign factors have equal represented logarithms. Equality of rounded
binary64 logarithms does not prove equality of the underlying factors.

For a valid D=1 regional control with region `[0, 2]`, transition width one,
strength `1e154`, radius `1e-16`, control location and kernel center
`c = 9.460674157303392e-18`, query `x = 1.78e-16`, and a unit Gaussian fixed
kernel and metric, the two analytic terms of

```text
b'(x) = A exp(-(x-c)^2/(2r^2)) [S'(x) - (x-c)S(x)/r^2]
```

receive the same rounded log magnitude `284.0495302377682`. The implementation
therefore marks their sum as mathematically zero. High-precision evaluation
from the exact represented inputs instead gives approximately
`b'(x) = -2.2122087785713344e107`, `b(c) = 8.467715431458748e103`, and a
complete local gradient of `-1.8732354411916995e211`. The evaluator drops that
term and retains only an approximately `-1.94495619321668e195` fixed-kernel
gradient term.

A Repair must first add a public compiled D=1 regional-control gradient
regression with those exact inputs and compare against independently evaluated
analytic truth. The smallest production repair must never promote equality of
rounded logarithmic magnitudes to mathematical exact-zero provenance.

### TREND002-REV-010 - P1: premature derivative conversion rejects a finite product

Affected code and contract:

- `crates/georbf/src/local_trend.rs:1430-1462`
- `crates/georbf/src/local_trend.rs:939-942`
- `docs/architecture/ANISOTROPY.md:359-362`

`WeightJet::try_from_stable` requires every individually demanded weight
derivative to be representable as `f64` before it can be multiplied by the
center factor and fixed kernel. A complete mixture contribution can remain
finite even when that individual derivative does not.

For a valid nonregional D=1 control with strength `1e154`, radius `1e-100`,
query and control location zero, kernel center `3.2e-99`, and a unit Gaussian
fixed kernel and metric, the query-weight Hessian has magnitude `1e354` and is
rejected. The center weight is approximately `4.377491037053051e-69`, so the
dominant complete Hessian contribution is approximately
`-4.3774910370537e285`, which is finite and representable; its log magnitude
`657.7132272409754` is below `ln(f64::MAX) = 709.782712893384`.

A Repair must first add a public compiled D=1 second-derivative regression for
those exact inputs, require successful evaluation, and compare the complete
Hessian against independent logarithmic analytic truth. Representability must
be decided after each complete mixture term is formed.

### TREND002-REV-011 - P1: fixed-kernel underflow is marked mathematically exact

Affected code and evidence:

- `crates/georbf/src/local_trend.rs:1540-1554`
- `crates/georbf/src/local_trend.rs:961-966`
- `crates/georbf/src/local_trend.rs:2055-2077`
- `changes/REQ-TREND-002.md:58-59`

`checked_stable_terms` wraps represented fixed-kernel values with
`StableFactor::from_factors`. A kernel value that merely rounded to zero is
therefore assigned mathematical exact-zero provenance and cannot be recovered
after multiplication by large but valid weight factors.

For a valid nonregional D=1 control with strength `1e154`, influence radius
`1000`, fixed Gaussian length one, query zero, center `39`, and a unit metric,
the represented fixed-kernel value is zero because `exp(-760.5)` underflows.
The complete local value is nevertheless

```text
exp(2 ln(1e154) - 0.5 (39/1000)^2 - 0.5 39^2)
    = 5.232584273707644e-23,
```

while the implementation returns zero. A unit-Gaussian background also
underflows at that separation, so a regression can isolate the local value.

A Repair must first add a public compiled D=1 value regression with those
exact inputs and independent logarithmic truth. The smallest production repair
must retain exact-zero or logarithmic provenance for fixed-kernel values through
complete mixture products, analogous to the repaired weight provenance.

### Re-review validation and disposition

- The reviewer verified the exact base and Repair head, the complete scoped PR
  and fourth-Repair diffs, and that the tail from `accad99` through `d8c5a9e`
  changes only this review record and the bounded handoff.
- The exact TREND002-REV-008 and TREND002-REV-007 focused regressions and all
  five private `local_trend` regressions pass. Complete PR diff whitespace
  validation also passes.
- The parent Review task independently passed all eleven public
  `trend_controls` tests, all five private `local_trend` regressions, workspace
  format, all 58 requirement checks, and working-diff whitespace validation.
- Draft Ubuntu CI passed its configured correctness gate on documentation-only
  evidence head `d8c5a9e`. The Ready-only Windows, Ubuntu, macOS, and
  benchmark-smoke matrix remains intentionally unexecuted.
- Exact stable fourth Repair head `accad99` retains the recorded complete local
  standard gate. No production, test, manifest, schema, CI, or build input
  changed in the documentation-only tail, so the full workspace gate was not
  rerun.
- The fixed-SPD diagonal-congruence construction, strict background, CPD
  rejection, C2 signs and dimensions, capability gating, interfaces,
  diagnostics, allocation behavior, and absence of hidden regularization
  otherwise satisfy the reviewed scope. Polynomial spaces, rank decisions,
  hard constraints, and solver infeasibility are not applicable.

No additional P0, P2, or P3 finding was identified. PR #109 remains Draft and
REQ-TREND-002 remains `implemented`, not `integrated`. A fresh Repair task must
address only TREND002-REV-009, TREND002-REV-010, and TREND002-REV-011, add the
specified regressions before the smallest production repair, run focused
checks and one complete stable-head standard gate after the last code change,
update this record and the bounded handoff, push, and stop for another fresh
independent re-review. Do not mark the PR ready, merge it, or begin another
requirement.

## Fifth Repair evidence pending fresh independent re-review

Fifth Repair code/test/contract and stable full-gate head:
`a2c04f0a9fd7990e9efd4f7a93ce7d6c4696290c`.

This Repair addresses only TREND002-REV-009, TREND002-REV-010, and
TREND002-REV-011 and remains pending a fresh independent re-review:

- TREND002-REV-009: signed factors now retain compensated direct products and
  sums. Equal rounded logarithmic magnitudes can use a finite compensated
  residual but can never create mathematical exact-zero provenance. The public
  compiled D=1 regional regression retains the independently evaluated
  approximately `-1.87e211` gradient scale instead of the former approximately
  `-1.94e195` residual.
- TREND002-REV-010: weight-jet derivatives remain signed stable factors until
  each complete mixture product is formed. The reviewed individually
  overflowing query-weight Hessian is scaled by its center weight and fixed
  kernel before representability is decided, returning the independently
  derived finite approximately `-4.38e285` complete Hessian.
- TREND002-REV-011: a fixed Gaussian kernel value retains its analytic
  logarithmic scale separately from the represented kernel jet. The reviewed
  underflowed fixed-kernel value combines with both valid large weight factors
  to recover the independently derived approximately `5.23e-23` local value.

The new regressions first reproduced all three reviewed failures. Focused
validation then passed all fourteen public `trend_controls` tests, all fifteen
`local_trend` integration tests, all five private local-trend regressions, and
warning-denying georbf all-target/all-feature Clippy. The release-mode focused
benchmark completed at approximately 23.4 us for four controls and 61.5 us for
sixteen controls over 10,000 compilations per case on this development machine.

After the final production, test, Rustdoc, architecture, and change-fragment
edit, exact stable head `a2c04f0` passed workspace format, warning-denying
workspace all-target/all-feature Clippy, all-feature workspace tests, workspace
Rustdoc, all 58 requirement checks, and complete diff whitespace validation.
The remaining evidence and bounded-handoff edits change only Markdown review
state and do not alter production, test, manifest, schema, CI, or build input.

This Repair evidence does not close its own findings. PR #109 remains Draft and
REQ-TREND-002 remains `implemented`, not `integrated`, until a fresh isolated
read-only `math_reviewer` verifies exact Repair head `a2c04f0` and checks for
new P0-P3 findings.

## Fresh independent re-review after fifth Repair

The fresh isolated read-only `math_reviewer` reviewed exact fifth Repair
code/test/contract head `a2c04f0a9fd7990e9efd4f7a93ce7d6c4696290c` without
the implementation task's reasoning transcript. The evidence-only tail through
`e7abea2` changes only this review record and the bounded handoff.

Result: TREND002-REV-009, TREND002-REV-010, and TREND002-REV-011 are closed for
their exact published regressions, and TREND002-REV-007/008 remain closed. One
new P1 finding, TREND002-REV-012, requires Repair. No P0, P2, or P3 finding was
identified.

### Closure of TREND002-REV-009 through TREND002-REV-011

- TREND002-REV-009 is closed for its exact input. Independent evaluation gave
  a regional query derivative of approximately `-2.2122087785713344e107`, a
  center weight of approximately `8.467715431458746e103`, and the complete
  gradient `-1.873235441191699523e211`. The regression's 15% tolerance is
  appropriate for its approximately `2.08e16` cancellation condition and
  excludes the former `1e195`-scale result.
- TREND002-REV-010 is closed. The independently derived complete Hessian is
  approximately `-4.377491037053051603e285`; delaying representability until
  the complete weight/kernel term is formed retains it within the published
  `512 * EPSILON` relative tolerance.
- TREND002-REV-011 is closed for its required value case. The independently
  derived local value is approximately `5.232584273707341641e-23`, retained
  within the published `1024 * EPSILON` relative tolerance.
- TREND002-REV-007 remains closed: mathematically exact compact query jets and
  center factors still short-circuit symmetrically before fixed-kernel
  evaluation through Hessian demand.
- TREND002-REV-008 remains closed: individually underflowed Gaussian weights
  retain non-exact-zero provenance and recover the required value in both
  argument orders.

### TREND002-REV-012 - P1: fixed Gaussian derivatives lose recoverable scale

Locations: `crates/georbf/src/local_trend.rs:977`,
`crates/georbf/src/local_trend.rs:989`,
`crates/georbf/src/local_trend.rs:2140`, and
`crates/georbf/src/local_trend.rs:2234`; the exposing public input begins at
`crates/georbf/tests/trend_controls.rs:980`.

The fifth Repair preserves an analytic stable factor only for the fixed
Gaussian kernel value. Kernel gradients and Hessians remain represented
`f64` values and enter `StableFactor::from_factors`; an underflowed represented
derivative zero is consequently treated as an exact multiplicative zero even
when the two weight factors make the complete product finite.

The existing TREND002-REV-011 input proves the gradient failure independently.
For the exact represented inputs `A = 1e154`, influence radius `R = 1000`,
query `q = 0`, center `y = 39`, and a unit fixed Gaussian, the retained local
value is

```text
L = A^2 exp(-0.5 (39 / R)^2 - 0.5 39^2)
  = 5.2325842737073416407e-23.
```

The query weight derivative is zero and the fixed-kernel query derivative is
`39 k`, so the complete local gradient is

```text
39 L = 2.0407078667458632399e-21
     = 0x1.34620fd0577b6p-69 after binary64 rounding.
```

The background derivative is approximately `5.10565e-330` and rounds to zero.
The represented fixed-kernel derivative also underflows because its log
magnitude is approximately `-756.84`, but unlike the background it becomes
finite after multiplication by the two large weights. Exact Repair head
`a2c04f0` nevertheless returns zero. The analogous represented Hessian path
can lose fixed-kernel derivative contributions for the same reason.

Required regression and Repair: extend
`fixed_gaussian_underflow_does_not_erase_representable_mixture_value` to demand
the first derivative for this same query/center and require approximately
`2.0407078667458633e-21` within about `1024 * EPSILON` relative tolerance. The
implementation must retain analytic stable Gaussian gradient and Hessian
factors until each complete mixture term is formed.

### Re-review validation and disposition

The reviewer inspected the complete `origin/main...a2c04f0` diff, the fifth
Repair delta, the requirement and dependency summaries, Issue #108, M6 plan
context, ANISOTROPY contract, ADR-0005/0008, source, tests, example, benchmark,
change fragment, registry, and prior review evidence. It independently ran:

- all 14 public `trend_controls` tests;
- all 15 public `local_trend` integration tests;
- all five private `local_trend` regressions;
- complete diff whitespace validation; and
- compact requirement `show` and `deps` commands.

The supplied stable-head full gate was not rerun because no code, test,
manifest, schema, CI, or build input changed after `a2c04f0`. Ready-only
Windows/Ubuntu/macOS and benchmark-smoke CI remain intentionally unexecuted
while PR #109 is Draft.

PR #109 must remain Draft and REQ-TREND-002 must remain `implemented`, not
`integrated`. A fresh Repair task must address only TREND002-REV-012, add the
specified independent derivative regression, run focused checks and one final
stable-head standard gate, update evidence, push, and stop for another fresh
independent re-review.
