# PR #109 Independent Review

- Requirement: REQ-TREND-002
- Issue: https://github.com/qingsonger/GeoRBF/issues/108
- Pull request: https://github.com/qingsonger/GeoRBF/pull/109
- Branch: `codex/req-trend-002-region-controls`
- Latest re-reviewed code/test/contract head: `2a25f4467588edd7ac040074a79e3aeed6b3f459`
- Latest reviewed evidence head: `8593ec5a9b5ba289964402401b16724d114a1d4c`
- Base head: `8535880c2d9cf2d580ac97bddf0610f9f6a68f61`
- Review date: 2026-07-22; latest re-review: 2026-07-23
- Result: TREND002-REV-001 through TREND002-REV-017 closed; P1 finding
  TREND002-REV-018 requires Repair; no other P0-P3 finding remains

## Scope and independence

A fresh read-only project `math_reviewer` received only the bounded
REQ-TREND-002 summary and integrated dependency closure, Issue #108 acceptance
criteria and exclusions, the M6 plan, ANISOTROPY and ADR-0005/ADR-0008
contracts, the exact PR diff, tests, benchmark and CI wiring, registry,
handoff, and validation evidence. It inherited no Implement or Repair reasoning
and made no repository or remote change.

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

## Sixth Repair evidence pending fresh independent re-review

Sixth Repair code/test/contract and stable full-gate head:
`cc5fa6f0593f40e57218e98a9322c0b1fc7ef012`.

This Repair addresses only TREND002-REV-012 and remains pending a fresh
independent re-review:

- The existing fixed-Gaussian-underflow public regression now demands Hessian
  order and independently checks the retained approximately `5.23e-23` value,
  `2.04e-21` gradient, and `7.95e-20` Hessian. Before the production change,
  the new gradient assertion reproduced the finding with expected
  `2.040707866745981e-21` and actual zero.
- Fixed Gaussian kernels now retain a complete internal stable jet. Its query
  gradient uses the analytic transformed-coordinate projection
  `-A^T A(x-y)/ell^2`, and its query Hessian retains both the projection outer
  product and `-A^T A/ell^2` curvature terms as signed stable factors. Value,
  gradient, and Hessian representability is decided only after the two weight
  factors and each complete mixture term are combined.
- Non-Gaussian kernels retain their prior represented-jet behavior. Exact
  compact-support short-circuits, fixed-SPD construction, CPD rejection, and
  the prior TREND002-REV-007 through TREND002-REV-011 regressions are unchanged.

Focused validation passed all fourteen public `trend_controls` tests, all
fifteen `local_trend` integration tests, all five private local-trend
regressions, warning-denying georbf all-target/all-feature Clippy, and complete
diff whitespace validation. The release-mode focused benchmark smoke completed
at approximately 8.7 us for four controls and 36.0 us for sixteen controls over
200 compilations per case on the development machine.

After the final production, test, Rustdoc, architecture, and change-fragment
edit, exact stable head `cc5fa6f` passed workspace format, warning-denying
workspace all-target/all-feature Clippy, all-feature workspace tests, workspace
Rustdoc, all 58 requirement checks, and complete diff whitespace validation.
The remaining review-record and bounded-handoff edits change only Markdown
review state and do not alter production, test, manifest, schema, CI, or build
input.

This Repair evidence does not close its own finding. PR #109 remains Draft and
REQ-TREND-002 remains `implemented`, not `integrated`, until a fresh isolated
read-only `math_reviewer` verifies exact Repair head `cc5fa6f` and checks for
new P0-P3 findings.

## Fresh independent re-review after sixth Repair

A fresh isolated read-only project `math_reviewer` reviewed exact sixth Repair
code/test/contract head `cc5fa6f0593f40e57218e98a9322c0b1fc7ef012`
against base and merge-base `8535880c2d9cf2d580ac97bddf0610f9f6a68f61`.
It received only the bounded REQ-TREND-002 summary and integrated dependency
closure, Issue #108 acceptance criteria and exclusions, M6 plan context,
ANISOTROPY and ADR-0005/ADR-0008 contracts, complete PR and Repair diffs,
source, tests, example, benchmark, registry, and validation evidence. It
inherited no Repair reasoning and made no repository or remote change. The
evidence-only tail from `cc5fa6f` through `860eed1` changes only this review
record and the bounded handoff.

Result: TREND002-REV-012 is closed for its exact published regression, and
TREND002-REV-007 through TREND002-REV-011 remain closed. One new P1 finding,
TREND002-REV-013, requires Repair. No P0, P2, or P3 finding was identified.

### Closure of TREND002-REV-012

For the exact represented inputs with strength `A = 1e154`, influence radius
`R = 1000`, query `q = 0`, center `y = 39`, and a unit fixed Gaussian, the
independent high-precision values are

```text
L       = 5.2325842737073416406723e-23
dL/dq   = 39 L = 2.0407078667458632398622e-21
d2L/dq2 = (39^2 - 1 - R^-2) L
        = 7.9535280908025750201146e-20.
```

