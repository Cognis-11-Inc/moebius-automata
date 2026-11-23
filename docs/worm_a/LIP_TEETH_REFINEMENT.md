# Upper-lip restraint and existing-tooth glaze — 2026-11-23

The user accepted the restored vertical-mouth direction, requesting less upper-lip stretch and a dark translucent tint on the existing human-like teeth. This revision keeps that scope.

The actual upper shape-key vertical deltas are45% of v4: maximum0.082028 mesh units instead of0.182285, a55% reduction. Its forward displacement also decreases. The lower lip closes toward a higher0.36-unit seam to meet it. Transverse X displacement remains zero. Rest positions, UVs and all25168 original sculpt triangles remain identical in connectivity;1547 vertices move and15778 remain fixed. No new teeth, throat or tongue geometry is introduced.

The original tooth crowns are selected from explicit pixel picks in the inspected neutral render, projected onto the existing surface and mirrored. A smooth0.040-unit selection falloff replaces the first hard-edged tint attempt. Only221 existing triangles receive the enamel material; they reuse the sculpt's position, UV, skin and morph accessors. This is a material partition, not replacement dentition. Dark smoky violet-charcoal vertex colour and alpha(center0.62), roughness0.21, transmission0.30 and IOR1.45 give a translucent glaze while preserving the source texture. Source face IDs and canonical pick positions are in rig.json and scripts/worm_teeth.py. Fused sculpt/gum boundaries make this an authored feathered selection, not a claim of anatomical segmentation.

29 viewer tests, strict workspace Clippy, formatting and diff checks pass. The stale two-primitive assertion failed initially and was corrected to verify the third material part reuses original positions. No new low-impact colour-only tests were added. Independent asset audits verify the55% upper reduction, no horizontal movement, original geometry/UV preservation and material settings. Sixteen live frames exactly match the unchanged controlled physics replay. Blender and actual live open/closed pixels were inspected. This remains authored visual rigging, with no new feeding, autonomous chewing, learning or tissue-optics validation.

The earlier flat dark material attempt, failed test and all studio/live witnesses are retained in immutable v5 evidence. Previous archives remain historical. Current editable .blend, runtime GLB, selection/deformation scripts and authoritative metadata stay in Git scope; raw evidence is external on Mermogon. Work is uncommitted; no push.

```sh
cargo run --release -p moebius-viewer -- --motor-nursery experiments/motor_land_demo.json --motor-control selected --mouth-view --jaw-exercise
python3 scripts/check_worm_sculpt.py RESTORED/mouth-v5-prechange/Worm_A_rigged.glb
python3 python/worm_a_artifacts.py verify
python3 python/worm_a_artifacts.py unpack NEW_DIRECTORY
```

Omit --jaw-exercise for native control. Space pauses; V shows the body. This finite-energy demo eventually dies. No additional visual redesign is included.
