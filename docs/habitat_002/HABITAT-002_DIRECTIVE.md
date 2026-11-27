# MŒBIUS AUTOMATA
## HABITAT-002 — The Scalable Atmosphere
### Sparse Dense-Block Architecture, Conservative Activation and GPU Computational Physics

## 0. Mission

Continue from published HABITAT-001 checkpoint `d7e1453`.

HABITAT-001 established a bounded experimental three-dimensional physical medium with deterministic carrier dynamics, material transport, gravity, particulate settling, passive body coupling and an independent Bevy Witness.

Its principal limitations are now computational scalability and unresolved body hydrodynamics.

HABITAT-002 shall address environmental computational architecture before introducing additional physical or biological complexity.

The objective is to establish an economical atmospheric substrate capable of eventually supporting an enormous persistent vessel containing heterogeneous matter, active disturbances, structural growth and large populations of artificial organisms.

**The atmosphere must evolve independently of the Witness and the presence of organisms.**

MORPH-005 remains PARTIAL.

MORPH-006 remains suspended.

Canonical ship physics must remain unchanged.

## 1. Entry verification

Read and reconcile:

- AGENTS.md.
- HANDOFF.md.
- HABITAT-001_RESULTS.md.
- HABITAT-001_RESULTS.json.
- HABITAT-001_SCALING.json.
- HABITAT_ENVIRONMENT_ARCHITECTURE.md.
- ENVIRONMENTAL_PHYSICS_CONTRACT.md.
- ENVIRONMENTAL_REUSE_AUDIT.md.
- Relevant validation, provenance and storage receipts.

Verify:

- Local and remote commit identities.
- Working-tree cleanliness.
- Clean-source checkpoint `8498c22894c269e31ad8d53d63116b9b3ecafa5b`.
- The reported 274-test certification.
- Both HABITAT-001 evidence archives on Mermogon.
- Existing environmental owner, transport implementation, numerical kernels and checkpoint contracts.

Inspect the current `primordial` implementation, especially:

- Chamber::step.
- D3Q19 carrier storage and streaming.
- Mass::Dense.
- Mass::Sparse.
- Scalar transport.
- Active-face construction.
- Numerical validation.
- Transactional state ownership.
- Snapshot and replay implementation.
- Bevy Witness integration.

Reuse established infrastructure.

Do not create a second environmental authority.

## 2. Established baseline

The HABITAT-001 reference chamber records:

- Dimensions: 0.8 m cube.
- Resolution: 16³ cells.
- Two statistical species.
- Nine inert spheres.
- 120 ordinary release ticks.
- Mean environmental update: 1.077040 ms.
- Empirical P99: 1.095660 ms.
- Requested allocations: approximately 3.114 MB per tick.

The reference passes the provisional environmental budget of 2 ms mean and 4 ms P99.

The independent resolution experiment records:

| Resolution | Dense mean ms | Sparse scalar mean ms |
|---|---:|---:|
| 8³ | 0.114 | 0.298 |
| 12³ | 0.383 | 0.979 |
| 16³ | 0.909 | 2.092 |
| 24³ | 3.036 | 5.552 |
| 32³ | 7.256 | 11.235 |

These are separate experimental workloads from the reference demo.

The sparse scalar implementation is slower in every measured nonempty resolution control.

The carrier remains dense in both configurations.

Reconcile these inherited measurements against the retained evidence before using them as benchmarks.

## 3. P0 — External donor archaeology

Before implementing new computational architecture, examine the following documented approaches.

### 3.1 Noita

Petri Purho, Exploring the Tech and Design of Noita.

Primary reference:

https://www.gdcvault.com/play/1025695/Exploring-the-Tech-and-Design/

Investigate:

- Spatial chunking.
- Dirty-region processing.
- Multithreaded update scheduling.
- Boundary interactions between chunks.
- Activation of previously inactive material.

