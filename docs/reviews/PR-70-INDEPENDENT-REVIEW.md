# PR #70 Independent Review

- Requirement: REQ-LEVEL-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/69
- Pull request: https://github.com/qingsonger/GeoRBF/pull/70
- Branch: `codex/req-level-001-explicit-level-variables`
- Reviewed head: `7d8d2834a539850ee73b0124faba6d4d88b20f27`
- Base head: `2904c64c8d99e0b6a3183dc6c232953a969922ad`
- Review date: 2026-07-17
- Result: five P1, three P2, and one P3 finding; Repair required
- Fresh re-review head: `93f85dd17e145042f4282208c361c9aac95b8181`
- Fresh re-review result: R70-001 through R70-009 closed; one new P1
  finding R70-010; Repair required
- R70-010 re-review head: `3b6cf1366f30b9285c1023e5b2c73810c8c1b282`
- R70-010 re-review result: R70-001 through R70-010 closed; one new P1
  finding R70-011; Repair required
- R70-011 re-review head: `b11d321961c3ac0448def84696046852a772ef26`
- R70-011 re-review result: R70-001 through R70-011 closed; new P1 R70-012
  and P2 R70-013; Repair required
- R70-012 and R70-013 re-review head:
  `49998ef4b18a803c84817415096dabe4eeabad63`
- R70-012 and R70-013 re-review result: R70-001 through R70-013 closed; one
  new P1 finding R70-014; Repair required
- R70-014 re-review head: `6a03fe6ae75df56613e1ae3ee7beda50aa5afb07`
- R70-014 re-review result: R70-001 through R70-014 closed; no P0-P3
  findings remain

## Scope and independence

A fresh read-only project `math_reviewer` received only the bounded
REQ-LEVEL-001 summary and integrated dependency closure, Issue #69 acceptance
criteria, the M4 plan, the constraint-semantics, ADR-0003, and level-layer
architecture contracts, the complete PR diff, tests, benchmark, registry
evidence, and exact-head validation state. It did not inherit the implementation
task's reasoning and made no repository or remote changes.

The reviewer checked level-row formulae, signs, constants and indices; fixed,
unknown, and prior semantics; dimensional validity; DAG and connected-component
logic; transitive gap arithmetic; conservative floating-point boundaries; hard
constraint preservation; source provenance; structured errors; D=1/D=2/D=3
bounds; allocations; interface dispositions; benchmark scope; and registry
truth.

## Findings

There are no P0 findings. The following P1-P3 findings are actionable.

### P1 R70-001: memberships accept dimensionally and gauge-invalid functionals

`LevelMembership` accepts an unrestricted `ObservationFunctional<D>` at
`crates/georbf/src/levels.rs:164-190`, while validation at lines 332-339 never
requires the contract's unit-weight Value functional `f(x_i)`. A derivative,
`2 f(x)`, or a value difference can therefore compile as `L(f) - h_k = 0`.
This can compare `f/length` with `f`, and the unconditional union at lines
1300-1302 then applies the ordinary joint-shift gauge argument even though a
derivative is shift-invariant and `2 f` shifts by `2 c`.

Repair must reject every membership that is not one coefficient-1 Value atom
unless the normative contract is explicitly changed. Regressions must reject a
directional derivative and a non-unit or multi-value expression with structured
semantic errors, while accepting one unit Value atom.

### P1 R70-002: extreme finite fixed endpoints hide a hard order conflict

At `crates/georbf/src/levels.rs:1225-1230`, lower fixed `f64::MAX`, upper fixed
`-f64::MAX`, and zero gap produce `fixed_gap = -infinity`; the scale and
tolerance also become infinity, so `infinity <= infinity` incorrectly accepts
the contradiction. All user inputs are finite.

An overflow-safe comparison must preserve the definite conflict. A regression
with separately located memberships must require `FixedOrderConflict` and all
endpoint and edge sources for this exact extreme case.

### P1 R70-003: accumulated-gap overflow rejects a feasible hard system

At `crates/georbf/src/levels.rs:1204-1208`, overflow in a derived path sum is
returned as a semantic error. Levels fixed at `-f64::MAX` and `f64::MAX`, an
unknown middle level, and two gaps of `f64::MAX` are feasible with the middle
level equal to zero: each original hard row has a finite difference of exactly
`f64::MAX`. The validator instead sums the two gaps to infinity and rejects the
problem.

Repair needs overflow-safe path comparison without changing the original hard
rows. The feasible three-level chain must validate, alongside a separate
overflow-safe contradictory-endpoint regression.

### P1 R70-008: functional provenance masks mathematical fixed conflicts