The background value and derivatives round below the binary64 subnormal range.
The regression's `f64` logarithmic oracle is approximately `5.77e-14`
relatively above the high-precision truth for all three quantities, within its
`1024 * EPSILON` relative tolerance. The repaired implementation has the
correct analytic signs and tensor form: the query gradient retains
`-A^T A(x-y)/ell^2`, the Hessian retains the projection outer product and
`-A^T A/ell^2` curvature, and value, gradient, and Hessian remain signed stable
factors until each complete term is combined with both weights.

The exact compact query and center-factor short-circuits remain symmetric
through Hessian demand, individually underflowed Gaussian weights retain
non-exact-zero provenance in both argument orders, equal rounded logarithms do
not invent mathematical exact-zero provenance, overflowing weight Hessians are
scaled only as complete terms, and fixed-Gaussian value underflow remains
recoverable. TREND002-REV-007 through TREND002-REV-011 therefore remain closed
for their published regressions.

### TREND002-REV-013 - P1: represented kernel preflight rejects finite complete Gaussian terms

Locations: `crates/georbf/src/local_trend.rs:2248-2262`,
`crates/georbf/src/model.rs:164-197`, and
`crates/georbf/src/anisotropy.rs:648-669`. This contradicts the demand-bounded
and complete-term representability contract in
`docs/architecture/ANISOTROPY.md:365-369`.

`component_kernel_jet` first invokes the generic represented
`try_spatial_jet` before constructing the stable Gaussian jet. That generic
path forms and anisotropy-transforms a full represented jet, including second
derivatives even for `Value` demand. An individually overflowing fixed-kernel
Hessian therefore aborts evaluation before two small weights can make the
complete local contribution finite.

An exact accepted D=1 reproducer uses a local Gaussian length `1e-100`,
spheroidal axial and transverse lengths `1e-154` with condition number one,
strength `1e-154`, influence radius one, control and query at zero, center
`5e-255`, and an ordinary strict Gaussian background. Exact sixth Repair head
`cc5fa6f` returns

```text
Kernel {
    component: 1,
    source: Anisotropy(
        NonFiniteSecondDerivative { row: 0, column: 0 }
    )
}
```

for `Value`, `First`, and `Second` demand. Yet after multiplication by both
weights, the complete query Hessian is independently finite at approximately
`-6.6187267693844664e199`.

Required regression and Repair: add one public compiled D=1 regression with
those exact inputs. `Value` demand must succeed without evaluating unused
derivatives, and `Second` demand must return the independently log-evaluated
finite Hessian. Route Gaussian evaluation directly through a demand-bounded
stable jet before requiring any individually represented derivative.

### Re-review validation and disposition

The reviewer independently passed all fourteen public `trend_controls` tests,
all fifteen `local_trend` integration tests, all five private local-trend
regressions, complete PR and sixth-Repair diff whitespace validation, and the
compact requirement `show` and dependency-closure commands. It confirmed the
fixed-SPD diagonal-congruence construction, strict background, CPD rejection,
C2 signs and dimensions, capability gating apart from TREND002-REV-013,
diagnostics, deterministic ordering, allocation behavior, interface
disposition, and absence of hidden regularization. Polynomial spaces,
scale-aware rank decisions, hard constraints, and solver infeasibility do not
apply.

The immutable `cc5fa6f` full workspace gate was not rerun. Ready-only Windows,
Ubuntu, macOS, and benchmark-smoke CI remain intentionally unexecuted. PR #109
must remain Draft and REQ-TREND-002 remains `implemented`, not `integrated`.
A fresh Repair task must address only TREND002-REV-013, add the specified
regression before the smallest production repair, run focused checks and one
final stable-head standard gate, update evidence, push, and stop for another
fresh independent re-review. Do not mark the PR ready, merge it, or begin
another requirement.

## Seventh Repair of TREND002-REV-013

Exact seventh Repair code/test/contract head:
`42c56862b61591a70d5c82bb17721bad7a96578a`.

The public D=1 regression uses the review's exact represented inputs: fixed
Gaussian length `1e-100`, condition-one axial and transverse metric lengths
`1e-154`, strength `1e-154`, unit influence radius, zero control and query,
and center `5e-255`. Before the production change, even `Value` demand
reproduced `NonFiniteSecondDerivative { row: 0, column: 0 }`. The regression
now proves that value-only evaluation succeeds without evaluating an unused
Hessian and that `Second` demand retains the independently log-evaluated
approximately `-6.6187267693844664e199` complete Hessian.

The smallest production repair routes the Gaussian variant directly through
the existing demand-bounded stable kernel jet. The generic represented
`try_spatial_jet` is now invoked only for non-Gaussian variants, so no
individually overflowing Gaussian derivative is required before both weights
scale the complete term. The fixed-SPD construction, derivative formulae,
capability checks, non-Gaussian evaluation, and structured error mapping are
otherwise unchanged.