Distinguish its two-dimensional falling-sand cellular simulation from our three-dimensional continuum carrier.

Do not assume its update-order or activation rules preserve our numerical model.

### 3.2 Space Station 13

Reference:

https://github.com/tgstation/tgstation/blob/master/code/modules/atmospherics/Atmospherics.md

Investigate:

- Active atmospheric tiles.
- Excited groups.
- Regional atmospheric equalization.
- Deactivation conditions.
- Reactivation following disturbances.
- Gas exchange between neighbouring regions.

Its equilibrium heuristics are architectural inspiration, not automatically valid physical laws for MŒBIUS AUTOMATA.

### 3.3 NVIDIA Flow

References:

https://github.com/NVIDIAGameWorks/Flow

https://github.com/NVIDIA-Omniverse/PhysX

Investigate:

- Adaptive sparse voxel blocks.
- GPU-resident block storage.
- Block activation.
- Indirection and addressing.
- Neighbour communication.
- Memory allocation policies.
- Disturbance-driven spatial refinement.

Identify the exact version and documentation consulted.

Distinguish visual-effects simulation requirements from conservative scientific simulation requirements.

### 3.4 FluidX3D

References:

https://github.com/ProjectPhysX/FluidX3D

https://doi.org/10.3390/computation10060092

Investigate published methods concerning:

- D3Q19 lattice Boltzmann computation.
- In-place streaming.
- Esoteric Pull.
- Memory layout.
- GPU memory bandwidth.
- Population storage.
- Mixed numerical precision.
- Boundary handling.

FluidX3D uses restrictive noncommercial licensing.

Treat its published numerical research as a theoretical reference.

Do not copy, translate, adapt or incorporate its restricted implementation into MŒBIUS AUTOMATA.

### 3.5 Sparse GPU lattice Boltzmann research

Primary reference:

A new GPU implementation for lattice-Boltzmann simulations on sparse geometries.

https://arxiv.org/abs/1611.02445

Investigate:

- Small three-dimensional tiles.
- Tile occupancy.
- Sparse addressing.
- Contiguous population storage.
- GPU memory-access patterns.
- Halo communication.
- Computational efficiency in irregular domains.

The paper reports high memory-bandwidth utilization for sparse geometries.

Determine which findings transfer to dynamically evolving atmospheric activity.

Static sparse geometry is not equivalent to dynamically activated fluid regions.

### 3.6 Additional game references

Investigate documented atmospheric approaches in Oxygen Not Included and Stationeers where sufficiently authoritative technical sources are available.

Do not reconstruct proprietary implementations from gameplay observations.

Do not treat visual behaviour as proof of internal architecture.

### 3.7 Donor provenance and licensing

For each source, record:

- Exact title and authors, where verified.
- URL or DOI.
- Publication date.
- Repository revision, where applicable.
- Accessed material.
- Applicable license and reuse restrictions.
- Numerical assumptions.
- Relevant architectural mechanism.
- Differences from our requirements.
- CONSULTED, PROPOSED, IMPLEMENTED or VALIDATED status.

No external code import is authorized during this phase.

Implement experimental candidates independently from documented numerical principles and our existing source.

## 4. P0 — Explain the sparse-scalar regression

The current transport implementation uses ordered sparse maps and constructs active faces through a BTreeSet.

This provides a plausible explanation for the observed regression, but its actual cost contributions remain unmeasured.

Instrument:

- Active-face discovery.
- Ordered-set insertion.
- Sparse cell lookup.
- Dense cell lookup.
- Neighbour calculation.
- Face-list construction.
- Temporary delta storage.
- Outgoing coefficient storage.
- Material accumulation.
- Entry insertion and removal.
- Allocation and deallocation.
- Carrier velocity extraction.
- Numerical validation.
- Transactional cloning.

Separate exclusive timings from enclosing overlapping timers.

Establish:

