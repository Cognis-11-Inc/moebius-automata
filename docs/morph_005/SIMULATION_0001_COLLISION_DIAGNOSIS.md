# SIMULATION-0001: trapped colony and authority lag

2026-11-24, America/Toronto. **Diagnosis VERIFIED; repair NOT IMPLEMENTED.**
Overall colony release remains **PARTIAL**. Entry main
`abdb664738a23c1aaf37c46810d26728f88c7d2f`; investigation and earlier
stabilization edits are uncommitted. No new physics representation was admitted.

## Reproduction and evidence boundary

The operator's screenshot shows tick 650/720, three Arachnitechs, one birth,
292 nodes, 32 Blood-Sacs, presentation about 60 Hz and authority tick rate 0.
It is the native session launched during this investigation:

```sh
target/release/moebius-world-witness --first-simulation --probe-world \
  --founders 2 --seed 11 --abundance 4000 --ticks 720 --mesh-budget 8 \
  --record NEW_DIRECTORY
```

The retained final snapshot is tick 720. Its initial-state replay matched exactly:
`b509ab9d302d3a6abe888558b582a8219562d895bdcc953cce7ce23a64b332bc`.
Parent 1 produced child 18 at event tick 360, quantity 64. This establishes
one paid natural birth in this window, not a viable or self-sustaining colony.

Native authority: mean **189.610 ms**, p95 **654.742 ms**, p99 **697.509 ms**,
maximum **2061.339 ms**. Retained presentation samples: mean draw CPU **2.458
ms**, mean frame interval **16.664 ms**. The 4096-frame recording cap was reached;
ticks continued through 720 and exact replay completed. Later presentation frames
are not included in those statistics. Other experiments ran concurrently, so
these are observed wall costs, not isolated comparative capacity certification.

## What actually stops them

At ticks 630–719, the same own-pipe contacts recur on every tick, together with
parent/child contact. A fresh restore of tick 720 followed by five real canonical
ticks reproduces two stationary bodies (8 and 18), while body 1 moves only a
few millimetres. Thus the frozen appearance is not solely a slow display.

Controlled interventions used the *same frozen snapshot and proposed motion*.
They excluded only selected contact classes in a temporary, explicitly passed
resolve filter. No altered world was published, saved as a valid colony, or
used as a product result. Root admission and other work remained present.

| Exclusion in resolve only | Resolve ms | Body 1 accepted displacement, mm | Body 8 | Body 18 |
|---|---:|---|---|---|
| None | 653.037 | (-2.519, 5.099, 1.566) | stopped | stopped |
| Own pipes | 134.241 | (-50.065, -24.664, 22.017) | stopped | stopped |
| Free parcels | 469.153 | (-2.519, 5.099, 1.566) | full proposed step | moved |
| Organism pairs and free parcels | 480.127 | (-2.075, 5.028, 2.514) | full proposed step | full proposed step |

**VERIFIED:** own pipes dominate this resolve's cost and heavily restrict body
1. They are not the sole trap. Body 8 is stopped by its own free parcel 35;
child 18 is also stopped by parent 1's parcel 133 and organism contacts.
Removing only own pipes does not release either body. Free parcels are stationary
2.5 cm collision balls in this suspended mode. Removing them is a diagnostic
intervention, not an acceptable repair or an excuse to discard material.

The cached tight hull already reports negative signed contact distances:
body 1 against its own pipes 7/8/9/10 about **-13.6/-13.5/-11.9/-8.2 cm**;
body 8 against own pipe 11 about **-9.3 cm**; parent/child about **-2.46 cm**.
These are Parry contact results for the authored envelopes, not measurements
of physical flesh. The separate visual-writhe mismatch cannot explain them away.

## Response defects

1. **VERIFIED wrong slide sign.** `collision.rs::along_surface` projects out
   the normal component when `travel.dot(normal) < 0`. Parry `normal1` points
   outward from the moving body toward the obstacle. Positive dot is inward.
   A focused test obtains that normal from two actual Parry balls. Proposed
   motion `(60, 10, 0)` mm into the obstacle remains `(60, 10, 0)` mm after
   sliding. The assertion fails: `slide preserved inward penetration` (exit 101).
   Escape motion is also projected incorrectly. This is a numerical sign defect,
   not a need to invent new biology or replace the hollow pipe representation.

2. **VERIFIED correction composition/recheck gap in source.** `resolve` captures
   `target = targets[&id]` once before the static-obstacle loop. Each pipe slide
   uses that stale target and overwrites `targets[id]`; constraints do not
   accumulate. The outer loop stops when the *number of stopped bodies* is
   unchanged, even if sliding changed a target. Consequently a corrected final
   path need not be swept again against the earlier obstacles. This is a
   source-level admission defect; the individual contribution to the accumulated
   penetration has not yet been isolated by a corrected full-history replay.

3. **VERIFIED no general recovery from an already overlapping start.** The
   swept detector reports contacts at time zero; response lacks a general
   separating-overlap recovery contract. `habitat.rs::prepare` also freezes
   yaw/roll after a blocked contact. These interactions help preserve a trapped
   state; their separate causal shares remain **UNRESOLVED**.

4. **VERIFIED permissive self-deposition seam.** `deposit_allowed` excludes the
   entire secreting organism from its body-overlap rejection. This is broader
   than exempting only the source outlet. Own-pipe braiding and admission through
   the body are different cases. Its contribution to this particular overlap
   history is **UNRESOLVED**, not proved merely by finding the condition.

## Why a stationary colony is expensive

Five canonical ticks after restoring the saved state take **486.7–525.7 ms**.
About **413–417 ms/tick** is in Parry distance evaluations and **32–34 ms/tick**
in contact construction. The body still attempts a roughly 60 mm step each
tick; rejecting it does not avoid the next tick's queries. Detailed hollow
trimeshes are repeatedly tested against compound anatomy in the same congested
configuration. The 512-visit bound is per cast, not a global tick budget.

The default resolve sampled 1267 interval visits. Removing only own pipes reduces
distance work from **417.236 ms to 106.359 ms** in that counterfactual. This
identifies expensive local geometry queries, not rendering, as the main stall.
Authored writhe animates independently of translation, making stationary bodies
look alive while the expensive authority thread keeps rejecting their steps.

## Preservation and next dependency

Temporary filtered-resolve hooks and saved-path tests were removed by restoring
the pre-forensics production source byte-identically. Their complete source,
logs, expected-red slide test, native snapshots and controls are retained in the
external receipt linked from `SIMULATION_0001_CAUSAL_STORAGE.json`. The filters
are not a product collision-off mode. No collision-response repair is claimed.

Next bounded repair: define separating-normal handling, compose/re-sweep final
candidate motion, test already touching/overlapping starts, and distinguish
source graft/self-deposit contact from arbitrary body penetration. Preserve
parcels and finite stock. Add regressions that fail on the demonstrated defects,
then replay this exact saved case and a fresh seed-11 history. A changed response
will change authoritative outcomes: preserve the old digest as a historical
receipt and document those differences; do not label it an equivalent cache
optimization. Only then assess performance again. Existing barrier tests and
the old clean suite do not certify these missing properties.
