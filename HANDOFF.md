# Current checkpoint — 2026-11-21

Default live mode is Farm M1 in a 256³ periodic cube. See
[the laws and limitations](docs/FARM_M1.md) and
[local validation evidence](docs/FARM_M1_VALIDATION.json).

Start: `cargo run -p moebius-viewer --release -- --live`.
Click captures mouse look; Escape releases it. WASD moves, Q/E descends/rises,
Shift moves faster, wheel zooms; F moves close to the nearest living creature. Space pauses; M mutes; brackets set volume.

The core has an adaptive linear motor policy, 12 spinal joints and two mandible joints with separate front/rear commands,
fixed-point inertia, actual feeding feedback, energy-split asexual reproduction,
parent/generation state, bounded inherited controller mutation, and paid,
material-bearing permanent flesh. Face-adjacent deposits weld into connected tissue; components with four or
more units cause damped contact. Directed closing bites excavate up to two units
of tissue into waste for a paid energy cost. The viewer meshes one implicit
surface and opens it as material is removed.
The renderer skins the body from joint state. Wet fading trails are viewer
history, distinct from permanent material. Food clusters are block sums of
local nutrient, not physical food pellets. No trained locomotion or learned
construction result is claimed. Softness is appearance and damped contact,
not a volumetric mechanics solver. Full articulated presentation is live;
recorded playback is lattice/accounting inspection.

M0 remains the historical 64³ experiment with its original state/config hashes.
`farm` absence selects M0; `experiments/farm_256.json` and `farm_smoke.json`
select M1 explicitly. Current FARM2 extensions persist the expanded brain, bend/twist rig and flesh.
FARM1 five-joint files are explicitly rejected by the new loader; they remain
unchanged as historical evidence and require their original implementation.

Validation for the current rig: 52 workspace tests, strict Clippy, formatting,
and independent Python ledgers pass. Two small 200-tick recordings match hashes.
The 256³ 160-tick record has 64 births, 128 deaths, 128 survivors and 67 flesh
units. Tests exercise separate body sections, twisting, jaw opening/closing,
material-conserving fusion, paid excavation opening a passage, and zero-energy
refusal. These are capability tests, not learned-tunnelling evidence.
See `docs/FARM_RIG2_VALIDATION.json`; prior five-joint results remain preserved
in `docs/FARM_M1_VALIDATION.json`. The rig remains an approximate kinematic
propulsion model, not a full articulated or soft-body physics solver.

This M1 checkpoint was developed from 37787ae. Recorded runs were generated
before the checkpoint commit and echo that base revision; the validation reports
preserve this provenance limitation and source hashes. Those historical report
fields describe validation time, not current publication status. Read the current
Git log and verify the remote ref when establishing delivery status.

The user wants an unsettling biomechanical ant farm: refine the basic visual,
then make increasingly strange ecosystems observable. Future experiments should
measure whether controller adaptation helps locomotion and whether secretion
becomes useful building; do not infer either from an interesting picture.

Genetics requirement: a future compact, versioned genome must generate actual
anatomy (segments/attachments, dimensions, joints/limits, effectors) and bounded
controller/metabolism/secretion traits. Separate inherited germline from learned
per-life state. Current weight inheritance includes learned values; current
body topology is fixed. Do not describe this prototype as that genetic system.
