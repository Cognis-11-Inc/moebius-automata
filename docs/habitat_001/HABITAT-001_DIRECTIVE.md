# MŒBIUS AUTOMATA
## HABITAT-001 — The Primordial Medium
### Environmental Physics Foundation

## 0. Strategic redirection

Continue from published checkpoint `966948a`.

Suspend conventional MORPH-005 performance stabilization temporarily.

MORPH-005 remains PARTIAL. Its existing acceptance gates, evidence, canonical implementation and unresolved limitations remain preserved.

MORPH-006 remains suspended.

The Operator has identified a foundational architectural omission: MŒBIUS AUTOMATA's organisms are intended to inhabit a weak-gravity environment containing an extraordinarily dense atmosphere rich in gases, chemical compounds and suspended particulates.

The current collision-enabled habitat instead requires a suspended, force-free environmental configuration.

Before further developing locomotion, articulated appendages, perception or biological intelligence, establish the physical substrate in which these capabilities will eventually operate.

Create a separate, explicitly experimental development track named HABITAT.

This directive authorizes the construction of HABITAT-001. It does not authorize replacement of the canonical ship runtime.

## 1. Environmental vision

The habitat is an enormous, ancient, sealed tubular vessel travelling through space.

Over aeons, its atmosphere has accumulated an extraordinary variety of gases, mineral particulates and chemical compounds.

Its material composition is heterogeneous.

Its atmosphere possesses inertia, viscosity, temperature, local movement and spatially distributed resources.

Weak gravity produces slow settling and directional environmental gradients.

External radiation and internal thermal sources can eventually supply energy.

The unusual hull's proposed ability to absorb external material during its travels remains a fictional environmental mechanism requiring a future explicit model.

Do not implement unexplained matter creation or passive pressure-defying intake.

For HABITAT-001, begin with a finite initialized inventory inside a sealed experimental chamber.

The environment must exist independently of biological organisms.

## 2. Mandatory repository archaeology

Inspect and reconcile:

- AGENTS.md.
- HANDOFF.md.
- PERFORMANCE_STABILIZATION_VII.md.
- FLAGELLAR_AUTHORITY_AUDIT.md.
- PHYSICAL_GEOMETRY_AUTHORITY_V.md.
- SENSORY_ARCHITECTURE.md.
- Existing MORPH-003 environmental and material contracts.
- Existing spatial locality and coordinate infrastructure.
- Existing nursery physical models.
- Existing environmental abstractions and material transport.

Specifically investigate:

- `crates/moebius-morph/src/emission.rs`
- `Environment`
- `MaterialBook`
- Parcel acceleration and drag integration.
- `crates/moebius-morph/src/habitat.rs`
- Habitat validation and force-free collision restrictions.
- The separate moebius-core nursery `Body::step` implementation.
- Existing spatial coordinates, cells and indices.
- Existing deterministic numerical integration and replay facilities.

Locate exact symbols, producers, consumers, tests and authority boundaries.

Determine which existing systems are genuinely reusable.

Do not create a duplicate material ledger, coordinate system, replay architecture or environmental authority without demonstrating why the existing implementation cannot lawfully serve the required purpose.

Do not simply remove the existing force-free habitat validation guard.

That guard protects a known physical contract.

## 3. Model hierarchy

Distinguish four future environmental layers.

### Layer A — Bulk medium

A numerical representation of the atmosphere's macroscopic properties:

- Density.
- Velocity.
- Temperature.
- Pressure, where supported by the selected model.
- Dynamic or effective viscosity.

### Layer B — Suspended matter

Spatially distributed concentrations of material species.

Represent bulk particulate populations statistically where appropriate.

Individual microscopic particles must not require independent simulation entities.

### Layer C — Explicit physical matter

Objects whose individual position, geometry or interactions matter:

- Large particles.
- Deposited material.
- Structural fragments.
- Future organisms.
- Future biological emissions.

