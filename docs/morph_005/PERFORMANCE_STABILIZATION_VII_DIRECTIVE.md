# MŒBIUS AUTOMATA
## MORPH-005 — Performance Stabilization VII
### Morphological Entanglement, Procedural Collision and Computational Economy

## 0. Mission

Continue from the published Stabilization VI checkpoint:

`7de3fa72d296d9574d6bdeaf514e6a229f6af687`

MORPH-005 remains PARTIAL.

Six stabilization phases have substantially improved collision processing, spatial locality, historical continuity and deterministic execution.

Nevertheless, population scaling, congested interaction costs, allocation requests and authoritative tick latency remain unacceptable.

The Operator has additionally observed that Arachnetechs frequently become entangled with one another and subsequently fail to construct clean snailpipes.

The Operator suspects the flagella contribute to this behaviour.

This is a hypothesis requiring experimental investigation, not an established diagnosis.

Stabilization VII must investigate three interconnected problems:

1. Morphological entanglement and its consequences for movement and secretion.
2. Computational cost inside expensive individual collision queries.
3. Alternative procedural representations of physical geometry.

**MORPH-006 and general biological expansion remain suspended. Controlled anatomical experimentation is authorized within this directive.**

## 1. Entry Verification and Containment

Reconcile the repository with:

- AGENTS.md.
- HANDOFF.md.
- PERFORMANCE_STABILIZATION_V.md.
- PERFORMANCE_STABILIZATION_VI.md.
- PERFORMANCE_STABILIZATION_VI_RESULTS.json.
- AUTHORITATIVE_TICK_DEPENDENCIES_V.md.
- PHYSICAL_GEOMETRY_AUTHORITY_V.md.
- QUERY_FRONTIER_AUTHORITY_VI.md.
- SENSORY_ARCHITECTURE.md.
- Relevant validation, provenance and storage receipts.

Verify:

- Local and remote commit identities.
- Working-tree status.
- Clean runtime checkpoint `c989033decd4d466a8d0a0a279975bb73344f810`.
- The reported 260-test clean-source certification.
- Mermogon archive identities and integrity.
- Authenticated collision fixtures.
- Existing anatomical definitions and editor configurations.
- Current flagellar geometry, animation, propulsion and collision consumers.

Reuse completed archaeology. Do not repeat previous investigations without a specific unresolved dependency.

Preserve previous source versions, experimental failures, benchmark executables, snapshots and immutable archives.

## 2. Established Performance Baseline

The VI report records:

| Scenario | Mean ms | P99 ms |
|---|---:|---:|
| One founder | 0.488 | 4.004 |
| Two founders | 4.909 | 31.215 |
| Four founders | 7.499 | 20.351 |
| Eight founders | 13.848 | 47.553 |
| Sixteen founders | 39.006 | 81.379 |
| Fresh colony | 10.945 | 33.736 |
| Eight-founder resume | 13.949 | 46.275 |
| Congested save | 32.106 | 79.747 |
| Native-input 720 | 21.420 | 71.017 |
| Endurance v2 | 54.893 | 103.930 |

These are VI candidate measurements, not universal sequential-default measurements.

Matched congested comparison:

- Sequential: 43.146 ms.
- Parallel recovery: 31.458 ms.

Complete endurance:

- Sequential: 193.275 seconds.
- Parallel recovery: 197.616 seconds.

Parallel endurance dispatched 15,440 queries, of which 4,837 were unnecessary.

Parallel execution increased requested allocation volume by approximately 1.70%.

Sequential execution remains the default.

Reconcile all inherited numerical claims against retained evidence before experimental use.

## 3. P0-A — Flagellar Authority and Anatomy Audit

### 3.1 Objective

Determine precisely how flagella participate in the authoritative physical and behavioural runtime.

Do not infer their implementation from rendered appearance.

Trace the exact producer and consumer paths for:

- Flagellar anatomical definitions.
- Attachment positions.
- Length and thickness.
- Segmentation.
- Joint or bending representation.
- Animation and phase.
- Collision geometry.
- Movement proposals.
- Propulsion.
- Contact resolution.
- Self-contact.
- Inter-organism contact.
- Body-pipe contact.
- Secretion-related geometry.