- Number of occupied scalar cells.
- Number of active faces.
- Number of newly activated cells.
- Number of removed cells.
- Actual occupancy over time.
- Bytes requested.
- Bytes retained.
- Cache-related metrics where supported.
- Cost per transported face.

Investigate how diffusion progressively expands the exact nonzero support of each material field.

Do not remove small concentrations simply to improve performance.

No concentration threshold may silently destroy material.

## 5. P0 — Optimize the dense reference

Before introducing a more elaborate sparse architecture, establish a computationally economical dense baseline.

Investigate:

- Reusable transport buffers.
- Reusable velocity extraction storage.
- Persistent delta and outgoing arrays.
- Elimination of unnecessary temporary collections.
- Precomputed neighbour relationships where economical.
- Contiguous population storage.
- Structure-of-arrays versus array-of-structures layouts.
- Reduced redundant carrier sampling.
- Bounded transactional scratch storage.
- Deterministic multicore processing.

Preserve the original physical equations, numerical precision, face order and admission semantics for exact-acceleration candidates.

Do not weaken rollback, positivity, stability or conservation guards.

Identify whether full-state cloning can be replaced by lawful staged computation and transactional buffer swapping.

Failed steps must leave committed environmental state unchanged.

Measure ordinary release performance separately from allocation-attribution builds.

## 6. P0 — Sparse global addressing with dense local blocks

Implement an isolated chunk-based scalar storage candidate.

The proposed representation consists of:

- Sparse global block addressing.
- Contiguous dense arrays within allocated blocks.
- Reusable local transport storage.
- Explicit neighbouring-block relationships.
- Deterministic block ordering.
- Bounded allocation ownership.

Do not use individual tree-map entries for every populated cell unless measurements justify that representation.

Evaluate multiple block dimensions, including where appropriate:

- 4³ cells.
- 8³ cells.
- 16³ cells.
- 32³ cells.

These are candidate dimensions, not prescribed production settings.

Investigate:

- Block occupancy.
- Internal fragmentation.
- Cross-block face exchange.
- Local memory access.
- Active-face construction.
- Block allocation cost.
- Block retirement.
- Spatial coherence.
- Concentration spreading.

Compare against both the original dense and original cell-level sparse implementations.

For the exact lane, preserve canonical face processing and floating-point accumulation order.

Require exact applicable dense/block state equivalence.

## 7. P0 — Sparse carrier architecture

The current D3Q19 carrier remains dense even when scalar fields are sparse.

HABITAT-002 must investigate this limitation directly.

### 7.1 Fundamental distinction

An unallocated atmospheric region is not a vacuum.

A region containing uniform atmospheric density and zero velocity still possesses:

- Mass.
- Pressure.
- Inertia.
- Material inventory.
- The capacity to receive disturbances.

A sparse representation must preserve the physical meaning of this background.

### 7.2 Implicit equilibrium background

Investigate whether an exactly known equilibrium state can be represented implicitly.

Potential candidate:

- Uniform analytical equilibrium background.
- Explicit D3Q19 populations in active blocks.
- Deterministic activation from background state.
- Explicit communication between active and background regions.

Determine the exact conditions under which implicit representation preserves the selected equations.

A region with no organism is not necessarily inactive.

A region containing a propagating disturbance cannot be discarded because the disturbance is visually insignificant.

### 7.3 Activation

Account for:

- Carrier streaming dependencies.
- Diagonal D3Q19 communication.
- Scalar advection.
- Diffusion.
- Body momentum exchange.
- Material sources.
- Wall boundary effects.
- Existing non-equilibrium populations.
- Future disturbances approaching block boundaries.

Derive the necessary halo and activation rules from the numerical stencil.

Do not select halo thickness arbitrarily.

### 7.4 Deactivation

Investigate exact deactivation only when the complete represented state is equivalent to its implicit replacement.

Checking velocity alone is insufficient.

Account for:

