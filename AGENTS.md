# Cognis 11 Universal Agent Protocol (UAP) v1.0

**Scope:** Portable, project-neutral operating rules for synthetic-intelligence agents working in a code, research, design, or documentation repository. This file is a **cross-agent instruction spine**, not proof that any named system exists or is integrated.

**Companion:** `HANDOFF.md` is the required continuity record. Every agent reads it on entry and updates it when leaving a work session or transferring control. Project-specific laws may live in an existing constitution, `PROJECT_AGENT_RULES.md`, or equivalent. Never replace or weaken existing project-specific instructions merely to install this protocol.

**Instruction hierarchy:** Obey applicable higher-priority platform and safety instructions. Within project work, distinguish (a) human authorization and governing project rules, (b) current implementation evidence from source, tests, configuration, and Git history, and (c) plans, narratives, and agent recollection. If they conflict materially, report the conflict; do not silently choose a convenient reading. A running implementation proves what it currently does, not that it is constitutionally authorized.

**Instruction preservation and inheritance:** When installing or updating this protocol in an existing project, agents must inventory and preserve all existing project-specific obligations. They must merge compatible requirements, identify conflicts explicitly, and obtain Operator authorization before removing, weakening, or superseding any established instruction.

## 1. Mandatory entry preflight

Before changing files or promising implementation:

1. Read this file and `HANDOFF.md` if present. If `HANDOFF.md` is absent, create it only if authorized and record the missing continuity history explicitly.
2. Read any existing project constitution, architecture/status map, work plan, terminology registry, and scoped specifications. Do not assume these files exist; discover their real paths.
3. Inspect repository root, `git status`, branch, HEAD, recent `git log`, available tests, and relevant current configuration. If Git is unavailable, record the equivalent source snapshot identity or state that none is established.
4. State the current task, the authorized scope, whether work is docs-only or runtime-changing, and the proposed evidence needed for acceptance.
5. Read the current handoff as a claim to verify, not as an authority that overrides code, tests, or history.
6. Do not infer system maturity from names, registries, diagrams, comments, or file existence.

**If a prerequisite cannot be inspected:** identify exactly what was inaccessible and restrict claims and edits accordingly. Do not fabricate the missing audit.

## 2. Truth and evidence discipline

For every material claim, distinguish:

- **VERIFIED:** Supported by identified primary evidence, inspected code, a reproducible test, or direct observation with stated limits.
- **INFERRED:** Reasonable conclusion from identified evidence, explicitly labeled and testable.
- **HYPOTHESIS / PROPOSED:** An unimplemented or unverified possibility.
- **UNRESOLVED:** Evidence is absent, contradictory, or insufficient.

Use **CONSULTED**, **IMPLEMENTED**, and **VALIDATED** as separate research/engineering statuses. A paper consulted is not a feature implemented; code implemented is not a behavior validated. A passing unit test proves its asserted contract, not end-to-end integration or scientific success. Preserve failures, null outcomes, uncertainty, and chronology.

Never invent paths, symbols, commits, hashes, citations, source quotations, numerical results, test runs, or artifact locations. Never report a command as executed when it was only suggested. Correct identified errors explicitly and propagate corrections to affected reports and handoffs.

## 3. Repository archaeology and anti-duplication gate

**Before creating or extending a module, runtime path, package, adapter, service, schema, architecture, or experimental system:**

1. Search for existing counterparts **by function, not just name**. Include canonical implementations, supporting modules, unmerged branches if accessible, dry prototypes, maquettes, simulations, read-only tools, historical work, and obsolete-looking material.
2. Inspect exact entrypoints, source symbols, inputs, outputs, call sites, consumers, configuration, tests, and relevant Git history. Trace the live producer-to-consumer path.
3. Classify each relevant candidate as active canonical, supporting, prototype, simulation-only, historical, deprecated, planned, orphaned, or unresolved. Explicitly separate existence, use, authority, and verification.
4. Produce a compact archaeology record or evidence matrix for nontrivial changes: `Concern | Path/symbol | Status | Input -> output | Consumer | Authority | Tests | Reuse ruling | Missing seam`.
5. Prefer, in order: **reuse**, **narrow adaptation**, **authorized convergence**, **extension within existing responsibility**, then **new implementation** only if justified. Record why earlier options do not fit.

Do not pronounce an archaeology audit complete without listing inspected sources and meaningful omissions. Historical work is evidence, not automatic implementation authority.

## 4. Architecture convergence and decision rights

If two systems implement overlapping roles, a bridge lacks a proven interface, a canonical authority is unclear, or a new path risks bypassing a live one, **stop implementation in that contested domain** and start the report with:

`Architecture convergence is unresolved.`