For every property, classify its current maturity:

- Implemented and authoritative.
- Implemented but presentation-only.
- Experimental.
- Reserved by contract.
- Unimplemented.
- Unresolved.

Identify the exact files, symbols, call sites, tests and historical design decisions supporting each classification.

Determine whether the visible flagella correspond to the actual physical collision envelope.

Do not assume that rendered animation represents articulated physical motion.

### 3.2 Physical questions

Establish whether the current flagellar model:

- Produces actual propulsive forces or contributes to prescribed movement.
- Changes authoritative collision geometry during motion.
- Can bend independently in response to contact.
- Can pass between nearby bodies.
- Can become mechanically interlocked.
- Participates in self-collision.
- Prevents rotation or translation of the organism.
- Can retract, relax or reverse its movement.
- Has physically meaningful disengagement mechanisms.

Document absent capabilities explicitly.

Do not introduce new organ behaviour merely to complete the audit.

## 4. P0-B — Entanglement and Snailpipe Failure Experiment

### 4.1 Objective

Test whether flagellar morphology contributes causally to persistent entanglement, failed secretion and disproportionate collision-processing costs.

The Operator's observation must be reproduced through controlled physical scenarios.

### 4.2 Operational definitions

Define measurable distinctions between:

- Ordinary transient contact.
- Sustained contact.
- Geometric interlocking.
- Physical penetration.
- Repeated movement rejection.
- Persistent solver freezing.
- Mechanical inability to disengage.
- Interrupted secretion.
- Failed deposition.
- Malformed or discontinuous snailpipes.

Do not classify all prolonged proximity as entanglement.

Do not classify all rejected deposition as a physical defect.

A legitimately obstructed organism may be unable to secrete safely.

The experiment must distinguish intended physical consequences from numerical pathology.

### 4.3 Anatomical experimental matrix

Use the existing canonical anatomy as the reference.

Where supported by the current anatomical representation, construct controlled variants:

| Variant | Purpose |
|---|---|
| Original flagellar anatomy | Reference |
| No flagella | Isolate main-body interactions |
| One short flagellum | Examine single-appendage contribution |
| Two short flagella | Examine multiple-appendage contribution |
| Original count with increased length | Examine length sensitivity |
| Original count with reduced thickness | Examine collision-envelope sensitivity |
| Original geometry with stationary flagella | Separate movement effects from geometry |
| Original geometry with controlled movement phase | Examine phase-dependent interactions |

Do not invent unsupported anatomical parameters.

If a variant cannot be produced through the existing model, classify it as NOT SUPPORTED or implement it only within a clearly isolated experimental harness.

Do not silently change canonical creature definitions.

### 4.4 Propulsion confounding

Removing or modifying flagella may change encounter rates, movement speed, orientation and available propulsion.

Therefore conduct two distinct experimental families.

**Family A — Natural movement**

Allow each experimental anatomy to use its actual existing movement mechanisms.

This measures the total consequences of the anatomical variation.

**Family B — Matched kinematic inputs**

Where lawful, provide equivalent controlled movement proposals, initial poses and encounter conditions.

This isolates collision geometry and contact behaviour independently of propulsion differences.

Do not treat a motionless organism having fewer collisions as proof of improved collision behaviour.

### 4.5 Encounter scenarios

Construct reproducible scenarios covering:

- Isolated organism producing an unobstructed snailpipe.
- Two organisms approaching head-on.
- Two organisms crossing at oblique angles.
- Organisms passing beside one another.
- Tail and flagellar contact.
- Multiple appendages contacting another organism.
- Two organisms secreting in proximity.
- Organisms approaching existing hollow snailpipes.
- Flagella contacting pipe openings.
- Organisms crossing recently deposited wet structures.
- Persistent contact followed by a physically available disengagement trajectory.
- Local congestion involving multiple organisms and structures.

Separate scenarios with a known available escape trajectory from those that are genuinely geometrically constrained.

