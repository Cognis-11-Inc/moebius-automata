# Neural foundation v1

M4 update: the default nursery now selects neural law 2 and disables the
authored spinal wave. Explicit historical law-1 configs retain their behavior.
See [M4 diagnosis and validation](../m4/REPORT.md); useful locomotion and
foraging remain unestablished. The earlier foundation measurements below
retain their original evidence scope.

This is an authored deterministic model, inspired by recurrent-network research,
not a reproduction or validation of a published biological model. Read the
[baseline audit](PHASE_0_AUDIT.md), [research record](../research/2026-11-21-neural-foundation.json)
and [measured results](artifacts/main_v2/REPORT.md) separately.

## Ownership and modes

`World::tick` clones the world, advances one fixed step, audits the candidate,
then commits atomically. The worker owns ticks; rendering only observes committed
snapshots and interpolates them. No network service, GPU inference or Python
trainer participates in organism control.

`SimConfig.neural = None` preserves historical M0/FARM2 laws. A version-1 neural
configuration requires farm rules and selects the new path. This path bypasses
legacy discrete action selection, authored strokes and heading impulses.
Legacy rig fields remain pose/accounting compatibility caches; neural state is
authoritative. `neural_farm_256()` is the live default.

## Inherited specification and lifetime state

`ControllerGenome` holds a versioned anatomy, eight recurrent neurons, dense
input/recurrent/output weights, neuron and motor biases, time constants,
input/output gains, plasticity rates and actuator strengths. Signed weights
permit excitation and inhibition. All founders share a seed-derived template;
four bounded per-founder mutations introduce individual differences.

`NeuralState` separately stores activations, integration remainders, learned
recurrent deltas, eligibility, plastic remainders, physiological baseline,
observations, commands, actual efforts, body state and cost/participation counters.
Offspring inherit the genetic specification with four seeded parameter mutations
when enabled. Learned deltas, eligibility and activations reset. Parent momentum
is divided between the two bodies; total linear momentum is preserved. There
is no Lamarckian inheritance in this mode.

Genotype IDs are BLAKE3 of the integer genetic JSON. They identify specification,
not competence. Parameter mutations can affect the executed controller and
actuators; an individual mutation may nevertheless have no observable effect in
a particular experiment. No anatomical or neural-topology mutation is enabled.

## Neural integration

Q = 1024 represents one normalized unit. Integer operations truncate toward zero;
explicit remainders retain small updates. For each of four synchronous substeps,
all recurrent inputs use the preceding activation vector:

```
f(x) = Q*x / (Q + abs(x))
drive_i = clamp(bias_i + sum_j((W_ij + P_ij)*f(x_j)/Q)
                        + sum_s(V_is*sensor_s*gain_s/Q²), -8Q, 8Q)
remainder_i += drive_i - x_i
x_i += remainder_i / (4*tau_i)
remainder_i %= 4*tau_i
command_a = clamp(gain_a * f(motor_bias_a + sum_i(O_ai*f(x_i)/Q))/Q, -Q, Q)
```

Time constants span 4–64 ticks; activation is bounded to ±8Q. Commands are
continuous in model intent but quantized to integers. They are simultaneous,
with no one-action winner. The bounded leaky update prevents numeric runaway;
this is not a convergence theorem or proof that default networks oscillate.
A constructed two-neuron test establishes that the architecture *can* sustain
oscillation under constant input, not that organisms learned a rhythm.

The recurrence ablation resets state and removes recurrent drive; it also changes
response amplitude and is not a gain-matched control. Random baseline commands
use an independent deterministic domain-separated stream with the same command
bounds. It still pays the defined neural and actuator fees.

## Sensor and motor ABI

`Anatomy` explicitly binds each axis to a joint and limit. The reference body has
12 spinal joints with bend and twist axes plus two independent jaw axes:
26 torque outputs, followed by secretion valve and body-local nozzle x/y/z,
for 30 outputs. `interfaces.json` in each study records every named binding.

For N rotational axes there are 3N+29 sensors (107 in the reference body):

