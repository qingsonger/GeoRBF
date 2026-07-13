# ADR-0002: No Surfe Model Types or Compatibility Layer

- Status: Accepted
- Date: 2026-07-13

## Context

Historical systems expose five model families and format conventions. Copying
that taxonomy would conflict with the unified functional formulation and create
an accidental compatibility and correctness obligation.

## Decision

GeoRBF is a new design. It will not expose Surfe APIs, file formats,
`model_type` values, five model classes, compatibility crates, or Surfe golden
results as truth. Surfe source is not copied, translated, or mechanically
rewritten. Explicitly approved future performance comparisons may treat it only
as an external benchmark.

## Consequences

Data migration is not a v1 feature. Public names and schemas describe GeoRBF's
own scalar-field concepts. Correctness evidence is mathematical and independent.