Use deterministic fixture identities and retained initial worlds.

### 4.6 Required measurements

Record:

- Contact frequency.
- Contact duration.
- Persistent contact graph.
- Participating anatomical components.
- Body and flagellar contact locations.
- Movement proposal acceptance and rejection.
- Solver freezing and slide responses.
- Contact-recovery invocations.
- Continuous collision-query multiplicity.
- Time spent resolving each interaction.
- Relative displacement and orientation.
- Successful spontaneous disengagement.
- Secretion attempts.
- Successful and rejected deposits.
- Rejection reasons.
- Snailpipe continuity and geometric integrity.
- Material conservation.
- Requested allocations and retained memory.
- Mean and tail tick latency.

Where supported, produce diagnostic visualization identifying the components responsible for sustained contact.

Diagnostic visualization remains an observer and must not alter physical authority.

### 4.7 Scientific verdicts

Classify the flagellar hypotheses independently:

- H1: Flagellar geometry increases persistent entanglement.
- H2: Flagellar movement increases persistent entanglement.
- H3: Existing flagellar collision envelopes are disproportionate to their intended physical anatomy.
- H4: Existing contact mechanics prevent disengagement despite an available physical trajectory.
- H5: Entanglement reduces successful snailpipe construction.
- H6: Flagellar entanglement generates disproportionately expensive collision processing.
- H7: Anatomical variation can improve disengagement without removing meaningful physical interaction.

Establish measurable decision criteria before evaluating final experimental outcomes.

Report each hypothesis as SUPPORTED, NOT SUPPORTED or INDETERMINATE within the tested scope.

Do not convert correlation into causal proof.

## 5. Anatomical Authority and Prohibited Shortcuts

The Arachnetech must remain a physically meaningful organism.

Do not solve entanglement by:

- Disabling flagellar collision globally.
- Making appendages decorative without explicit authorization.
- Introducing arbitrary inter-organism repulsion.
- Teleporting trapped organisms.
- Automatically separating bodies after a timeout.
- Lowering collision accuracy.
- Ignoring obstructed secretion.
- Allowing material to pass through physical obstacles.
- Introducing scripted avoidance intelligence.
- Removing difficult anatomical configurations from benchmarks.

Entanglement itself is not prohibited.

Organisms should be capable of entering physically meaningful constrained configurations.

The objective is to determine whether the underlying physical model permits lawful disengagement where such disengagement should exist.

If the current anatomy lacks necessary compliance or articulation, propose an explicit experimental correction.

Any replacement of canonical flagellar physical authority requires separate Operator approval.

## 6. P0-C — Intra-Query Computational Parallelism

### 6.1 Objective

Investigate whether expensive individual collision queries contain parallelizable geometric work.

The previous batch executor demonstrated substantial throughput for independent queries but did not substantially accelerate isolated expensive queries.

V recorded approximately:

- Eight-founder individual query: 17–18 ms.
- Fresh-colony individual query: 60 ms.
- Endurance query batch: 5.35× throughput at eight workers.

Do not treat the batch result as an achievable single-query acceleration factor.

### 6.2 Query decomposition

Trace the dominant individual nonlinear and linear collision queries.

Attribute work to:

- Compound components.
- Mesh acceleration traversal.
- Support-point calculations.
- Bounding-volume tests.
- Candidate triangle selection.
- Leaf evaluation.
- Distance calculation.
- Contact refinement.
- Time-of-impact search.
- Contact recovery.
- Ordered result selection.

Determine which operations can be subdivided without changing canonical traversal semantics.

Identify dependencies involving:

- Current best distance.
- Pruning thresholds.
- Earliest time of impact.
- Contact-normal selection.
- Equal-distance ties.
- Numerical operation ordering.
- Early termination.
- Error ordering.

### 6.3 Experimental implementation

Investigate lawful deterministic subdivision where supported.

Potential candidates:

- Independent immutable compound-component evaluation.
- Parallel evaluation of authenticated candidate leaf groups.
- Parallel conservative bounds.
- Ordered collection of exact local collision results.