Focused validation passed all fifteen public `trend_controls` tests, all
fifteen `local_trend` integration tests, all five private local-trend
regressions, and complete diff whitespace validation. After the final
production/test change, exact head `42c5686` passed workspace format,
warning-denying workspace all-target/all-feature Clippy, all-feature workspace
tests, workspace Rustdoc, and all 58 requirement checks. The later evidence
tail changes Markdown only.

TREND002-REV-013 is repaired pending a fresh independent re-review. PR #109
remains Draft and REQ-TREND-002 remains `implemented`, not `integrated`.
Ready-only Windows, Ubuntu, macOS, and benchmark-smoke CI remain intentionally
unexecuted. No unavailable check is claimed as passed.

## Fresh independent re-review after seventh Repair

- Exact reviewed evidence head:
  `b0ff092195595862a9a314c4d3cc3975c1c94490`
- Seventh Repair code/test/contract head:
  `42c56862b61591a70d5c82bb17721bad7a96578a`
- Base and merge-base:
  `8535880c2d9cf2d580ac97bddf0610f9f6a68f61`
- Re-review date: 2026-07-22
- Result: TREND002-REV-013 closed; P1 TREND002-REV-014 requires Repair; no
  other P0-P3 finding remains

A fresh isolated read-only project `math_reviewer` received only the bounded
REQ-TREND-002 summary and integrated dependency closure, Issue #108 acceptance
criteria and exclusions, M6 plan context, ANISOTROPY and ADR-0005/ADR-0008
contracts, the complete PR and seventh-Repair diffs, directly relevant source,
tests, example, benchmark, registry, change evidence, handoff, and recorded
validation evidence. It inherited no Implement or Repair reasoning transcript
and made no repository, Git, or GitHub change.

### Closure of TREND002-REV-013

TREND002-REV-013 is closed for its published regression. Gaussian evaluation
now enters the demand-bounded stable jet directly; the generic represented
`try_spatial_jet` call is confined to non-Gaussian variants, so value demand no
longer evaluates the unused overflowing Hessian. For the reviewed D=1 inputs,
the independent complete local Hessian is

```text
-(0.75) exp(-0.125) * 10^200 = -6.6187267693844664e199,
```

and the exact public regression passes. The analytic signs, metric scaling,
and complete-term representability are correct for that case.

### TREND002-REV-014 - P1: rounded displacement erases a representable regional Hessian

Affected code:

- `crates/georbf/src/local_trend.rs:1914-1953`
- `crates/georbf/src/local_trend.rs:1989-2015`

`gaussian_weight_state` computes the exact subtraction residual, but the
Gaussian exponent and Hessian factors use only the rounded displacement. The
factor `displacement - radius` can therefore become represented zero even when
the exact difference between the represented input coordinates is nonzero.

An accepted D=1 counterexample uses a regional control whose query and kernel
center are on the plateau of region `[-2, 2]` with transition width `0.25`,
control location `c = -2^-53`, query and kernel center `x = y = 1`, unit
strength and influence radius, unit fixed anisotropy, fixed Gaussian length
`1e100`, and a valid negligible strict background such as constant weight
`2^-537`. For the exact represented coordinates,

```text
d = x - c = 1 + 2^-53,
b(x) b(y) = exp(-d^2),
H_local = exp(-d^2) (d^2 - 1 - 1e-200)
        = 8.168564517495419e-17 approximately.
```

Binary64 subtraction rounds `1 - (-2^-53)` to exactly one. The current
Hessian factors consequently erase `d^2 - 1` and retain only the fixed-kernel
curvature,

```text
-exp(-1) * 1e-200 = -3.678794411714423e-201 approximately.
```

An independent 100-digit decimal evaluation reproduced both values and the
sign reversal. Accepted input can therefore lose a representable derivative
and return the wrong Hessian sign, violating the regional analytic product-rule
and complete representability contracts.

A fresh Repair must first add one public compiled D=1 regional-control
regression with the exact inputs above and both arguments on the plateau.
Second-derivative demand must return the independently evaluated positive
Hessian near `8.168564517495419e-17`, not the current tiny negative result,
before the smallest residual-aware displacement repair is implemented.

### Re-review validation and disposition

The reviewer inspected the complete base-to-`b0ff092` PR diff and exact
seventh-Repair delta and independently reran the published REV-013 regression.
The parent Review task passed all fifteen public `trend_controls` tests, all
fifteen public `local_trend` tests, all five private local-trend regressions,
and complete PR diff whitespace validation. Exact code/test/contract head
`42c5686` retains its recorded complete standard local gate; the tail to
reviewed head `b0ff092` changes only the bounded Markdown handoff.

