# Neural foundation — measured results

112 trials; eight matched seeds per condition; zero failed trials.

Published studies are context. These results validate only the stated project mechanisms and observations.

| Environment / condition | Intake mean ± SD | COM distance, sites | Excavated units | Plastic delta L1 |
|---|---:|---:|---:|---:|
| foraging / plastic_on | 8.125 ± 0.354 | 0.1365 | 0.125 | 124.625 |
| foraging / plastic_off | 8.125 ± 0.354 | 0.1365 | 0.125 | 0.000 |
| foraging / random | 25.250 ± 37.625 | 2.9323 | 0.500 | 0.000 |
| foraging / recurrence_off | 8.000 ± 0.000 | 0.0000 | 0.000 | 0.000 |
| foraging / proprioception_off | 8.125 ± 0.354 | 0.1295 | 0.125 | 135.125 |
| foraging / chemistry_off | 8.125 ± 0.354 | 0.1365 | 0.125 | 130.000 |
| contact / plastic_on | 8.125 ± 0.354 | 0.1305 | 7.000 | 136.250 |
| contact / plastic_off | 8.125 ± 0.354 | 0.1305 | 7.000 | 0.000 |
| contact / random | 10.125 ± 2.900 | 1.6770 | 12.875 | 0.000 |
| contact / recurrence_off | 8.000 ± 0.000 | 0.0000 | 1.000 | 0.000 |
| contact / proprioception_off | 8.125 ± 0.354 | 0.2598 | 7.625 | 133.625 |
| contact / chemistry_off | 8.125 ± 0.354 | 0.1305 | 6.750 | 132.000 |
| lineage / plastic_on | 127.750 ± 29.227 | 2.7800 | 1.875 | 253.875 |
| lineage / mutation_off | 129.625 ± 33.649 | 3.2451 | 1.625 | 265.250 |

Plasticity changed synaptic state, but plastic-on and plastic-off had the same mean intake and endpoint energy in these trials. The random baseline obtained more food and displacement in the foraging environment. This does not establish learned locomotion.

Mutation produced 97 changed descendant genotypes among 97 measured descendants, with 94 different fresh-state command probes. Mutation-off produced zero changed genotypes/probes. All eight lineage runs in each condition ended extinct. This is heritable divergence, not demonstrated evolutionary adaptation.

## Uncertainty and controls

All seed values, sample standard deviations, paired differences and deterministic 95% percentile bootstrap intervals (2000 resamples) are in `analysis.json`. Identical initial state hashes and founder genotypes were checked across matched conditions. These small exploratory comparisons are not corrected for multiple testing.

All 3 saved final snapshots independently pass Python material, energy and linear-momentum checks. BLAKE3 is not independently recomputed by this script; Rust snapshot decode/replay tests cover hashing and continuation. All trials pass the core audits at every tick.

## Performance

| Initial population | ms / World tick, 16³ | Final population |
|---|---:|---:|
| 1 | 0.0140 | 1 |
| 16 | 0.1460 | 16 |
| 64 | 0.6007 | 64 |
| 192 | 1.8344 | 192 |
| 512 | 5.1193 | 512 |

Measurements include cloning, neural/actuator integration and integrity audits. Rendering, snapshot hashing and the dense 256³ world are excluded. No broader scalability guarantee is inferred.

## Limits

- 8 seeds; exploratory multiple comparisons; no retention/generalization transfer test
- mutation/common-garden probe measures commands, not all possible phenotype effects
- chemistry ablation removes named external chemical channels; feedstock readiness and intake remain
- recurrence knockout also changes dynamic response amplitude; not a gain-matched control
- no full articulated rigid-body, capsule contact or hydrodynamic solver; angular counter-rotation approximate
- effective inertia is not tracked biomass; mechanical energy is not the action-energy ledger
- local material supplies secretion; a stored internal secretion reservoir is not modeled
- microbenchmark on 16^3 excludes rendering/hashing; not a 256^3 scalability guarantee
