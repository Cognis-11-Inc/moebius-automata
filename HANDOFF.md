# Current checkpoint — nursery habitat (2026-11-21 local)

Start: `cargo run -p moebius-viewer --release -- --live --inspect`.
The default is the 256³ nursery: the neural farm plus fixed edible clots and
mouth-contact feeding. WASD/mouse fly, Q/E rise/fall, wheel zoom, F selects
and approaches, N toggles inspection, Tab cycles six pages. Space pauses; M
mutes. The viewer only observes committed ticks.

Every neural organism carries the same alien tissue genes: muscle follow,
ligament stiffness, and a resting curl. Nerves are the recurrent controller.
Lifetime motor synapses can change after a meal and are not inherited. Germline
mutation can change the tissue genes. That is a substrate for later evolution.
It is not a demonstration that a new body plan has evolved. The next species
is waiting on you.

One bite of the basic clot is 1 nutrient and 4 energy. Maintenance is 1, and
birth still costs a reserve of 256, so contact feeds them without turning a
clot into a feast. Founders start at 128. A clot holds 512. Smell reaches 8
sites and is silent beyond that.

A site crossing condenses 1 reservoir unit into flesh on the vacated site.
That thread stays. Four linked units still fuse into a soft wall. The green
ribbon is only the wet gloss on recent steps. The next species is not started.

Read [AGENTS.md](AGENTS.md) first: before searching scientific work, announce
intent; afterwards preserve sources, knowledge transfer, implementation trace
and separate CONSULTED / IMPLEMENTED / VALIDATED status under docs/research.
This is a mandatory notification/evidence rule, not a permission gate.

The starting published checkpoint was `32a6b030a521e2faa7d7848e5e6fc0beeb140806`.
Neural work is in the working tree; check live Git state before asserting commit,
push or CI. Historical reports retain their original base revision and results.

Read these current records:

- [Preimplementation audit](docs/neural/PHASE_0_AUDIT.md)
- [Authored architecture and explicit physical limits](docs/neural/ARCHITECTURE.md)
- [Protocol and sensor-defect amendment](docs/neural/STUDY_PROTOCOL.json)
- [Corrected 112-trial study](docs/neural/artifacts/main_v2/REPORT.md)
- [Full matched analysis](docs/neural/artifacts/main_v2/analysis.json)
- [Instrumentation replay and lineage audit](docs/neural/artifacts/main_v2/REPLAY_AUDIT.json)
- [Research provenance](docs/research/2026-11-21-neural-foundation.json)
- [Nursery protocol, registered before the probe](docs/nursery/PROTOCOL.json)
- [Nursery probe measurements](docs/nursery/PROBE_V1.md)

Eight recurrent neurons command 26 torque axes plus valve and local nozzle xyz.
Persistent bounded fixed-point state, physiological plasticity, and inherited
parameter mutation are distinct. Offspring reset learned state. No authored gait
or world-heading motor impulse runs in neural mode. Internal articulation cannot
translate the COM; secretion jets exchange momentum with the body. All bodies,
jets and the environment close an exact integer linear-momentum ledger. Material
includes in-flight droplets. Energy is action/metabolic accounting, not mechanical
energy conservation. Anatomical mapping is extensible in controller helpers;
world/rendering still require the reference 12-spine/two-jaw body.

Physical prerequisites remain partial: kinematic joints, approximate internal
counter-rotation, site contact/shared head jaw target, automatic site feeding,
no fluid solver/full rigid-body or soft-body mechanics, no biomass/inertial mass
stock, no internal secretion reservoir. Do not describe this as full embodiment.

The corrected study contains 112 trials, eight matched seeds, 256 ticks, zero
trial failures. Plasticity changed synapses but did not improve foraging intake
over plasticity-off. Random commands obtained more food. Mutation produced 97
changed measured descendant genomes and 94 changed fresh-state command probes;
mutation-off produced zero differences. Extinction was 8/8 with mutation and 7/8 without mutation.
These establish mechanisms and inherited divergence, not useful learning,
adaptive evolution, purposeful tunnelling, construction or ecological emergence.
The command probe does not integrate a body. No retention/generalization test
has established learned competence.

The first main dataset and pilot are preserved. A final review found acceleration
sensing omitted immediate recoil; correction and regression were recorded before
a separate corrected run. No goal/reward/genotype/scenario tuning followed nulls.
An instrumentation-only rerun reproduces all 112 corrected trial results except
wall time and exports auditable parent-child genomes/probes (257 records).

`cargo test --workspace` now passes 70 tests. Strict Clippy was not re-run.
M0 1000-tick acceptance remains
`facbb77bb992e4213416a9b4d81aaec29da38fff6f1a3a5832d26bd0e3562096`.
Two new neural 200-tick runs match
`a26fd626d752906c9e31f56b36f884c14b4c877e6a8f6138cf527374a4ffb3d0`;
independent Python material/energy checks pass. See current validation report for
additional artifact/replay/render evidence and its limits. FARM3 persists neural
state; FARM2/M0 retain historical laws/encoding; FARM1 remains intentionally refused.

The nursery is an authored habitat. `experiments/nursery_smoke.json` is 32³
with 12 founders. The registered probe is
[docs/nursery/PROTOCOL.json](docs/nursery/PROTOCOL.json). Its measured record is
[docs/nursery/PROBE_V1.md](docs/nursery/PROBE_V1.md). No new scientific papers
were consulted for this slice. Feeding is oral capture at distance-squared ≤ 2,
not a food-seeking policy. Production birth cooldown stays 128 ticks. A smaller
replay fixture uses 32 so a birth can occur inside its window.

On that 32³ probe, 1024 ticks, eight seeds: every provisioned trial reproduced
(36–199 births). Five of eight still had living cells at the end, and three
were extinct. Every food-absent trial had zero intake, zero births, and twelve
deaths. Material and action-energy residuals were 0. `World::tick` also refuses
a nonzero linear-momentum residual, and the probe reported no trial failures.
This is habitat mechanics. It is not learned foraging, motivation, or adaptive
evolution.

The user wants an unsettling biomechanical artificial-life ant farm, and will
introduce the next species. Future anatomical evolution needs a versioned
development genome and a complete mapping. The tissue genes are a start, not
that mapping. No result should be inferred from attractive motion. The 32³
probe above used the older bite, 2 units at 16 energy.