- Density.
- All relevant carrier populations.
- Non-equilibrium information.
- Scalar concentrations.
- Material inventories.
- Momentum.
- Pending boundary exchange.

If exact deactivation is impractical, report that result.

Approximate deactivation belongs to a separate research lane with an explicit error budget.

No unexplained disappearance of matter or momentum is permitted.

## 8. P1 — Conservative multiresolution atmosphere

Investigate a hierarchical atmospheric architecture.

Potential model:

- Coarse background atmosphere.
- Intermediate regional disturbances.
- Fine-resolution blocks near physically significant events.

Physical activity, not the Witness, determines computational resolution.

Identify lawful mechanisms for:

- Restriction from fine to coarse state.
- Prolongation from coarse to fine state.
- Cross-resolution mass exchange.
- Momentum exchange.
- Scalar flux conservation.
- Interface communication.
- Compatible temporal subcycling.
- Deterministic refinement and coarsening.

Changing lattice resolution may change numerical viscosity, relaxation parameters, timestep requirements and physical error.

Do not directly interpolate carrier populations across resolutions without establishing the numerical consequences.

Begin with the smallest controlled experiment crossing a single resolution interface.

Compare against a uniformly fine reference.

Measure:

- Material and momentum residuals.
- Wave transmission and reflection.
- Diffusion error.
- Disturbance attenuation.
- Interface artefacts.
- Computational savings.

This is an exploratory approximation lane.

It must not replace the established reference merely because it is faster.

## 9. P0 — GPU feasibility

The Forge reportedly provides an NVIDIA RTX 5060 Ti with 16 GB VRAM.

Verify the actual hardware, driver, available compute APIs, memory capacity and device capabilities.

Investigate GPU execution of the existing D3Q19 carrier.

### 9.1 Backend selection

Evaluate available and suitable compute mechanisms, including where supported:

- Vulkan compute.
- wgpu compute.
- OpenCL.
- CUDA.

Inspect existing Rust/Bevy dependencies before selecting a backend.

Do not introduce multiple permanent compute stacks without justification.

Select the smallest independently testable backend capable of exercising the existing numerical model.

### 9.2 Initial GPU prototype

Begin with the dense D3Q19 reference.

Implement and measure:

- Population update.
- Collision/relaxation.
- Streaming.
- Boundary handling.
- Carrier-state sampling.
- Numerical validation.

Do not immediately migrate the entire environmental model to the GPU.

First establish the actual computational benefit of the carrier kernel.

### 9.3 Memory residency

Investigate persistent device-resident populations and reusable buffers.

Avoid transferring the complete carrier state between CPU and GPU every tick.

Measure:

- Kernel execution.
- Dispatch overhead.
- Synchronization.
- Host-to-device transfer.
- Device-to-host transfer.
- Checkpoint transfer.
- Rendering contention.
- VRAM usage.
- Total end-to-end authoritative update.

A fast kernel surrounded by expensive synchronization is not a qualified environmental acceleration.

### 9.4 Precision and determinism

Maintain the existing CPU implementation as the numerical reference.

Begin with matching numerical precision where supported.

Do not assume GPU and CPU arithmetic will be bit-identical.

Distinguish:

- Exact same-backend replay.
- Exact cross-backend equivalence.
- Tolerance-based physical equivalence.
- Changed numerical precision.
- Changed discretization.

Do not silently enable fast-math or reduced precision.

Any mixed-precision candidate requires separate conservation, stability, error and convergence experiments.

Checkpoint identities must include backend and numerical configuration.

No source-bound checkpoint protection may be bypassed.

### 9.5 GPU qualification

Compare actual wall-clock performance across multiple grid sizes.

Include small workloads where GPU dispatch overhead may dominate.

Measure kernel throughput in lattice updates per second and complete environmental tick duration.

Do not claim GPU superiority from a carrier-only kernel benchmark.

GPU adoption requires separate runtime, replay and rendering qualification.

