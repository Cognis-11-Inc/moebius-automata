# Authoritative tick dependencies — Stabilization V

2026-11-26, Codex. VERIFIED by source inspection at entry `4e3eea6`.
V adds diagnostic observers and an isolated query assay. It does not install a
parallel tick or transfer mutation rights. No external publications consulted.

Classification: **I** independently executable; **R** reads an immutable snapshot;
**D** needs deterministic ordered collection; **S** sequential causal mutation;
**U** integration not established. R does not imply that admission is independent.

| Concern | Path/symbol under `crates/moebius-morph/src` | Status; input -> output | Consumer | Authority | Tests / reuse ruling / missing seam |
|---|---|---|---|---|---|
| Tick transaction | `runtime.rs: ShipRuntime::tick` | active, S; committed state + ordered pending operations -> provisional `next`, then committed state | next tick, snapshot readers | sole world commit owner | runtime rollback/replay tests; retain transaction |
| Movement proposal | `habitat.rs: prepare` | active, S; provisional state, agents, sacs, food -> `BTreeMap<u64, ConstructorState>` and provisional updates | `collision::resolve` | proposal only until collision admission | habitat tests; cloning or per-agent parallelism alone does not establish independence of shared updates |
| Body geometry | `collision.rs: CollisionCache::body`, `geometry` | active, R/I per definition/quantity; posed anatomy -> immutable compound/hulls | casts, overlap, placement | derived geometry, no world mutation | cache/invalidation tests; reuse constructors, explicit preparation before workers |
| Pipe geometry | `filament.rs: filament_chunk_at`; `CollisionCache::pipe_in`, `pipe_key` | active, R/I per chunk; paid nodes, section, nozzle, tick -> exact oriented hollow mesh | casts, deposition, roots, placement | derived physical geometry | wet/future-node/dry/cache tests; topology and revision must remain authoritative inputs |
| Spatial candidates | `CollisionCache::{sync_pipe_horizon,interacting_chunks}`, `pipe_separated` | active, R/D after index maintenance; assembly revision + query reach -> ordered conservative candidates | five collision walks | exclusion only, never contact authority | same-tick mutation and horizon tests; preserve corrected receipt and conservative fallback |
| Body-body sweep | `collision.rs: resolve`, inner `a,b` loop | active, query R, admission S; starts + **current** targets -> hit or clear | target changes, contact events | existing response code | a previous pair can change either target; bulk results need exact pose receipt validation |
| Body-static sweep | `resolve`, `static_shapes` loop | active, query R, admission S; current body target + solids/pipes/roots/parcels -> hit | graft check, recovery, slide/freeze | existing response code | earlier static hit can change the next query target; prey-root contact can freeze another sac |
| Continuous query | `CollisionCache::cast`, private `cast` | active, R/I per fixed descriptor; geometry + four endpoint poses -> optional TOI/normal/point | admission | no direct state mutation | exact memo and fixture tests; immutable descriptor evaluation is the tested parallel seam |
| Source graft | `resolve`, `shapes[id].parts` casts | active, R/D; non-outlet components + same poses -> source allowed only if all clear | SourceAttachment vs Blocked | existing ordered short-circuit rule | doing every part concurrently may waste work; preserve first error/short-circuit semantics |
| Recovery | `nondeepening_recovery`, `cached_world_contact` | active, R/D for fixed samples; fixed orientations + five sample poses -> nondeepening predicate | continue vs slide/freeze | predicate does not itself move a body | existing recovery/workspace tests; monotonic comparisons stay ordered; witnesses need renewed pose/geometry validation |
| Collision admission | `resolve`, `record`, `along_surface`, final target application | active, S; hits + current solver state -> contacts, stopped set, plans, rigid poses | material/emission phase | canonical collision response | bounded passes and freeze fallback retained; no worker may write these structures |
| Emission/material exchange | `runtime.rs: tick`, `emission.rs: MaterialBook`, `collision::emission_obstacle` | active, S; admitted plans + finite stocks + paid emissions -> parcels/books/events | deposition, feeding, conservation guards | book and world owners | material/replay tests; physical snapshot alone does not make shared assembly/exchange independent |
| Structural deposition | `collision::deposit_allowed`, `Assembly::deposit` | active, S around R preflight; candidate paid parcel + current assembly -> admission, binding, nodes, junctions | later deposits, roots, next tick | book debit and Assembly mutation | paid/self-deposit tests; later operations see preceding mutations; candidate meshes are not authority |
| Feeding/reproduction/roots | `runtime::blood_tick`, `habitat::finish`, `collision::admit_roots` | active, S; updated material/world -> births, intake, roots, contacts | final guards and history | canonical world/book paths | habitat/root/conservation tests; reproduction placement sees earlier births |
| History | `tick` event-commit block; `history.rs: Store::append` opt-in | active v1 / opt-in v2, S; ordered deltas + checked state -> retained events, optionally sealed prefix | snapshots, validation, observers | runtime admits, explicit store persists | unchanged history fault tests; no unordered event merge or fingerprint bypass |
| Snapshot | `ShipRuntime::snapshot`, `WorldSnapshot::{save,digest}`, opt-in `save_checkpoint_v2` | active, R after commit; committed world -> clone/bytes/hash | Witness, recorder, restore | observer/persistence interface | snapshot/replay tests; ordinary snapshot cloning is separate from physical query workers |

The existing Witness `Worker` owns one numerical thread; Bevy rendering has its
own executor. Neither supplies a collision worker interface to the headless
crate. No Rayon dependency or global executor was introduced. The new
`collision_fixtures::batch::Batch` is supporting research behind the existing
`collision-fixture` feature, with no call site in `ShipRuntime`.

The assay constructs a bounded ordered descriptor array, authenticates snapshot,
radii, full collider fingerprints and pipe revision, then lends immutable
geometry to a persistent scoped pool. Each worker owns its contact/EPA/support
workspace. Atomic scheduling selects an index; results return through bounded
channels and are restored to that index before comparison. No numerical reduction
depends on arrival order. Contacts are cleared between queries to prevent repeat
rounds from becoming memo benchmarks. Worker count 1..32, descriptor count <=8192,
round count 1..1000; threads are created once per assay, never once per query.

The endurance capture contains 370 queries, 23 distinct body/target recipes and
42 consecutive pose groups. Some bodies have two or three targets. This is
positive evidence against treating the complete capture as one live immutable
admission wave. Grouping is an offline aid; it does not prove independence even
inside a group, because it lacks skipped candidates and solver decisions.

PROPOSED runtime seam: prepare bounded descriptors at an explicitly fixed solver
frontier; return receipts containing both geometry revisions and all four poses;
consume in existing order; discard stale receipts and execute the canonical
query at the current inputs; preserve lazy error visibility and early exit.
Measure discarded work, preparation, workspace duplication, synchronization and
native resource contention before accepting it. This is feasible architecture,
not an implemented or validated speculative runtime.

Inspected: entry rules/handoff, IV reports and retained builds, runtime, habitat,
collision, collision fixtures, support index, filament, history, Witness worker,
native Witness call sites, Cargo manifests/lock, local Parry 0.27 source. Only
main/origin/main exists locally; stash is empty. Remote unpublished branches,
all historical prototypes, GPU scheduling and every rendering task were not
audited. Completed IV locality/translation archaeology was not restarted.