The fixed-SPD diagonal-congruence construction, strict background, CPD
rejection, metric formulae, C2 signs, reference normalization, capability
checks, deterministic diagnostics, allocation behavior, interface disposition,
and absence of hidden regularization otherwise satisfy the reviewed scope.
Polynomial spaces, rank decisions, hard constraints, and infeasibility are not
applicable. Draft Ubuntu CI passed on exact reviewed head `b0ff092`; Ready-only
Windows, Ubuntu, macOS, and benchmark-smoke CI remain unexecuted and are not
claimed as passed.

PR #109 must remain Draft and REQ-TREND-002 remains `implemented`, not
`integrated`. Open a fresh bounded Repair task for TREND002-REV-014 only. Do
not repair production code, mark the PR Ready, merge it, or begin another
requirement in this Review task.

## Eighth Repair of TREND002-REV-014

Exact eighth Repair code/test/contract head:
`d42ccb5692a72e90d970329236cb8a402c6763ef`.

The new public compiled D=1 regression uses the exact reviewed inputs: control
location `-2^-53`, query and kernel center one on the plateau of region
`[-2, 2]` with transition width `0.25`, unit strength, influence radius, and
fixed anisotropy, fixed Gaussian length `1e100`, and strict-background weight
`2^-537`. Before the production repair it reproduced the review's tiny
negative `-3.6787944117144235e-201` Hessian instead of the independent positive
`8.168564517495419e-17` truth. It now retains the positive truth within
`64 * EPSILON` relative tolerance.

The smallest production repair consumes the existing error-free subtraction
residual throughout the regional Gaussian displacement path. Squared scaled
distance is accumulated from the two-component displacement, and Hessian
curvature uses the residual-aware product `(d-r)(d+r)`; mixed curvature and
region-gradient product-rule terms use the same stable displacement factor.
The gate, fixed kernel, public API, diagnostics, and non-regional weight path
are otherwise unchanged.

Focused validation passed all sixteen public `trend_controls` tests, all
fifteen public `local_trend` integration tests, all five private local-trend
regressions, and complete diff whitespace validation. After the last
production/test change, exact stable head `d42ccb5` passed workspace format,
warning-denying workspace all-target/all-feature Clippy, all-feature workspace
tests, workspace Rustdoc, all 58 requirement checks, and complete PR diff
whitespace validation.

TREND002-REV-014 is repaired pending a fresh isolated read-only re-review; this
Repair does not close its own finding. PR #109 remains Draft and
REQ-TREND-002 remains `implemented`, not `integrated`. Ready-only Windows,
Ubuntu, macOS, and benchmark-smoke CI remain intentionally unexecuted. No
unavailable check is claimed as passed.

## Fresh independent re-review after eighth Repair

- Exact reviewed evidence head:
  `473f831ecf55030bdcdcdf807184047219fc48d5`
- Eighth Repair code/test/contract head:
  `d42ccb5692a72e90d970329236cb8a402c6763ef`
- Base and merge-base:
  `8535880c2d9cf2d580ac97bddf0610f9f6a68f61`
- Re-review date: 2026-07-22
- Result: TREND002-REV-014 closed; P1 TREND002-REV-015 requires Repair; no
  other P0-P3 finding remains

A fresh isolated read-only project `math_reviewer` received only the bounded
REQ-TREND-002 summary and integrated dependency closure, Issue #108 acceptance
criteria and exclusions, M6 plan context, ANISOTROPY and ADR-0005/ADR-0008
contracts, the complete PR and eighth-Repair diffs, directly relevant source,
tests, example, benchmark, registry, change evidence, handoff, and recorded
validation evidence. It inherited no Implement or Repair reasoning transcript
and made no repository, Git, or GitHub change.

### Closure of TREND002-REV-014

TREND002-REV-014 is closed for its published regression. The exact public
`regional_hessian_preserves_rounded_displacement_residual` regression passes,
and an independent 100-decimal evaluation from the represented inputs gives

```text
true Hessian = 8.1685645174954193488e-17
old rounded-displacement result = -3.6787944117144231e-201.
```

For this case the residual-aware exponent, diagonal and mixed curvature, both
region-gradient product-rule factors, gate Hessian signs, and complete mixture
Hessian scaling are correct.

### TREND002-REV-015 - P1: diagonal curvature underflows before recoverable scaling

Affected code:

- `crates/georbf/src/local_trend.rs:1508-1512`
- `crates/georbf/src/local_trend.rs:1578-1580`
- `crates/georbf/src/local_trend.rs:1923-1927`
- `crates/georbf/src/local_trend.rs:1970-1975`

`GaussianWeightState::diagonal_curvature` forms residual-aware
`(d-r)(d+r)` with `double_product`, but `double_product` returns exact zero
when its immediate binary64 product underflows. `StableFactor::from_double`
then classifies the curvature as mathematically zero before the later two
inverse-radius-square factors can restore a representable complete term. This
violates the complete-term scaling contract in ANISOTROPY.

