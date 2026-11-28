# Hierarchical storage comparison — no new physical layout

Authoritative cells remain the existing BTreeMap<u64,Payload>. A derived Vec<Face>
cache is IMPLEMENTED, but it is not a contiguous hierarchy or a second physical
inventory. No Morton leaf array, SoA/AoSoA carrier, block payload or comparison
between competing atmospheric storage owners is implemented. Those experiments
are NOT RUN; no locality/bandwidth superiority is inferred from a cache flag.

Measured003 stage attribution preceded the cache change: refined shear32 monorate
mean0.528811ms; mean face construction305184ns,numerics176980ns,validation40698ns,
clone3968ns. This supports investigating reusable topology, not guessing cache misses.
004 cached/rebuilt operation order and like-model complete state/digest equivalence
are tested, including actual refine/coarsen invalidation and missing-neighbor refusal.
Three ordinary timing windows are registered for both first-order and higher-order
cases. Independent hardware array/irregular-access microbenchmarks do not validate
an atmospheric layout that was never implemented.

## Final measured qualification

| Scheme | Mean paired speed ratio | Each paired ratio | Cache bytes |
|---|---|---|---|
| Constant/Euler | 3.35789 | 3.37187, 3.34605, 3.35576 | 720896 |
| Minmod/SSP2 | 1.27795 | 1.27687, 1.28562, 1.27135 | 720896 |

Only derived face topology reuse was changed. Ordered-map versus packed payload, SoA, block-octree and recursive layouts remain NOT RUN; cache results do not rank those representations.
