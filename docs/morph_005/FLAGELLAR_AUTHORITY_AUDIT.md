# Flagellar authority audit — VII

2026-11-26. VERIFIED source at entry 7de3fa7; experimental tools added separately.
This is the implemented kinematic model, not a physical fluid/elasticity model.

| Concern | Path/symbol | Status | Input -> output / consumer | Authority/tests | Reuse ruling / missing seam |
|---|---|---|---|---|---|
| Definition, attachments, dimensions | anatomy.rs Component::flagellum, Shape::Flagellum, AnatomySpec::validate | authoritative authored input | station/azimuth, orientation, length/radius/taper/curvature/amplitude/frequency/phase -> compose | validates length0.15..3m, radius0.008..0.15m and aspect; existing composition tests | reuse supported fields; no joint stiffness, mass, actuation or contact compliance |
| Attachment frame | anatomy.rs compose/component_frame; compiler.rs attachment | authoritative generator | body surface sample + component orientation -> component tube | attached to body0; child appendage joints not implemented | no independent articulated degrees of freedom |
| Segmentation | anatomy.rs compose | implemented generator | 29 axial rings,10 sides plus caps/tip -> surface | tessellation is not a chain of physical links | keep mesh convention; no hinge dynamics |
| Animation | compose, filament.rs construction_pose, collision.rs habitat_presentation_pose | presentation-only tick motion | phase/tick -> traveling wave and larger body bend | renderer/witness consumer; new fixed-collider test | same generator, different pose sample; visible motion is not contact-responsive |
| Collider | collision.rs CollisionCache::body, collision_pose | authoritative collision envelope | spec hash/quantity -> per-surface convex hull Compound; phase0,bend0.12,flagella1 | existing cold/memo/invalidation tests; new tick/hash test | immutable hull per definition; use exact canonical constructor |
| Visible surface relation | collision.rs presentation_slack | distinct presentation and physical envelopes | eight visual phases/both eating states -> maximum outside-hull distance plus0.03m | clearance enlarges solids only, not body/pipe/prey | visible appendages need not lie inside the fixed hull; slack is not intended-anatomy error tolerance |
| Propulsion | habitat.rs prepare, turn_heading/explore | prescribed movement authoritative; flagellar force UNIMPLEMENTED | heading/food/blocked history + habitat.rules.step -> rigid proposal | no Shape::Flagellum/amplitude/phase consumer in movement path | removing flagella does not remove a force term; contacts can still alter later steering |
| Contact admission | collision.rs resolve/cast/nondeepening_recovery/along_surface/retreat | authoritative kinematic admission | compound, proposed rigid endpoints, obstacles -> admitted poses/observations | bounded ordered passes, unchanged fallback freeze and original recovery tests | appendages can block translation/rotation, cannot individually bend/retract/relax |
| Self-contact | resolve pair loops; diagnostics separate IDs | unimplemented within one organism | distinct-body/static queries only | no self-contact solver; overlapping convex parts are a union envelope, not interacting joints | do not infer self-interlocking from geometry |
| Inter-organism | resolve distinct ids, static full body hulls | implemented authoritative | actual individual component hulls contribute to compound queries | contacts record organism/target, not winning component ID | component-pair diagnostic queries are observer evidence, not authoritative winner attribution |
| Body/pipe | resolve -> pipe horizon/sample sphere -> hollow TriMesh | implemented authoritative | rigid organism hull versus oriented inner/outer pipe surfaces | exact fixture capture/replay tests | frozen appendage can meet pipe wall/opening; traversability needs query evidence |
| Secretion | runtime.rs tick; emission.rs place_output; collision.rs deposit_allowed; Assembly::deposit | implemented finite-stock path | actual posed nozzle -> parcel; collision preflight -> paid binding/nodes | material/assembly validation, rejected deposit events | no flagellar secretion organ; appendages can obstruct geometrically; source exemptions remain canonical |
| Sensing | docs/morph_005/SENSORY_ARCHITECTURE.md | reserved contract, unimplemented | no flagellar touch sensor feeds an organism controller | contact observations are world records | no new sensory path authorized |
| Creator/editor | morph-lab main.rs, editor controls; world-witness creator | implemented authoring/presentation | add/remove/duplicate/edit supported fields; load/save anatomy -> runtime definition at Start | default creator bare(seed); benchmark uses showcase(seed), runtime adds food storage | benchmark anatomy is not every user-edited organism; actual user save was not supplied |

**Physical answers.** The appendage changes the fixed collision envelope and can
obstruct rigid motion. It has no propulsive force, independent bending response,
retraction or relaxation mechanism. Whole-organism heading changes/retreat exist;
these are authored kinematic responses, not appendage disengagement mechanics.
Nearby bodies can pass only if their actual compound sweeps permit it. Mechanical
interlocking/impossibility cannot be inferred from finite solver freezes. Animation
changes neither collider geometry nor propulsion. Controlled Shape.phase edits
change the frozen generated shape; they are not moving articulated flagella.

Historical evidence inspected: cdc3acd (anatomy/collision allocation work), 8748f66
(exact local kernels and history prototype), VI source c989033 and its contract.
Read current anatomy/compiler, habitat movement, collision/fixtures/executor,
filament constructor, runtime material/event path, Witness and editor call sites.
Main/origin/main are the only visible branches; stash empty. IV/V dependency and
physical-geometry archaeology reused. No remote unpublished history, user's exact
edited creature/save, fluid solver, GPU trace or all historical prototypes audited.
No conflicting canonical owner found; research tools remain feature-gated observers.

Additional counterpart check: spec.rs/compiler.rs retain v1 WormSpec/CompiledWorm
flagellar mesh generation; AnatomySpec::migrate converts those authored appendages
to v2 components. They do not supply a ship flagellar force solver. Separate
moebius-core neural/body tests (torque_has_rates_limits_and_no_vacuum_com_thrust,
a_hard_bend_lunges_in_a_medium_and_vacuum_stays_put) exercise nursery Body::step,
not ShipRuntime habitat physics. No adapter or shared articulation authority was
found; these paths were not conflated with the active ship model.