These are candidates, not implementation mandates.

A parallel traversal must not silently replace an ordered branch-and-bound algorithm with a different numerical selection rule.

Do not introduce uncontrolled nested parallelism between existing recovery workers and new intra-query workers.

Reuse bounded executor infrastructure where its contract remains suitable.

### 6.4 Qualification

Use the authenticated isolated pathological fixtures retained from IV and V.

Require:

- Exact hit or clear outcomes.
- Exact time-of-impact representation.
- Exact contact points and normals.
- Preserved canonical tie-breaking.
- Preserved error visibility.
- Complete authoritative input authentication.
- Meaningful ordinary-release acceleration.
- Bounded memory and allocation costs.

Then integrate only candidates that establish lawful runtime consumption.

Run matched colony continuations before full-matrix qualification.

An isolated result does not establish a runtime implementation.

## 7. P0-D — Procedural Physical Geometry Research

### 7.1 Objective

Determine whether the mathematical description of organically generated structures can reduce the cost of collision processing.

Current snailpipes are represented by procedurally generated, oriented hollow triangulated meshes.

The engine already possesses structural information including:

- Paid centreline nodes.
- Curvature.
- Inner and outer radii.
- Deposit ages.
- Wall thickness.
- Openings and seals.
- Endpoint status.
- Nozzle-dependent geometry.
- Structural connectivity.

Verify the actual representation and consumers before implementing alternatives.

Investigate whether that information can reduce the need for complete mesh construction and traversal.

### 7.2 Lane A — Exact procedural acceleration

Preserve all existing physical geometry and collision semantics.

Investigate:

- Conservative curved-segment bounds.
- Hierarchical procedural exclusion.
- Exact local mesh generation on demand.
- Selective generation of physically relevant sections.
- Reusable immutable construction intermediates.
- Segment-level acceleration where exact global ordering can be preserved.
- Rejection of regions without constructing their complete geometry.

A procedural bound must enclose every relevant authoritative surface.

It must account for inner walls, curvature, elliptical sections, ridges, wobble, age-dependent radii, openings and caps.

Any uncertain result must fall through to the canonical exact query.

No admissible contact may be discarded.

### 7.3 Lane B — Alternative procedural collision representation

Implement a small, isolated experimental prototype of a procedural hollow-pipe collision representation.

This research lane is explicitly authorized.

It must not replace canonical collision authority.

Investigate mathematical or hybrid representations using the actual existing structural inputs.

Possible candidates include:

- Curved swept hollow sections.
- Piecewise procedural surfaces.
- Hierarchical inner and outer wall representations.
- Adaptive local surface evaluation.
- Conservative procedural candidates followed by exact local refinement.

Choose the smallest prototype capable of falsifying the proposed computational advantage.

Do not build a second complete simulation engine.

### 7.4 Physical fidelity experiment

Compare the prototype with the canonical triangulated representation using controlled queries.

Measure:

- Hit and clear agreement.
- False-negative contacts.
- False-positive contacts.
- Distance deviation.
- Time-of-impact deviation.
- Contact-point deviation.
- Contact-normal deviation.
- Inner-wall interaction.
- Outer-wall interaction.
- Traversable clearance.
- Opening and seal preservation.
- Junction behaviour.
- Wet-to-dry geometric evolution.
- Construction time.
- Query time.
- Requested allocations.
- Retained memory.

Include difficult configurations rather than exclusively uncomplicated straight pipes.

A mathematically smoother surface may legitimately differ from the current triangulated physical surface.

Such differences must be explicitly reported.

Do not claim exact equivalence for a changed physical model.

### 7.5 Authority boundary

Alternative physical representations remain experimental.

Any proposal for canonical adoption must include:

- The revised physical law.
- Properties preserved.
- Properties changed.
- Known approximation and failure boundaries.
- Material-ledger compatibility.
- Contact and adhesion implications.
- Historical replay implications.
- Snapshot and migration requirements.
- Performance evidence.
- Explicit Operator approval.

Do not combine alternative-representation results with exact-acceleration certification.