`validate_fixed_memberships` at `crates/georbf/src/levels.rs:1101-1103` uses
derived `PartialEq` on `ObservationFunctional`. That equality includes
`FunctionalProvenance` and expression insertion order, so independently sourced
unit Value evaluations at the same point compare unequal. The hard equations
`f(x) = a` and `f(x) = b` remain contradictory for `a != b` regardless of
provenance, but the precheck misses them. The current test at
`crates/georbf/tests/levels.rs:225-237` clones one functional and cannot expose
the defect.

Repair must compare mathematical functionals independently of source identity.
A regression must construct the same Value evaluation with distinct functional
provenance and require `FixedMembershipConflict`; a reordered-expression case
is also required if multi-term memberships remain permitted.

### P1 R70-009: graph connectivity does not prove field contrast

The union-find and contrast checks at `crates/georbf/src/levels.rs:1349-1382`
accept any positive order gap or distinct anchors in the component containing a
field membership. With only `f(x_A) = h_A`, a membershipless `B` can absorb
`h_B - h_A >= delta` while the field remains constant. A distinct fixed but
membershipless `B` has the same defect.

Repair must prove contrast on at least two membership-coupled field
functionals, not merely on connected level vertices. Regressions must require
`MissingContrast` for one membership plus a positive-gap membershipless level,
and for one membership plus a distinct fixed membershipless level joined by a
zero-gap edge; each has a constant-field feasibility witness.

### P2 R70-004: cycle diagnostics include downstream non-cycle edges

Kahn's unemitted vertices at `crates/georbf/src/levels.rs:1155-1167` include
cycle vertices and vertices reachable from a cycle. Collecting every edge whose
endpoints are unemitted therefore reports downstream DAG edges as cyclic,
contrary to `changes/REQ-LEVEL-001.md:21-22`. The pure-cycle test at
`crates/georbf/tests/levels.rs:173-191` asserts only a count.

A regression with `A <-> B` and `B -> C` must identify exactly the two cycle
edge observation IDs in stable order.

### P2 R70-005: fixed-membership conflicts omit fixed-definition sources

The source vector at `crates/georbf/src/levels.rs:1104-1125` contains only the
two memberships even though the contradiction also depends on the two fixed
definitions. This is incomplete compared with the error's complete-source
contract and with fixed-order conflict evidence.

A regression must require both fixed-level definition sources and both
membership sources.

### P2 R70-006: missing-contrast diagnostics cite unrelated levels

When contrast is absent, `crates/georbf/src/levels.rs:1382-1385` always reports
`levels[0]` and `levels[1]`. Those may be isolated anchored components and may
themselves have distinct values, so they are not evidence for the failing field
component required by `docs/math/CONSTRAINT_SEMANTICS.md:47-51`.

A regression must place two distinct isolated fixed anchors first and a later
field-connected pair without contrast, then require component-specific level or
equivalent source evidence.

### P3 R70-007: test and registry evidence overstate independent coverage

The registry list at `requirements/v1.yaml:395` and the claims at
`changes/REQ-LEVEL-001.md:35-40` overstate the tests. The DAG case has a unique
topological order and does not prove deterministic tie-breaking; the cycle test
checks only a source count; and the fixed-functional test clones one functional
rather than independently constructing the same mathematical evaluation.

Repair must add a DAG with simultaneously eligible nodes and assert insertion-
order tie-breaking, assert exact source identifiers, add the provenance-
independent conflict regression from R70-008, and then make the registry and
change fragment no stronger than the resulting evidence.

## Independent truth and unaffected contracts

- Membership, fixed-value, and order canonical rows have the documented signs,
  constants, variable offsets, and insertion order for valid inputs.
- Priors remain explicit metadata and are neither hardened nor claimed as
  solved. Individual hard rows are not dropped, softened, jittered,
  regularized, pseudoinverted, or otherwise repaired.
- Compile-time D=1/D=2/D=3 bounds and the tested `Send + Sync` properties are
  consistent with the requirement.
- Rust is the only current interface. CLI deferral to M8 and C, C++, and Python
  deferral to M9 match the milestone plan.
- SPD/CPD classification, polynomial spaces, center limits, positive
  definiteness, and Hessian capability are not applicable to this semantic
  layer.
- Fixed-path validation remains a residual performance risk: it is
  `Theta(F * V * E)` and allocates two work vectors per fixed source, while the
  64-level benchmark has only one fixed source and no acceptance threshold.

## Independently verified evidence

- The worktree was clean before review evidence was added, and local HEAD and
  remote PR head matched reviewed commit `7d8d283`.
- Exact-head Draft Ubuntu CI run 29561377945 passed its complete correctness
  gate. The Ready-only Windows, Ubuntu, macOS, and benchmark-smoke matrix did not
  run, as expected for a Draft PR.
- The parent Review task independently reran the focused level suite (10
  passed), core Rustdoc, and the complete PR whitespace check; all passed.
