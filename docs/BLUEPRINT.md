# Möbius Automata M0 design record

This file records the operator brief that this repository implements, including
the amendments made while the first version was being written. A separate
manuscript titled "Research and Engineering Blueprint v0.1" was not in the
workspace. This page does not invent that manuscript, and it is not evidence
that a behavior exists. The tests and the run manifest are the evidence.

## What M0 is

A headless, seeded, deterministic simulation of persistent digital cells on a
finite 3D lattice, plus a read-only viewer of committed snapshots.

- Each live cell has a unique nonzero id, one site, a nonnegative action-energy
  balance at most `energy_cap`, an age, and eight inert genome bytes.
- The grid is at most 64×64×64. The default experiment is that size. Edges wrap.
  The neighborhood is the six face neighbors. At most one live cell occupies a site.
- One material nutrient field has an explicit reservoir, external input, waste,
  and external output. Every conversion is in the ledger.
- The tick reads a snapshot, resolves movement and uptake and maintenance, audits,
  and commits atomically. Observers do not receive mutable state.
- Decisions that affect the next state use integers and ChaCha12 streams derived
  from the seed. They do not use wall-clock time, a frame rate, or hash-map
  iteration order.
- The command line runner writes a manifest, metrics, per-tick trails, and
  snapshots. Invalid scientific inputs fail with a diagnostic.

## Operator amendments included here

- The lattice is 3D. 64×64×64 replaces an earlier 2D 128×128 candidate.
- The viewer is a dark 3D reading of committed data. Cell brightness follows
  action-energy. It is not a species color.
- Filaments are the last 24 committed movement steps, split where a step wraps.
  They are not bonds.
- Fluid is a strided block-sum of the nutrient field, labeled as a downsample.
- Sound is deterministic PCM in the viewer. It is not part of the state hash
  and it does not advance ticks. Byte-identical audio across platforms is not claimed.
- A Python 3.13 virtual environment can check manifests. It is not a second simulator.
- Hypothesis layers can hold annotations so a later organism question has a place
  to be written down. M0 does not detect organisms.

## What M0 does not do

Binding, division, mutation, cell roles, signal networks, collective
classification, organs, evolutionary success, and emergent multicellularity are
not implemented. A hypothesis mark is not membership. The genome is not read by
the tick. This crate does not connect to Proto-Pineal or any other Cognis system.

Cross-platform bitwise identity was not tested outside this machine. Do not
claim it from a single Linux run.

## Layer scaffold

`ObservationDeck` stores named hypothesis layers outside the world. A layer has
a name, a note, and optional weighted sites. Names are lowercase identifiers.
The empty scaffold note says that no organism detector writes the layer. The
viewer leaves that layer off until the operator presses `4`, and the HUD says
the marks are not simulation state.

Parsing accepts at most 4096 marks, rejects weight 0, rejects duplicate sites,
and rejects unknown JSON fields. The core tick never calls that parser.
