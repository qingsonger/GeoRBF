# Independent Review of PR #26

- Requirement: REQ-ORIENT-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/25
- Pull request: https://github.com/qingsonger/GeoRBF/pull/26
- Branch: `codex/req-orient-001-geological-orientation`
- Initial reviewed head: `e4ec9bc`
- Repaired and re-reviewed head: `bf00f73`
- Review date: 2026-07-14
- Mode: Independent review and repair

## Scope and evidence read

The independent subagent read `AGENTS.md`, `V1_SCOPE.md`, all requirement and
handoff constraints relevant to REQ-ORIENT-001, the milestone plan, applicable
mathematical and architecture contracts and ADRs, Issue #25, Draft PR #26, and
the complete diff and tests relative to `main`. It independently derived the
D=2/D=3 conversions and reviewed dimensions, units, signs, ranges, polarity,
rotation covariance, numerical boundaries, errors, allocation, safety,
interfaces, documentation, requirement state, and exclusions.

The review remained within orientation conversion. It did not begin normal or
tangent observations, orthogonal-complement construction, angular cones,
anisotropy, fields, functionals, assembly, solvers, schemas, persistence, or
language bindings.

## Finding repaired

1. **P1 — polarity application recreated negative zero.** Angle constructors
   canonicalized exact zero components before applying polarity, but negative
   polarity then negated every component. Component constructors also retained
   caller-supplied `-0.0`. Because `UnitDirection` preserves zero sign during
   normalization, these values violated the normative positive-zero boundary
   contract while still passing tolerance comparisons. The repair applies
   polarity first and then canonicalizes every zero component immediately
   before `UnitDirection` construction. An exact-bit regression now covers
   D=2/D=3 planar and linear horizontal, vertical, compass-quadrant, and raw
   component paths, including Positive, Negative, and Unknown polarity.

The independent re-review of `bf00f73` reported no remaining or new finding.

## Mathematical and convention conclusion

- In the documented right-handed local frame X east, Y north, and Z up,
  azimuth increases clockwise from +Y toward +X. For azimuth `a`, dip `d`, and
  downward-positive signed plunge `p`, the implemented reference vectors are
  exactly

  ```text
  n(a,d) = [sin(d) sin(a), sin(d) cos(a),  cos(d)]
  l(a,p) = [cos(p) sin(a), cos(p) cos(a), -sin(p)].
  ```

  Independent derivation confirmed that `n` is orthogonal to the strike and
  down-dip tangents and points upward, and that the lineation sign is correct.
- Right-hand-rule strike maps to down-dip azimuth
  `strike + pi/2 (mod 2pi)`. Cardinal and oblique equivalence tests would fail
  for a reversed offset or an east/north axis swap.
- D=2 signed dip `[sin(d), cos(d)]` and signed plunge
  `[cos(p), -sin(p)]` use X horizontal and Y up and have the documented signs.
- Degree and radian inputs use the same bounded domains before conversion:
  azimuth/strike/dip direction `[0, 2pi)`, plane dip `[0, pi/2]`, and signed
  D=2 dip or plunge `[-pi/2, pi/2]`. Non-finite and next-representable
  out-of-range inputs return field- and unit-aware structured errors.
- Exact horizontal, vertical, and compass branches remove irrelevant pole
  azimuth noise. The repaired post-polarity canonicalization makes every stored
  zero positive without changing nonzero antipodes.
- Positive keeps the reference vector, Negative stores its antipode, and
  Unknown preserves a representative with explicit axial metadata. No path
  infers or imposes a nonzero gradient magnitude.
- Horizontal rotations produce the independently expected transformed X/Y
  components while preserving Z. Raw component conversion also covaries under
  a D=2 rotation. D=1 and D=4 fail to compile.
- SPD/CPD classification, kernel center limits, polynomial space, numerical
  rank, hard constraints, infeasibility, positive definiteness, and Hessian
  capability are not introduced and therefore are not claimed by this PR.

The planar/linear distinction and explicit convention follow
[OGC GeoSciML 4.1](https://docs.ogc.org/is/16-008/16-008r1.html); compass
azimuth and right-hand-rule meaning agree with
[USGS OFR 01-223](https://pubs.usgs.gov/of/2001/of01-223/richard2.html).

## Safety, API, performance, and interface conclusion

- Public values have private fixed-array representations backed by the existing
  overflow/underflow-safe `UnitDirection`. User input cannot cause a production
  panic. There is no unsafe code, global mutable state, core output, placeholder
  path, heap allocation, dynamic numerical dispatch, or new dependency.
- Planar normals and linear directions remain distinct types. Polarity and
  angle-field errors are explicit, immutable values are `Send + Sync`, and no
  third-party type crosses the public geometry API.
- Every conversion is fixed bounded work over D=2 or D=3 and no batch hot path
  is introduced. The registry's benchmark N/A disposition is truthful.
- Rust is the only applicable interface in this increment. CLI, C, C++, and
  Python wait for versioned project inputs and their separately reviewed API
  freeze/binding requirements; no placeholder adapter capability was added.
- Rustdoc, normative math and architecture text, the runnable example, change
  record, registry, and progress handoff agree with the implementation and its
  exclusions.

## Verification

Passed locally on Windows with Rust 1.96.1 against repaired head `bf00f73`:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features` (102 passed, 0 failed)
- `cargo test --doc --workspace` (19 passed, including 17 compile-fail cases)
- `cargo test -p georbf --release --all-features` (88 integration tests and 19
  doctests passed)
- `RUSTDOCFLAGS="-D warnings" cargo doc -p georbf --all-features --no-deps`
- `cargo xtask requirements check` (58 requirements)
- `cargo metadata --format-version 1 --no-deps`
- `cargo tree --workspace --duplicates` (no duplicate tree)
- the geological-orientation example, scoped forbidden-pattern/allocation
  scan, and `git diff --check`

The independent reviewer separately ran the 12 orientation tests, formatting,
the requirement checker, and diff checks for both the repair and full PR. PR
CI run 29304961733 passed on Windows, Ubuntu, and macOS before review repair.
Repair-head run 29305373648 passed the complete three-platform job set,
including all 102 workspace tests, 19 doctests, four existing benchmark smoke
workloads, strict Clippy, formatting, and all 58 requirement checks.

## Checks not run

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Rust 1.96.1 does not provide the Miri component. Sanitizers,
executable fuzzing, mutation testing, allocation instrumentation, and
API/ABI/schema snapshot tooling are not yet implemented. No orientation
benchmark was run because the requirement truthfully records constant-bounded
fixed-size conversion as N/A. These are residual release-program risks, not a
remaining finding in REQ-ORIENT-001.