Reuse existing material authority where applicable.

### Layer D — Energy and chemistry

Future thermal transfer, radiation, chemical reactions, nutrient exchange and energetic gradients.

Reserve explicit interfaces for these mechanisms.

Do not implement speculative chemistry merely to populate an environmental registry.

## 4. Initial environmental model

Create the smallest experimentally useful three-dimensional physical chamber.

Begin with:

- A finite sealed volume.
- Configurable weak gravity.
- Configurable medium density.
- Configurable viscosity.
- A bounded spatial velocity representation.
- Multiple distinguishable passive material concentrations.
- Explicit inert test bodies and particles.
- Controlled initial conditions.

Density and viscosity must remain independent parameters.

Do not infer one directly from the other.

Select a numerical model appropriate to a dense gaseous or aerosol-rich medium.

Determine whether a reduced-order continuum approximation, an incompressible approximation, a compressible model or another bounded method is justified for the initial experiment.

Document the validity conditions and physical effects omitted by the selected approximation.

Do not claim complete computational fluid dynamics merely because a velocity field exists.

## 5. Units and physical contract

Establish a coherent units convention before implementation.

Explicitly identify:

- Length.
- Time.
- Mass.
- Velocity.
- Acceleration.
- Force.
- Density.
- Dynamic viscosity.
- Pressure.
- Temperature.
- Material concentration.
- Momentum.
- Energy.

Reconcile these units with the existing simulation coordinate and tick conventions.

If a dimensionless model is selected, document its reference scales and conversion semantics.

Environmental parameters must not be arbitrary multipliers with unexplained physical interpretations.

Define:

- Fixed timestep.
- Numerical integration scheme.
- Spatial discretization.
- Boundary conditions.
- Stability limits.
- Conservation expectations.
- Numerical dissipation.
- Error tolerances.

Do not assign physical values to the fictional atmosphere without identifying them as experimental design assumptions.

## 6. Gravity

Implement configurable uniform weak gravity in the experimental chamber.

Support:

- Zero gravity.
- Multiple nonzero weak-gravity configurations.
- Configurable gravity direction.

Do not assume the mechanism producing gravity inside the fictional vessel.

For this phase, gravity is an explicit environmental acceleration field.

Validate:

- Vacuum free fall.
- Settling under drag.
- Directional consistency.
- Zero-gravity behaviour.
- Timestep convergence.

Future gravity variation is outside the initial scope.

## 7. Passive body–medium interaction

Before introducing actuated organs, implement a simple inert test body interacting with the medium.

Investigate:

- Relative velocity between body and local medium.
- Drag.
- Buoyancy.
- Gravity.
- Inertial response.
- Momentum transfer.

Distinguish physically prescribed environmental forces from numerical constraints.

Where the selected approximation permits, momentum transferred to a body must have a corresponding opposite contribution to the medium.

If the initial prototype uses a prescribed infinite-reservoir background, identify that approximation explicitly. It must not be represented as a closed, momentum-conserving atmosphere.

Test bodies must not move through prescribed displacement masquerading as physical force.

## 8. Local flow and environmental disturbance

Investigate a bounded, locally sampled velocity field.

The desired future environment permits organisms and objects to disturb surrounding flow and subsequently interact with those disturbances.

For HABITAT-001, use controlled nonbiological perturbations.

Test:

- Initially stationary medium.
- Uniform flow.
- Local velocity perturbation.
- Transport of a passive concentration.
- Viscous dissipation.
- Interaction between two perturbations.
- Momentum exchange with an inert body.
- Boundary interactions.

If pressure projection, diffusion or transport requires communication beyond the local neighbourhood, document that dependency.

Do not artificially truncate physically relevant influence at the Witness boundary.

The Witness remains an observer.

## 9. Particulate atmosphere

The habitat must eventually contain vast quantities of suspended material without requiring billions of simulated particle entities.