| Observation | Normalization / model mechanism |
|---|---|
| Each actual angle, angular rate, actuator effort | angle / largest absolute limit; rate /64; effort /2048 |
| Local linear velocity and acceleration, approximate angular velocity | /256, /64, /64 respectively |
| Available energy | energy / configured energy cap |
| Secretion feedstock readiness | binary local nutrient present and energy ≥2; not an internal reservoir |
| Last intake, net metabolic signal, last actuator cost | intake/4 and cost/4 saturated; metabolic signal described below |
| Local nutrient and six chemical rays | local / site cap; each facing sums stock over 8 steps, nearer stock weighted more, scaled by site cap. Silent beyond 8 sites |
| Six directional contacts and two jaw loads | short-range local tissue/occupancy probes; jaws share head target |

All channels clamp to ±Q. Proprioception ablation zeros angle/rate/effort and
motion channels. Chemistry ablation zeros the named chemical channels; intake
and feedstock readiness remain. No coordinates, IDs, distant targets, future
outcomes or whole-field navigation enter the brain. No damage, fatigue or
contacted-matter velocity channels are invented: those mechanisms are absent.

The schema/controller/body helpers accept compatible chains of 3–24 spinal
joints and 0–2 jaws. The integrated world currently requires the reference
anatomy because rendering and contact bindings have not been generalized.
Future development must version the mapping, allocate/remap matrices explicitly,
validate dimensions and update body/contact/rendering together. Varying neuron
count or connectivity similarly requires a new controller schema; neither is
silently interpreted by version 1.

## Mechanics and costs

The nerve writes a command. A heritable muscle follows it by
`muscle += (command - muscle) / step` (`muscle_rate` 2–16, founder 3; a stored
0 still means 8 so older genomes load). `step` is 2 when the command gap
exceeds 128, otherwise the gene and at least 2. A real contraction arrives in
a few ticks. The body receives that smoothed drive, not the raw flicker. Effort
is drive × strength / Q, clamped to ±1024, plus a ligament spring
`-(angle - rest) / stiffness`. Stiffness is 2–24, founder 6. `ligament_rest`
is one heritable angle per joint, founder values a shallow alien curl, jaws at 0.
Rate integrates as `1/2 previous + effort/8`, capped at ±10, ±40 when absolute
effort exceeds 64, or ±64 when it exceeds 256. In law 1 a limit bleeds reported rate even when the angle is fixed. Law 2
reports the actual bounded angle change, so a held stop has zero rate. These
three genes mutate.

When `spinal_wave` is enabled and energy remains, an authored spinal wave pulls bend muscles toward a
traveling target. The ordinary stroke aims at ±640 with pull 512/1024. Hunger
(action-energy under 96, or no intake after the first eight ticks) aims at
±960 with pull 720/1024. The phase is `tick % 12`. Jaws and the valve are not
driven. It is not a path to the nearest clot, and a fed animal still works
the spine.

With drag, a same-direction throw (the absolute value of the signed sum of
spinal bend rates above 280) lunges along the local spine, scaled by
`excess * drag / 256`, and the sign follows the throw. Opposing bends in the
wave cancel, so the stroke does not become a one-way rocket. The impulse is
clamped into the body momentum bound and the opposite amount is added to the
environment ledger.
Vacuum still has no center-of-mass thrust from internal motion. Lifetime muscle
length is not inherited. Half of the lifetime recurrent and motor weights is
written into the child genes at birth; the traces themselves reset. A learned
weight of 1 truncates to nothing. That write is an authored assimilation, not
a measured evolutionary result.

Jaw load still limits closing rate to -8. Internal joint motion produces
approximate opposite body rotation. In vacuum (`drag = 0`) that motion still
creates **no COM thrust**. With drag, spine deformation feels anisotropic
resistance: normal drag is four times tangential drag. The finite-difference
force is divided by 512 before it is entered on the body and, with the opposite
sign, on the environment ledger. That divisor keeps a hard stroke off the
momentum cap.
This is a coarse authored coupling, not a Stokes solution. A held pose does
not swim. Bend limits remain ±768, twist ±512, jaw -256..768.