- The stable implementation tree retains the recorded complete standard local
  gate. This Review task changes only review and bounded-handoff documentation.

## Repair disposition

PR #70 must remain Draft and REQ-LEVEL-001 remains `implemented`. A fresh
Repair task must address only R70-001 through R70-009, add the specified
independent regressions, rerun focused checks during development, and run the
complete standard gate after the final production or test change. This Review
task does not repair production code, mark the PR ready, merge it, integrate
the requirement, or begin another requirement.

## Repair evidence

Repair commit `a56e7ad24a9eaa4768534d3cd897ee74a6355659` addresses only
R70-001 through R70-009:

- memberships now accept exactly one coefficient-1 Value atom;
- fixed-path propagation and endpoint comparison use an overflow-safe scaled
  magnitude while retaining every original hard row;
- mathematical Value equality ignores functional provenance, and fixed
  membership conflicts retain both definitions and both membership sources;
- cycle evidence contains only edges that have a return path and therefore
  participate in a directed cycle;
- contrast requires two membership-coupled levels, and missing-contrast
  evidence is selected from the failing field component; and
- the registry and change fragment now match deterministic-tie, exact-source,
  provenance-independent, overflow, and membership-coupled regression evidence.

The repaired focused level suite has 16 passing tests. Core all-target,
all-feature Clippy, core Rustdoc, the 64-level benchmark smoke, the 58-entry
requirement check, and whitespace review passed. After the final production,
test, registry, and normative-document change, the complete stable-tree gate
passed: workspace formatting, warning-denying all-target/all-feature workspace
Clippy, all-feature workspace tests, workspace Rustdoc, requirement validation,
and `git diff --check`.

This repair evidence is not an independent re-review and does not close the
findings. PR #70 remains Draft and REQ-LEVEL-001 remains `implemented`; a fresh
read-only re-review must confirm closure and check for new findings.

## Fresh independent re-review

A new read-only project `math_reviewer` independently reviewed exact PR head
`93f85dd17e145042f4282208c361c9aac95b8181` against base
`2904c64c8d99e0b6a3183dc6c232953a969922ad` and repair evidence base
`809e580969673867315980bfec17e70472dc3677`. It received only the bounded
REQ-LEVEL-001 summary and integrated dependency closure, the M4 plan, relevant
mathematical, ADR, and architecture contracts, the original review, complete
PR and repair diffs, tests, benchmark, and validation evidence. It inherited
no implementation reasoning and made no repository or remote changes.

The reviewer independently confirmed R70-001 through R70-009 are closed:
membership shape is restricted to one unit Value atom; extreme endpoint and
path arithmetic is overflow safe without changing hard rows; cycle evidence
contains only cycle edges; mathematical functional comparison ignores
provenance and retains complete fixed-conflict sources; deterministic ties and
exact evidence are tested; and membershipless levels no longer manufacture
field contrast or unrelated diagnostics.

### P1 R70-010: identical memberships plus a positive order path are accepted

`validate_fixed_memberships` at `crates/georbf/src/levels.rs:1127-1140`
compares mathematically identical membership functionals only when both levels
are fixed. Fixed-path validation at lines 1322-1326 likewise skips an unknown
target. Contrast validation at lines 1583-1617 then treats any positive order
path between two membership-bearing level identifiers as valid contrast
without checking whether their Value functionals are mathematically identical.

For fixed `A`, unknown `B`, two independently sourced memberships at the same
point, and `A -> B` with `delta > 0`, the emitted hard constraints are

```text
f(x) - h_A = 0
f(x) - h_B = 0
h_B - h_A >= delta
```

The equalities force `h_B - h_A = 0`, contradicting the positive gap, but
`LevelProblem::try_new` succeeds because the path sets `has_gap = true`. This
is a hard semantic infeasibility that requires source-aware diagnosis, not a
rank, roundoff, or solver issue. Existing regressions cover two fixed levels
with identical functionals and membershipless contrast separately, but do not
cover their combination with an order gap.

Repair must add a regression with `A = Fixed(0)`, `B = Unknown`, independently
constructed unit Value evaluations at the same point with distinct functional
and semantic provenance, and a direct `A -> B` gap of `1.0`. Construction must
return a structured infeasibility retaining both membership sources and the
order-edge source. The smallest production repair must reject the same
mathematical contradiction without changing, dropping, softening, or
regularizing any hard row.

No additional P0, P2, or P3 finding remains. The exact reviewed head and remote
branch matched, `a56e7ad..93f85dd` was documentation-only, the full PR diff
passed `git diff --check`, and exact-head Draft Ubuntu CI run 29563643533 passed
the complete correctness gate. PR #70 must remain Draft and REQ-LEVEL-001 must
remain `implemented`. A fresh Repair task must address only R70-010, run the
focused and complete stable-head gates, push, and stop for another fresh
independent re-review.

