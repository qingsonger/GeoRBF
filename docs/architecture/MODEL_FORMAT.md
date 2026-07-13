# Project and Model Format Policy

## Project documents

A versioned project contains coordinate metadata and one or more independent
fields. Each field records dimension, kernel, anisotropy, levels, observation
sets, constraints, fit options, and contour options. There is no `model_type`.

Coordinate metadata includes length unit, CRS/EPSG/WKT metadata, axis order,
positive-Z convention, handedness, and angle convention. The reader rejects
incompatible units and preserves CRS metadata without performing reprojection.

Length units use exact validated identifiers; aliases and conversions are an
input-adapter responsibility. Axis order is a permutation from stored component
positions to canonical axes. Canonical axes are X, X/Y, or X/Y/Z for D=1, D=2,
or D=3 respectively, and the final canonical axis is vertical. CRS metadata may
contain EPSG, WKT, both, or neither, and is preserved as opaque data. Combining
coordinate sources requires exactly matching unit, CRS, axis-order,
vertical-direction, handedness, and angle-unit metadata unless an explicit
conversion has already produced a common convention.

Normalization records finite `mu`, finite invertible `S`, and the contract
`x_tilde = S^-1 (x - mu)`. Readers must validate invertibility without hidden
regularization and reject an inverse or transformed result that is not finite.
Data-derived equilibration may change the inversion representation but never
the stored matrix or user semantics. If that representation cannot remain in
the finite nonzero domain, readers retry the original unscaled matrix with the
same exact-zero singularity policy.
The persistence schema and migration rules remain deferred to REQ-SCHEMA-001;
this policy does not claim that a public project or model format exists yet.

## Fitted models

A deterministic model records schema version, dimension, coordinate metadata,
normalization, kernel and anisotropy definitions, centers, center functionals,
kernel and polynomial coefficients, solved level values, capabilities,
diagnostics, build version, and checksums. It does not depend on an input
builder after loading.

Readers reject unsupported versions, missing required fields, inconsistent
dimensions, nonfinite coefficients, invalid transforms, capability conflicts,
and checksum failures with structured field paths. Semantic and tolerance
compatibility is required across platforms; bitwise equality across all CPUs or
backends is not.

Schema snapshots, round-trip fixtures, corrupt-input tests, migration policy,
and maximum allocation limits are release gates before public persistence is
declared complete.