## 8. P1 — Adaptive Parallel Recovery

VI established meaningful congestion acceleration but failed to improve general endurance performance.

Investigate whether recovery dispatch can be restricted to workloads where parallel execution is demonstrably profitable.

Attribute:

- Eligible recovery samples.
- Existing memo hits.
- Consumed speculative results.
- Unnecessary results.
- Worker execution costs.
- Sequential short-circuit frequency.
- Synchronization overhead.
- Workspace duplication.
- Geometry complexity.
- Local interaction density.

Do not equate unnecessary-query counts with wasted CPU-time fractions.

Evaluate deterministic, bounded dispatch policies using information available before execution.

Potential policies:

- Minimum number of unresolved samples.
- Existing memo availability.
- Authenticated geometry complexity.
- Measured query-cost predictors based on deterministic structural properties.
- Ordered small waves instead of dispatching every sample immediately.

Runtime timing measurements may guide offline policy selection but must not introduce nondeterministic physical decisions.

Preserve canonical sequential fallback, event ordering and transaction semantics.

Compare policies on both calibration and held-out workloads.

Reject policies that merely specialize to the congested fixture while regressing broader workloads.

Sequential remains the default until a more general policy qualifies.

## 9. P1 — Exact Geometry Allocation Economy

The previous transactional geometry handoff did not demonstrate meaningful general benefit.

Do not repeat the same experiment without new evidence.

Investigate the remaining measured geometry-construction costs, particularly:

- Wet mesh construction.
- Repeated centreline interpolation.
- Frame reconstruction.
- Ring and index generation.
- Temporary construction buffers.
- Collision pseudo-normal generation.
- Acceleration-structure construction.

Determine whether immutable intermediates can be reused while changing radii and authoritative geometry continue to evolve correctly.

Preserve exact vertex/index ordering, supported dependency APIs and complete mutation invalidation.

No unsafe dependency modification or unbounded retention is authorized.

Compare geometry economy independently from parallel execution and anatomical variations.

## 10. Endurance Regression and Outlier

The VI report identifies:

- Approximately 4.34 seconds of total endurance regression.
- Approximately 4.09 seconds of that regression inside collision resolution.
- One 415 ms tick coinciding with history segment sealing.
- Failure to reproduce the original pause in subsequent diagnostic execution.

The original syscall-level cause remains unresolved.

Do not assume history persistence caused the pause.

Do not redesign historical continuity in response to an unverified attribution.

Investigate the outlier only if new evidence or reproducible conditions justify further work.

Prioritize systematic collision cost over isolated unexplained timing events.

## 11. Experimental Separation

Maintain independent experimental lanes:

- A: Canonical sequential reference.
- B: Qualified exact intra-query acceleration.
- C: Qualified adaptive recovery.
- D: Qualified exact procedural geometry economy.
- E: Controlled anatomical variants.
- F: Alternative physical representation prototype.

E and F are research lanes with potentially different physical inputs or semantics.

Do not use their performance results to claim exact legacy-state equivalence.

Only independently qualified exact optimizations may be combined for canonical runtime certification.

Record source, executable, configuration, anatomy and fixture identities for every comparison.

## 12. Qualification Sequence

### Gate A — Verified authority and attribution

Identify actual implementation, causal dependencies and dominant costs.

### Gate B — Controlled reproduction

Reproduce the pathological collision workload or observed entanglement behaviour.

### Gate C — Isolated experimental result

Demonstrate meaningful improvement or establish a falsified hypothesis.

### Gate D — Focused correctness

Pass applicable contact, geometry, material, ordering, invalidation and rollback tests.

### Gate E — Matched continuation

For exact acceleration candidates, preserve all required intermediate rows, contact results, event payloads and final worlds.

For alternative anatomy or physical representations, establish separate deterministic experimental replay and report actual physical differences.

### Gate F — Ordinary release performance

Measure integrated performance without diagnostic instrumentation.

Separate cold-start and steady-state behaviour.

### Gate G — Full matrix

Run the established full matrix only after an exact runtime candidate qualifies.

Do not repeatedly execute the matrix for unqualified prototypes.