## R70-010 repair evidence

Repair implementation commit `612aa0d34f2c75740cb0d26cb57392249d31a892`
addresses only R70-010. Before the production change, the new direct-path
regression failed because `LevelProblem::try_new` returned success. The repair
now performs a deterministic DAG pass from each membership-coupled level,
rejects a positive direct or transitive path to a mathematically identical
membership, and returns `MembershipOrderConflict`. Its source vector contains
the lower membership, every selected path edge in order, and the upper
membership. Mathematical functional comparison ignores functional provenance;
the original hard rows are neither emitted nor changed on this semantic error
path. Graph work buffers are allocated once and reused across membership
sources.

The source-aware regression uses fixed `A`, unknown `B`, independently
constructed unit Value evaluations at the same point with distinct functional
and semantic provenance, and a direct `A -> B` gap of `1.0`. It asserts the
exact membership/order/membership source sequence. The same test also checks a
two-edge path and both edge sources. The focused level suite has 17 passing
tests; core all-target/all-feature Clippy, all 29 core Rustdoc tests, and the
64-level benchmark smoke passed.

After the final production, test, registry, and normative-document change, the
complete stable-tree standard gate passed: workspace formatting,
warning-denying all-target/all-feature workspace Clippy, all-feature workspace
tests, workspace Rustdoc, all 58 requirement checks, and `git diff --check`.
This repair evidence is not an independent re-review and does not itself close
R70-010. PR #70 remains Draft and REQ-LEVEL-001 remains `implemented`; a fresh
read-only re-review must confirm closure and check for new findings.

## Fresh independent re-review after R70-010 repair

A fresh read-only project `math_reviewer` independently reviewed exact PR head
`3b6cf1366f30b9285c1023e5b2c73810c8c1b282` against base
`2904c64c8d99e0b6a3183dc6c232953a969922ad`. It received only the bounded
REQ-LEVEL-001 summary and integrated dependency closure, the M4 plan, relevant
mathematical, ADR, and architecture contracts, the complete PR and R70-010
repair diffs, tests, benchmark, prior review record, and validation evidence.
It inherited no implementation reasoning and made no repository or remote
changes.

The reviewer independently confirmed R70-010 is closed. The repaired DAG pass
deterministically propagates positive reachability, compares mathematical Value
evaluations independently of provenance, and reports both memberships plus
every selected direct or transitive path edge. The required direct and
transitive regressions assert the complete source sequences. R70-001 through
R70-009 also remain closed.

### P1 R70-011: identical memberships with distinct fixed/prior anchors falsely satisfy contrast

`has_distinct_anchors` at `crates/georbf/src/levels.rs:1734-1757` compares a
fixed value with a prior mean without first proving that the anchored
membership evaluations are mathematically distinct. It therefore accepts
fixed level A at `0`, prior level B with mean `1`, independently sourced
memberships for A and B at the same point, and no order edges.

The hard equations are

```text
f(x) - h_A = 0
f(x) - h_B = 0
h_A = 0
```

and force `h_A = h_B = f(x) = 0`. B's prior is only a soft objective residual;
it cannot establish nonzero field contrast. Comparing the fixed value with the
prior mean nonetheless sets `has_distinct_anchors` and construction succeeds.
This contradicts the normative requirement that contrast be proved between
membership-coupled functionals.

Repair must add an independent regression near
`crates/georbf/tests/levels.rs:531` with fixed A and prior B, distinct anchor
values, distinct functional and semantic provenance, identical Value points,
and no orders. `LevelProblem::try_new` must return `MissingContrast` naming A
and B. The production repair must not change, drop, soften, or regularize any
hard row or turn the prior into a hard equality.

No P0, P2, or P3 finding remains. SPD/CPD classification, center limits,
polynomial spaces, rank decisions, positive definiteness, rotation invariance,
and Hessian capabilities are not applicable to this semantic layer. The review
also covered membership units and signs, overflow-safe path arithmetic,
conservative roundoff behavior, hard-row preservation, the canonical solver
boundary, provenance, allocations, interface dispositions, and requirement
status.

Exact-head Draft Ubuntu CI run 29565567615 passed the complete correctness gate.
The parent Review task independently reran all 17 focused level tests, all 29
core Rustdoc tests, and the complete PR `git diff --check`; all passed. Exact
HEAD and the remote branch matched before this review-only evidence change, and
`612aa0d..3b6cf13` changed only this review record and the bounded handoff.

