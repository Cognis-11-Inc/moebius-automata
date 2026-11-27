# HABITAT-001 — The Primordial Medium

2026-11-27. **PARTIAL implementation checkpoint; clean-source final qualification
pending.** This is a separate explicitly selected HABITAT experiment, not a
replacement for canonical ship admission. MORPH-005 remains PARTIAL and its
performance programme is paused; MORPH-006 remains suspended.

## Implemented and contained

The gated `moebius_morph::primordial::Chamber` owns a finite3D isothermal D3Q19
carrier, conservative passive species, deposited wall inventories and inert
spheres. Bulk density and viscosity are independent SI inputs. Weak gravity,
buoyancy approximation, exact reduced-mass drag coupling, finite carrier reaction,
sealed bounceback walls, scalar advection/diffusion and source-bound bit-exact
checkpoints have executable consumers. The Bevy `habitat-witness` and headless
`habitat_environment` invoke the same numerical owner. There are no organisms,
actuation, chemistry, thermal convection, matter intake or ship runtime consumers.

[Reuse audit](ENVIRONMENTAL_REUSE_AUDIT.md),
[architecture](HABITAT_ENVIRONMENT_ARCHITECTURE.md) and
[physical contract](ENVIRONMENTAL_PHYSICS_CONTRACT.md) identify ownership, existing
contracts, units, models, discretization, validity limits, closures and omissions.
The inherited Environment/MaterialBook and nursery rig are preserved. Existing
Position/tick/atomic persistence/source-identity conventions are reused; no ship
material transfer or history migration is installed. Local SI atmospheric
inventories have no unauthorized mapping to paid secretion quanta.

## Prequalification evidence

Eleven focused tests pass. Opposite reaction omission, source-check omission and
premature mutation on a rejected step each fail an explicit assertion (exit101),
then pass after source restoration. Strict workspace/all-target/all-feature Clippy
and formatting pass after retained initial parser/style failures.

The first ordinary/counting experimental suites ran source-identified binaries
and replayed their physical records. A16³ two-species/nine-object demonstration
averaged1.082ms/update with empiricalP991.110ms in120 observations. The native
Witness reached the identical physical digest. These are preliminary captured
source results, not the later clean-source certificate or a production recommendation.

Scalar and shear spatial errors decrease over8/16/32 cells per edge. The coarse8³
shear result has roughly5.8–6.0percent error and timestep sensitivity; it is not a
general accuracy certificate. Sparse scalar and dense-reference physical digests
match in all18 paired scaling settings. Dense carrier work still scales with grid
volume; sparse scalar fields do not establish sparse atmosphere scalability.

The initial Witness capture exposed missing font glyphs and HUD overlap. Both
original and corrected actual rendered captures are retained. The corrected view
uses legible SI labels, a separate control column, chamber/flow/concentration
samples and explicit-object markers. No aesthetic acceptance or human mouse test
is inferred from recording.

## Required final gates

A local source commit will precede the clean workspace regression and final
E01–E12 ordinary/counting suites/native capture. Final results must retain all
per-tick observations, errors, conservation residuals, convergence and factorial
costs. Closed-system fluid+explicit-body momentum is checked against external
gravity, prescribed hydrostatic support and walls; passive tracer momentum and
thermal energy are explicitly not included. No model is certified merely by a
velocity field existing or a reference paper being consulted.

Raw working copy: `/home/cognis/habitat-001-20261127`. Authoritative external
archive and restore instructions: `HABITAT-001_STORAGE.json` and later validation/
qualification supplement. Source, protocols, reports, compact tables and receipts
belong in Git; raw traces, snapshots, captures and binaries remain on verified
Mermogon. The plotting environment/target directories are regenerable caches,
not authoritative evidence. No previous source/data/archive is removed.
