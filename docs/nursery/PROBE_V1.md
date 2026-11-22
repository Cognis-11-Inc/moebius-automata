# Nursery probe v1

Registered beforehand in [PROTOCOL.json](PROTOCOL.json). Executed once into
`runs/nursery_v1`, which is gitignored. The versioned table is
[artifacts/probe_v1_summary.json](artifacts/probe_v1_summary.json). Full
per-tick observations and the two final snapshots remain in that local run.
The registered protocol file was not edited after the run.

## Search

No papers were consulted for this slice. The food sites, mouth reach,
starting reserve, and birth timing are authored habitat choices. They are not
a published feeding model and they do not show that the controllers learned
to forage.

## What was run

`experiments/nursery_smoke.json`: a 32³ periodic grid, 12 founders, population
cap 96, nursery spacing 8, initial clot stock 2048. Eight seeds. 1024 ticks.
Two conditions. `provisioned` keeps the clot stock and the config inflow.
`food_absent` sets initial stock, inflow, and injection to zero and keeps the
same founders and body reserves. The command was:

```bash
cargo run -p moebius-core --release --example nursery_probe -- \
  experiments/nursery_smoke.json runs/nursery_v1
```

The probe process reported 16 trials and 0 failures. A failure here means the
trial returned an error. Extinction is recorded, not treated as a crash.

## Measurements

| Condition | Seeds | Births | Alive at tick 1024 | Intake |
| --- | --- | --- | --- | --- |
| provisioned | 8 | 36–199 | 5 of 8 trials; 0–96 cells | 720–20778 |
| food_absent | 8 | 0 | 0 of 8 trials; 12 dead in each | 0 |

Provisioned survivors reached generation 3–7. The three extinct provisioned
trials, seeds 5, 6, and 7, had already recorded 36 births each. Their final
generation is 0 because no brain was left. Every trial ended with material
residual 0 and action-energy residual 0.

`World::tick` audits linear momentum and refuses a nonzero residual. All 16
trials completed, so that check held on every committed tick. The probe does
not export the momentum vector itself.

## Evidence labels

- **IMPLEMENTED:** mouth-contact feeding, clot placement, nursery config, and
  the live clot mesh with an uptake filament.
- **VALIDATED as mechanism:** the oral-contact test places food at the mouth
  and one site farther away. Only the near site is consumed. The replay test
  checks that a provisioned founder can reproduce and that a restored snapshot
  matches. The production nursery cooldown remains 128 ticks. The replay
  fixture sets cooldown to 32 so a birth can occur in its window.
- **EXPERIMENTAL OBSERVATION:** the table above.
- **NOT VALIDATED:** learned foraging, hunger-driven motivation, adaptive
  evolution, distinct body kinds, or an emergent ecology.

The 256³ viewer habitat uses the same feeding law with wider spacing. This
probe did not run that 256³ config.

The bite in this probe was the earlier rich setting: 2 units, 16 energy each,
clot stock 2048. The live nursery is now one unit worth 4 energy, clot stock
512. These rows were not re-run after that change.