PR #70 must remain Draft and REQ-LEVEL-001 must remain `implemented`. A fresh
Repair task must address only R70-011, add the specified regression, run focused
checks and the complete stable-head gate, update repair evidence and the bounded
handoff, push, and stop for another fresh independent re-review. This Review
task does not repair production code, mark the PR ready, merge it, integrate
the requirement, or begin another requirement.

## R70-011 repair evidence

Repair implementation commit `914c1eaf6b85991c2bb2f3d51c99bcf4e29de6c3`
addresses only R70-011. Before the production change, the independently
specified regression failed because `LevelProblem::try_new` returned success.
The regression uses fixed A at `0`, prior B with mean `1`, independently
constructed unit Value evaluations at the same point with distinct functional
and semantic provenance, and no order edges. It now requires
`MissingContrast` naming A and B. The same test confirms that distinct
fixed/prior anchors on mathematically distinct memberships remain accepted.

The contrast validator now compares anchored membership evaluations before
accepting different fixed values or prior means as evidence. Any mathematically
identical cross-level Value membership hard-couples the two level variables, so
that anchor pair cannot manufacture field contrast. This check allocates no
new work storage, does not emit or change a canonical row, and leaves priors as
soft objective metadata rather than hard equalities.

The focused level suite has 18 passing tests. Core all-target/all-feature
Clippy, all 29 core Rustdoc tests, and the 64-level benchmark smoke passed.
After the final production, test, registry, and normative-document change, the
complete stable-tree standard gate passed: workspace formatting,
warning-denying all-target/all-feature workspace Clippy, all-feature workspace
tests, workspace Rustdoc, all 58 requirement checks, and `git diff --check`.

This repair evidence is not an independent re-review and does not itself close
R70-011. PR #70 remains Draft and REQ-LEVEL-001 remains `implemented`; a fresh
read-only re-review must confirm R70-011 closure, reconfirm R70-001 through
R70-010, and check for new findings.

## Fresh independent re-review after R70-011 repair

A fresh read-only project `math_reviewer` independently reviewed exact PR head
`b11d321961c3ac0448def84696046852a772ef26` against base
`2904c64c8d99e0b6a3183dc6c232953a969922ad`. It received only the bounded
REQ-LEVEL-001 summary and integrated dependency closure, Issue #69 acceptance
criteria, the M4 plan, relevant mathematical, ADR, and architecture contracts,
the complete PR and R70-011 diffs, tests, benchmark, prior review record, and
validation evidence. It inherited no implementation reasoning and made no
repository or remote changes.

The reviewer independently confirmed R70-011 is closed. Direct identical
cross-level Value memberships now prevent distinct fixed/prior anchors from
proving contrast, while distinct-point memberships remain accepted. Priors
remain soft objective metadata and only fixed levels emit hard rows. The exact
R70-001 through R70-010 regression cases also remain closed.

### P1 R70-012: transitive membership equalities are treated only pairwise

The fixed-membership checks at `crates/georbf/src/levels.rs:1148-1160`, the
membership-order checks at `crates/georbf/src/levels.rs:1246-1283`, and the
anchor-membership checks at `crates/georbf/src/levels.rs:1782-1792` and
`crates/georbf/src/levels.rs:1809-1821` compare only memberships directly
attached to two endpoint levels. They do not form the transitive equivalence
classes induced by shared Value evaluations.

For three levels with memberships

```text
f(x) - h_A = 0
f(x) - h_B = 0
f(y) - h_B = 0
f(y) - h_C = 0
```

the hard rows imply `h_A = h_B = h_C`, although A and C have no directly
identical membership. The current implementation therefore accepts three
invalid systems: distinct fixed values on A and C; a fixed A and distinct prior
mean on C that falsely proves field contrast; and a positive A-to-C order gap.
The first and third are hard infeasibilities, while the second has no nonzero
field contrast because the prior remains soft. This violates the hard-conflict
and contrast contracts in `docs/math/CONSTRAINT_SEMANTICS.md:45-63`.

Repair must add one independently constructed three-level equality-chain
fixture with distinct functional and semantic provenance. Distinct fixed A/C
must return a structured fixed conflict retaining both definitions and all four
membership sources; fixed/prior A/C must return `MissingContrast`; and a
positive A-to-C order must return structured infeasibility retaining all
equality-chain memberships and the order source. The production repair must
reason over the complete equality closure without changing, dropping,
softening, regularizing, or otherwise repairing any hard row.

### P2 R70-013: a single-level field component cites an unrelated anchor

When the field component contains only one membership-bearing level, the
missing-contrast diagnostic search at `crates/georbf/src/levels.rs:1744-1754`
finds no second level in that component and falls back to any unrelated level.
For fixed A with the only membership plus an isolated fixed or prior B, the
error is `MissingContrast(A, B)` even though B is not part of the failing field
component. This contradicts `docs/math/CONSTRAINT_SEMANTICS.md:64-65` and
`changes/REQ-LEVEL-001.md:33-34`.