An accepted D=1 counterexample uses `eta = 2^-1074`, influence radius
`r = 2^-500`, control location `c = -eta`, and query and kernel center
`x = y = r`. It has unit strength, unit fixed anisotropy, fixed Gaussian
length one, region `[-1, 1]` with transition width `0.25`, and an ordinary
constant-`0.5` strict Gaussian background. Both arguments are on the region
plateau, so all gate derivatives are exactly zero.

For the exact represented displacement `d = r + eta`, although its leading
binary64 subtraction rounds to `r`,

```text
(d^2 - r^2) / r^4 = 2 eta / r^3 + eta^2 / r^4
                      = 2^427 + 2^-148.
```

The independently derived complete Hessian is therefore

```text
exp(-(d/r)^2) * (2^427 + 2^-148 - 1) - 0.25
    approximately 1.2750102220326992e128.
```

The current path instead multiplies lower factor `eta` by upper factor
`2r + eta`; the leading `2^-1573` product underflows and the curvature becomes
exact zero. It consequently returns approximately
`-exp(-1) - 0.25 = -0.6178794411714423`, reversing the sign and losing a large
representable Hessian.

A fresh Repair must first add one public compiled D=1 regional-control
regression with those exact inputs and assert the positive high-precision
value near `1.2750102220326992e128`. It must then preserve the diagonal
curvature until multiplication by both inverse-radius-square factors, rather
than materializing or classifying `(d-r)(d+r)` as zero first.

### Re-review validation and disposition

The reviewer independently passed the exact REV-014 regression, all sixteen
public `trend_controls` tests, all fifteen public `local_trend` tests, all five
private local-trend regressions, complete PR diff whitespace validation, and
the compact requirement `show` and dependency-closure commands. It verified
that `d42ccb5..473f831` changes only the three recorded Markdown evidence
files. Draft Ubuntu CI passed on exact evidence head `473f831`.

The analytic Gaussian and gate product rules, fixed-SPD construction, strict
background, CPD rejection, C2 behavior, reference normalization, capability
checks, deterministic diagnostics and order, allocation behavior, interface
disposition, and absence of hidden regularization otherwise satisfy the
reviewed scope. Polynomial spaces, rank decisions, hard constraints, solver
infeasibility, and solver regularization do not apply. The immutable
`d42ccb5` complete standard local gate remains valid; Ready-only Windows,
Ubuntu, macOS, and benchmark-smoke CI remain unexecuted and are not claimed as
passed.

PR #109 must remain Draft and REQ-TREND-002 remains `implemented`, not
`integrated`. Open a fresh bounded Repair task for TREND002-REV-015 only. Do
not repair production code, mark the PR Ready, merge it, or begin another
requirement in this Review task.

## Ninth Repair of TREND002-REV-015

Exact ninth Repair code/test/contract head:
`144a018f697e3c6e9f23fc9621b434337543be7f`.

The new public compiled D=1 regression uses the exact reviewed inputs:
influence radius `2^-500`, control location `-2^-1074`, query and kernel center
`2^-500`, unit strength, fixed anisotropy, and fixed Gaussian length, region
`[-1, 1]` with transition width `0.25`, and the constant-`0.5` strict Gaussian
background. Before the production repair it reproduced the reviewed negative
`-0.6178794411714423` Hessian instead of the independent positive
`1.2750102220326992e128` truth. It now retains that positive truth within
`64 * EPSILON` relative tolerance.

The smallest production repair keeps the residual-aware `d-r` and `d+r`
factors separate until each has been multiplied by one inverse-radius-square
factor. Their subsequent product is directly representable, so neither
`double_product` nor exact-zero provenance can erase the diagonal curvature
before the complete Gaussian Hessian is formed. Mixed curvature, gate
product-rule terms, fixed-kernel evaluation, public APIs, and diagnostics are
otherwise unchanged.

Focused validation passed all seventeen public `trend_controls` tests, all
fifteen public `local_trend` integration tests, all five private local-trend
regressions, the exact REV-015 regression, and complete diff whitespace
validation. After the last production/test change, exact stable head `144a018`
passed workspace format, warning-denying workspace all-target/all-feature
Clippy, all-feature workspace tests, workspace Rustdoc, all 58 requirement
checks, and complete diff whitespace validation.

TREND002-REV-015 is repaired pending a fresh isolated read-only re-review; this
Repair does not close its own finding. PR #109 remains Draft and REQ-TREND-002
remains `implemented`, not `integrated`. Ready-only Windows, Ubuntu, macOS, and
benchmark-smoke CI remain intentionally unexecuted. No unavailable check is
claimed as passed.

## Fresh independent re-review after ninth Repair

- Exact reviewed evidence head:
  `d516be797f385b120c1d6ea7a988dd43039b5ac9`
- Ninth Repair code/test/contract head:
  `144a018f697e3c6e9f23fc9621b434337543be7f`
- Base and merge-base:
  `8535880c2d9cf2d580ac97bddf0610f9f6a68f61`
- Re-review date: 2026-07-23
- Result: TREND002-REV-015 closed; P1 findings TREND002-REV-016 and
  TREND002-REV-017 require Repair; no P0, P2, or P3 finding remains

