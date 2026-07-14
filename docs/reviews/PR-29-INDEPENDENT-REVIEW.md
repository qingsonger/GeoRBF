# Independent Review of PR #29

- Requirement: REQ-ANISO-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/28
- Pull request: https://github.com/qingsonger/GeoRBF/pull/29
- Branch: `codex/req-aniso-001-global-anisotropy`
- Initial reviewed head: `83a46e9`
- Repaired and re-reviewed head: `960496a`
- Review date: 2026-07-14
- Mode: Independent mathematical and numerical review and repair

## Scope and evidence read

The independent subagent read `AGENTS.md`, `V1_SCOPE.md`, the requirement and
handoff state, the milestone plan, applicable mathematical and architecture
contracts, accepted ADRs, Issue #28, Draft PR #29, and the complete diff and
tests relative to `main`. It independently reviewed the D=1/D=2/D=3 metric
constructions, dimensions, units, signs, SPD classification, singular-value
diagnostics, condition policy, center limits, rotation and scaling covariance,
derivative chain rules through third order, errors, allocation, safety,
performance, interfaces, documentation, requirement state, and exclusions.

The review remained within fixed global anisotropy. It did not begin local
anisotropy, orientation-tensor estimation, SPD kernel mixtures, observations,
center selection, assembly, fitting, solvers, persistence, schemas, or language
bindings.

## Findings repaired

1. **P1 — difference-form spheroidal factor lost valid high axis ratios.**
   The original projector expression subtracted two large reciprocal scales.
   For a principal coordinate direction this cancelled a representable small
   axial reciprocal, making valid D=1/D=2 `1e100` ratios singular. The repair
   constructs a stable dimension-specific orthonormal frame whose principal
   row is scaled by the axial reciprocal and whose transverse rows are scaled
   independently. Exact high-ratio regressions cover D=1 and D=2; independent
   D=3 singular-value truth covers a `1e150` condition number.
2. **P1 — a rounded transform-derived metric could lose SPD.** A finite,
   invertible transform such as `[[1,1],[1e-12,2e-12]]` can produce the rounded
   public metric `[[1,1],[1,1]]`. The original path exposed that singular
   `A^T A`. The repair certifies the exact signs of the leading principal
   minors of the rounded stored metric and returns a structured error if
   rounding loses SPD.
3. **P1 — ordinary floating Cholesky could accept a slightly indefinite user
   metric.** The reported D=2 boundary matrix has an exact negative determinant
   although an ordinary computed pivot rounded positive. The repair applies an
   exact-sign expansion to every leading principal minor before unregularized
   Cholesky. D3 equicorrelation regressions straddle the exact determinant
   boundary while all pairwise minors remain positive.
4. **P1 — a rounded square-root prefilter rejected valid boundary SPD input.**
   For `[[1,1],[1,1+EPSILON]]`, the exact second leading minor is positive but
   `sqrt(1+EPSILON)` rounds to one. The redundant `>= sqrt(...)` prefilter
   rejected that valid matrix before exact evaluation. The repair removes that
   decision and adds D=2 and D=3 block-embedded regressions. A strict necessary
   `|b_ij| > max(b_ii,b_jj)` guard rejects only mathematically impossible SPD
   entries before exact-product formation and prevents invalid `f64::MAX`
   off-diagonal input from overflowing the expansion.

The independent final re-review of `960496a` reported no remaining or new
P0--P3 finding.

## Mathematical and numerical conclusion

- The distance convention is `r_A(x,y)=||A(x-y)||` with the stored metric
  `B=A^T A`. Isotropic, spheroidal, ellipsoidal, arbitrary-transform, and
  exactly symmetric SPD metric construction use positive coordinate-length
  scales and expose consistent fixed-size transforms, inverses, metrics,
  singular values, and Euclidean condition numbers.
- The repaired spheroidal construction is an orthonormal factor of
  `B=uu^T/ell_a^2 + (I-uu^T)/ell_t^2`. It never forms the large-minus-large
  reciprocal-scale projector and retains the specified principal direction in
  D=1, D=2, and D=3.
- User and transform-derived metrics are finite and exactly symmetric.
  Power-of-two congruence equilibration preserves every principal-minor sign.
  Exact floating expansions use error-free `two_sum`, FMA two-products, and
  FMA triple-product decomposition. The D=2 expansion needs at most four
  components and the D=3 six-term determinant needs at most 24; the fixed
  capacity of 64 is sufficient.
