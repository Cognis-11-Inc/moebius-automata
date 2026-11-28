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