A fresh isolated read-only project `math_reviewer` received only the bounded
REQ-TREND-002 summary and integrated dependency closure, Issue #108 acceptance
criteria and exclusions, the M6 plan, ANISOTROPY and ADR-0005/ADR-0008
contracts, the complete PR and ninth-Repair diffs, directly relevant source,
tests, example, benchmark, registry, change evidence, handoff, and recorded
validation evidence. It inherited no Repair reasoning and made no repository,
Git, or GitHub change. The tail from `144a018` through `d516be7` changes only
the requirement change fragment, this review record, and the bounded handoff.

### Closure of TREND002-REV-015

TREND002-REV-015 is closed for its published regional D=1 regression. With
`r = 2^-500`, `eta = 2^-1074`, and the exact represented displacement
`d = r + eta`, the independent curvature scale is

```text
(d^2 - r^2) / r^4 = 2 eta / r^3 + eta^2 / r^4
                    = 2^427 + 2^-148.
```

The Repair scales the residual-aware `d-r` and `d+r` separately by `r^-2`.
Those factors remain representable near `2^-74` and `2^501` before their
product, including the residual term. After the fixed-kernel and background
contributions, the complete Hessian is

```text
exp(-(1 + 2^-574)^2) * (2^427 + 2^-148 - 1) - 0.25
    approximately 1.2750102220326992e128.
```

The exact public regression returns that positive result within
`64 * EPSILON` relative tolerance. Its formula, signs, scale, and complete-term
representability are correct.

### TREND002-REV-016 - P1: non-regional weights erase the same displacement residual

Affected code:

- `crates/georbf/src/local_trend.rs:2047-2055`
- `crates/georbf/src/local_trend.rs:2101-2107`

The non-regional `gaussian_weight_jet` retains only the rounded subtraction
`point - center`. Its diagonal Hessian then forms `(d-r)(d+r)` from that
rounded value. It retains neither the error-free subtraction residual nor the
separately scaled diagonal factors used by the repaired regional path.

The exact accepted public counterexample is the REV-015 configuration with
only `region = None`: D=1, `eta = 2^-1074`, influence radius `r = 2^-500`,
control location `-eta`, query and kernel center `r`, unit strength, unit fixed
anisotropy and fixed Gaussian length, and the constant-`0.5` strict Gaussian
background. The independent positive truth is the same
`1.2750102220326992e128` Hessian above. The current implementation rounds `d`
to `r`, treats `d-r` as exact zero, and returns
`-exp(-1) - 0.25 = -0.6178794411714423`, reversing the sign.

A fresh Repair must first add a public D=1 regression cloned from
`regional_hessian_scales_diagonal_curvature_before_underflow` with only the
region removed, then make the smallest residual-aware non-regional repair.

### TREND002-REV-017 - P1: fixed-Gaussian inverse-length square becomes exact zero too early

Affected code:

- `crates/georbf/src/local_trend.rs:2178-2188`
- `crates/georbf/src/local_trend.rs:2203-2215`
- `crates/georbf/src/kernel/smooth_global.rs:456-468`

The stable fixed-Gaussian path forms `inverse_length * inverse_length` as one
binary64 value. An accepted length `1e200` retains the represented reciprocal
`1e-200`, but its square underflows to zero. Passing that zero through
`StableFactor::from_factors` promotes analytic Gaussian gradient and curvature
to mathematical exact zero before two large spatial weights can make the
complete Hessian finite. The smooth-kernel constructor rejects non-finite
cached inverse powers but permits this zero square.

An exact accepted public D=1 counterexample uses control location zero, query
and kernel center `1e154`, strength and influence radius `1e154`, no region,
unit fixed anisotropy, fixed Gaussian length `1e200`, and a unit-Gaussian
constant background weight `2^-537` with the same policy minimum. At `d = r`
the spatial-weight Hessian and fixed-kernel gradient are exactly zero, while

```text
H_local = -strength^2 exp(-1) / length^2
        = -exp(2 ln(1e154) - 1 - 2 ln(1e200))
        approximately -3.67879441171431e-93.
```

The background adds only `-2^-1074`. The current implementation returns that
single minimum-subnormal background term and loses the finite local Hessian.
This contradicts the complete fixed-Gaussian scaling evidence in the
requirement change fragment.

A fresh Repair must first add the exact public compiled-control regression and
then retain two reciprocal-length factors, or mathematically equivalent
logarithmic scale, until complete mixture-term formation.

### Re-review validation and disposition

The reviewer and parent Review task each passed all seventeen public
`trend_controls` tests, all fifteen public `local_trend` tests, all five private
local-trend regressions, the exact REV-015 regression, compact requirement
`show` and dependency review, and complete PR diff whitespace validation. The
parent also independently reproduced both new findings through temporary
public-API tests: REV-016 returned `-0.6178794411714423`, and REV-017 returned
only `-2^-1074`; both temporary tests were removed and the worktree restored
before recording evidence. Independent high-precision calculations establish
the positive REV-016 truth and the finite negative REV-017 truth above.

