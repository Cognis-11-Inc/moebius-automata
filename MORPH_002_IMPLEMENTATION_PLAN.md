# MORPH-002 implementation plan — 2026-11-23

Baseline: clean main at9171f0dada9e66c4e92dc548c6ed8a552f9ab1fa (MORPH-001 release).
Audit: WormSpec v1, CompiledWorm, parametric body/mouth, normalized surface
coordinates compiled to barycentric frames; Session transactional edits,
strict JSON persistence, independently reusable CPU geometry. Existing tests
and population/replay consumers are rerun before accepting extensions.

Keep v1 public interfaces and exact geometry available. Add a version2
AnatomySpec wrapping the existing body parameters with independently identified
components. Saved surface coordinates are anatomical definitions; triangle
bindings are recomputed every pose/topology change. Only trunk attachments
are supported; unknown/dangling parents rejected. Body and primary mouth have
reserved identities. No arbitrary secondary mouths claimed.

Add independent flagella, closed surface spines and recessed secretion nozzles
using one swept construction primitive. Nozzle output frame/metadata is
anatomical only. Preserve working mouth interior and positive closure seam.
Roots remain overlap-mounted, not welded; no global intersection guarantee.

Extend the native lab with component selection and parameter fields, add,
duplicate, delete and attachment controls. Keep legacy Session for baseline
consumers; native lab uses a compositional session. Remove default plane.
Keyboard selection first; no spatial gizmo claim. Transactional validation
before accepted state replacement. Save v2, accept v1 through explicit migration.

Gates: baseline117 tests/population800 poses/render replay; composition0/2/8/16;
100 varied definitions with animated poses, independent JSON/headless replay;
finite/winding/closure/root audits; repeated edit/save/reload and malformed
input protection; timings and vertex counts; rendered interactive walkthrough.
Record all failures and numerical limitations, not just final passes.

Atlas override: verified Stable Diffusion Drive UUID8CA451CDA451BA82, mounted
at /run/media/cognis/Stable Diffusion Drive. Inspected ZEITGEIST_Vault_Architecture
00_START_HERE architecture/status guide and99_TEMPLATES/Decision_Record.
Use numbered sections, explicit frontmatter statuses and wikilinks in separate
MOEBIUS_AUTOMATA_Atlas on that drive. No vault in source repository. Lightweight
location/provenance reference only. Evidence remains independently archived on
Mermogon UUID D0A02EEBA02ED82C. No commits/push authorized for this slice.

Status 2026-11-23, completion pass: the slice criterion is met in the dirty
working tree. Focused tests, honest-red, and the existing Mermogon archive
verification are recorded in MORPH_002_RESULTS.md. The archive was not
replaced. Clean-commit workspace certification remains blocked until a
commit is explicitly authorized.
