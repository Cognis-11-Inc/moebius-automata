# Higher-order finite-volume ablations

IMPLEMENTED separate Config Reconstruction::{Constant,Minmod} and
Integrator::{Euler,Ssp2}. All four combinations are explicit ablations. The
original constant/Euler numerical path remains a reference and reproduces the
legacy physical state exactly after removal of declared new metadata/zero-ledger
fields. Historical003 source/executables remain independently restorable.

Primitive gradients use area-weighted left/right differences divided by physical
centre distance, then minmod. Both side states are reconstructed at the actual
coarse/fine subface centroid, including coarse tangential displacement. Existing
centre-gradient diffusion/viscosity is retained. No blanket3D/nonuniform TVD or
second-order theorem is claimed. Negative/nonfinite reconstructed state refuses;
there is no clipping or floor. A natural red test exposed multiplication-underflow
in minmod's sign comparison; explicit sign tests preserve small finite gradients.

SSP2 uses two Euler evaluations and a half/half convex combination of extensive
inventories and changed impulse/wall reservoirs. Each intermediate state is fully
validated before the next stage or averaging. The SSP name does not establish
positivity of the selected space operator. Higher-order subcycling is refused
pending its own interface qualification. Hydrostatic treatment is tested separately;
no well-balanced reconstruction was implemented. Final physical/cost outcomes are
in RESULTS/SCALING; short pilots are not promoted to qualification.

## Final measured qualification

Family results are in THREE_DIMENSIONAL_NUMERICAL_FIDELITY.md. Wave error improvement ratios n8/12/16: 1.79896, 2.42516, 2.89865. Coarsest wave still fails .15. Time-only does not remove dominant spatial dissipation. Pure diffusion gets no benefit from reconstruction of advective/acoustic states.

| Control | Scheme | Error | Mean ms |
|---|---|---|---|
| Diffusion3 | reference | 0.000322142 | 1.54489 |
| Diffusion3 | space | 0.000322142 | 6.28048 |
| Diffusion3 | time | 0.000326223 | 3.47397 |
| Diffusion3 | combined | 0.000326223 | 12.9411 |
| Diagonal | reference | 0.00119822 | 1.55287 |
| Diagonal | space | 0.000371915 | 6.26473 |
| Diagonal | time | 0.00119823 | 3.4748 |
| Diagonal | combined | 0.000371764 | 12.8967 |
| Shear3 | reference | 0.146853 | 1.52975 |
| Shear3 | space | 0.0413196 | 6.27407 |
| Shear3 | time | 0.146635 | 3.4567 |
| Shear3 | combined | 0.0412659 | 12.8737 |
| Oblique | reference | 0.261298 | 1.53449 |
| Oblique | space | 0.101355 | 6.22662 |
| Oblique | time | 0.267252 | 3.44935 |
| Oblique | combined | 0.107745 | 12.8129 |
| Vortex | reference | 0.0997363 | 1.59825 |
| Vortex | space | 0.0341582 | 6.40677 |
| Vortex | time | 0.0995888 | 3.64391 |
| Vortex | combined | 0.0340596 | 13.1502 |

Steep-scalar extrema/conservation checks: [{"case": "steep-16-reference", "status": "COMPLETED", "min": 5.73346934406551e-21, "max": 9.998864201885935e-05, "scalar_error": 0.013302546278511306, "registered_extrema_pass": true}, {"case": "steep-16-space", "status": "COMPLETED", "min": 3.4184694680921946e-24, "max": 9.999836441120622e-05, "scalar_error": 0.007184215964865078, "registered_extrema_pass": true}, {"case": "steep-16-combined", "status": "COMPLETED", "min": 3.6509551924277335e-24, "max": 9.999835489365415e-05, "scalar_error": 0.007191364318983003, "registered_extrema_pass": true}]. No positivity clipping. No formal second-order/TVD proof on arbitrary AMR.
