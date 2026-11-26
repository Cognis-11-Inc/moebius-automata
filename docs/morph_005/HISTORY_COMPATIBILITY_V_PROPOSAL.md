# History compatibility across engine revisions — V proposal

2026-11-26, Codex. PROPOSED only; no migration or source bypass implemented.
Legacy v1 and opt-in v2 runtime code are unchanged in V. The retained IV source,
executable and archive remain the restoration authority for its 3600-tick run.

Verified `history.rs`: `source_digest` hashes the complete embedded source map,
including diagnostic/examples sources. `Store::open` requires encoding 2 and
the current source digest. `visit` requires every immutable segment's source
digest to match the checkpoint/store. A diagnostic-only source change therefore
still invalidates direct restoration. Editing STORE/checkpoint fingerprints
cannot lawfully migrate a chain: segments retain old fingerprints and hashes.

| Compatibility dimension | Required evidence before a transition |
|---|---|
| Serialization | old/new schema readers, lossless fields/order and explicit unknown-field refusal; readable JSON alone is insufficient |
| Physical laws | unchanged geometry, arithmetic, admission/tolerances, material rules and effective configuration; matched every-tick states/contacts |
| Historical events | complete ordered payloads and provenance, unchanged old segment bytes; event transformations require a new declared contract |
| Replay executable | retained old binary/source/compiler/dependency identity and inputs; a new executable does not become the old replay oracle |
| Snapshot migration | independently identified converter/version, immutable old checkpoint and new boundary checkpoint, exact mapping and hashes |
| Provenance transition | explicit old/new source and physical-law identities, boundary tick/event ordinal, chain head, witness results, operator authorization |

Proposed smallest future research slice: a transition envelope referencing the
old immutable checkpoint, complete old chain head/count, old executable/source,
new source, migration tool, policy, classification and validation receipt. The
new era starts at a separately identified checkpoint; old segments remain under
their original verifier and are never relabelled or rewritten. A future bounded
reader would validate a chain of eras and refuse missing/unknown transitions.
This requires a separately versioned encoding; it cannot be slipped into current
encoding-2 verification by making the source test optional.

First candidate class: exact, non-physical source changes such as diagnostic
code. Qualification must still include independently witnessed old/new
continuations from the same authenticated state, every required intermediate
state and event, old-history streamed provenance, and a retained executable.
Physical-law changes require a distinct decision and declared divergence; a
serializer success or selected matching ticks cannot certify them as exact.

Fail-closed tests for a future prototype: absent/forged transition; wrong source
pair; changed rules or policy; altered origin/segment; omitted/reordered era;
wrong boundary ordinal/tick; incomplete backing copy; converter failure;
write exhaustion/crash before publication; old restore still available; portable
bundle restoration and exact old/new matched continuation. Transition metadata
must have explicit resident/count and provenance bounds, not grow an unbounded
in-memory source registry.

Current limitations remain: finite 16384 provenance ceilings, no automatic
v2-to-v2 migration, no unrestricted lifetime guarantee, no physical power-loss
proof, and no whole-engine memory stability result. This proposal establishes
neither canonical adoption nor a bypass of those limits.
