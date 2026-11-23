# MORPH-003 implementation plan — 2026-11-23

Baseline: `main` at `1296ce4d338c5863eb023863e39a45554447f47f` (MORPH-002 pushed).
The MŒBIUS Ship note is design canon. It does not add vessel physics.

## Archaeology

| Concern | Path | What it actually is | Ruling |
|---|---|---|---|
| Anatomical nozzle | `crates/moebius-morph/src/anatomy.rs` `SecretionNozzle`, `OutputFrame` | Pose-dependent origin and direction. `operational: true` is rejected. Channel string is `future_conserved_substrate`. | Keep. Anatomy does not admit material. |
| Body deformation | `compiler.rs` `center`, `CompiledAnatomy::pose` | Bend and phase move the trunk. Attachments are recomputed. | Sample that frame. Do not invent a second mesh. |
| Nursery secretion | `world.rs` `SECRETE_ACTION` | Spends nutrient and energy, writes flesh on the cell site. | Do not call. |
| Nursery jet | `neural/body.rs` `Jet`, `world.rs` `advance_jets` | Lattice fixed-point, life 4, periodic wrap, nutrient and energy debit, reaction impulse, then flesh or waste. | Do not modify and do not reuse. It is locomotion feedstock. |
| Fluid samples | `layers.rs` `FluidSample` | Block sums of the nutrient field. Not a new stock. | Do not reuse. |
| Trail points | `layers.rs` `TrailPoint` | Observation rows. The comment says they are not bonds. | Do not reuse as material. |
| Hypothesis layer | `layers.rs` `HypothesisLayer` | Non-authoritative annotation. | Do not reuse. |
| Snapshots | `snapshot.rs` | Nursery world persistence. | Do not route this book through it. |
| Preview clock | `moebius-morph-lab` `advance` | Wall-clock phase, capped at 99999, not a material step. | Leave it for the anatomy preview. Emission uses the book tick. |

`moebius-morph` does not depend on `moebius-core`. No file under `crates/moebius-core` is part of this slice.

## Convergence decision

The lattice jet conflicts with a persistent free-space trail. Changing it would alter nursery conservation and couple emission to locomotion, which this slice forbids.

The authorized substrate is a new experiment-local book in `moebius-morph`, not a second `World`. The nursery remains the authority for cells, nutrient, flesh, and jets. The book owns only quanta that an explicit emit transferred out of its own reservoir. The laboratory reads the book. Rendered lines are not parcels and not pipes.

This is the operator-visible ruling. It is not a silent third world.

## Smallest model

Integer quanta. Position and velocity are micro-units (`1_000_000` = 1 anatomical world unit). One `step` is one tick.

```
couple = drag_per_mille * response_per_mille / 1000
velocity = velocity + acceleration
velocity = (velocity * (1000 - couple) + flow * couple) / 1000
position = position + velocity
```

Division truncates toward zero. Drag, acceleration, and flow are fields on the book. Baseline is zero drag and zero acceleration. Full drag with zero flow stops the parcel before that tick's displacement. There is no floor, no gravity name, and no atmosphere.

Invariant: `initial + external_input = reservoir + in_world + sinks`. `external_input` stays 0. A failed emit does not debit. Capacity above the configured bound returns `CapacityExhausted` and leaves the book unchanged. The prototype bound is 4096 parcels.

Runtime operational is true only after `Admitted`. The anatomical flag stays false.

The benchmark schedule samples the nozzle, emits, then steps. The preview steps, then emits, so the newest parcel is still on the nozzle until the next tick. Both are sequences of the same two operations. Reproduction uses the benchmark schedule.

## Gates

Headless experiments A–F and the nozzle, fault, and capacity checks live in `crates/moebius-morph/tests/emission.rs` and `evaluate_morph003`. The preview clock and ledger live in `emission_preview.rs` and are drawn by the existing Bevy lab. Workspace certification follows a clean commit. No push. No MORPH-004.