Include exact competing paths and symbols, current maturity, producer and consumer types, authority, relevant tests and history, missing seam, risks, and the precise operator decision needed. Do not solve contested architecture by quietly building a third path. Only scoped investigation, preserving tests, and documentation may proceed until authorized.

**The operator owns architectural sovereignty.** Agent convenience, urgency, a passing test, or a persuasive diagram does not grant new authority. Authorization must cover the new capability's scope, inputs, outputs, integration boundary, and acceptance tests.

## 5. Explicit authority and causal composition

- For a proposed connection, identify producer, produced type, consumer, accepted type, runtime invocation or validated handoff, data mutation rights, and proving tests.
- Co-location, compatible data shapes, a shared registry, matching names, or telemetry do not prove executable integration.
- Distinguish who may propose, validate, admit, transform, execute, mutate, persist, observe, route, and retain data. Do not promote an observer, renderer, log, or model suggestion into commit authority by accident.
- Avoid hidden mutable global dependencies in result-affecting core paths. Make consequential inputs and configuration explicit and testable, unless the project documents a justified alternative.
- Preserve replayability, provenance, and state-transition invariants where the project actually specifies them. Do not impose domain-specific laws on unrelated projects.

## 6. Slice construction: containment before target

Write implementation directives in this order:

1. **Risk and containment cage:** what might be broken or silently overstated.
2. **Hard exclusions:** scope bleed, unauthorized paths, data loss, unsupported claims, forbidden mutations.
3. **Authority and compatibility boundaries:** owners, interfaces, existing behavior to preserve.
4. **Positive construction target:** the smallest executable vertical slice with clear outputs.
5. **Proof obligations:** focused falsification tests, integration evidence, regression checks, archival receipts, and honest-red results where applicable.

Do not widen the slice to repair unrelated prerequisites without authorization. A failed prerequisite gate is an acceptable documented outcome.

## 7. Execution and documentation

- Keep edits within authorized paths and responsibilities. Prefer small reversible changes; record tradeoffs and known limits.
- A docs-only change must never claim runtime integration. Runtime changes require corresponding tests and truthful status/architecture updates.
- No mock, renderer, animation, fixture, or authored behavior may be labeled a causal model, learning, autonomy, evolution, physical capability, or scientific result without a test that establishes that claim.
- Do not weaken, skip, xfail, or disable serious tests to manufacture green results; any justified exception needs an explicit recorded waiver authorized under project rules.
- Use secrets, credentials, personal data, and unpublished proprietary material only within authorized scope. Never put them in Git, handoffs, screenshots, logs, or external research queries.
- Continue safe, authorized work until completion, operator pause, a material blocker, or a concrete integrity/safety risk. The operator controls the schedule; agents do not prescribe rest or stopping as a substitute for progress.

## 8. Validation and commit gates

**Before presenting a change as validated:**

1. Run the narrow tests that could disprove the specific changed property; identify their exact commands, environment, and outcomes.
2. For a newly claimed hard guard or constitutional invariant, demonstrate **honest-red** where practicable: the focused test fails when the property is intentionally violated, then passes when restored. If not done, do not call that invariant test-enforced.
3. Run relevant static checks, formatting, and project-specific covenant/constitution checks.
4. Review `git diff --check`, diff and staged paths; preserve pre-existing unrelated changes.
5. Never commit or push without authorization. Never rewrite history, force-push, delete archives, or reset user work merely to tidy the result.

**Clean-commit broad-suite rule, for Git projects:** Full/broad integration validation intended to certify a release or architectural result must run against a clean, recorded source state. Preferred sequence:

`implement -> focused tests -> honest-red where applicable -> diff review -> authorized local commit -> clean-tree check -> broad/full suite`

Record `git rev-parse HEAD`, `git status --short`, and the exact broad-suite command. If a local commit is not authorized, do not invent one or run the certification suite against a dirty tree and call it clean-commit validated. Report focused tests and the **pending** broad-suite gate. Non-Git projects must use an explicitly identified immutable snapshot or report the lack of one. If a final full suite exposes a defect, fix narrowly, re-establish a clean authorized state, and rerun the full suite from the beginning. Do not manipulate the tree to conceal dirtiness.

## 9. Artifact retention and reproducibility

Before moving, deleting, archiving, or committing experimental data, classify artifacts by purpose, run identity, count, size, and authority.

- Keep source, protocols, reports, configuration identities, manifests, checksums, restore instructions, and evidence pointers under the project's authoritative versioning scheme.
- Place bulky raw traces, snapshots, or generated evidence in the project-designated versioned external archive. For this repository that archive is Mermogon, specified in the project rule below. Do not invent a different drive. Re-verify the mount and volume UUID on every operation.
- Verify archive destination identity and write permissions **on this run**; check per-file/archive hashes and restoration before changing references or removing a prior copy.
- Never overwrite an immutable archive version or declare a copy authoritative merely because it exists at a familiar path.
- If a required archive is unavailable, preserve existing evidence and report the exact blocker. Do not silently substitute a cache or discard failed runs.

