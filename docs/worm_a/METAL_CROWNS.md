# Raised metal crowns on the existing dental patches — 2026-11-23

The user accepted the closing motion, rejected the translucent tooth treatment as paint on the mesh rather than anatomy, and requested a metallic finish with slightly more exposure above the gum. Closing is preserved exactly.

The v5 soft tint is removed. Existing crown regions are bounded around the picked source sites, mirror-deduplicated and restricted to the connected surface nearest each tip. The current candidate contains15 selected crown patches, from78 original source triangles. This is an authored surface selection, not a validated count or segmentation of biological teeth.

Those same source patches are lifted toward the opening by0.028 mesh units and forward by0.006; root collars connect them back to their original gum boundary. Maximum exposure is0.028636. The finished dental surface has290 triangles. Root positions remain at the original surface; source IDs for every finished vertex and cap face are recorded in rig.json. This deliberately changes only the localized crown surface; it is no longer just a material split. No generic cone dentition or new tooth sites are added. The finish is opaque brushed gunmetal: metallic0.86, roughness0.34, alpha1 and no transmission. Collar walls have separate normals so the crowns read as raised forms rather than stickers.

All original non-dental skin vertex attributes, including rest positions, UVs, colour, normals and skin weights, are byte-value identical to v5. Both closing position and normal targets are also exactly unchanged. Finished tooth vertices inherit their source vertex's skin weights and both closing targets. All source faces are accounted for as retained skin or projected crown caps; additional triangles form collars. Existing physical feeding and controller laws are unchanged. Visual raised teeth are not new collision surfaces.

## Evidence and limitations

29 viewer tests, strict workspace Clippy, formatting and diff checks pass. scripts/check_worm_teeth.py independently verifies source-face accounting, fixed skin, exact closing preservation, bounded exposure, root positions and opaque metal.16 actual live-render frames match controlled replay state hashes and jaw angles. Open/closed Blender and live pixels were inspected. Early shiny fragmented and flat-cap attempts are preserved with their source and renders. Current teeth remain angular because their shape follows the low-resolution source patches; polished final anatomy/user approval is NOT DEMONSTRATED.

```sh
cargo run --release -p moebius-viewer -- --motor-nursery experiments/motor_land_demo.json --motor-control selected --mouth-view --jaw-exercise
python3 scripts/check_worm_teeth.py RESTORED/mouth-v6-prechange/Worm_A_rigged.glb
python3 python/worm_a_artifacts.py verify
python3 python/worm_a_artifacts.py unpack NEW_DIRECTORY
```

Omit --jaw-exercise for native control. The finite-energy demonstration eventually dies. Source/authoring assets, report and reproduction metadata belong in Git scope; raw iterations, inspection renders and live evidence are immutable in Mermogon v6. Prior archives remain untouched. Work remains local and uncommitted; no push.
