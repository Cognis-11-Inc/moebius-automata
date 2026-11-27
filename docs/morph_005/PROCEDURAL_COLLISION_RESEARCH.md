# Procedural collision research — VII

**IMPLEMENTED isolated alternative; REJECTED for canonical adoption/exactness.**
No external paper was consulted. Published science is not claimed: this is an
authored falsifiable mathematical prototype using locally inspected structures.

## Lane A: existing exact acceleration

Paid Hermite nodes, interpolated deposit ages, elliptical inner/outer rings,
ridges/wobble, seals, caps and nozzle ID remain authoritative. Existing
chunk_mesh_radius_bound, pipe_separated and the spatial horizon already exclude
chunks before constructing meshes. Their bounds explicitly include every sampled
outer/inner ring and displaced cap centre with slack; existing sphere coverage,
wall-vertex, horizon and mutation tests remain. No additional unproved bound or
new third acceleration tree was inserted. Exact selective leaf generation must
preserve oriented pseudo-normal adjacency, cap/seam topology and global query
ties; the source audit did not establish that seam. New exact acceleration: NONE.

## Lane B: independent physical representation

The feature-gated prototype stores piecewise circular annular cylinders from
node positions, section radii, age and end-seal inputs. It computes a scalar
sphere/surface distance. It intentionally lacks Hermite curvature, ellipses,
ridges/wobble, interpolated per-ring ages and canonical cap construction. Shared
segments are a union-like minimum-distance approximation, not structural junction
mechanics. Synthetic control recipes derive from a retained paid pipe but are
explicitly unpaid isolated geometry inputs; they never enter a live ledger/world.

Ten controls cover straight/curved, open/sealed, age-gradient and neighbouring
segment geometry, each wet/dry. Each has1400 static sphere probes, including lumen,
inner wall, outer wall, openings and caps. Canonical reference is the actual
oriented TriMesh contact kernel, radius0.01m sphere, prediction10m. No point
classification is inferred from appearance. Aggregate:116 false-negative and1624
false-positive contacts out of14000 queries. These are probe-grid counts, not
population rates. Distances deviate by up to0.080029m. Raw exact points/normals
and all distance/classification mismatches are retained.

| Control | False negative | False positive | Max distance error m |
|---|---:|---:|---:|
| straight-open-age0 | 0 | 258 | 0.042399 |
| straight-open-age256 | 4 | 12 | 0.017299 |
| curved-open-age0 | 31 | 249 | 0.055057 |
| curved-open-age256 | 21 | 125 | 0.047246 |
| sealed-age0 | 0 | 260 | 0.080029 |
| sealed-age256 | 4 | 12 | 0.076007 |
| age-gradient-age0 | 0 | 322 | 0.045931 |
| age-gradient-age256 | 4 | 12 | 0.017299 |
| junction-neighbour-age0 | 31 | 249 | 0.055057 |
| junction-neighbour-age256 | 21 | 125 | 0.047246 |

Construction/query timings and requested/retained allocation deltas are in
PERFORMANCE_STABILIZATION_VII_PROCEDURAL.json. The scalar model is cheaper in this
assay but does not compute the same geometry/result. No exact economy claim uses
those timings. Continuous TOI/contact-point/normal deviations are NOT RUN / NOT
SUPPORTED by this scalar prototype; general junction behaviour and end-to-end
traversal are NOT VALIDATED. The neighbouring-segment recipe is not a multi-pipe
junction experiment. This is the smallest falsification slice, not a complete
alternative collision engine or full fidelity qualification.

Any future adoption would revise the physical surface law, preserve finite paid
nodes only through an explicit ledger compatibility argument, and reassess contact,
adhesion, opening clearance, snapshots, source-bound history and migration. No
migration/adoption law is proposed as already implemented. Operator approval and
a separately versioned law remain indispensable. No canonical biology is changed.
