# Post-ruling transfer update

Operator chose option2. Independent world ownership, immutable observation,
bounded snapshot publication and explicit intervention admission have now been
implemented and tested locally in runtime.rs/witness.rs and the native witness.
No donor code or assets imported. These tests validate stated local contracts,
not donor behavior, ecological equivalence or biology. See MORPH_004_RESULTS.md.
The original consultation assessment below is preserved as a preimplementation
record; its transfer status describes that earlier point in chronology.

---

# MORPH-004 donor architecture assessment — 2026-11-23

CONSULTED only. No donor mechanism implemented or validated locally in this
slice. No proprietary code copied. Primary developer documentation unless
explicitly qualified below. This is an engineering architecture investigation,
not biological evidence. Search chronology/queries and source provenance are
in docs/research/2026-11-23-morph-004-runtime.json.

## Factorio

Kovarex, [Friday Facts70 — The smooth FPS](https://www.factorio.com/blog/post/fff-70),
2015-01-23: deterministic scripting needed stable iteration; expensive rendering
can skip frames while updates continue, while slow simulation reduces update
rate. [Official download](https://www.factorio.com/download) offers a Linux
server without graphics/audio. Rseding, [Friday Facts415](https://www.factorio.com/blog/post/fff-415),
2024-06-14: CPU-count-dependent chunk generation caused a determinism defect;
server auto-pause is a deliberate policy when players leave.

Reuse concept: explicit simulation ticks, bounded observation production,
headless state ownership. Adapt publication/backpressure to zero listeners.
Reject observer absence as an automatic pause trigger; reject assuming thread
parallelism preserves deterministic event order. Limits: proprietary simulation
source not inspected, binaries not executed, 2015 loop description is historical,
and no Factorio performance numbers are transferred to MŒBIUS.

## Zenith (Factorio visualizer, identity resolved)

The initially broad name search returned unrelated CUDA-fluid and robotics
projects. Those were disambiguation leads, not evidence for this visualizer.
`unjustodin/Zenith` redirects to [Zenith-3D-Team/Zenith](https://github.com/Zenith-3D-Team/Zenith).
Inspected source pinned at7779697b7e5f013f4a178b9b40065469f85a26b2:
[architecture](https://github.com/Zenith-3D-Team/Zenith/blob/7779697b7e5f013f4a178b9b40065469f85a26b2/ARCHITECTURE.md),
`mod/factorio-3d-bridge/control.lua`, `scripts/{world,transport}.lua`,
`client/scripts/net/bridge.gd`, and `docs/PROTOCOL.md`.

Factorio remains authoritative. The Lua bridge subscribes to nearby chunks,
sends snapshots and deltas; the Godot client mirrors them. Reliable numbered
updates and resync differ from replaceable live-state samples. The bridge's
on_tick order and work budgets bound export work. Input commands are separate
from downstream messages; the client can request actual gameplay actions.

Reuse ownership split, stable identities, resync and presentation-work budgets.
Adapt to immutable snapshots and explicit tick-targeted validated interventions.
Reject copying client inputs into owner state without admission; reject treating
this source inspection as an observer-neutrality experiment. Root LICENSE is
GPLv3; no code/assets/dependencies imported. Selected consulted source and license
are preserved externally with hashes. Neither Godot nor Factorio was run.

## Dwarf Fortress

[Bay12 Features](https://bay12games.com/dwarves/features.html), undated developer
page: persistent worlds and historical events span multiple games/roles.
[Tarn Adams in Dwarf Fortress Talk13](https://bay12games.com/media/df_talk_13_transcript.html)
discusses reconciling visited cities with summarized changes. The transcript
mixes current behavior and future intentions; it does not certify every described
feature as implemented, nor continuous identical per-creature dynamics off-screen.

Reuse persistence/identity and historical event records as requirements. Adapt
them to explicit tick/causal-event digests and snapshot continuation. Reject
assuming summarized history is equivalent to the same detailed ecology. No
proprietary source, executable or same-input distant/local comparison inspected.

## X-series

[Egosoft-hosted X Rebirth Combat guide](https://wiki.egosoft.com/X%20Rebirth%20Wiki/XR%20Manual%20and%20Guides/Player%20guides%20%28spoiler%20warning%29/Combat/?xpage=print),
last modified Daniel Turner2023-11-02: high-attention detailed combat versus
low-attention abstract approximation. The guide lists different fighter damage
bonuses and missile interactions in low attention. This is hosted community
manual evidence, not proprietary engine source or a developer-certified
quantitative outcome study. It concerns X Rebirth, not an unverified claim about
all X-series versions. Followed links/other engines were not assumed inspected.

Reuse explicit accounting of attention-dependent work. Reject switching causal
rules with the camera for MORPH-004. Rendering can simplify; any later numerical
approximation needs paired tests and explicit tolerance/outcome boundaries. No
low/high combat equivalence or measured outcome rates established here.

## Minecraft

Adrian Östergård, [Snapshot21w38a](https://www.minecraft.net/en-us/article/minecraft-snapshot-21w38a),
2021-09-23: render distance is separated from simulation distance, and entities
outside simulation distance are not updated. This is direct release-note evidence
for that snapshot; it is not a complete current-version code audit.

Reuse separate presentation controls. Reject distance-based suspension of
organisms/material. CPU savings from suspended activity cannot support a claim
of an unobserved living MŒBIUS world. No Minecraft source/runtime benchmark used.

## Locked Bevy0.20 inspection

Local Cargo.lock/cache, not an assumption based on latest online APIs:
`bevy_pbr-0.20.0/src/volumetric_fog/{mod.rs,render.rs}` contains
VolumetricFogPlugin and FogVolume extraction into RenderApp. The plugin does
not supply authoritative environmental exposure or heat. A bounded local fog
prototype and quality/cost measurements remain NOT_RUN pending lawful world
ownership. No ship-sized fog volume added, no GPU timings claimed.

## Transfer status

CONSULTED: the mechanisms above and local source boundaries.
IMPLEMENTED: none of these donor transfers in MORPH-004.
VALIDATED: none of MORPH-004's new runtime/Witness claims.
Existing regression tests are separately recorded; they cannot validate these
proposals. See MORPH_004_ARCHITECTURE.md for the unresolved convergence ruling.
