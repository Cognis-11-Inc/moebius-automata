Architecture convergence is unresolved.

# MORPH-004 — Investigation and containment result

2026-11-23. **BLOCKED for implementation.** The user directive authorizes
MORPH-004 and in-scope local commits, not pushing or an unspecified replacement
of canonical authority. The mandatory archaeology identified a missing
convergence decision. No runtime code was changed. This is not a successful
world-runtime slice and does not satisfy its success criterion.

Entry main `20f0febb0cdc8e613ae36dfc6ed1abe529994d52`. The only pre-existing
worktree change was the Codex rehydration note in HANDOFF.md. Current changes
are audit/research/results, provenance/archive tooling/metadata, Atlas reference
metadata, and continuity records. No local commit or push performed this pass.

## Precise blocker and reviewable ruling

See [authority audit](MORPH_004_ARCHITECTURE.md). core `World` already owns a
renderer-free canonical lattice simulation, its clock, organisms, material and
snapshots. MORPH-003 `MaterialBook` is expressly experiment-local, with its own
reservoir/parcel ledger, tick, identity, environment and persistence. AnatomySpec
is not core Anatomy/NeuralGenome. There is no validated bridge, common unit
mapping, stock-transfer rule, event order or composite snapshot contract.
The existing lab also discards preview backlog after4 steps and is not an
independent authoritative world clock. These facts are source-inspected, not
assumptions based on file names.

AGENTS.md section4 requires stopping implementation in the contested domain.
Choices presented to the operator, with consequences documented in the audit:

- Extend core World as canonical owner, explicitly authorizing the versioned
  free-space mode, unchanged-book delegation and anatomy-instance seam.
- Explicitly promote and extend the M3 substrate as the ship runtime, keeping
  the nursery's existing World/laws as a preserved separate experiment.
- Narrow004 to the existing lattice World; acknowledge that combined M3/ship
  world requirements are deferred rather than calling the full slice complete.

No observer, scheduler, IPC adapter or third state owner was created to bypass
this ruling. Scope-safe donor/source investigation, preserved tests and
interpretive Atlas updates proceeded while the ruling was requested.

## Work delivered

[MORPH_004_DONOR_ASSESSMENT.md](MORPH_004_DONOR_ASSESSMENT.md): primary developer
accounts for Factorio, pinned source for the intended Zenith Factorio visualizer,
Bay12 persistence/history documentation, Egosoft-hosted X Rebirth attention
comparison with its community-documentation limitation, and Mojang's explicit
simulation-distance suspension. Mechanisms to reuse/adapt/reject are separated.
No donor code/assets imported; no donor executable run. [Research record](docs/research/2026-11-23-morph-004-runtime.json)
contains announced search intent, actual queries, exact sources/dates, selected
source hashes, ambiguity chronology, transfer proposals and status boundaries.

The architecture report includes the functional archaeology matrix, exact
missing seams, witness/intervention/illumination/coordinate proposals, and an
existing-versus-proposed Rust/Bevy/Python/Lisp producer/consumer map. Locked
Bevy0.20 fog implementation was inspected locally. GPU performance is NOT_RUN.
No large-coordinate correctness result is inferred from i64 capacity.

Atlas: `/run/media/cognis/Stable Diffusion Drive/MOEBIUS_AUTOMATA_Atlas`, UUID
`8CA451CDA451BA82`. Added9 linked notes covering runtime authority, Witness,
interventions, hybrid stack, light ports, proposed30,000km/3,000km dimensions,
donors, validation limits and004 experiment. Existing subject indexes point to
them. Pre/post note snapshots are preserved independently. No source tree or
raw evidence was copied into the Atlas.

## Validation gates

