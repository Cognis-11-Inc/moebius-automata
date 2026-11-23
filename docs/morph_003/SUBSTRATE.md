# MORPH-003 environmental substrate

2026-11-23. Experiment-local material book. Not the nursery `World`.

## Separation

| Layer | Owner | This slice |
|---|---|---|
| Anatomical structure | `AnatomySpec`, `CompiledAnatomy` | Where the nozzle is, and the pose that moves it. `operational` stays false. |
| Material transport | `MaterialBook` | Reservoir debit, parcel identity, world position, explicit sink. |
| Environmental forces | `Environment` on the book | Drag per mille, uniform acceleration, uniform flow, response per mille. Inputs, not hidden globals. |
| Visual representation | Bevy gizmos and the body transform | Copies parcel coordinates. Line segments are not bonds, pipes, or solids. |
| Future metabolism | Not this book | No harvest, no energy, no nutrient field. |
| Future pipe construction | Not this book | No stiffness, no joint, no curing. |

## State

`MaterialBook` version 1. Organism id, recorded seed, tick, initial quanta, reservoir, sinks, `external_input` (always 0), monotonic parcel ids from 1, and the parcel list.

Each parcel stores id, organism id, nozzle id, position, velocity, quantity, material `Secretion`, creation tick, and the response per mille copied at admission.

Seed is stored so a repeat names the same experiment. The motion update does not read it. There is no random term.

## Motion

Fixed tick. No wall-clock inside the book.

`couple = drag_per_mille * parcel.response_per_mille / 1000`, integer, at most 1000.

Then acceleration is added, velocity is blended toward `medium_flow`, and position advances by the new velocity. Overflow returns an error and does not commit the step. The lattice wrap used by nursery jets is not used. There is no ground plane.

Preferred baseline: drag 0, acceleration 0, flow 0. A comparison may set drag to 1000 or a small acceleration such as 4000 micro-units per tick on +X. That acceleration is not called gravity.

Full drag and zero flow make the pre-step velocity disappear before the position update, so a newly launched parcel stays at its birth coordinate on that step.

Negative velocity uses the same truncating division. The mechanism test `blend(-5, 0, 500) = -2` is the record of that rule.

## Emission

`try_emit` is the only admission path.

Disabled, zero quantity, wrong channel, unknown nozzle, second emit on the same tick, a non-unit direction, a launch speed outside `0..=1_000_000_000`, a full parcel list, or a reservoir smaller than the request all return a status and do not change quantity. The request is all or nothing.

A parcel is integrated only by a later `step`. Birth position is the placed nozzle position.

`sink_parcel` is the only deletion path. It adds the same quantity to `sinks`. It is an accounting tool, not decay and not metabolism.

`fault_duplicate_last` and `fault_drop_last` exist so tests can break the ledger on purpose. They are not emission.

## Placement

`place_output` quantizes the anatomical frame into micro-units and applies a rigid yaw about Y plus a translation. Yaw sine and cosine are micro-units of a unit angle. `sin = SCALE`, `cos = 0` sends +X to -Z. The anatomical compiler's f32 pose is same-build deterministic. Integer motion after that quantization does not depend on the platform beyond Rust's toward-zero division.

An authored trajectory is a test input. It is not locomotion. Flagellar motion in the preview is still the anatomical phase. It is not propulsion.

## Persistence

`save` / `load` use the anatomical crate's atomic replace. Load rejects a book whose residual is not zero, whose ids collide, or whose version, capacity, or channel assumptions fail. Capacity is `1..=4096`.

## Preview clock

The lab accumulates 1/60 s and spends at most four ticks per frame. Extra stalled time is dropped and the note says so. That limit is not part of the benchmark. Pose phase for the preview is `min(tick, 6_399_936) / 64`, which reaches 99999 and stops. It does not wrap. The anatomy preview's wall-clock phase still runs only while the material clock is off.

## What this does not decide

The ship interior, breathing hull, particulate atmosphere, pipe growth, and metabolism remain design hypotheses. This book does not implement them.
