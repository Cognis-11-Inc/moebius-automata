# Dynamic refinement — bounded two-level experiment

Existing physical jump/hysteresis indicators are reused. The new registered matched
3D diagonal-scalar controls run2.5s with/without indicators: density.001,velocity
.01m/s,scalar2e-5kg/m³,nonequilibrium.01,coarsen fraction.4,quiet8 cycles. All are
versioned physical configuration; no view/player position or timing enters them.
Actual refine/coarsen counters, cache invalidations, final partition and analytic
error are measured. Final outcomes are recorded below.

This is a translated sinusoidal field on two levels, not a certified moving localized
cloud tracker or arbitrary-depth split/merge system. Remapping remains piecewise
constant and conservative; lost subcell information affects accuracy. No positive
tail is deleted to save work. Coordinate and balance semantics stay those of003.
A lawful next extension needs localized disturbance tracking, measured region
trajectories/threshold sensitivity and multiple-level controls before stronger claims.

## Final measured qualification

| Case | Refinements / coarsenings | Leaves | Scalar error | Cache builds | Mean ms |
|---|---|---|---|---|---|
| physical-adaptation-False | {'refinements': 0, 'coarsenings': 0} | 512 | 0.021178 | 1 | 3.33142 |
| physical-adaptation-True | {'refinements': 256, 'coarsenings': 8} | 2248 | 0.026741 | 3 | 19.9557 |

Matched duration2.5s. This translated periodic sinusoid has two physical levels and fixed existing threshold logic; it is not a compact moving cloud, recursive mesh or split/merge test. Scalar error beyond .15 is a failed fidelity gate even when inventories remain conserved.

Observed negative result: both endpoints pass the .15 scalar bound, but dynamic error is 1.263× static and ordinary mean cost is 5.990× static. The main refinement is256 roots, followed by8 coarsenings; no continued moving-cloud tracking is established. Piecewise-constant transfer is a possible mechanism, not an isolated causal result; a matched intervention is needed.
