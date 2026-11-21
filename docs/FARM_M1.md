# Farm M1 — first biomechanical vivarium slice

2026-11-21. The live default is 256×256×256: twice the previous side length,
eight times the volume. The 64³ M0 acceptance experiment is unchanged.

## Body and motor controller

Each cell owns a twelve-joint spine and two independently rendered mandible joints, three fixed-point velocities, and a
subsite offset. A tiny linear neural policy has 32 outputs: coast, 24 motor choices
(two body sections, two opposed strokes, six thrust headings), secrete, and six
directed digging choices. It senses adjacent nutrient differences, momentum, available joint
stroke, local occupancy/flesh, and a bias. Initial preferences are seeded; the
policy explores on roughly one quarter of decisions. It adjusts bounded integer
weights using feeding income minus maintenance, motor, secretion, and excavation costs,
a running reward baseline, and decaying eligibility traces.

This is an adaptive policy over authored motor primitives, not independent
control of arbitrary muscles. A motor choice actuates one six-joint section with broad S-bends and twisting;
the other section retains its pose. Bends reach 160/256 radians and twists
96/256 radians per joint. Front and rear sections can hold different poses. Joint displacement produces an effective impulse in the selected
heading. Motors cost one energy unit when they move. No energy means no powered
stroke. Coasting still integrates velocity. The medium damps velocity by 127/128
per tick, with fixed-point rounding; velocity is capped at a quarter site per
axis per tick. Occupied endpoints and mature flesh cause damped rebound.

The medium supplies the reaction to strokes. This is a simplified propulsion
law, not vacuum propulsion, rigid-body dynamics, or a hydrodynamics solver.
Only the cell endpoint occupies a lattice site; mesh overlap is not a complete
body collision calculation. Diagonal motion can arise from combined momentum.

The GPU skin follows the committed joints; the renderer blends positions,
orientation, and joints between observations. The body does not spin on an
independent idle loop in farm mode. Learning updates are real state changes;
**improved locomotion or learned construction has not been demonstrated**.

## Feeding and reproduction

Cells consume nutrient at their actual lattice site, converting it to waste
one-for-one and receiving action-energy. Lobed green clusters summarize nutrient
in 64³ blocks at the default cube size; they are display samples, not isolated
physical pellets. A mouth extends and contracts only when the committed cell
reports positive uptake. Shell and feeding tissue meshes are presentation.

The default reproduction rules are: at least 80 energy, age at least 64 ticks,
128 ticks between births, and a total population ceiling of 768. A parent needs
an empty face-neighbor site without mature flesh. It pays 8 energy, then splits
the remaining energy between parent and offspring. No new energy is credited.
The child inherits the controller weights, with a bounded one-weight mutation,
and the parent's velocity. Joint pose, learning traces, and reward memory reset.
Parent ID and generation are recorded. Display genome bytes are inherited;
body material/biomass is not yet a stock in this model.

Births are resolved in ascending parent ID order. The cap is a stated operating
limit, not evidence of ecological equilibrium. Deaths and extinction remain
valid outcomes and are retained in records.

## Secretion and surfaces

Secretion is an available controller action, with no architecture objective or
scripted nest. When affordable, it converts one local nutrient unit into one
persistent flesh unit and disposes two energy units. A site holds at most 64
units; at most 8192 flesh sites are retained. A controller senses nearby flesh.
Face-adjacent deposits weld immediately into connected tissue, including across
periodic seams. A connected component with at least four material units becomes
a contact surface throughout, arresting entry and damping/reversing momentum.
Fusion does not create, delete, or move material. The display derives a smooth
implicit surface from the field rather than drawing separate intersecting lobes. New offspring cannot be placed inside one.

Brown-red tissue deposits accumulate and remain after their creator dies. Their
material is included in the stock ledger and in hashed snapshots. Wet, fading
movement traces are a separate viewer history, not free construction material.
Softness is currently visual appearance and damped contact; volumetric tissue
elasticity and a structural mechanics solver are not implemented.

A directed digging action opens the paired mandibles on one stroke and closes
them on the next. Closing against adjacent tissue can remove up to two material
units. Jaw motion costs one energy unit; successful excavation disposes one more.
Removed tissue becomes waste. An empty energy balance cannot operate the jaws.
Carving changes the real material field and contact topology; the smooth surface
is rebuilt when that field changes. Opening a route to food can provide delayed
feedback through the existing eligibility traces. Useful learned tunnelling has
not been evaluated; the focused test commands the action explicitly.

Material accounting is:

`initial + external_input = nutrient_field + flesh + reservoir + waste + external_output`

Energy accounting is:

`credited = live + maintenance_and_motors + disposed_by_death_birth_secretion_and_excavation`

## Controls and evidence

Click to capture mouse look; Escape releases it. WASD flies, Q/E descends/rises,
Shift accelerates, and the mouse wheel zooms the lens. F moves close to the
nearest living creature. The camera has no physical
presence. Space pauses, 1/2/3 toggle bodies/food/traces, M mutes, and brackets
adjust volume. Audio is a one-second faded call every eight seconds at a quiet
default level of 3/10. It is not simulation state.

`experiments/farm_smoke.json` is a small recorded test; `farm_256.json` matches
the full live configuration. See `FARM_RIG2_VALIDATION.json` for current local evidence;
`FARM_M1_VALIDATION.json` preserves the earlier five-joint results.
Current FARM2 snapshots preserve complete brain, rig, lineage, and flesh state and
restore deterministically. The current full articulated presentation is live;
recorded viewer playback shows lattice positions and accounting only.

This first slice supplies the means to move, feed, inherit, and alter the world.
Intentional building, an ecosystem of distinct functional species, stable
population dynamics, and evolutionary improvement remain questions for later
experiments. No such result is inferred from a visually interesting run.

## Genetics direction

The requested future genetic system must describe actual anatomy as well as
controller parameters: a compact versioned body plan with segment lengths,
branching/attachment, joints and limits, paired jaws and other effectors, plus
bounded sensory, metabolism, and secretion traits. It should decode into a
valid rig deterministically, with bounded mutation and recorded parentage.
Keep germline traits distinct from learned per-life state; make inheritance
policy explicit. The current prototype inherits learned controller weights
and mutates one, while the display genome is eight bytes and the rig topology
is fixed. It is not yet the requested body-generating genetic system.