Linear velocity is momentum / effective inertia 16; positions use 1024 subunits
per lattice site. Drag absorbs momentum × drag/1024 into an environment ledger.
Blocked site transitions reverse/damp momentum and transfer the difference to
that ledger. Full segment/capsule collision and a 3D angular-momentum solver are
absent: this is a torque-driven kinematic rig with approximate counter-rotation.

One neural energy unit is charged every eight ticks. Aggregate absolute
strength-weighted axis effort /8 accumulates debt; each 4096 debt units costs
one energy unit. Law 1 uses raw neural commands here, omitting downstream wave
drive; law 2 bills the actual smoothed muscle request including any enabled
wave. Law 2 suppresses unaffordable muscle drive. Passive coasting remains
possible. Valve/nozzle strength affects actual release and recoil, not only
stored genes. The accumulator advances only when the smoothed valve drive
exceeds 480, near the squashed ceiling of a saturated open. The spinal wave
does not write that channel. A release costs two energy plus one local nutrient unit, creates
one material-bearing jet, and gives the body exactly the opposite linear
impulse. A 4096-jet cap and body-momentum bounds refuse release without losing
material. There is no hidden desired velocity.

Jets move with their momentum. In the nonzero-drag medium they cure after four
ticks or on contact; their momentum transfers to the environment. In vacuum
they keep flying until contact. Arrivals become permanent flesh, or waste if
the site is saturated. Closing loaded jaws can convert up to two flesh units
to waste for one energy. Face-connected flesh fuses under the existing rules.
Those are authored local laws, not a fluid solver or learned construction.

Exact integer audits check body + jet + environment linear momentum = zero and
material/energy balances every committed tick. In-flight material is included.
Effective inertia is not tracked organism biomass. Action-energy accounting is
not conservation of mechanical kinetic energy. Feeding still automatically
consumes nutrient at the organism's lattice site, not through mouth geometry;
secretion has no stored internal reservoir. These are explicit prerequisites
for a fuller embodied feeding/biomechanics model.

## Lifetime plasticity

After physiological consequences, raw modulation is the clamped change in
energy ×128. Subtract a slowly adapting baseline (1/16 per tick). Eligibility
is a bounded leaky correlation of presynaptic and postsynaptic firing:

```
e_ij = clamp(7*e_ij/8 + f(x_i)*f(x_j)/(8Q), -Q, Q)
plastic_remainder_ij += inherited_rate_i * modulation * e_ij
P_ij += plastic_remainder_ij / (8Q²)
P_ij = clamp(P_ij, -512, 512)
plastic_remainder_ij %= 8Q²
```

The same authored rule also updates motor synapses from the smoothed muscle
drive. Those weights stay lifetime state until birth. Offspring assimilate half
of the recurrent plastic weights and half of the motor plastic weights into the
germline, then start with empty traces. Changing a synapse, or finding it in a
child genome, establishes the mechanism. It does not show that a lineage
learned to forage. The matched study did not demonstrate that benefit, and the
nursery has no new foraging trial since assimilation was added.

## Persistence and observation

FARM3 persists genetic and lifetime state, remainders, body, jets and environment
momentum, including when all organisms die. Decode validates bounds and mode;
restore/continue tests compare subsequent hashes. Optional new fields are
omitted from historical M0/FARM2 canonical encodings.

F selects a nearby creature; N toggles inspection; Tab cycles six read-only
pages: summary/activity history, all axis commands/efforts/angles/rates and
participation, three sensor pages, and inherited/learned recurrent matrices.
`--live --inspect` opens the panel immediately. Display history is observed
history, not additional neural state. The panel identifies committed state and
rendered interpolation. Portable study traces expose actual state each tick.

Run the documented [protocol](STUDY_PROTOCOL.json) using `neural-study`, then
`python/analyze_neural_study.py`. Reports distinguish motor activity, actual
displacement, experience-dependent response, useful learning, inherited
divergence, adaptive evolution and ecological emergence.
