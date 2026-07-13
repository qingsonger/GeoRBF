# GeoRBF v1.0.0 Release Checklist

Release mode is allowed only when every item below has objective evidence.

## Requirements and source

- [ ] Every mandatory entry in `requirements/v1.yaml` is `integrated`.
- [ ] `cargo xtask requirements check` passes.
- [ ] Production code contains no unresolved placeholders.
- [ ] The core crate contains no unsafe code.
- [ ] Public APIs are documented and API/schema/ABI snapshots are frozen.
- [ ] No unresolved P0 or P1 issue remains.

## Mathematical and numerical verification

- [ ] D=1, D=2, and D=3 truth suites pass.
- [ ] Kernel derivatives, center limits, SPD/CPD, polynomial reproduction, and
      rank-degenerate cases pass independently.
- [ ] Hard, soft, bound, cone, level, normal, tangent, gauge, contrast,
      thickness, and infeasibility suites pass.
- [ ] Coordinate, rotation, scaling, anisotropy, and local-mixture invariance
      tests pass.
- [ ] Level point, isoline, and isosurface topology suites pass.
- [ ] Dense and sparse paths, center selection, tuning, and memory estimates
      pass their correctness and performance gates.
- [ ] No blocking regression exists against the previous stable GeoRBF
      benchmark baseline.

## Interfaces and platforms

- [ ] Rust, CLI, C, C++, and Python capability matrices are complete.
- [ ] Cross-language parity and model round trips pass.
- [ ] The C ABI snapshot and dynamic-analysis suites pass.
- [ ] Python wheels install and run on supported targets.
- [ ] Windows, Linux, and macOS artifacts pass clean-install tests.
- [ ] Miri, sanitizers, fuzzing, mutation, and dependency security checks pass.

## Documentation and supply chain

- [ ] Chinese and English README, user guide, API references, examples, and
      release notes are complete.
- [ ] SBOM, third-party license inventory, checksums, and provenance are
      generated and verified.
- [ ] Release credentials and permissions are configured without exposing
      secrets.

## Publication

- [ ] At least one full v1.0.0 release-candidate rehearsal succeeded.
- [ ] The source tag and GitHub Release exist.
- [ ] crates.io and PyPI packages are actually published and installable.
- [ ] CLI binaries and C/C++ SDK artifacts are actually attached and verified.

If credentials are unavailable, report publication as blocked; never mark the
release successful based only on locally built artifacts.
