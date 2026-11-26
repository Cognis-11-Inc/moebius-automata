# Historical continuity assessment and proposed contract

2026-11-25 America/Toronto / 2026-11-26 UTC. **PROTOTYPE AUTHORIZED AND IMPLEMENTED; PARTIAL VALIDATION.**
Operator approval is required before this changes authoritative history. The
current HistoryBudget refusal, event cap, snapshot schema and digest stay intact.

## Verified active path

`WorldSnapshot.events: Vec<Event>` in runtime.rs is serialized world state.
`WorldSnapshot::digest` hashes the entire serialized snapshot, including that
ordered vector. `save` and `load` validate by restoring the whole snapshot.
There is no lazy-history or archival-segment resolver in that interface.
`ShipRuntime::snapshot` clones the whole state for an observer.

`submit` appends Accepted and reserves room against the same lifetime event cap;
pending interventions are sorted by tick and sequence. `tick` clones mutable
state without past events, records this tick's deltas, executes fallible gates,
checks the lifetime HistoryBudget, then appends the deltas to the existing vector
at commit. It does not discard or clone the old event vector inside that commit.
Records include Accepted,Applied,Contact,Reproduced,Fed,Emitted,Deposited,Chewed,
FoodBurst and explicit rejection events. Producers are runtime.rs,habitat.rs and
collision.rs; their current execution order is the authoritative order.

The next numerical tick does not scan all historical events to select motion.
However, full validation *does* use history: runtime::validate sums constructor
Fed quantities against book.external_input; habitat::validate requires the paid
Reproduced provenance of each offspring. Material books/assembly own quantities;
events are nevertheless part of their checked historical provenance.

Witness presentation scans the last256 events for recent feeding flashes, and
counts births over the complete event vector. Recorders calculate lineage,
intake and birth summaries from the log. Tests compare complete event-inclusive
snapshots/digests. Changing resident history changes these consumers and identity.

The current recorder's replay restores its initial snapshot and advances the
canonical runtime. This is not proof of an implemented general event-sourced
reconstruction API from an arbitrary archived log. Arbitrary later interventions
need their original submission order/times as well as initial state and rules.

III endurance evidence:65505 events at tick1863; cap65536; next tick refused.
Replay of committed ticks and material gap0 hold. Raising the cap or dropping a
prefix would hide the defect and violate the current contract.

`submit` also scans Accepted events to reject a reused intervention sequence.
A segmented implementation must preserve this check across sealed history;
a missing prefix cannot become permission to reuse a sequence.

Version clarification: current construction worlds already use WorldSnapshot
and WorldConfig version2. Here v1/v2 name the legacy/full-log and proposed
segmented *history encodings*, not those existing physical format numbers.
Do not reuse an occupied snapshot version number or change physical rules.

## Proposed v2 boundary (prototype now authorized)

Keep v1 behavior unchanged, including its refusal and digest. Introduce an
explicitly selected v2 checkpoint/history contract; never silently migrate a
running v1 world. V2 preserves every ordered event and numerical state transition,
but replaces the event-inclusive v1 serialized identity after migration with an
explicit new identity. It cannot claim byte equality between the two schemas.

The same canonical runtime owns a HistoryStore interface and admission. Witness
and renderers remain read-only consumers. An explicitly supplied backing store
owns durable bytes; no implicit global path, background observer writer or hidden
cache becomes persistence authority. Reuse the existing atomic-write conventions
where sufficient, adding durability/failure checks that those helpers lack.

V2 checkpoints contain active state, pending interventions, a bounded recent-event
window, total committed count, immutable history-chain head and checked provenance
summaries. Immutable segments contain sequence range, tick range, previous-segment
hash, rule/source identity and complete event payload/hash. The resident checkpoint
has a fixed-size chain head, not an ever-growing vector of segment descriptors.
Historical indices, if needed, live on disk and remain derived/read-only.

Use a separately named resident policy, provisionally no more than65536events and
an explicit byte ceiling. Do not reinterpret or increase v1 max_events. Bound both
resident event count and payload bytes: rejection reasons are variable-length.
Storage capacity remains finite and exhaustion must refuse the next authoritative
tick without partial event/material commit.

