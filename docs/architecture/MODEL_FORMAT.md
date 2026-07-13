# Project and Model Format Policy

## Project documents

A versioned project contains coordinate metadata and one or more independent
fields. Each field records dimension, kernel, anisotropy, levels, observation
sets, constraints, fit options, and contour options. There is no `model_type`.

Coordinate metadata includes length unit, CRS/EPSG/WKT metadata, axis order,
positive-Z convention, handedness, and angle convention. The reader rejects
incompatible units and preserves CRS metadata without performing reprojection.

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