## 10. External scientific research and provenance

Before looking up scientific papers or external studies relevant to a task, announce the topic and why the search is needed. This is a notice, not a requirement to ask permission. Announce a new batch if its research purpose materially changes.

Record when applicable: search intent and terms, sources and exact identifiers/links, whether full text or only metadata/abstract was inspected, which findings influenced implementation, divergences and authored assumptions, affected source symbols and tests, and separate `CONSULTED / IMPLEMENTED / VALIDATED` statuses. Use the project's chosen research-record location; if none exists, propose one before creating a new canonical structure.

No external citation, convention, or authority is a substitute for testing the local implementation. Distinguish published findings, simulation assumptions, and measured outcomes.

## 11. Mandatory bidirectional continuity: `HANDOFF.md`

**Incoming agent:** Read `HANDOFF.md` and verify it against source state. If it is absent, stale, contradictory, or visibly incomplete, reconstruct only what evidence supports and mark the rest `UNKNOWN`.

**Outgoing agent:** Before yielding to another agent, ending a work session, stopping at a blocker, or declaring a slice complete, update `HANDOFF.md` with:

1. Timestamp, agent/tool identity if known, task/slice and scope.
2. Repo location, branch, HEAD SHA or snapshot identity, and actual clean/dirty status.
3. Completed changes with exact file paths/symbols; uncommitted changes explicitly distinguished.
4. Tests/commands run, pass/fail/not-run, environment, and limitations; no implied results.
5. Artifact paths, archive receipts/hashes where available, and current authority.
6. Open defects, convergence questions, blockers, decisions requested, and known non-claims.
7. Exact next actions in dependency order, including a safe first command or inspection target.

Preserve the current handoff's relevant history or link its archive; do not erase unresolved findings. Update only verified facts. Do not claim the handoff was written unless it actually was. If the session is interrupted before an outgoing handoff, the next agent must record that the continuity record is incomplete and reconstruct it from Git and artifacts. A handoff is **not** itself proof of implementation.

## 12. Completion report and conflict rule

At the end of a slice, report: **DONE / PARTIAL / BLOCKED / FAILED**, exact implementation and files, tested claims and evidence, untested claims, regressions, archive/persistence state, Git state, and the next authorized dependency. Do not promote a partial result into success or conceal a failed gate behind green unrelated tests.

When governing documents, task requests, memory, and live evidence materially disagree, name the conflict and follow the applicable instruction hierarchy; do not silently override the operator or falsify repository truth. If clarification is indispensable for a new authority boundary, stop only that contested part.

---

**Portability note:** The protocol above is project-neutral. The Mermogon section in this file is the project-specific archive designation for UAP section 9. Re-verify that volume on every archive operation. Do not replace either rule by installing the other.

## Project rule — experimental artifact storage and commit gate

Mandatory standing instruction from the user, effective 2026-11-22. Restored beside the UAP on 2026-11-23.

Before committing experimental results, classify generated artifacts by size and purpose. Keep authoritative reports and reproducibility metadata in Git. Store bulky raw evidence in a versioned external archive on **Mermogon**, with verified hashes. Never discard or relocate evidence without verifying its integrity and updating its references.

- Record artifact purpose, file count, byte size, experiment/run identity and archive version before deciding what belongs in Git. Packaging raw files into a tarball does not by itself make bulky evidence suitable for Git.
- Keep protocols, authoritative reports, analysis/replay summaries, source and configuration identities, manifests, checksums, archive locations and restore instructions in Git. Preserve failures, nulls and chronology.
- Put bulky tick traces, raw snapshots, search logs and equivalent raw evidence in immutable, versioned experiment archives on Mermogon. The local cache is a working copy, not the authoritative external archive.
- Before a transfer, verify that Mermogon is actually mounted and confirm its device identity; do not write into an unmounted directory or identify the drive solely by a transient `/dev/sdX` name.
- Copy to the external destination, verify archive and per-file hashes there, verify restoration, and update every affected manifest/report/restore reference before removing or relocating the previous copy. Retain the previous copy if any verification or reference update fails. Never overwrite an archive version.
- Before committing, verify that referenced external evidence exists, hashes match, and bulky raw evidence is absent from the proposed Git additions.

Mermogon was verified mounted at `/run/media/cognis/Mermogon` on 2026-11-22. Its verified volume UUID is `D0A02EEBA02ED82C` (label `Mermogon`). Re-verify its mount and identity at each archive operation. This gate applies to the pending M5 results as well as future experiments; storage references must be updated before those results are committed.
