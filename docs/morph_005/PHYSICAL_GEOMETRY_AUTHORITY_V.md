# Physical geometry authority and incremental maintenance — V

2026-11-26, Codex. Source audit, not a changed physical contract. Canonical
constructors remain `filament_chunk_at` and `CollisionCache::{body,pipe_in}`.
No property below was removed or reassigned to presentation.

| Property | Verified source and physical consumer | Material/connectivity relationship | Classification and remaining uncertainty |
|---|---|---|---|
| Cubic centreline | `filament_chunk_at`: Hermite position/derivative using neighbouring nodes; all hollow-mesh queries consume vertices | paid ordered nodes supply anchors; curvature does not create ledger units | physical occupancy/contact; simplifying the curve changes clearance; no force solver inferred |
| Elliptical sections | outer and inner rings use second radius `0.94 * radius` | book quantities do not integrate tessellated wall volume | physical outer/inner contact and clearance; no equality between mesh volume and ledger volume claimed |
| Ridges | outer radius multiplied by `1 + 0.065*sin(phase*2.1)^6` | no separate material debit per ridge | physical outer contact; not presentation-only; future perception unresolved |
| Nozzle-dependent radius wobble | outer radius factor `1 + 0.035*sin(phase*0.37 + nozzle)` | nozzle is a geometry-key input | physical outer occupancy, including contact/admission; sensory role unimplemented |
| Deposit age/radius | interpolated deposit time; `PipeSection::radii_at` scales both radii from 1 to 0.7 over 192 ticks | drying changes geometry without deleting paid units | physical outer/inner contact and interior clearance; not a mechanical curing law |
| Hollow interior | reversed inner bridges and oriented mesh; `geometry` uses `TriMeshFlags::ORIENTED` | `Assembly::lumen_connected` checks open nodes on the same filament | interior surface is physical; traversing an entire lumen is not demonstrated by this audit |
| Start/end seals and caps | displaced inner cap centre, outer/inner fans and endpoint bridges | seals deny open-node connectivity | physical closures and contact; dropping caps/ORIENTED flags is not an exact optimization |
| Structural junctions | `Assembly::{deposit,validate}`, paid node pairs within bounded gap | no new bridge material; no cross-filament lumen passage admitted | graph connectivity is separate from mesh union; general load-bearing/adhesion forces are not implemented by these records |
| Colour/UV/normal presentation attributes | ring colour varies with wetness; physical `geometry` reads positions/indices and computes collision pseudo-normals itself | no ledger mutation | mesh colour/UV are presentation attributes; visual shading normals are distinct from physical pseudo-normals |
| Sac attachment/root | `habitat::{nearest_anchor,anchor,prepare}`, `collision::{root_geometry,admit_roots}` | attachment to existing paid nodes, physical root/graft checks | actual attachment/contact consumer; not evidence of a general adhesive-force or structural equilibrium solver |

Future biological perception remains CONTRACT ONLY in SENSORY_ARCHITECTURE.md.
No missing sensory/force consumer is used to declare geometry expendable.
Material conservation currently checks paid discrete stocks, binding and graph
provenance; it is not a mesh-volume conservation equation.

## Exact maintenance assessment (Lane A)

For edge k, positions/tangents require nodes k-1 through k+2, clamped at ends;
age needs its two deposit ticks plus filament creation time and query tick.
Origin is the chunk's first node. Moving that origin can change every local
floating-point coordinate in the chunk. Section radii/seals and nozzle affect
many rings. Appending changes endpoint tangent/lookahead and final-chunk status.
Chewing can remove nodes and junctions. A single-node edit is therefore not a
license to retain all neighbouring chunk acceleration.

Existing `pipe_key` hashes the one-neighbour span, section, nozzle, creation,
endpoint status and wet absolute mesh time. A fully dry unchanged chunk reuses
the existing immutable mesh; wet meshes continue to change. Horizon radius
supersets are independent from exact-mesh invalidation. Preserve the IV mutation
receipt that detects same-tick middle-node changes.

PROPOSED reuse: cache centreline/frame/topology work for unchanged local inputs,
then regenerate the exact ring positions in the original arithmetic order;
reuse topology only when node count, seam/cap status and ordering match. Such
reuse does not eliminate acceleration/pseudo-normal reconstruction. A proposed
immutable segment cache needs measured duplicate-build attribution and exact
whole-chunk witnesses before changing ownership. It must remain within the
existing collision cache, with bounded retention and complete revision keys.

Verified local dependency: Parry 0.27.0 `TriMesh` exposes immutable `vertices()`,
`indices()`, `bvh()`, rigid `transform_vertices`, and consuming `scaled`.
`rebuild_bvh` is private. No public arbitrary vertex update/refit API was found
in the inspected implementation. Rigid transforms or uniform scaling cannot
express independently ageing curved rings; they also need bit-equivalence proof.
No unsafe access, library patch, flag removal or approximate collider was used.

## Alternative representations (Lane B, design review only)

| Candidate | Potential economy | Physical differences / required proof |
|---|---|---|
| Separate physical and presentation tessellation | fewer authoritative triangles | changing current tessellation changes occupancy; needs separate versioned law and migration, even if appearance is retained |
| Procedural hollow sections | analytical local contact | must preserve both walls, ellipse, curvature, ridges, wobble, caps and junction semantics; circular tubes do not establish equivalence |
| Hierarchical curved segments/adaptive tessellation | localized candidate/refinement work | conservative bounds may be Lane A; substituted surfaces are Lane B; require error and missed-contact bounds |
| Conservative bounds + exact local mesh on demand | prune empty work, avoid unused mesh construction | retain exact final kernels, prove bounds include openings/caps/inner walls and preserve query tie/order rules |
| Persistent per-segment acceleration | amortize unchanged regions | prove global contact traversal and tie order; Parry mesh traversal does not directly supply this interface |

No alternative collider prototype was selected or benchmarked in V. Geometry,
normal, contact-point, TOI and clearance deviation distributions: **NOT RUN**.
Material/adhesion interaction compatibility: **NOT VALIDATED**. No alternative
results enter the exact throughput matrix. Replacement still needs explicit
operator approval and a separate versioned contract; this report is no approval.
