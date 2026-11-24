# MORPH-005 evidence restoration

Git retains source, reports, compact measurements and identities. Raw evidence
is independently versioned on Mermogon. The local cache is a working copy.

Run `python python/morph_005_artifacts.py verify` to check the mounted UUID,
external index, every archive and every member. Run
`python python/morph_005_artifacts.py unpack NEW_DIRECTORY` for a byte-verified
restoration into a new directory. Never overwrite an existing destination.
All original and superseded trials, failure logs and prior versions remain.

`ARTIFACT_CLASSIFICATION.json` inventories purpose/count/bytes before transfer;
`ARTIFACT_STORAGE.json` identifies immutable archive versions and SHA-256 hashes.
`RESTORE_VERIFICATION.json` records complete restoration. Later addenda extend
references without overwriting the earlier archives or their indexes.

For source reproduction, create a new checkout from Git revision
`d6fa1ac41ec73f7c6ab0372032d32fe6faf0c047`, then apply only the verified members
listed in `source-final/SOURCE_MAP.json` to their `repository_path` targets.
The overlay includes both morphology crates and root Cargo/AGENTS files.
`SOURCE_REBUILD.json` records all ten new tests passing in a newly restored
source directory, using this host's existing offline Cargo cache and shared
target. Fresh-machine or cross-platform reproduction is not established.

Atlas archives use flattened safe member names. `ATLAS_SNAPSHOTS.json` maps
them to original note paths. Restore into a new vault only; the current live
Atlas remains on the Stable Diffusion Drive. The full original baseline source
manifest is `logs-and-procedures/full-baseline.json`; its external restoration
was verified before replacing the broad Git copy with a scoped reference.

The final clean-HEAD certification is a local Git note, preserving the tested
main tree: `git notes --ref=morph-005-validation show HEAD`. The note includes
its own immutable final transcript/certificate archive and restoration receipt.
The original MORPH-005 certificate applies to its recorded historical commit.
The subsequent authorized publication includes that notes ref. For the new
feeding/constructor release, read `git notes --ref=first-simulation-validation
show HEAD`: its note identifies the exact clean tested commit, command, result
and independently hashed/restored certification archive. If the corresponding
note is absent, that certification is pending. Fetch notes refs explicitly in
other clones; main alone does not transfer them.


The 2026-11-24 camera/first-simulation diagnostic addendum is immutable v6:
`first-simulation-camera-v6`. Its six raw PNGs are external; classification,
hashes, member list and verified restoration are recorded in
`ADDENDUM_V6_CLASSIFICATION.json`, `ARTIFACT_STORAGE.json` and
`RESTORE_VERIFICATION.json`. Existing v1-v5 archives remain unchanged.
