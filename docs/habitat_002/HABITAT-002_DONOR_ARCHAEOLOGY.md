# HABITAT-002 donor archaeology

Consulted before new candidates; URLs, revisions, access scope, licenses and failed
accesses in [provenance](HABITAT-002_LICENSE_AND_PROVENANCE.json). No external solver
code imported. Local source/tests alone determine implemented/validated status.

| Donor | Inspected principle | Reuse ruling and mismatch |
|---|---|---|
| Noita / Purho / developer interview | Local dirty regions, chunk boundary scheduling | Independent block worklists only.2D particle update order and checkerboard ownership cannot replace our3D simultaneous flux/stream semantics. GDC video not watched; developer interview supplies detailed mechanism evidence. |
| tgstation Atmospherics, pinned revision | Active tiles, archived mixtures, excited-group equalization and reactivation | No gas equalization/threshold code reused. Our face order and non-equilibrium state prohibit a velocity-only sleep test. |
| NVIDIA Flow1.0.1; current Flow docs105.1.1 / pinned PhysX | Sparse voxel block pools, addressing, resource lifetime/versioning | Bounded blocks studied, no SDK/shader import. Effects-driven allocation is not a conservative proof; emitter visibility cannot govern atmospheric state. |
| FluidX3D | Repository README and restrictive license only | Implementation excluded completely. Separately licensed numerical paper is a theoretical reference. |
| Lehmann2022 published methods | In-place split streaming, orientation/parity | Own permutation-cycle method derived from existing push mapping, not Esoteric Pull/Push code. Canonical decoded orientation retained; no parity reinterpretation in checkpoints. |
| Tomczak/Szafran arXiv1611.02445v2 | Small contiguous tiles, occupancy/addressing overhead | Own block candidate, sizes4/8/16/32. Static solid-domain sparsity cannot justify dropping live equilibrium atmosphere. Published bandwidth results are not our measurements. |
| Oxygen Not Included / Stationeers | Official product metadata, targeted searches | Solver internals UNRESOLVED. No proprietary reconstruction, no gameplay-derived numerical claim. |
| Khronos OpenCL | Strict build options and event API | Minimal ctypes experiment using installed library; no permanent second Rust compute stack. |

These searches do not establish exhaustive historical/proprietary archaeology.
Flow guide.html was inaccessible; selected PDF sections, not entire papers, were
inspected. The research record retains exact access boundaries. License findings
are evidence of consulted terms, not authorization to import code.