## 13. Behavioural Qualification

The stabilization programme must no longer treat physical reproducibility as sufficient evidence of desirable organism behaviour.

Add a dedicated behavioural evidence report containing:

- Entanglement frequency and persistence.
- Anatomical contribution to contact.
- Availability of physical disengagement trajectories.
- Solver behaviour during sustained constraint.
- Snailpipe construction success under controlled conditions.
- Secretion failure attribution.
- Physical versus numerical pathology classification.
- Relationship between morphology and computational cost.

Where appropriate, propose explicit behavioural acceptance criteria for future canonical development.

Do not silently redefine existing MORPH-005 performance gates.

## 14. Full Runtime Matrix and Acceptance

Retain the established MORPH-005 cases:

- One founder, 120 ticks.
- Two founders, 120 ticks.
- Four founders, 120 ticks.
- Eight founders, 120 ticks.
- Sixteen founders, 120 ticks.
- Fresh colony, 420 ticks.
- Eight-founder resume, 120 ticks.
- Congested save, 30 ticks.
- Native validation, 720 ticks.
- Opt-in v2 endurance, 3,600 ticks.

Retain the original acceptance targets:

- Provisional mean ≤ 5 ms.
- Preferred mean ≤ 2 ms.
- P99 ≤ 8 ms.
- Authoritative real-time budget approximately 16.667 ms.
- Requested allocation reduction ≥ 95%.
- Complete 3,600-tick endurance within 180 seconds.
- Exact applicable authoritative equivalence.
- Material conservation.
- No new missed admissible contacts relative to the canonical reference.
- Correct geometry invalidation.
- Bounded memory under tested workloads.

Report each gate independently.

Do not declare stabilization complete merely because an anatomical experiment reduces collision frequency.

## 15. Historical Continuity and Scope

Preserve history v1 and opt-in v2.

Retain strict source-bound restoration and existing historical provenance.

Do not introduce history migration, coordinate-system redesign, population expansion, GPU offloading or new sensory systems during VII.

Controlled anatomical and collision experiments are authorized.

Canonical biological expansion remains suspended.

## 16. Deliverables

Produce:

- PERFORMANCE_STABILIZATION_VII.md.
- FLAGELLAR_AUTHORITY_AUDIT.md.
- ARACHNETECH_ENTANGLEMENT_RESULTS.md.
- SNAILPIPE_CONSTRUCTION_BEHAVIOUR.md.
- INTRA_QUERY_PARALLELISM_RESULTS.md.
- PROCEDURAL_COLLISION_RESEARCH.md.
- ADAPTIVE_RECOVERY_RESULTS.md.
- Exact geometry allocation attribution.
- Controlled anatomical fixture definitions.
- Entanglement and contact diagnostic evidence.
- Alternative representation fidelity and performance evidence.
- Accepted and rejected optimization evidence.
- Matched runtime qualification where supported.
- Clean-source test and validation receipts.
- Updated HANDOFF and Atlas.
- Immutable, restore-verified Mermogon evidence archive.

Include diagnostic visual recordings where useful for understanding observed entanglement and malformed snailpipes.

Distinguish observations, verified implementation, experimental results, hypotheses and unresolved mechanisms.

## 17. Publication

Commit and push validated work and reproducible evidence.

A useful checkpoint may remain PARTIAL.

Do not promote alternative anatomical authority or physical collision representations without explicit Operator approval.

Do not commence MORPH-006.

## 18. Governing Principles

The objective of MŒBIUS AUTOMATA is not merely to execute a deterministic simulation quickly.

It is to sustain a rich physical environment in which complex artificial organisms can interact, construct, adapt and eventually discover their own strategies.

Entanglement may be meaningful.

Physical obstruction may be meaningful.

Secretion failure may be meaningful.

Numerical imprisonment is not an ecological mechanism.

The physics must permit a solution before intelligence can discover one.

Parallelize computation, not authority.

Preserve biological richness while reducing its computational cost.

**Optimize the cost of causality, not its richness.**

Proceed autonomously within these boundaries.