Draft Ubuntu CI run 29931521124 passed on exact reviewed evidence head
`d516be7`. Exact stable code/test/contract head `144a018` retains its recorded
complete local standard gate. Ready-only Windows, Ubuntu, macOS, and benchmark
smoke remain unexecuted and are not claimed as passed.

The regional REV-015 path, analytic Gaussian and gate product rules within its
published case, fixed-SPD construction, strict background, CPD rejection, C2
behavior, reference normalization, capability checks, deterministic
diagnostics/order, allocation behavior, interface disposition, and absence of
hidden regularization otherwise satisfy the reviewed scope. Polynomial
spaces, scale-aware rank decisions, hard constraints, solver infeasibility,
and solver regularization do not apply because this compiler constructs only
fixed-SPD mixture components and rejects CPD kernels before any polynomial or
solver path.

PR #109 must remain Draft and REQ-TREND-002 remains `implemented`, not
`integrated`. A fresh Repair task must address only TREND002-REV-016 and
TREND002-REV-017, add both exact regressions before the smallest production
repairs, run focused checks and one final stable-head standard gate, update
evidence, push, and stop for another fresh independent re-review. This Review
task does not repair production code, mark the PR Ready, merge it, or begin
another requirement.

## Tenth Repair of TREND002-REV-016 and TREND002-REV-017

Exact tenth Repair code/test/contract head:
`2a25f4467588edd7ac040074a79e3aeed6b3f459`.

The new public non-regional D=1 regression uses the exact reviewed REV-016
inputs: influence radius `2^-500`, control location `-2^-1074`, query and
kernel center `2^-500`, unit strength, fixed anisotropy and Gaussian length,
no region, and the constant-`0.5` strict Gaussian background. Before the
production repair it reproduced `-0.6178794411714423`; it now retains the
independent positive `1.2750102220326992e128` Hessian within `64 * EPSILON`
relative tolerance.

The smallest REV-016 repair reuses the residual-aware Gaussian state already
required by the regional path. Non-regional value, gradient, diagonal Hessian,
and mixed Hessian formation now preserve the error-free subtraction residual;
the diagonal factors `d-r` and `d+r` each receive one represented
inverse-radius-square factor before their product. No region semantics or
public API changes.

The new public compiled D=1 REV-017 regression uses control location zero,
query and kernel center `1e154`, strength and influence radius `1e154`, no
region, unit fixed anisotropy, fixed Gaussian length `1e200`, and a constant
background weight and policy minimum of `2^-537`. Before the repair it
reproduced only the `-2^-1074` background Hessian; it now retains the
independent finite approximately `-3.67879441171431e-93` complete Hessian
within `1024 * EPSILON` relative tolerance.

The smallest REV-017 repair no longer squares the represented reciprocal
fixed-Gaussian length before stable-factor construction. Both gradient
projections and Hessian curvature retain two reciprocal-length factors until
they are combined with anisotropy, displacement, Gaussian value, and both
spatial weights. The ordinary smooth-kernel representation and public API are
unchanged.

Focused validation passed all nineteen public `trend_controls` tests, all
fifteen public `local_trend` integration tests, all five private local-trend
regressions, both exact new regressions, and complete diff whitespace
validation. After the last production/test change, exact stable head `2a25f44`
passed workspace format, warning-denying workspace all-target/all-feature
Clippy, all-feature workspace tests, workspace Rustdoc, all 58 requirement
checks, and complete diff whitespace validation.

TREND002-REV-016 and TREND002-REV-017 are repaired pending a fresh isolated
read-only re-review; this Repair does not close its own findings. PR #109
remains Draft and REQ-TREND-002 remains `implemented`, not `integrated`.
Ready-only Windows, Ubuntu, macOS, and benchmark-smoke CI remain intentionally
unexecuted. No unavailable check is claimed as passed.

## Fresh independent re-review after tenth Repair

- Exact reviewed evidence head:
  `8593ec5a9b5ba289964402401b16724d114a1d4c`
- Tenth Repair code/test/contract head:
  `2a25f4467588edd7ac040074a79e3aeed6b3f459`
- Base and merge-base:
  `8535880c2d9cf2d580ac97bddf0610f9f6a68f61`
- Re-review date: 2026-07-23
- Result: TREND002-REV-016 and TREND002-REV-017 closed; P1 finding
  TREND002-REV-018 requires Repair; no other P0-P3 finding remains

A fresh isolated read-only project `math_reviewer` received only the bounded
REQ-TREND-002 summary and integrated dependency closure, Issue #108 acceptance
criteria and exclusions, the M6 plan, ANISOTROPY and ADR-0005/ADR-0008
contracts, the complete PR and tenth-Repair diffs, directly relevant source,
tests, example, benchmark, registry, change evidence, handoff, and recorded
validation evidence. It inherited no Implement or Repair reasoning and made no
repository, Git, or GitHub change. The tail from `2a25f44` through `8593ec5`
changes only the requirement change fragment, this review record, and the
bounded handoff.

