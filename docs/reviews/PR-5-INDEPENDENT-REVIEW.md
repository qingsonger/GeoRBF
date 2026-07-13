# Independent Review of PR #5

- Requirement: REQ-DIM-001
- Issue: https://github.com/qingsonger/GeoRBF/issues/4
- Pull request: https://github.com/qingsonger/GeoRBF/pull/5
- Branch: `codex/req-dim-001-dimension-safe-geometry`
- Reviewed code head: `6db7097`
- Review date: 2026-07-13
- Mode: Independent review and repair

## Scope and evidence read

The review covered `AGENTS.md`, `V1_SCOPE.md`, `requirements/v1.yaml`,
`docs/math/`, `docs/architecture/`, all accepted ADRs,
`docs/MASTER_PLAN.md`, `docs/progress/CURRENT.md`, Issue #4, the complete PR
diff and commit list, PR discussion and review state, the three-platform CI
logs, and the applicable test and benchmark obligations. PR #5 had no submitted
review, inline thread, or comment before this review.

## Finding repaired

`docs/progress/CURRENT.md` recorded GitHub Actions run 29243178924 for commit
`635327c`, although the reviewed PR head was `6db7097` and its successful run
was 29243260972. The handoff record now identifies the reviewed code head and
its actual CI run. This review document records the independent conclusion and
the checks repeated by the reviewer.

## Mathematical and numerical conclusion

- The public geometry contract is exactly Euclidean `Point`, `Vector`, nonzero
  `Direction`, and `UnitDirection` for D=1, D=2, and D=3. The const-generic
  dimension seal has implementations only for those dimensions; compile-fail
  doctests independently reject D=0 and D=4.
- Unit normalization divides first by the largest absolute component. For a
  finite nonzero direction, the scale is finite and positive, at least one
  scaled component has magnitude one, and every scaled component has magnitude
  at most one. With D at most three, the scaled squared norm is in `[1, 3]`, so
  it cannot overflow, underflow to zero, or create a non-finite result. Tests
  cover `f64::MAX`, the smallest positive subnormal, signs, scale invariance,
  and known 3-4-5 values.
- Kernels, SPD/CPD classification, polynomial spaces, null spaces, derivatives,
  center limits, numerical rank, condition estimation, hard constraints,
  infeasibility, normals, tangents, polarity, cones, anisotropy, and Hessian
  evaluation are outside REQ-DIM-001. The PR exposes none of those capabilities
  and therefore makes no premature mathematical or numerical promise.

## Safety, API, performance, and test conclusion

- Geometry fields are private and construction rejects indexed non-finite
  components and zero directions through structured errors. Production code
  has no `unsafe`, panic path on user input, global mutable state, dependency on
  adapters, heap allocation, or third-party public type.
- The immutable fixed-size values are `Send + Sync`; this is asserted at
  compile time. Normalization is deterministic and constant-time for the only
  supported dimensions. No benchmark is required for this non-hot-path
  primitive, and the registry records that obligation as N/A rather than as a
  passed benchmark.
- Tests use analytic values and mathematical properties rather than another
  implementation or proprietary output. They cover construction errors,
  conversions, extreme magnitudes, dimension boundaries, normalization
  invariants, and thread-safety. Release-mode tests and warning-free rustdoc
  generation also pass.
- Rust is the only applicable API surface for this primitive requirement. CLI,
  C, C++, and Python are correctly N/A because later adapters consume validated
  arrays, shapes, and dimension tags rather than expose Rust geometry types.
  No semantic mismatch or false adapter capability is present.
- Rustdoc, the runnable example, the change record, the requirement registry,
  and the progress handoff cover the implemented API. API/ABI/schema snapshot
  tooling is not yet present and is not claimed as passed.

No blocking mathematical, numerical, safety, API, performance, or test finding
remains in REQ-DIM-001 after the progress-evidence repair. The requirement must
remain `documented`, not `integrated`, until PR #5 is merged.

## Verification

Passed locally on Windows with Rust 1.96.1 against reviewed code head
`6db7097`:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features` (23 passed, 0 failed)
- `cargo test --doc --workspace` (3 passed, including two compile-fail cases)
- `cargo test -p georbf --release --all-features` (9 integration tests and 3
  doctests passed)
- `cargo xtask requirements check` (58 requirements)
- `RUSTDOCFLAGS="-D warnings" cargo doc -p georbf --all-features --no-deps`
- `cargo metadata --format-version 1 --no-deps` (5 workspace members)
- `cargo tree --workspace --duplicates` (no duplicates)
- the dimension-safe geometry example and actual CLI success/error paths
- forbidden-code scan and `git diff --check`

GitHub Actions run 29243260972 passed for the reviewed code head on Windows,
Ubuntu, and macOS. Each job passed formatting, Clippy, workspace tests,
doctests, and all 58 requirement checks. After this record was committed,
three-platform run 29244426487 also passed for review-record head `3eaf97f`;
PR #5 was then marked ready for maintainer review.

## Checks not run

`cargo-nextest`, `cargo-deny`, `cargo-audit`, `cargo-semver-checks`, Miri,
sanitizers, fuzzing, mutation testing, and API/ABI/schema snapshot tooling are
not installed or not yet implemented. The pinned Rust toolchain does not offer
the Miri component. No benchmark was run because fixed-size validation and
normalization for D at most three are constant-time and the requirement records
the obligation as N/A.
