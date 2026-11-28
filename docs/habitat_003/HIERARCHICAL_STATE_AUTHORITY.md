# Native hierarchical physical authority

IMPLEMENTED as primordial::scientific::{Chamber,State,Payload}, behind the existing
habitat-experiment feature. No ShipRuntime consumer. Existing primordial::Chamber
is retained unchanged as the legacy experimental model. This is an explicitly
selected alternative numerical owner authorized by003, not a bridge into the ship.

A root key is root_index<<4. Its inventory is either at that key or at all eight
child keys root|1..8, never both. Child bits locate x/y/z subdivisions. Two levels
only, ratio2, isotropic Cartesian cells. Volume partitions are checked before
admission. A cell payload is either eight FV integrated variables or19 LBM
populations. There is no duplicate FV/LBM carrier and no finest-volume shadow.

The face graph enumerates actual leaf faces with finest face-tile addresses,
merging identical neighbor pairs. This temporary face representation is not a
finest physical-volume allocation. Cells, faces, sorting, validation, checkpoint
and reductions still scale with actual leaf/coarse-background count. It is not
an infinite sparse universe or an arbitrary-depth hierarchy. Coarse roots and
all positive scalar tails remain represented. BTreeMap ordering fixes traversal.

Chamber stages changes in owned clones; it validates then swaps one committed
state. Refusal preserves the committed digest. Pending temporal work is separately
marked and cannot be sampled as committed physics. Checkpoint schema binds full
payload, geometry, clocks, thresholds, ledgers, source, backend and float bits;
load rejects altered source/schema/backend or invalid ownership. This is corruption
and provenance checking, not cryptographic authentication against a hostile author.
