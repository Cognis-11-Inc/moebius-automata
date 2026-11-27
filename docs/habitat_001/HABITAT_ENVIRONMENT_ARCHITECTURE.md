# HABITAT environment architecture — experimental

Single owner: gated moebius_morph::primordial::Chamber. Config/State contain bounded
carrier populations, species fields/wall stocks, inert spheres, initialized
inventories, tick and external-impulse diagnostics. Only Chamber::step advances
physical state. A headless harness and separate Bevy Witness invoke it; rendering
receives immutable samples. Reset/parameter controls explicitly create a new run.
No function is called by ShipRuntime, MaterialBook, nursery World or old Witness.

Layer A: D3Q19 isothermal small dense carrier reference. Layer B: bounded statistical
passive fields, optional active-face sparse scalar storage, deposited wall stocks.
Layer C: finite initialized inert spheres using SI mass; no canonical parcels,
paid filaments or bodies transferred. Layer D: reserved heat/chemistry/radiation/
nutrient/hull exchange interfaces only; no placeholder registry implies physics.

Future integration must type producer local density/u/mu/sample frame, consumer
appendage solver and opposite cell force/torque, explicit actuation-energy debit,
validated source/recipient ownership, collision/contact authority and historical
transition. Nursery Body::step's returned reaction ledger is useful precedent;
its quantized coefficients and authored lunge term are not a lawful fluid adapter.
No actuation interface is implemented merely by defining a proposed connection.

The dense finite reference is deliberate. Sparse scalar storage is compared with
identical dense transport; it does not make dense carrier cost sparse. Field
activation, halo communication, pressure influence and large-volume equivalence
remain a future measured seam. Existing region/local Position stays global frame
owner. No new global voxel map, replay service, material exchange ledger or
canonical environmental authority is installed.