Repair must add a regression with one anchored membership level, one isolated
anchored level, and no orders. Every item of missing-contrast evidence must
belong to the field component; the diagnostic representation may need to
support a one-level failing component.

No P0 or P3 finding remains. SPD/CPD classification, center limits, polynomial
spaces, rank decisions, rotation invariance, positive definiteness, and Hessian
capabilities are not applicable to this semantic layer. The review also
covered membership units and signs, overflow-safe path arithmetic, hard-row
preservation, deterministic paths and sources, the canonical solver boundary,
provenance, allocations, interface dispositions, and requirement status.

The reviewer passed all 18 focused level tests, all 29 core Rustdoc tests, the
64-level benchmark smoke at approximately 350 microseconds per validation and
compile iteration, the complete PR and R70-011 `git diff --check`, and the
requirement show/dependency review. The parent Review task independently passed
the same focused level and core Rustdoc suites and the complete PR whitespace
check. Exact-head Draft Ubuntu CI run 29630380600 passed at `b11d321`.

PR #70 must remain Draft and REQ-LEVEL-001 must remain `implemented`. A fresh
Repair task must address only R70-012 and R70-013, add the specified
regressions, run focused checks and the complete stable-head gate, update repair
evidence and the bounded handoff, push, and stop for another fresh independent
re-review. This Review task does not repair production code, mark the PR ready,
merge it, integrate the requirement, or begin another requirement.

## R70-012 and R70-013 repair evidence

Repair implementation commit `0df0550777e8ca95b0d17a9ac08d1ec5a4d5d561`
addresses only R70-012 and R70-013. Before the production change, the new
three-level equality-chain regression failed because distinct fixed A/C values
were accepted, and the one-level field-component regression failed because its
`MissingContrast` diagnostic named unrelated isolated level B.

The repair now builds a deterministic spanning forest over levels joined by
mathematically identical Value evaluations. Its union state supplies the full
transitive equality closure without quadratic edge storage, while a selected
forest path retains every membership source needed to prove one conflict.
Distinct fixed values within a component return both definitions and the full
membership chain. A positive order path within a component returns the full
membership chain and every selected order source. Fixed/prior anchor contrast
is accepted only across different equality components. No canonical row is
changed, dropped, softened, regularized, or emitted on these semantic error
paths, and priors remain soft objective metadata.

`ContrastDiagnostic` now represents either two levels or one membership-bearing
level. When no second level belongs to the failing field component, the
diagnostic retains only that level instead of falling back to an isolated
anchor.

The focused level suite has 20 passing tests and the focused diagnostics suite
has 6. Core all-target/all-feature Clippy, all 29 core Rustdoc tests, and the
64-level benchmark smoke passed; the benchmark completed at approximately 369
microseconds per validation and compile iteration. After the final production,
test, registry, and normative-document change, the exact implementation tree
passed the complete standard workspace gate: formatting, warning-denying
all-target/all-feature workspace Clippy, all-feature workspace tests, workspace
Rustdoc, all 58 requirement checks, and `git diff --check`.

This repair evidence is not an independent re-review and does not itself close
R70-012 or R70-013. PR #70 remains Draft and REQ-LEVEL-001 remains
`implemented`; a fresh read-only re-review must confirm both repairs, reconfirm
R70-001 through R70-011, and check for new findings.

## Fresh independent re-review after R70-012 and R70-013 repair

A fresh read-only project `math_reviewer` independently reviewed exact PR head
`49998ef4b18a803c84817415096dabe4eeabad63` against base and merge base
`2904c64c8d99e0b6a3183dc6c232953a969922ad`. The local and remote heads
matched and the worktree was clean. The reviewer received only the bounded
REQ-LEVEL-001 summary and integrated dependency closure, Issue #69 acceptance
criteria, the M4 plan, relevant mathematical, ADR, and architecture contracts,
the complete PR and latest repair diffs, tests, benchmark, prior review record,
and exact-head validation evidence. It inherited no implementation reasoning
and made no repository or remote changes.

The reviewer independently confirmed R70-012 and R70-013 are closed. The
deterministic membership-equality spanning forest supplies the complete
transitive closure and selected proving membership chain to fixed and order
conflicts, while anchor contrast is accepted only across different equality
components. `ContrastDiagnostic` represents a one-level failing field
component without borrowing an unrelated isolated anchor. The required
three-level equality-chain and one-level diagnostic regressions pass. R70-001
through R70-011 also remain closed.

### P1 R70-014: fixed-order infeasibility depends on the scalar unit

