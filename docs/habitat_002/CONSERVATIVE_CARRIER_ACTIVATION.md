# Implicit background carrier: lane D, bounded kernel adapter

Uniform atmospheric background is finite density, not vacuum. Every implicit cell
represents all19 populations. Critically, the001 equilibrium is not an IEEE-bit
fixed point of BGK: its homogeneous recurrence must advance every substep. The
workspace caches that deterministic recurrence by Config/microstep; failed or
restored executions recompute from a lawful earlier point, never from a future
scratch clock. No externally mutable telemetry affects the physics.

Input blocks encode every population differing in bits from the current homogeneous
background. The D3Q19 stencil traverses at most one cell per axis per substep and
includes face diagonals. Required output blocks include the source block and its
D3Q19 neighboring block offsets (a conservative superset at block boundaries).
Pull destinations sample exact source postcollision populations or the same-time
background. Missing halo is assertion-tested honest-red. No visibility/organism
input exists. Zero velocity is insufficient: a dedicated non-equilibrium stress
perturbation remains allocated and propagates despite zero initial velocity.

All19 output values must match implicit replacement bits for carrier-block
retirement. This is ONLY carrier encoding; scalar/wall/body state is preserved
independently. It never declares an entire environment region disposable. Empty
scalar blocks retire independently only at exact zero; tests retire/reactivate a
synthetic eligible scalar block and retain minimum subnormal mass.

The bounded adapter re-encodes dense authoritative input and decodes dense output.
Original ordered wall pressure impulses still visit boundary exchanges, including
quiet background, because skipping them changes IEEE accumulation. Config/State
validation/diagnostics and scalar sampling still visit dense cells. Therefore D
CANNOT qualify production sparse residency or domain-independent whole-tick cost.
It is a falsifiable exact carrier-kernel experiment within Chamber, not a second
owner. Replacing dense State and ordered diagnostics with lawful sparse authority
is a remaining architectural seam; publication does not silently adopt it.

## Final measured status

Final D matches all compared physical bits but full-demo mean2.257422ms and dense encode/decode/diagnostic/wall-mask scans prevent sparse-atmosphere qualification. Largest fixed-h1.6m chamber retains121/512 encoded blocks; full update6.7712ms. This adapter is rejected as the production scalability solution.
