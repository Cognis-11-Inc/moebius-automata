# Latest: raised opaque metal crowns

See [metal crown refinement](METAL_CROWNS.md). The translucent tint was rejected. Existing crown patches now have an opaque metal finish and a small rooted lift above the gums; the accepted closing targets are unchanged. Earlier sections are historical.

# Latest refinement

See [upper lip and existing-tooth glaze](LIP_TEETH_REFINEMENT.md). Upper-lip travel is reduced55%; the original human-like tooth crowns have a dark translucent glaze. The sculpt and vertical closure remain intact. Three material parts share original geometry; there are no added teeth. Older sections below are historical.

# Current visual: restored sculpt and vertical rim closure

See [sculpt repair](SCULPT_REPAIR.md). V3 lip plates were rejected and removed. The accepted pre-v3 mesh is restored exactly; its existing lips now use localized upper/lower shape keys. Silver paint and throat/tongue redesign remain deferred. The sections below record earlier revisions; their v3 appearance claims are historical.

# Worm_A — flesh, subtle cuticle and an articulated mouth

The user's root `Worm_A.glb` is preserved unchanged. The live reference creature now uses its derived rigged model by default: supple painted tissue, thin muted cuticle ridges and the original recognizable mouth. The rear hood is reprofiled into a narrower worm contour, with a separate flexible tissue layer beneath it. This is a render/model change, not a new species, anatomy, motor controller or physics law.

Production assets:

- `assets/creatures/worm_a/Worm_A_rigged.blend`: editable neutral Blender rig,14 named bones, skin weights, two painted surfaces, original embedded texture.
- `assets/creatures/worm_a/Worm_A_rigged.glb`: runtime skin,17325 sculpt vertices plus1560 tissue vertices; no baked animation clips.
- `assets/creatures/worm_a/rig.json`: source/output hashes, binding positions, jaw mapping and authored mouth landmark.
- `Worm_A.glb`: original source. `Worm_A_collision.glb`: preserved supplied proxy, not activated as simulation collision. The new visual contour does not constitute a validated physical collider.

## Inspect locally

```sh
cargo run --release -p moebius-viewer -- --motor-nursery experiments/motor_land_demo.json --motor-control selected --mouth-view
```

The mouth view follows the actual head-joint orientation and keeps native movement running. Space pauses/resumes movement. V toggles the mouth/body view. It stays open. `--creature-skin procedural` selects the historical visual. Recorded legacy Reel playback retains its old visual path; the new skin is the live neural reference rig.

`--capture-frame NEW.png --capture-tick 16 --capture-keep-open` records the close-up while leaving it open. Capture mode suppresses manual camera input; normal viewing controls remain available.

## Binding and mouth use

The existing12 spinal joints and2 jaws remain authoritative. Meshes use their native bend/twist/jaw angles; there is no authored swimming animation. The mouth landmark maps to spinal joint9, which is the actual articulated feeding anchor in `Body::head_site`, rather than the last spinal tip. Reference bind translations use the core's exact297/1024-site step and-1720/1024-site start at the viewer scale. Float rendering approximates integer kinematics; four sampled bent/twisted poses have mouth-anchor error below0.004 viewer units.

The mouth rim has left/right jaw influences, with closing/opening directions tested. The skull is not a separate rigid shell. No procedural replacement mandibles cover the sculpt. A small ingestion indicator inside the mouth is hidden unless real intake was recorded; it does not animate the skin or generate feeding. Positive/negative tests place food at the articulated mouth or four sites away: only the reachable mouth fixture feeds, with closed material/energy ledgers. The existing rounded-head feeding stencil is unchanged; triangle-level biting and soft-tissue mechanics are not implemented.

Painting is local procedural vertex painting: living red tissue, darker oral recesses and muted bone only on raised convex ridges. It retains source UV/image data and does not use image-generation or remote services. The underlayer uses a smoother multi-bone blend; source skin has a continuous mouth/neck binding. Both surfaces follow actual simulation joints.

## Honest revision history and validation

The first linear binding had an abrupt head-weight switch and produced long strips. A rigid-shell revision removed much of that distortion but did not match the user's requested worm-like flesh. Both were rejected and preserved. The final revision reprofiles the rear, adds tissue, paints the surfaces and keeps soft bending. Several neck-weight variants were evaluated visually and against six recorded native poses; all reports and retained prototype assets remain external.

An independent linear-skin edge audit on those six poses measured maximum elongation43.07× for the rejected first binding versus3.28× for the final model; the worst per-pose99th percentile fell from4.54× to2.09×. These are visual engineering measurements on a small pose sample, not a claim that the skin never stretches. Extreme joint states can still compress/stretch or self-intersect. No volume-preserving soft-body solver or new mechanical constraints were installed, and the visual is not evidence of better propulsion.

107 workspace tests pass, including new mouth-anchor, jaw-direction and actual-ingestion checks. Strict Clippy and formatting pass. Source, studio neutral/open/closed/bent frames, live comparison and head-following mouth captures were inspected; the initial view that followed only body orientation exposed the wrong side of a strongly bent head and is retained as a failed witness. The corrected view follows the accumulated head frame. Live appearance changes preserve the same simulation config/state hashes as the earlier demo.

**Swimming status:** M6 established sustained paid propulsion for one genetically selected recurrent controller under the existing authored laws. It did not establish active steering or useful individual lifetime motor learning. This rig/paint update changes none of those verdicts.

Build/check locally:

```sh
python3 scripts/rig_worm_a.py
blender -b -t 4 --python scripts/preview_worm_a.py -- NEW_PREVIEW_DIRECTORY
python3 scripts/check_worm_a.py RESTORED_M6/heldout-v1/episodes.jsonl.gz
python3 python/worm_a_artifacts.py verify
python3 python/worm_a_artifacts.py unpack NEW_RESTORE_DIRECTORY
```

The original source and final authoring/runtime assets belong in Git. Bulky previews, prototype models, rejected bindings, authoring backups and logs are classified separately and stored in immutable Mermogon archives with archive/member/restored hashes. No prior M6 or cathedral evidence was overwritten. Source identities in those earlier slices remain historical. No commit/push was made.

## Moving-mouth correction

The first mouth-view revision accidentally paused native ticking at tick16; keep-open screenshots also paused it. Both automatic pauses are removed. See [jaw-motion evidence](JAW_MOTION_FIX.md). `--record-mouth NEW_DIRECTORY` records16 native rendered frames and metadata without commanding or pausing the organism. The food-free demonstration eventually dies; the HUD reports that explicitly.

## Full closure, silver teeth and recessed throat

See [the mouth closure correction](MOUTH_CLOSURE.md). The asset now has closing lips, silver hooked teeth, a deep throat and a separate tongue. Add `--jaw-exercise` to the command above for an explicitly labelled close/open motor diagnostic. The native controller still does not fully close both jaws in the sampled run; no autonomous chewing or larger physical feeding capacity is claimed.