Before an append would exceed the resident allowance, seal the required existing
prefix to an immutable segment and verify its durable content/hash. Publish the
new checkpoint/chain head atomically with admitted state only after all required
writes succeed. Write failure leaves the committed world unchanged. Orphaned
unreferenced segments from an interrupted publication are recoverable evidence,
not permission to discard the old checkpoint. Never overwrite a sealed segment.

Intake totals and paid-birth proofs are derived summaries over the sealed prefix
plus resident tail, not invented replacement history. Load validates them by
streaming the chain and checking contiguous event sequence/order and hashes;
normal numerical ticks do not rescan it. Current living populations are bounded;
future death/ancestry retention would need an explicit disk-backed query contract,
not an unbounded resident map slipped into this phase.

A v2 snapshot may reference durable history; it is not a self-contained v1 JSON.
Portable saves therefore bundle checkpoint plus required immutable segments and
restore metadata. Missing/corrupt segments, changed rules or unsupported versions
fail closed. Preserve the original v1 checkpoint and legacy digest during explicit
migration. A streaming history reader supplies full audit/replay queries; recent
Witness effects use only the bounded recent view, and total birth display uses a
verified cumulative summary.

Experimental evidence continues to use versioned, hash/restore-verified Mermogon
archives (UUID D0A02EEBA02ED82C). A runtime backing-store path/policy is an explicit
configuration choice; a working system-disk copy is not an authoritative external
experimental archive. This proposal does not authorize a different archive drive.

## Required implementation and migration proofs

Before acceptance: unchanged v1 fixtures/refusals; exact event-stream equality and
numerical/contact/material equality on all shared ticks; digest/version separation
made explicit; deterministic reconstruction with original interventions; streaming
history/provenance checks; migration retaining the original checkpoint; bounded
resident count/bytes under long execution; corrupted/missing/reordered segment,
write failure,disk exhaustion and crash-between-write/publication tests; restored
portable bundle equality; Witness neutrality. Then a3600tick run must actually
complete under the existing time target. Storage failure is never a skipped tick.

This is a proposal, not a persistence implementation or approval request to raise
the event cap. Collision work can proceed while the history-model decision waits.

Operator decision: the v2 prototype and required validation were explicitly
authorized in this session. Legacy retention/digest behavior must remain intact.
Implementation and evidence are still pending; approval is not a result.

## Prototype status, 2026-11-26 UTC

IMPLEMENTED behind `history-v2-prototype`; explicit opt-in only. The existing
physical snapshot versions stay unchanged. Metadata and segments carry a history
encoding2 and compiled-source fingerprint, requiring explicit migration after a
source change. Portable copies validate every chained segment and the retained
legacy origin. Source and origin digests enter the new logical identity; filesystem
paths do not. Full legacy guards for emissions/depositions/chews/interventions
were an additional consumer omitted from the initial assessment. They are now
shared exactly through two streaming passes, rather than replaced or weakened.

VALIDATED so far: three focused history tests (512 exact paired physical/event
steps, portable continuation, corrupt/missing backing, sealed duplicate admission,
explicit provenance refusal and failed-seal atomicity) plus nine runtime tests.
Full colony/endurance, reordered/crash/exhaustion variants and native neutrality
remain pending. A missing backing directory is a tested IO failure, not proof of
an actual full disk or power-loss experiment.

Resident default4096 events/4MiB payload, segments1024, recent256. Validation maps
have explicit16384-entry ceilings. Admission conservatively bounds the lifetime
Accepted/Emitted/Deposited/Chewed event counts by that same ceiling, in addition
to the count/byte resident policy. This makes the prototype finite and fail-closed;
it does not establish unlimited arbitrary command/material provenance. A disk
index or lawful lifecycle compaction would require further validated extension.
Active geometry/material/population memory and the eager initial checkpoint JSON
loader are separate from bounded historical resident storage; no global bounded
memory claim is made. Ordinary ticks do not read the whole archive; restoration,
full validation and sealed duplicate-command checks do. Storage exhaustion fails
before visible state mutation; orphan immutable segments remain reusable evidence.