### Closure of TREND002-REV-016

TREND002-REV-016 is closed. The non-regional Gaussian-weight path now reuses
the residual-aware `GaussianWeightState`. Its value, gradient, mixed Hessian,
and diagonal Hessian therefore retain the error-free displacement residual.
The diagonal factors `d-r` and `d+r` each receive one represented
inverse-radius-square factor before their product. For the published D=1
counterexample, the independently derived complete Hessian remains
approximately `1.2750102220326992e128`; the exact public regression passes
within `64 * EPSILON` relative tolerance.

### Closure of TREND002-REV-017

TREND002-REV-017 is closed. Stable fixed-Gaussian gradient projections and
Hessian curvatures retain two represented reciprocal-length factors rather
than forming an underflowed reciprocal square. For the published D=1
counterexample, the complete local Hessian remains approximately
`-3.67879441171431e-93`, rather than only the `-2^-1074` background term; the
exact public regression passes within `1024 * EPSILON` relative tolerance.

### TREND002-REV-018 - P1: fixed-Gaussian normalization erases a represented transverse gradient

Affected code:

- `crates/georbf/src/local_trend.rs:2151-2169`

The stable fixed-Gaussian path obtains a transformed radial separation, then
reconstructs each transformed displacement component as
`unit_displacement[axis] * radius`. Normalizing a highly unbalanced but finite
represented displacement can underflow a small unit component to exact zero.
The reconstruction then promotes that represented displacement component to a
mathematical exact zero before two large accepted spatial weights can make its
kernel-gradient contribution representable.

A concrete accepted D=2 compiled-control counterexample uses identity fixed
anisotropy; control location and query `x = [1e160, 1e-170]`; kernel center
`y = [0, 0]`; fixed Gaussian length `L = 1e160`; influence radius `R = 1e161`;
strength `s = 1e154`; no region; and a valid tiny constant Gaussian
background. The radial separation is approximately `1e160`, so normalization
underflows the second unit component `1e-170 / 1e160` to zero. Because the
query equals the control location, the query spatial-weight gradient is zero.
For identity anisotropy the omitted fixed-kernel derivative is

```text
partial_x2 k = -k(x, y) (x2 - y2) / L^2.
```

The center spatial weight contributes
`w_c = -0.5 ||x / R||^2`, approximately `-0.005`, and the fixed kernel
contributes `-0.5`. Evaluated independently at 300 decimal digits from the
exact represented binary64 inputs, the complete local term is

```text
-s^2 exp(w_c - 0.5) (x2 - y2) / L^2
    approximately -6.035055754270406e-183.
```

That value is finite and representable, but the current implementation returns
exactly zero. The parent Review task independently reproduced both values with
a temporary public-API compiled-control test and removed the test before
recording this evidence. This contradicts the documented rule that fixed
Gaussian gradients retain analytic signed-logarithmic scale through complete
mixture-term formation.

A fresh Repair must first add this public D=2 compiled-control regression, then
preserve the original transformed separation components, or mathematically
equivalent signed-log scale, without reconstructing them through normalized
unit components. The Repair must remain limited to TREND002-REV-018.

### Re-review validation and disposition

The parent Review task passed all nineteen public `trend_controls` tests, all
fifteen public `local_trend` tests, all five private local-trend regressions,
both exact tenth-Repair regressions, compact requirement `show` and dependency
review, and complete PR diff whitespace validation. Independent 300-digit
arithmetic and the temporary public-API reproduction establish the finite
TREND002-REV-018 truth and the current exact-zero result. The temporary test
was removed and the worktree restored before this evidence change.

Exact stable code/test/contract head `2a25f44` retains its recorded complete
local standard gate. Draft CI run 29964780295 passed its Ubuntu correctness
gate on exact evidence head `8593ec5`. This Review changes only this review
record and the bounded Markdown handoff. Ready-only Windows, Ubuntu, macOS,
and benchmark smoke remain unexecuted and are not claimed as passed.

The SPD-mixture proof, CPD rejection, regional C2 gate and complete product
rules, center capability checks, deterministic diagnostics and allocation
interfaces, and adapter dispositions otherwise satisfy the reviewed scope.
Polynomial spaces, scale-aware rank decisions, hard constraints, solver
infeasibility, and solver regularization do not apply to this fixed-SPD
compiler path.

PR #109 must remain Draft and REQ-TREND-002 remains `implemented`, not
`integrated`. A fresh Repair task must address only TREND002-REV-018, add the
required regression before the smallest production repair, run focused checks
and one final stable-head standard gate, update evidence, push, and stop for
another fresh independent re-review. This Review task does not repair
production code, mark the PR Ready, merge it, or begin another requirement.
