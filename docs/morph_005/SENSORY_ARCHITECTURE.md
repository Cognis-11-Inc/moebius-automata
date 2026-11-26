# Sensory architecture contract

Status: **CONTRACT ONLY**. Nothing in this document is implemented by this
change. It does not add organs, fields, events, or a new runtime authority.

2026-11-26. MORPH-005 stabilization III. Biological expansion stays suspended.
The nursery neural vector in `moebius-core` (`Genome::sensory`,
`sensory_gains`) is a separate lattice controller. This contract does not
attach the ship runtime to that vector and does not treat it as ship perception.

## Ownership

The simulation owns authoritative physical reality: bodies, pipes, solids,
roots, parcels, food sites, and the material ledger.

An organism may perceive that reality only through sensory organs configured
on that organism. A missing organ is the absence of a sample. It is not the
absence of the physical object.

`collision::resolve` remains the only contact authority. Habitat
`ContactObservation` values are world records of physical outcomes. They are
not a sensory organ and they must not be reused as one. An organism that
cannot perceive an obstacle still collides with it.

Sensory code must not read global coordinates, object identifiers, hidden
ledger fields, another organism's internal state, or any other fact that the
configured organ cannot physically obtain. Where a sample needs a position,
that position is the organ's own sample frame, not a world catalog.

## Per-modality configuration

Each modality is configured independently. One organism may carry several.
None is granted by having a larger neural architecture.

- Detection range
- Directionality
- Data type
- Spatial resolution
- Temporal resolution
- Sensitivity
- Signal latency
- Metabolic cost
- Environmental attenuation that applies to that modality

Anticipated modalities, none of which are implemented here:

- Mechanical touch
- Echolocation
- Vision
- Chemical perception
- Thermoreception
- Electroreception
- Structural vibration perception

Mechanical touch, when it exists, samples contact that the collision
authority has already admitted. It does not decide contact.

## Shared physical work

Where a physical phenomenon is lawful to compute once, compute it once and
let organs sample it. Examples that already exist and may be sampled later,
without becoming sensory authorities:

- The collision pipe horizon and the hollow-pipe meshes it guards
- Food sites and illumination, which remain world fields

A sensory query must not rebuild collision acceleration, and collision must
not depend on which organs are awake, on Witness position, or on rendering
cadence.

## Echolocation reservation

Reserve a future localized acoustic event with:

- Emission origin
- Frequency
- Intensity
- Directionality
- Propagation delay
- Medium-dependent attenuation
- Reflection properties
- Receiver sensitivity

Full volumetric acoustic simulation is not a prerequisite. Any approximation
must state its physical assumptions and its limits in the same change that
introduces it. A missed echo is not a missed collision.

Acoustic emissions should eventually be able to drive audible Witness
presentation from actual organism activity. That presentation is an observer.
It does not write the world.

Do not script synchronization, communication, or collective behaviour on top
of this reservation. If those appear, they have to come from organisms using
organs in the environment.

## Non-claims

This contract does not minimize what an organ may lawfully report. It forbids
privileged access. It does not rank organisms by neural size. It does not
authorize MORPH-006 or a sensory implementation in the stabilization tree.
