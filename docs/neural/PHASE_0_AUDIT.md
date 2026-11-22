# Mandatory repository audit — 2026-11-21

Baseline inspected: `32a6b030a521e2faa7d7848e5e6fc0beeb140806`. This audit
precedes controller implementation. The new scientific-research notification
rule is in `AGENTS.md`. Source references below describe this baseline.

| Capability | Status | Exact authority and finding |
|---|---|---|
| Authoritative state and atomic mutation | IMPLEMENTED | `crates/moebius-core/src/world.rs:179`, `World`; `World::tick` at line 377 clones, advances, audits, then commits. `advance` at line 514 owns tick order. |
| Simulation timestep ownership | IMPLEMENTED | `crates/moebius-viewer/src/live.rs:296`, `pump`, owns a World on the worker thread; `wait` at line 335 paces commits. Headless `moebius-cli::run_batch` calls the same World tick. Render FPS is not simulation frequency. |
| Determinism and accounting | IMPLEMENTED | `World::audit` at `world.rs:855`, integer stocks and BLAKE3-domain-separated draws in `prng.rs::domain_seed`. Material includes field, flesh, reservoir, waste and exports. Action-energy is bookkeeping, not physical kinetic energy. |
| Controller | PARTIAL | `farm.rs:67`, `BrainRig::choose`, computes 32 linear scores, chooses one discrete action, and explores with a seeded draw. There is no recurrent activation state and no simultaneous continuous actuator output. |
| Joint articulation | PARTIAL | `farm.rs:19`, `BrainRig` stores 14 bend/jaw values and 14 twists. `actuate` at line 134 selects one of two authored six-joint patterns (lines 145–149), or a paired jaw target. Joint angular velocities, independent torque integration and body angular velocity are absent. |
| Propulsion and reaction forces | PARTIAL | `farm.rs:166–174` converts the selected world-axis heading and summed joint displacement directly into velocity. The stated implicit medium has no measured segment resistance or opposite momentum ledger. This is an authored thrust proxy, not defensible evidence of discovered swimming. |
| Body orientation and interpolation | PARTIAL | `show.rs:1064`, `hall_bodies`, aims the rendered body toward velocity or a dig heading. `rig_joints` at line 1126 adds an X rotation proportional to bend. Those display choices are not authoritative rotational mechanics. Skinning is real presentation, not full body physics. |
| Proprioception | PARTIAL | `BrainRig::choose` sees one section's remaining stroke and velocity projections. Full joint positions/rates, actuator loads, body acceleration, and jaw resistance are absent from its input. |
| Local environmental sensing | PARTIAL | `choose` samples current and face-neighbor nutrient, occupancy and flesh. It has no global navigator, but inputs are tied to authored action headings rather than a versioned body-local sensory mapping. |
| Feeding | IMPLEMENTED | `world.rs:604–638` consumes nutrient at the cell site, credits energy and creates waste one-for-one. A distinct physical mouth reach and fluid nutrient transport are not modeled. |
| Secretion, fusion, excavation | IMPLEMENTED / PARTIAL | `world.rs:640–687` charges secretion and closing-bite excavation; `farm.rs:273`, `fused_surface`, defines face-connected tissue. Contact is endpoint occupancy/rebound, not complete articulated collision mechanics. No adhesion, moving walls, fatigue or damage model exists. |
| Lifetime plasticity | IMPLEMENTED / PARTIAL | `farm.rs:199`, `learn`, changes bounded policy weights with physiological feedback and eligibility traces. These are adaptations of discrete motor choices; learned coordination or useful tunnelling has not been demonstrated. No neural compute fee exists. |
| Reproduction | IMPLEMENTED | `world.rs:759`, `reproduce`, checks age/energy/cooldown/cap/site and pays/splits energy. Cells have no accounted body biomass. |
| Genotype versus phenotype | ABSENT | `farm.rs:220`, `child`, clones the parent's current learned weights and mutates one. This is Lamarckian inheritance; no immutable inherited controller genome is separated from lifetime changes. The eight display genome bytes do not generate the rig. |
| Persistence | IMPLEMENTED | `snapshot.rs:124`, `canonical_bytes`, hashes FARM2 brain and flesh data. `World::restore` at line 307 reconstructs/audits it. FARM1 is explicitly refused. No new data must silently reinterpret existing FARM2 recordings. |
| Inspection and logs | PARTIAL | `live.rs:350`, `shoot`, copies brain/rig state into immutable pictures. The HUD displays stocks, lineage summary and hashes; it does not expose activations, motor commands, sensors or realized actuator responses. |
| Scientific tests | IMPLEMENTED / PARTIAL | `tests/farm.rs` covers paid work, deterministic continuation, births, distinct sections, fusion and commanded excavation; `tests/invariants.rs` preserves M0 invariants. These are capability tests. Matched plasticity controls, random baselines and causal learning/evolution experiments are absent. |
| Neural/anatomical co-evolution | PLANNED | `docs/FARM_M1.md`, Genetics direction, records a future anatomy-generating genome. Current rig topology is fixed. No topology evolution should be claimed in this milestone. |

## First defensible slice

Preserve M0 and explicit legacy FARM2 behaviour. Introduce a separately identified
recurrent-controller mode, with versioned genotype, lifetime state and body-local
sensor/actuator descriptors. Use bounded deterministic fixed-step integration
and independently commanded axes. Remove authored gait selection from that mode.

The propulsion prerequisite must be handled explicitly. Internal joint motion
cannot accelerate the centre of mass in a vacuum. The smallest supported
propulsion extension is paid secretion as reaction mass, with material and
opposite momentum accounted through flight and deposition. Articulation-powered
swimming remains a separate physical-model question; it must not be recreated
by sending neural outputs directly to a velocity vector.

Joint torque/rate/contact dynamics, sensing and lifetime plasticity need their
own tests before any claim of discovery. Reproduction must inherit germline
controller parameters and reset activations/plastic changes by default. Tests
and experiments must distinguish motor activity, realized mechanics, locomotion,
experience-dependent changes, learning, and evolution.

## External references already disclosed to the user

- Beer, R. D. (1995), [On the Dynamics of Small Continuous-Time Recurrent Neural Networks](https://journals.sagepub.com/doi/pdf/10.1177/105971239500300405): motivates investigating persistent nonlinear neural dynamics. Our integration/nonlinearity choices require their own specification and tests.
- Pak, O. S. and Lauga, E. (2015; 2014 author draft), [Theoretical Models of Low-Reynolds-Number Locomotion](https://www.damtp.cam.ac.uk/user/lauga/papers/96.pdf): provides context for resistance-mediated swimming. No resistive-force solver or swimming result is imported or validated by citing it.

Neither source establishes that organisms in this repository learn locomotion.
