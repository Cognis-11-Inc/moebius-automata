# Numerical precision and replay boundaries

A/B/C/D/F CPU lanes retain f64, original equations and ordered face/wall updates.
Deterministic workers parallelize independent collision only. Compare all decoded
population/scalar bits and body/impulse/tick state; representation tags and derived
scratch/timing are excluded. `physical_digest` normalizes only scalar backend tag;
checkpoint data digest still includes its actual Config and represented inventories.

HABITAT002 CPU checkpoint adapter reuses existing atomic write, bit encoding and
source identity/digest checks, adds explicit CPU/f64/lane/worker identity and a
separate schema. `load_candidate` requires matching identity. Ordinary001 load
cannot silently read002 execution records. No historic checkpoint conversion or
new history authority. Rebuild fresh scratch after restoration. State remains
canonical decoded orientation even for F's cycle permutation; interrupted tentative
permutations never become visible. Snapshot between half-streams is forbidden.

E has its own detached carrier-fixture envelope identifying CPU fixture source,
GPU device/driver/FP64/kernel/runner hashes/config and payload hash. Source/backend
changes are refused by the actual loader. Same-device final-repeat and restored
continuation checks are distinct from cross-backend bit comparison. Local tolerance
limits were preregistered; a tolerance PASS is not automatic exact equivalence or
cross-driver portability. GPU scalar/body/full-environment adoption is NOT RUN.

F is own collision-then-permutation-cycle implementation, not a copy of restricted
FluidX3D or published appendix code. A naive forward overwrite counterexample is
retained as a rejected algorithm. Unlike Esoteric schemes, decoded orientation is
fixed after every admitted substep; visited flags and a single carry value replace
one streaming array. Committed plus tentative state still exist for rollback.