| Gate | Verdict in this pass | Evidence / limitation |
|---|---|---|
| G1 runtime independence | EXISTING CAPABILITY VERIFIED IN SOURCE; combined004 BLOCKED | CLI run_batch calls World::tick without Bevy/GPU; no new combined runtime executed |
| G2 observer neutrality | NOT_RUN for004 | Existing M4 observation regression passed, but is not headless/stationary/moving sphere comparison |
| G3 boundary continuity | NOT_RUN | No Witness Sphere implemented |
| G4 temporal integrity | NOT_RUN for004 | Existing preview4-step backlog discard identified in source; no fix silently applied to M3 |
| G5 interventions | BLOCKED | Owner/admission/replay contract undecided; no UI mutation promoted to world authority |
| G6 state/persistence/replay | EXISTING TESTS PASS; integrated004 NOT_RUN | Core invariant, M4 observation/restore and morph persistence regressions; separate formats remain separate |
| G7 large coordinates | NOT_RUN | Proposed hierarchical frames, unit mapping unresolved; no dense ship allocation |
| G8 performance | NOT_RUN | No100/1000/10000-population, witness, crossing, snapshot, replay or GPU benchmark claimed |
| G9 regression | FOCUSED PASS | 43 existing tests; no runtime changes; full clean-commit workspace certification NOT_RUN this slice |
| G10 architectural provenance | AUDIT DELIVERED; implementation BLOCKED | Existing authorities/consumers inspected; no invented cross-language integration or new owner |

Honest-red for observation mutation, tick loss and new severe guards: NOT_RUN;
no new guard or runtime was implemented. Existing historical falsification is
not relabelled as004 evidence. No numerical/ecological approximation validated.

## Executed commands and outcomes

rustc1.99.1 (`58cf8f9c3`2026-10-13), existing local dependencies, offline tests.
[VALIDATION.json](docs/morph_004/VALIDATION.json) records commands/counts.

```
cargo test -p moebius-core --test invariants --offline
# exit0,13 existing tests
cargo test -p moebius-morph -p moebius-morph-lab --offline -- --test-threads=8
# exit0,29 existing tests
cargo test -p moebius-core --test m4 observation_preserves_transitions_accounts_deaths_births_and_restore --offline
# exit0,1 existing test
python3 python/morph_003_artifacts.py verify
# PASS, existing archive1/file1; previous evidence unchanged
python3 python/morph_002_atlas.py
# PASS after note updates,39 notes; link count recorded in ATLAS_VALIDATION.json
python3 python/morph_004_artifacts.py verify
# PASS,4 archives/83 files
```

Git runtime diff is empty. Source identities under crates/ are recorded in
[SOURCE_IDENTITY.json](docs/morph_004/SOURCE_IDENTITY.json). No full suite was
run against the dirty documentation tree and labelled clean certification.
The136-test result in the previous handoff remains the historical a8cb705 run.

## Evidence storage and restoration

Before transfer, mount and write access verified on Mermogon UUID
`D0A02EEBA02ED82C`. [Classification](docs/morph_004/ARTIFACT_CLASSIFICATION.json)
records purpose/count/bytes before packaging. New immutable investigation root:

`/run/media/cognis/Mermogon/Moebius-Automata/experiments/MORPH-004/2026-11-23-investigation`

4 archives,83 raw files,481191 raw bytes:9 consulted source/license documents,
pre/post Atlas notes, regression and clone/Atlas-check logs. Existing M1–M3
archives and cache copies retained. Git holds reports, manifests and hashes,
not raw consulted-source snapshots/logs. [Storage manifest](docs/morph_004/ARTIFACT_STORAGE.json)
provides exact index/archive/member hashes. [Restore receipt](docs/morph_004/RESTORE_VERIFICATION.json)
records destination verification and complete83-file restoration.

```
python3 python/morph_004_artifacts.py verify
python3 python/morph_004_artifacts.py unpack NEW_DIRECTORY
```

The existing m6_artifacts verifier is reused; no separate archive format added.
Atlas flat archive members map to vault-relative paths through
ATLAS_SNAPSHOTS.json. Reconstruct only into a NEW vault, never overwrite living
notes. No standalone runtime reproduction is claimed:004 has no implementation.

## Next dependency

Operator convergence ruling, then a narrow recorded contract for owner, clock,
identity, unit/region mapping, disjoint/transferable material and persistence.
Only then implement one lawful runtime/Witness/intervention path, falsify it,
measure it, archive results, make an authorized local commit, and run the full
suite on that clean recorded source state. No push without separate authorization.
Do not begin MORPH-005. No propulsion, metabolism, biological intelligence,
hull exchange, radiative solver, floor or full biosphere was introduced.