## 10. P1 — In-place population streaming

Investigate independently implemented in-place streaming methods derived from published numerical research.

Determine whether eliminating redundant population buffers can reduce:

- Resident memory.
- Memory bandwidth.
- Allocation requests.
- Cache traffic.
- GPU storage requirements.

Account for:

- Even/odd timestep representation.
- Population orientation.
- Boundary conditions.
- Checkpoint serialization.
- Interrupted steps.
- Transaction rollback.
- Concurrent reading by observers.

A changed internal storage representation must not silently alter the numerical state represented.

Compare decoded physical states against the reference.

If an exact internal transformation is not possible, classify the candidate under the appropriate numerical-variation lane.

## 11. Independent experimental lanes

Maintain the following comparisons:

- A: Original HABITAT-001 dense reference.
- B: Optimized dense CPU.
- C: Dense-block sparse scalar storage.
- D: Implicit-background sparse carrier.
- E: Dense GPU carrier.
- F: In-place streaming.
- G: Multiresolution or approximate activation research.

Qualify each candidate independently before combining them.

Where practical, use the same source revision, physical inputs and execution harness.

Do not claim causal attribution from a combined configuration alone.

## 12. Controlled scaling experiments

Independently vary:

- Physical chamber dimensions.
- Spatial resolution.
- Number of allocated blocks.
- Occupied fraction.
- Number of disconnected active regions.
- Material species count.
- Diffusion strength.
- Disturbance intensity.
- Disturbance propagation distance.
- Explicit body count.
- Worker count.
- Compute backend.

Test occupancy fractions ranging from highly sparse to fully occupied.

Determine empirical crossover points at which dense storage becomes preferable to sparse storage.

Distinguish two different scaling experiments:

1. Increasing resolution at fixed physical chamber size.
2. Increasing physical chamber size at fixed cell dimensions.

The second is essential to our intended enormous vessel.

Do not relabel coordinate translation as physical domain enlargement.

Compare equivalent physical durations and valid timesteps.

Retain exact and approximate candidate results separately.

## 13. Mandatory physical challenges

### E01 — Equilibrium background

Verify a large uniform atmosphere without disturbances.

### E02 — Single active cloud

Transport and diffuse a localized concentration through an otherwise uniform domain.

### E03 — Multiple disconnected clouds

Test independent active regions before and after interaction.

### E04 — Block boundary crossing

Transport material and carrier disturbances across allocated-block boundaries.

### E05 — Activation propagation

Verify that previously implicit atmospheric regions activate before required numerical updates.

### E06 — Deactivation and reactivation

Where supported, retire an eligible region, reactivate it and compare against the reference.

### E07 — Diffusion saturation

Determine the computational consequences when initially sparse fields become increasingly occupied.

### E08 — Wall interaction

Verify carrier and material exchange near sealed chamber boundaries.

### E09 — Body coupling

Preserve finite-cell momentum exchange with existing inert spheres.

### E10 — Large-domain scaling

Increase physical extent while keeping localized activity and cell dimensions controlled.

### E11 — Numerical equivalence

Compare each exact candidate against decoded reference states and authoritative observations.

### E12 — Approximation fidelity

For numerical variations, establish preregistered tolerances and convergence behaviour.

### E13 — Replay and restoration

Require same-source, same-backend deterministic continuation and verified checkpoint restoration.

### E14 — Failure and rollback

Inject bounded-capacity, allocation, invalid-state and backend failures where supported.

No partial environmental state may become authoritative.

### E15 — Native Witness contention

Measure environmental execution while rendering the experimental chamber.

### E16 — Sustained evolution

Run extended workloads to measure activity growth, memory retention, cumulative numerical error and computational degradation.

Do not classify finite successful execution as unlimited lifetime stability.

## 14. Performance and acceptance

Retain the HABITAT-001 provisional reference targets:

- Environmental update mean ≤ 2 ms.
- Environmental update P99 ≤ 4 ms.

