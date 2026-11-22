# Project instructions

## Experimental artifact storage and commit gate

Mandatory standing instruction from the user, effective 2026-11-22:

Before committing experimental results, classify generated artifacts by size
and purpose. Keep authoritative reports and reproducibility metadata in Git.
Store bulky raw evidence in a versioned external archive on **Mermogon**, with
verified hashes. Never discard or relocate evidence without verifying its
integrity and updating its references.

- Record artifact purpose, file count, byte size, experiment/run identity and
  archive version before deciding what belongs in Git. Packaging raw files into
  a tarball does not by itself make bulky evidence suitable for Git.
- Keep protocols, authoritative reports, analysis/replay summaries, source and
  configuration identities, manifests, checksums, archive locations and restore
  instructions in Git. Preserve failures, nulls and chronology.
- Put bulky tick traces, raw snapshots, search logs and equivalent raw evidence
  in immutable, versioned experiment archives on Mermogon. The local cache is a
  working copy, not the authoritative external archive.
- Before a transfer, verify that Mermogon is actually mounted and confirm its
  device identity; do not write into an unmounted directory or identify the
  drive solely by a transient `/dev/sdX` name.
- Copy to the external destination, verify archive and per-file hashes there,
  verify restoration, and update every affected manifest/report/restore reference
  before removing or relocating the previous copy. Retain the previous copy if
  any verification or reference update fails. Never overwrite an archive version.
- Before committing, verify that referenced external evidence exists, hashes
  match, and bulky raw evidence is absent from the proposed Git additions.

Mermogon was verified mounted at `/run/media/cognis/Mermogon` on 2026-11-22.
Its verified volume UUID is `D0A02EEBA02ED82C` (label `Mermogon`). Re-verify
its mount and identity at each archive operation. This gate applies to
the pending M5 results as well as future experiments; storage references must be
updated before those results are committed.

## External scientific research

This is a mandatory standing rule requested by the user for future work.

Before looking up studies, papers, or other scientists' work related to the
user's prompt, announce the research topic and explain why it matters to the
task. Do not conduct that scientific research silently. Announce further
research batches when their topic or purpose changes.

Afterward, identify and link the relevant sources, explain their pertinent
findings in plain language, and connect them to the design or experiment under
consideration. Explain useful concepts so the user can learn from the research,
including uncertainty and differences between the source and our implementation.

This is a notification and educational explanation requirement, not a
requirement to ask permission or stop already authorized work.

Distinguish published findings, authored simulation assumptions, and results
actually measured in this project. A reference does not validate our simulation
or demonstrate learning, locomotion, evolution, or emergent behaviour.

## Mandatory scientific research evidence record

Announce the search before conducting it, and preserve the evidence afterward.
Maintain a version-controlled research record under `docs/research/` containing:

1. Search intent: the scientific question, proposed use, and search queries.
2. Source provenance: exact links and verified bibliographic details of consulted
   publications; identify sources seen only through search metadata or abstracts.
3. Knowledge transfer: the particular claims or techniques that influenced a
   design decision, including deviations and authored assumptions.
4. Implementation trace: affected source symbols, files, tests, and experiments.
5. Evidence status: separately label CONSULTED, IMPLEMENTED, and VALIDATED;
   distinguish mechanism tests, experimental observations, nulls, and untested
   claims. Never promote consultation or implementation into empirical validation.

Record chronology honestly, including any search conducted before this rule was
introduced or any missing evidence. Preserve prior records and failed or null
results. Update implementation and validation links as work proceeds.
