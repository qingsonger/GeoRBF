# REQ-ORIENT-001

Added separate planar-normal and linear-direction orientation values for D=2
and D=3. Fixed-size constructors accept finite nonzero components or explicit
degree/radian geological angles, normalize through the existing stable geometry
path, and preserve positive, negative, or unknown/axial polarity metadata.

The D=3 convention is a local right-handed X-easting/Y-northing/Z-up frame,
with compass azimuth clockwise from north, signed plunge positive downward,
and right-hand-rule strike/dip. D=2 uses an X-horizontal/Y-up vertical section.
Canonical angle intervals, non-finite values, zero directions, and non-finite
components return structured errors. Exact horizontal, vertical, and compass
boundary branches remove irrelevant pole-azimuth noise.

Independent analytic direction truth, degree/radian parity, strike versus dip-
direction equivalence, polarity, D=2/D=3 rotation covariance, extreme component
normalization, angle and component error paths, compile-fail dimensions, and
thread-safety tests cover the conversion API. A runnable example and normative
convention citations accompany the implementation.

No normal or tangent observation, gradient-magnitude claim, cone, anisotropy,
orientation-tensor estimation, reprojection, field, assembly, solver, schema,
binding, or compatibility behavior is included.