These targets apply to the established reference workload.

They do not certify production-scale atmospheric performance.

Establish additional preregistered scaling criteria before evaluating new large-domain candidates.

Require:

- Meaningful measured computational or memory improvement.
- Appropriate conservation.
- Exact replay within the declared execution contract.
- No missed required activation.
- Correct boundary exchange.
- Bounded resources under tested workloads.
- Preserved transaction rollback.
- Explicit approximation classification.

Report:

- Mean, median, P95, P99 and maximum duration.
- Cold and steady-state costs.
- Requested allocation bytes.
- Retained memory.
- Peak RSS.
- VRAM consumption.
- Kernel and complete-update timing.
- Occupancy and active-block evolution.
- Material and momentum residuals.

Do not alter acceptance criteria after observing experimental results.

## 15. Environmental Witness

Extend the existing experimental Witness.

Expose:

- Active and implicit blocks.
- Block boundaries.
- Occupancy.
- Carrier density.
- Scalar concentrations.
- Local velocity.
- Activation and deactivation events.
- Resolution levels, where applicable.
- Selected computational backend.
- CPU and GPU memory measurements.
- Conservation residuals.
- Simulation and rendering timing.

Provide reproducible scenario selection and reset.

The Witness must remain an observer.

Rendering must not determine atmospheric activation.

Do not build a second environmental simulator inside Bevy.

## 16. Scope restrictions

Do not implement during HABITAT-002:

- Canonical ship atmosphere integration.
- Flagellar propulsion.
- New biological organs.
- Resolved deformable-body hydrodynamics.
- Chemical reaction networks.
- Thermal convection.
- Neural systems.
- Population expansion.
- New history-authority models.
- MORPH-006.

Preserve the existing passive body-coupling interface and future mechanical reservations.

If the architecture requires a change in physical equations, distinguish that change from an optimization and retain it as an isolated proposal.

## 17. Deliverables

Produce:

- HABITAT-002_DIRECTIVE.md.
- HABITAT-002_DONOR_ARCHAEOLOGY.md.
- HABITAT-002_LICENSE_AND_PROVENANCE.json.
- SPARSE_TRANSPORT_ATTRIBUTION.md.
- DENSE_REFERENCE_OPTIMIZATION.md.
- DENSE_BLOCK_ARCHITECTURE.md.
- CONSERVATIVE_CARRIER_ACTIVATION.md.
- MULTIRESOLUTION_RESEARCH.md.
- GPU_LBM_FEASIBILITY.md.
- NUMERICAL_PRECISION_AND_REPLAY.md.
- Controlled scaling results and occupancy crossover analysis.
- Accepted and rejected experimental implementations.
- Environmental Witness updates.
- HABITAT-002_RESULTS.md.
- Machine-readable results and reproducibility instructions.
- Clean-source validation receipts.
- Updated HANDOFF and Atlas.
- Immutable restore-verified Mermogon evidence archives.

Retain failed experiments, original source identities, executable fingerprints and numerical counterexamples.

If an experimental lane cannot qualify, report its actual blocking mechanism rather than inventing a successful substitute.

## 18. Publication

Commit and push validated research and experimental implementations.

HABITAT-002 may conclude as PARTIAL.

No new architecture becomes canonical through publication alone.

Preserve HABITAT-001 as the validated reference and retain all previous archives.

Canonical adoption requires separate Operator approval.

## 19. Governing principles

Do not simulate empty computational work.

Do not mistake physical inactivity for physical absence.

An unobserved atmosphere must continue to exist and evolve.

Material and momentum must cross computational boundaries lawfully.

Use the cheapest representation that preserves the required physical information.

Optimize memory movement before multiplying computation.

Study existing discoveries before reinventing their mechanisms.

**The ship must be enormous. Its computational burden need not be proportional to every cubic metre it contains.**

Proceed autonomously within these authority boundaries.