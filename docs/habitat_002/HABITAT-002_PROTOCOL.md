# HABITAT-002 containment and preregistration

2026-11-27. Entry d7e14530325652be172564f3d00bb8bff1358608, main/remote equal,
clean. Numerical Rust/Cargo sources equal clean274-test source8498c22. Both001
archives rehashed including530 members; original reference timings reconciled.

Risk: optimizing storage may drop exact nonzero tails, alter face accumulation,
miss diagonal dependencies, mutate committed state on error or misrepresent a
carrier-only gain as environmental acceleration. No thresholds, clipping, equation
changes, GPU fast math, canonical adoption, historical migration or foreign code.
Chamber remains environmental owner; reusable Workspace is disposable scratch.
Original step and dense/sparse-cell representations remain laneA references.

Acceptance registered before candidate measurements: exact lanes B/C/D must match
all decoded population/species/body/impulse/tick bits after each step; storage tags
and nonphysical workspace/timing excluded. Same lane replay and source-bound
checkpoint continuation required. Reuse001 inventory/impulse/low-Mach/CFL guards.
Failed-step committed digest must remain unchanged. Provisional16³ demo <=2ms
mean/4ms P99 retained. Improvement: >=10percent steady mean reduction OR >=50percent
requested-byte reduction on the same workload; report both and cold costs. Large
extent scaling: fixed h=.05m, N8/16/24/32, identical compact activity and duration;
report actual costs, no production acceptance from this finite range. Occupancy
0/1/10/50/100percent, edge4/8/16/32; compare equal inputs including nonzero tails.
Report mean/median/P95/P99/max, first update separately, second-half steady state,
requested/live/peak bytes, RSS, active support and block fragmentation. No fit of
an enormous-domain law to four small chambers.

GPU laneE starts FP64 with contraction disabled and no fast math. CPU/GPU exactness
is measured, not assumed. Tolerance lane preregistration: finite nonnegative
populations, relative carrier mass drift<=1e-10, population max absolute error
<=1e-10kg/m³ and sampled velocity max error<=1e-10m/s over120 carrier substeps;
same-GPU repeat exact. No GPU full-tick adoption without full Chamber integration,
ordered wall accounting, rollback, source/backend checkpoint and Witness proof.
Report kernel/device, enqueue/sync/transfers/checkpoint costs and memory. Resource
or API failure is a result, not a substitute success. CPU remains reference.

P1 lanesF/G: investigate in-place orientation/rollback and conservative coarse-fine
interfaces; no unqualified implementation becomes selectable Chamber backend.
If physical equations/interface consequences cannot qualify, mark NOT IMPLEMENTED
or REJECTED with counterexample rather than fabricate an acceleration.

# Functional archaeology and reuse ruling

| Concern | Existing source/symbol | Status/input -> output/consumer | Authority/tests | Reuse/missing seam |
|---|---|---|---|---|
| Numerical owner | primordial/mod.rs Chamber::step, State::validate | active experiment; state -> next state / harness and Witness | sole Chamber commit;11 tests | extend explicit selected stepping; preserve reference |
| Carrier | carrier_step, equilibrium, moments | dense AoS f64 D3Q19 -> collision/push/bounceback | ordered wall ledger | reusable buffer/address table; sparse decode must preserve all bits |
| Scalars | transport::step, Mass::{Dense,Sparse} | active; kg + velocity/slip -> kg/wall | ordered faces/CFL/inventory | reuse flux equation; measure discovery/tree/storage costs before replacement |
| Transaction | next=self.state.clone, validate, self.state=next | active staged commit | atomic failure test | reusable tentative state, swap only after validation |
| Persistence | Chamber::{save,load,digest}, write_atomic | source-bound bit checkpoint -> validated Chamber | source/data guards | retain; workspace not serialized; backend recorded by harness identity |
| Witness | morph-lab habitat-witness Authority,advance,draw,record | Chamber -> immutable samples | no view activation | extend selected CPU lane and block observation, no Bevy solver |
| GPU | Bevy/wgpu rendering dependencies; no compute carrier | renderer only | not environmental authority | external OpenCL experiment using installed runtime; no permanent new Rust stack |
| Global coordinates/material/history | Position, MaterialBook, Store | canonical unrelated types | unchanged existing tests | no duplicate owners or adapters |

Inspected primordial mod/transport/tests/experiments, example, native producer and
consumer, Cargo/workspace/lock,001 contract/architecture/reuse/results/scaling and
receipts, MORPH005 architecture, AGENTS/HANDOFF and recent history. Searched
chunk/sparse/GPU/neighbor counterparts. No other atmospheric compute backend found.
Unmerged remote prototype branches and proprietary game internals not audited.
No contested canonical path is changed. Block storage is an alternative internal
Mass representation, not a second atmospheric or material authority.

P1 G scalar-only interface probe, registered before execution: 1D finite-volume
advection/diffusion, left h=.025m/right h=.0125m, shared face flux, u=.05m/s,
D=.0005m²/s, dt=.016666667s, Gaussian centre .4m/sigma .025m,240steps. Compare to
uniform80-cell reference over equal duration; mass drift<=1e-12kg per unit area;
report L1 and first moment error without claiming a carrier/pressure-wave test.
Restriction sums fine masses; piecewise-constant prolongation divides by child
volume. Both maps preserve finite scalar inventory; no LBM populations remapped.
This probe cannot qualify atmospheric multiresolution without nonequilibrium,
acoustic reflection, viscosity and subcycling validation. No post-hoc L1 threshold.