- The D=3 determinant permutation indices and signs are correct. Sylvester's
  leading-principal-minor criterion is sufficient because the matrix is exactly
  symmetric. No eigenvalue tolerance replaces the exact signs.
- After equilibration every diagonal is in `[1,4)`. For SPD,
  `|b_ij| < sqrt(b_ii b_jj) <= max(b_ii,b_jj)`, so the strict maximum guard
  cannot reject SPD or an equality boundary. Every component reaching the
  exact expansion has magnitude at most four; D=2 products remain below 16 and
  D=3 triple products below 64, avoiding overflow and NaN.
- Exact leading-minor certification precedes unregularized Cholesky. There is
  no hidden symmetry tolerance, jitter, regularization, eigenvalue clipping,
  pseudoinverse, or automatic condition cutoff. The only condition rejection
  policy is an explicit caller value recorded in structured diagnostics.
- One-sided Jacobi singular values and exact partial-pivot inversion produce
  structured singular or nonrepresentable errors. The repaired boundary
  examples report the expected finite or rejected states.
- The constant-map chain rule is correct through third order:
  `g_x=A^T g_z`, `H_x=A^T H_z A`, and each third-tensor index contracts with
  one column of `A`. Independent polynomial truth, query/center exchange signs,
  tensor symmetry, analytic center limits, and rotation covariance confirm the
  implementation.
- Local anisotropy, CPD local mixtures, observations, polynomial spaces, rank
  decisions, hard constraints, infeasibility, and solvers are not introduced
  and therefore are not claimed by this PR.

## Safety, API, performance, and interface conclusion

- Public immutable values use private fixed arrays. User input does not cause a
  production panic. There is no unsafe code, global mutable state, core output,
  placeholder path, per-evaluation heap allocation, dynamic numerical dispatch,
  or new dependency.
- The numerical hot path computes `A(x-y)` directly and reuses existing stable
  radial separation and jet machinery. It does not clone kernels or perform
  unnecessary full-matrix work. The deterministic D=1/D=2/D=3 benchmark and CI
  smoke workload truthfully cover this new runtime path.
- Rust is the only applicable interface in this increment. CLI, C, C++, and
  Python wait for versioned project schemas and later API-freeze/binding
  requirements; no placeholder adapter capability was added.
- Rustdoc, normative math and architecture text, the runnable example, change
  record, benchmark record, requirement registry, and progress handoff agree
  with the implementation and exclusions.

## Verification

Passed locally on Windows with Rust 1.96.1 against repaired head `960496a`:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features` (115 passed, 0 failed)
- `cargo test --doc --workspace` (21 passed, including 18 compile-fail cases)
- `cargo test -p georbf --release --all-features` (101 integration tests and
  21 doctests passed)
- `RUSTDOCFLAGS="-D warnings" cargo doc -p georbf --all-features --no-deps`
- `cargo bench -p georbf --bench global_anisotropy -- --smoke`
- `cargo xtask requirements check` (58 requirements)
- the global-anisotropy example, dependency metadata/tree checks, scoped
  forbidden-pattern/allocation scans, and `git diff --check`

The independent reviewer separately ran all 13 anisotropy tests, focused
warning-denying Clippy, the requirement checker, repair-range diff checks, and
independent reproductions for every finding. Initial PR run 29308438441,
first-repair run 29309340493, square-root-repair run 29309631907, and final
repair run 29309762439 passed on Windows, Ubuntu, and macOS. The final run
included all 115 workspace tests, 21 doctests, all five benchmark smoke
workloads, strict formatting and Clippy, and all 58 requirement checks.

## Checks not run and residual risk

`cargo-nextest`, `cargo-deny`, `cargo-audit`, and `cargo-semver-checks` are not
installed. Rust 1.96.1 does not provide the Miri component. Sanitizers,
executable fuzzing, mutation testing, allocation instrumentation, and
API/ABI/schema snapshot tooling are not yet implemented. Workspace-wide
rustdoc is not counted as passed because Cargo's same-output-name collision
between the core library and CLI binary caused a Windows target-doc access
error; warning-denying core rustdoc passed.

The remaining numerical coverage risk is that extreme D=3 minimum singular
values do not yet have an exhaustive high-precision oracle. Existing closed-
form and extreme diagnostic truth, mathematical review, independent boundary
reproductions, and three-platform CI show no open defect. These are residual
release-program risks, not remaining findings in REQ-ANISO-001.