The fixed-order endpoint check at `crates/georbf/src/levels.rs:1645-1650` and
`crates/georbf/src/levels.rs:1780-1788` computes a relative tolerance from
machine epsilon and level count, but floors its comparison scale at the
dimensioned constant `1.0`. This creates an absolute tolerance in whichever
scalar-field unit the caller selected.

For two fixed levels at exactly zero, memberships at distinct Value points,
and the hard order

```text
h_B - h_A >= 1e-20,
```

the available gap is exactly zero and the system is infeasible. With two
levels, the current tolerance is approximately `2.842e-14`, so construction
accepts the contradiction. Multiplying every level value and gap by `1e20`
makes the otherwise equivalent positive-unit rescaling reject the system.
Feasibility must be invariant under a positive change of scalar unit. This is
not a derived-subtraction boundary: both fixed values and their difference are
exactly zero, and the positive required gap is representable.

Repair must add a regression with two distinct membership points, two fixed
zero levels, and a direct `1e-20` order gap. It must require
`FixedOrderConflict` with the exact lower-definition, order, and
upper-definition source sequence, then repeat after a positive unit rescaling
such as `1e20` and require the same verdict and evidence. The roundoff allowance
must use actual problem magnitudes with an exact-zero case instead of an
unscaled `1.0` floor. No hard row may be changed, dropped, softened,
regularized, or otherwise repaired.

No new P0, P2, or P3 finding remains. Membership, fixed, and order canonical
row signs and insertion order are otherwise correct; priors remain soft
metadata; provenance and hard rows are preserved; and no hidden jitter,
pseudoinverse, regularization, or constraint repair occurs. SPD/CPD
classification, polynomial spaces, center limits, rank decisions, rotation
invariance, positive definiteness, and Hessian capability are not applicable
to this semantic compilation layer.

The reviewer passed all 20 focused level tests, all 6 diagnostics tests, all 29
core Rustdoc tests, the level benchmark smoke at approximately 253 microseconds
per validation and compile iteration, the complete PR and latest-repair
`git diff --check`, and the requirement show/dependency review. Exact-head
Draft Ubuntu CI passed at `49998ef`; the Ready-only platform and benchmark
matrix remained skipped as expected. The complete standard workspace gate was
not rerun because exact implementation tree `0df0550` had already passed it and
`0df0550..49998ef` changes only this review record and the bounded handoff.
After recording this review, the parent task passed all 58 requirement checks,
the complete PR whitespace check, and the scoped review-evidence whitespace
check; only this review record and the bounded handoff changed.

PR #70 must remain Draft and REQ-LEVEL-001 must remain `implemented`. A fresh
Repair task must address only R70-014, add the unit-rescaling regression, run
focused checks and the complete stable-head gate after the last code or test
change, update repair evidence and the bounded handoff, push, and stop for a
fresh independent re-review. This Review task does not repair production code,
mark the PR ready, merge it, integrate the requirement, or begin another
requirement.

## R70-014 repair evidence

Repair implementation commit `61fa6d328c79d5236ed937c3c565b344226371d8`
addresses only R70-014. Before the production change, the independently
specified regression failed because the fixed-zero system with a direct
`1e-20` positive order gap returned success. The same fixture now requires
`FixedOrderConflict` both in the original unit and after multiplying the gap by
`1e20`, with the exact lower-definition, order, and upper-definition source
sequence in both cases.

The scaled-magnitude comparison now treats its exact-zero representation
explicitly, so every positive magnitude orders above zero regardless of its
binary exponent. Once the available gap is nonzero, the conservative relative
allowance is scaled only by the actual compared gap rather than a dimensioned
`1.0` floor. The repair changes no canonical hard row, source, or solver input
and adds no jitter, regularization, pseudoinverse, softening, or automatic
constraint repair.

The focused level suite has 21 passing tests and the focused diagnostics suite
has 6. Core all-target/all-feature Clippy, all 29 core Rustdoc tests, and the
64-level benchmark smoke passed; the benchmark completed at approximately 247
microseconds per validation and compile iteration. After the final production,
test, registry, and normative-document change, exact implementation tree
`61fa6d3` passed the complete standard workspace gate: formatting,
warning-denying all-target/all-feature workspace Clippy, all-feature workspace
tests, workspace Rustdoc, all 58 requirement checks, and `git diff --check`.

This repair evidence is not an independent re-review and does not itself close
R70-014. PR #70 remains Draft and REQ-LEVEL-001 remains `implemented`; a fresh
read-only re-review must confirm R70-014 closure, reconfirm R70-001 through
R70-013, and check for new findings before any Ready transition.

## Fresh independent re-review after R70-014 repair