Investigate a hybrid representation combining:

- Bulk atmospheric fields.
- Material concentration fields.
- Statistical particulate populations.
- A bounded number of explicit test particles.

For particulate species, investigate physically relevant descriptors such as:

- Material identity.
- Particle size class.
- Material density.
- Concentration.
- Coupling to the carrier medium.
- Settling behaviour.
- Diffusion or dispersion.

Only implement properties that have actual consumers and validation.

Do not introduce decorative schema fields and claim an implemented environmental system.

Demonstrate that particulate species with differing physical properties can exhibit different transport and settling behaviour.

Preserve finite material inventories.

## 10. Thermal and chemical reservations

Reserve a lawful path for future:

- Internal heat sources.
- Radiation entering through portholes.
- Temperature gradients.
- Convection.
- Chemical reactions.
- Resource concentration gradients.
- Biological nutrient uptake.
- Material exchange with the hull.

HABITAT-001 may include simple passive temperature or concentration transport if justified by the chosen model.

Do not claim thermal convection unless the necessary temperature-dependent force or density coupling is implemented and validated.

Do not introduce metabolism, chemical reaction networks or biological intelligence during this phase.

## 11. Computational architecture

The long-term habitat is enormous.

The computational cost of empty or undisturbed regions must not scale unnecessarily with the total theoretical ship volume.

Investigate:

- Sparse spatial fields.
- Uniform background states.
- Locally active regions.
- Bounded field sampling.
- Shared environmental state.
- Conservative regional activation.
- Explicit boundary treatment.
- Controlled spatial resolution.

A finite dense reference chamber is acceptable for scientific validation.

However, do not mistake a successful small dense-grid experiment for a scalable production architecture.

Any sparse representation must be compared against an appropriate reference.

Distinguish physical approximation error from computational optimization.

Do not freeze environmental evolution merely because no player is observing it.

## 12. Conservation and numerical validation

Define the conservation properties appropriate to the selected physical model before running the final experiments.

At minimum, investigate:

- Material inventory conservation.
- Fluid mass conservation where represented.
- Particulate material conservation.
- Momentum exchange.
- External gravity impulse.
- Boundary impulse.
- Mechanical energy dissipation.
- Numerical stability.

Account explicitly for external forces, constrained boundaries and dissipative processes.

Do not claim fluid-only momentum conservation where fixed chamber walls exchange momentum with the medium.

Do not claim total energy conservation unless the model accounts for dissipated energy and every relevant reservoir.

Reject unexplained material creation or destruction.

## 13. Mandatory experimental suite

### E01 — Empty equilibrium

A sealed uniform medium without organisms, forces or disturbances.

Verify that it remains appropriately stable.

### E02 — Vacuum control

An inert body in a zero-density, zero-viscosity environment.

Verify the selected vacuum dynamics and absence of fluid-generated forces.

### E03 — Weak-gravity control

Validate acceleration and displacement against the applicable analytical reference.

### E04 — Drag

Compare inert-body deceleration with the selected drag law and its analytical or numerical reference.

### E05 — Buoyancy

Compare bodies of differing densities in the same medium.

Where applicable, validate neutral buoyancy and opposing rise or settling behaviour.

### E06 — Particulate settling

Compare multiple particulate species under matched environmental conditions.

Verify material conservation and model-appropriate settling behaviour.

### E07 — Advection and diffusion

Transport a localized passive concentration through a known flow.

Measure numerical spreading, conservation and spatial error.

### E08 — Local disturbance

Introduce a controlled medium perturbation.

Measure its propagation and dissipation against the selected physical model.

### E09 — Two-way momentum coupling

Exchange momentum between a test body and the medium.

Account for both participants and any external impulses.

### E10 — Numerical convergence

Repeat appropriate experiments across multiple spatial resolutions and timesteps.

Do not certify results that depend on an unconverged numerical instability.

### E11 — Deterministic replay

Repeat identical experiments with identical initial state and configuration.

Require exact replay within the explicitly defined execution contract.

### E12 — Computational scaling

Vary active-region count, field resolution, particulate representation and explicit object count independently.

Measure actual computational and memory costs.

Do not derive scaling laws from unrelated physical configurations.

## 14. Future flagellar interface

Do not implement canonical flagellar propulsion during HABITAT-001.

Reserve an explicit interface through which a future articulated appendage can:

- Sample local medium properties.
- Displace or disturb the medium.
- Experience contact and hydrodynamic resistance.
- Exchange momentum.
- Consume actuation energy.
- Produce forces and torques on its parent body.

Investigate reuse of the earlier nursery physical model before designing a new actuation architecture.

The next experimental phase will test whether locomotion emerges from a controlled appendage interacting with the medium.

No prescribed forward displacement or hidden thrust term may be used to manufacture successful propulsion.

## 15. Environmental Witness

Create or adapt an experimental visualization using the existing Rust/Bevy infrastructure.

The visualization should expose:

- Chamber boundaries.
- Gravity direction and magnitude.
- Density or concentration distribution.
- Local flow vectors.
- Particulate transport.
- Explicit test bodies.
- Disturbance propagation.
- Relevant conservation residuals.
- Simulation and rendering timing.

Provide controls for experimental parameters and repeatable initial conditions.

The environment should be observable independently of any creature.

Rendering particles may sample the authoritative environmental fields, but they must remain distinguishable from explicit physical particles.

The visualization must not become a second environmental simulator.

Maintain headless execution and deterministic replay.

## 16. Performance

Measure:

- Environmental update time.
- Body–medium coupling time.
- Field sampling cost.
- Transport and diffusion cost.
- Requested allocations.
- Resident memory.
- Spatial resolution sensitivity.
- Active-region scaling.
- Rendering cost.

Establish a dedicated environmental performance budget.

Do not consume the entire future authoritative tick budget before organisms have been introduced.

Report actual measured costs before proposing a production resolution or update cadence.

The environmental model must eventually coexist with collision, material, biological and cognitive workloads.

## 17. Integration boundary

HABITAT-001 remains isolated from canonical ship admission.

Do not:

- Remove the existing suspended-medium restriction.
- Replace canonical movement rules.
- Change existing collision authority.
- Introduce flagellar thrust into the current colony runtime.
- Rewrite historical snapshots.
- Modify paid structural material semantics.
- Declare MORPH-005 complete.
- Commence MORPH-006.

Future integration requires an explicit versioned physical contract and separate Operator approval.

## 18. Deliverables

Produce:

- HABITAT-001_DIRECTIVE.md.
- HABITAT_ENVIRONMENT_ARCHITECTURE.md.
- ENVIRONMENTAL_PHYSICS_CONTRACT.md.
- ENVIRONMENTAL_REUSE_AUDIT.md.
- Implemented isolated environmental prototype.
- Deterministic headless experimental harness.
- Experimental environmental Witness.
- Units and numerical-model specification.
- Conservation and convergence results.
- Computational scaling results.
- Accepted and rejected model evidence.
- HABITAT-001_RESULTS.md.
- Updated HANDOFF and Atlas.
- Clean-source validation receipts.
- Immutable restore-verified Mermogon evidence archive.

Classify every proposed capability as implemented, validated, experimental, reserved, rejected or unresolved.

Commit and push the validated experimental checkpoint without modifying canonical physical authority.

## 19. Governing principle

The environment is not a background.

It is a physical participant in every organism's existence.

It must possess its own state, history, material distribution and dynamics.

The organisms will eventually inhabit, disturb, consume, transform and exploit this substrate.

Their intelligence must discover the opportunities provided by its laws rather than receive those opportunities through scripted behaviour.

First establish the world.

Then allow the creatures to inhabit it.

Proceed autonomously within these authority boundaries.