A fresh read-only project `math_reviewer` independently reviewed exact local
and remote PR head `6a03fe6ae75df56613e1ae3ee7beda50aa5afb07`
against base and merge base `2904c64c8d99e0b6a3183dc6c232953a969922ad`.
It received only the bounded REQ-LEVEL-001 summary and integrated dependency
closure, Issue #69 acceptance criteria, the M4 plan, relevant mathematical,
ADR, and architecture contracts, the complete PR and latest repair diffs,
tests, benchmark, prior review record, and exact-head validation evidence. It
inherited no implementation reasoning and made no repository or remote
changes.

The reviewer independently confirmed R70-014 is closed. For fixed endpoints
`h_A = h_B = 0` and any positive gap, the exact-zero available magnitude is
ordered below the required magnitude before exponent comparison. For nonzero
availability, the dimensionless comparison scales the conservative tolerance
only from the required gap, which is the larger compared magnitude. No
dimensioned floor remains, so positive scalar-unit rescaling preserves the
verdict. The exact `1e-20` and `1e20`-rescaled regression requires
`FixedOrderConflict` with the lower definition, order edge, and upper
definition sources in that order. The original hard rows remain unchanged.

R70-001 through R70-013 also remain closed. The review reconfirmed membership
units and signs, the transitive membership-equality forest and proving chains,
membership/order infeasibility, exact cycle sources, overflow-safe fixed paths,
component gauge and contrast, canonical row ordering, prior metadata, source
provenance, D=1/D=2/D=3 bounds, interface dispositions, and registry truth. No
P0-P3 finding remains. SPD/CPD classification, polynomial spaces, center
limits, rank decisions, rotation invariance, positive definiteness, and Hessian
capability are not applicable to this semantic compilation layer.

The independent reviewer passed all 21 focused level tests, all 6 diagnostics
tests, all 29 core Rustdoc tests, the 64-level benchmark smoke at approximately
360 microseconds per validation and compile iteration, the complete PR and
latest-repair whitespace checks, and an exact-rational scaled-arithmetic probe.
The probe covered 20,007 representations, 50,000 ordering pairs, 19,977
feasible accumulated paths without a false conflict, and nonzero-boundary
power-of-two rescaling cases. The parent Review task independently passed the
same focused Rust tests and Rustdoc, the benchmark smoke at approximately 192
microseconds per iteration, all 58 requirement checks, and the complete PR
whitespace check. Exact-head Draft Ubuntu CI passed; the Ready-only three-
platform and benchmark-smoke matrix remained skipped as expected.

The complete workspace gate was not rerun because immutable implementation
tree `61fa6d3` already passed it and `61fa6d3..6a03fe6` changes only this review
record and the bounded handoff. Residual non-blocking risks are the existing
quadratic membership comparison and fixed-path scaling costs, the benchmark's
lack of an acceptance threshold, and the absence of exhaustive allocation,
fuzz, and mutation instrumentation.

PR #70 remains Draft and REQ-LEVEL-001 remains `implemented`. This clean
re-review task does not mark the PR ready, merge it, integrate the requirement,
or begin another requirement. A fresh integration Review task may verify that
every commit after reviewed head `6a03fe6` is limited to this review record and
the bounded handoff, synchronize the clean evidence, mark that evidence-only
descendant head ready, wait for the complete Windows, Ubuntu, macOS, and
benchmark-smoke CI on that exact ready head, merge exactly once only if all
required checks are green, and then record truthful integration state.

## Integration evidence

The integration task confirmed that exact Ready head
`5bfa52f81f31785a660d7446c55099e570e29521` differs from independently
reviewed head `6a03fe6ae75df56613e1ae3ee7beda50aa5afb07` only through
this review record and the bounded handoff. Those evidence-only changes alter
no production code, tests, manifests, registry, schemas, CI, build input, API,
normative contract, numerical behavior, dependency, tag, or release.

Exact Ready head `5bfa52f` passed the complete Windows, Ubuntu, and macOS
correctness matrix with every backend and benchmark-smoke workload in CI run
29646041086. PR #70 then squash-merged exactly once as
`11e0659319ae08731f083749974d9ad6fb316616`, and Issue #69 closed as
completed. Post-merge `main` run 29646382654 passed the same complete
three-platform correctness, benchmark-smoke, and requirement-registry gate on
that exact merge commit.

The isolated integration-state change updates only the registry, this review
evidence, the history index, and the bounded handoff. It changes no production
code, tests, manifests, schemas, CI, build inputs, APIs, normative contracts,
numerical behavior, dependencies, tags, or releases. REQ-LEVEL-001 may
therefore be recorded as `integrated`; the next requirement must begin only in
a fresh task after the isolated integration-state pull request is green and
merged.

The isolated integration-state registry tree passed the complete local
standard gate and `git diff --check`. Any subsequent pull-request-link update
is documentation-only and changes no production, test, manifest, registry,
schema, CI, or build input. The isolated integration-state evidence is
published in PR #71.
