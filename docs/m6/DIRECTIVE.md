# CODEX DIRECTIVE — MŒBIUS AUTOMATA
## M6: THE MOTOR NURSERY
### Embodied Exploration, Motor Learning and Locomotion Acquisition

**Primary objective: Teach the existing organisms to discover useful movement through physical experimentation.**

The user is dissatisfied because organisms possess articulated bodies but do not meaningfully use them to explore their environment.

We will investigate whether the existing recurrent controller can acquire coordinated locomotion through systematic motor exploration and training.

**Do not prescribe a swimming gait. Create conditions in which useful motor coordination can be discovered and measured.**

---

### PHASE 0 — Read, audit and establish the baseline

Read:

- `AGENTS.md`
- `HANDOFF.md`
- `docs/m4/REPORT.md`
- `docs/m5/REPORT.md`
- `docs/m5/PROTOCOL.json`
- `docs/m5/VALIDATION.json`
- Existing neural controller, body dynamics, sensory system, energy accounting and experimental harnesses.

Inspect the actual current branch, changes and integration state.

Establish the current contracts for:

- Eight-neuron recurrent controller.
- Twenty-six joint actuator axes and other motor outputs.
- Joint angle, rate and effort feedback.
- Motor smoothing.
- Law-2 physical dynamics.
- Neural plasticity and inherited parameters.
- Momentum and energy accounting.
- Diagnostic externally controlled ticks.

Do not infer integration from file existence.

Preserve historical replay compatibility.

The head and external eyes may be visually unsatisfactory, but **head aesthetics are explicitly outside M6 scope**.

---

### PHASE 1 — Determine the teachable motor envelope

Before training, establish the movement capabilities physically available to the controller.

M5 demonstrated limited COM propulsion through imposed commands, but not competent continuous swimming.

Build a compact capability test that measures:

1. Sustained COM translation.
2. Actual travel versus net displacement.
3. Rotational control.
4. Direction reversal.
5. Coordinated joint participation.
6. Motor energy expenditure.
7. Motion in different initial orientations.
8. Behaviour under different medium conditions.

Use genuine physics transitions, not manipulated position or velocity.

Distinguish translation caused by body articulation from translation caused by secretion jets.

Do not score mouth extension as COM locomotion.

**Gate:** If the present actuator/physics system cannot support a useful controllable movement pattern, investigate and correct the specific mechanical blocker before launching an expensive neural search.

Any physics correction must be separately versioned, justified and regression-tested.

Do not silently change production coefficients to make the training succeed.

---

### PHASE 2 — Build a motor-learning nursery

Create a controlled experimental environment independent of the ordinary food-and-reproduction nursery.

Initial training conditions:

- One existing organism.
- Existing fourteen-joint anatomy.
- No food-search objective.
- No reproduction.
- No predators or obstacles.
- Configurable medium.
- Explicitly provisioned training energy.
- Deterministic initial states.
- Full neural, motor and movement telemetry.

Training energy may exceed the ecological default, but this must be a documented training condition, not a production physiology change.

The organism must receive only lawful local sensory and proprioceptive signals.

No global coordinates, target vectors or privileged physics data may enter its production-compatible neural inputs.

A separate training evaluator may observe global state to calculate scores.

Resetting episodes is allowed in the training harness; it must not be misrepresented as biological survival.

---

### PHASE 3 — Motor babbling

Begin with exploratory motor commands.

We want the organism to experience relationships between action and physical consequence.

Explore bounded:

- Individual joint contractions.
- Reversals.
- Simultaneous contractions.
- Sequential activations.
- Different phase relationships.
- Different amplitudes and durations.
- Recovery from different initial poses.

Capture:

`observation → neural state → motor command → actual joint response → physical consequence`

Include:

- COM displacement.
- Velocity.
- Orientation.
- Joint position and rate.
- Motor energy.
- Net momentum exchange.
- Action outcome.

Avoid hand-authoring the successful coordination pattern.

If a useful movement pattern emerges from random exploration, preserve the exact commands and conditions for replay.

**Motor activity alone earns no success credit.**

---

### PHASE 4 — Train the recurrent controller

Start with a bounded evolutionary search over the existing recurrent controller parameters.

Prefer adapting the existing Rust implementation over adding an unnecessary machine-learning dependency.

Candidate genomes may vary:

- Recurrent weights.
- Sensor weights.
- Motor output weights.
- Motor biases.
- Relevant neural dynamics parameters, if currently supported.

Enforce numeric bounds and valid controller structures.

Use multiple short evaluation episodes per candidate.

Fitness must reward measurable physical competence, not the appearance of movement.

#### Training objectives

**A. Translation**

Reward actual net COM movement relative to the starting pose.

**B. Efficiency**

Measure motor energy per unit of useful travel, with defined treatment of near-zero displacement.

**C. Directional competence**

Evaluate movement toward changing training directions.

Privileged directional goals may be used by the evaluator, but not secretly inserted into organism sensors.

If a directional command interface is required, design and validate an explicit local sensor-compatible cue.

**D. Control**

Measure the ability to alter movement and reduce velocity when challenged.

**E. Robustness**

Evaluate across initial orientations and unseen physics-compatible starting poses.

Do not combine everything into one opaque fitness number without recording its components.

Avoid rewarding endlessly rotating in place, mouth-only movement, useless oscillation or numerical artifacts.

Maintain population diversity and preserve the full lineage of selected controllers.

No scripted waveform, target-specific lookup table or privileged oracle behaviour may be installed as the learned policy.

---

### PHASE 5 — Distinguish evolution from individual learning

This distinction is mandatory.

**Track A: Evolutionary optimization**

Controller parameters improve through evaluation, selection and mutation across candidate genomes.

This demonstrates optimization across candidates, not learning during an organism's lifetime.

**Track B: Lifetime plasticity**

Test whether the current plasticity mechanism improves motor performance through an individual's experience.

Compare:

- Plasticity enabled.
- Plasticity disabled.
- Training feedback disabled.
- Untrained matched controls.
- Frozen post-training controller.
- Fresh organism initialized from selected inherited parameters.

For lifetime learning, evaluate the same organism before and after experience under matched probes.

Test whether any improvement persists when feedback is removed and whether it generalizes to unfamiliar starting conditions.

Do not call ordinary synaptic change or inheritance evidence of acquired competence.

If the existing plasticity rule cannot use motor consequences effectively, document that limitation and propose the smallest explicitly testable change.

Do not silently implement a new learning law without identifying it.

---

### PHASE 6 — Progressive curriculum

Only advance when the previous competence is demonstrated.

**Stage A: Produce movement**

Can the controller create meaningful COM translation?

**Stage B: Sustain movement**

Can it continue travelling instead of producing one impulse and stopping?

**Stage C: Change direction**

Can it respond to changing environmental cues?

**Stage D: Control and recover**

Can it brake, reorient, and recover from unfamiliar poses?

**Stage E: Explore**

Place the organism in an initially unfamiliar bounded habitat.

Measure unique visited regions, physical path length, revisit behaviour and cost of exploration.

No food objective yet.

**Stage F: Ecological transfer**

Place a frozen trained controller into the ordinary energy-and-food environment.

Restore normal resource accounting and mortality.

Test whether its acquired motor competence creates opportunities for reaching additional food.

Do not train directly on the final held-out test scenarios.

---

### PHASE 7 — Mandatory controls and falsification

Compare against:

1. Existing M5 production recurrent controller.
2. Deterministic random controller.
3. Rest controller.
4. Fixed diagnostic actuator patterns.
5. Best independently demonstrated mechanical control.
6. Trained controller with fitness-relevant observations ablated.
7. Plasticity-disabled version of the trained organism.

Use matched starts where appropriate.

Pre-register exploratory and held-out seeds and evaluation criteria before optimizing.

Report variation and uncertainty, not only the most successful seed.

Specifically investigate whether performance reflects:

- Genuine joint coordination.
- Exploitation of fixed-point rounding.
- Accumulation of free momentum.
- Training-specific initial poses.
- An undisclosed scripted actuator pattern.
- Hidden energy or sensor advantages.

A controller that performs well only under one initial orientation is not a robust locomotor learner.

---

### PHASE 8 — Scientific acceptance gates

Report separate verdicts for:

| Gate | Required evidence |
|---|---|
| M6-G1 | Physical movement capability under controlled commands |
| M6-G2 | Genuine exploratory joint activity |
| M6-G3 | Measurable optimization over untrained controls |
| M6-G4 | Sustained COM locomotion |
| M6-G5 | Directional control |
| M6-G6 | Robustness on held-out initial conditions |
| M6-G7 | Evidence of individual lifetime motor learning |
| M6-G8 | Transfer to ecological conditions |
| M6-G9 | Reproducible deterministic replay and lawful accounting |

Distinguish PASS, FAIL, PARTIAL, NOT RUN and INDETERMINATE.

**M6-G7 must remain unproven if only evolutionary selection succeeds.**

Do not claim acquired exploration intelligence merely because a candidate travels farther.

An unsuccessful training result is acceptable if its causes and limitations are preserved.

---

### PHASE 9 — Visual witness

The user needs to observe actual behaviour, not just read numerical reports.

Provide a selectable trained-organism demonstration in the existing viewer, without replacing the ordinary nursery by default.

Allow a meaningful comparison between an untrained and trained controller.

Make actual body articulation, orientation and COM travel clearly observable.

If feasible in the available environment, capture a short reproducible visual demonstration or inspect saved rendered frames.

Do not claim live visual improvement unless the visual output has actually been inspected.

Do not disguise poor mechanics with procedural animation.

---

### PHASE 10 — Resource discipline

This experiment must not generate another 10,000-file Git change set.

Follow the standing `AGENTS.md` artifact storage policy.

- Preserve raw evidence in verified versioned archives on Mermogon.
- Keep protocols, source, reports, metrics, configuration identities and checksum manifests in Git.
- Store large raw traces and training checkpoints outside Git.
- Record unsuccessful searches and failed candidates in reproducible form.
- Verify archived data before relocating any source files.
- Use bounded pilot experiments before larger optimization campaigns.
- Define the experimental budget before starting each campaign.
- Stop or revise the protocol when evidence shows the training objective is being exploited or is mechanically unattainable.

Do not commit or push unless explicitly instructed.

Follow existing research provenance rules. Announce any external scientific literature consultation before searching and document what it actually influenced.

---

## Required delivery

Provide:

1. Exact repository audit and integration map.
2. Motor capability baseline.
3. Training architecture and objective definitions.
4. Working motor-babbling and controller-optimization harness.
5. Trained and untrained controller checkpoints.
6. Held-out comparative measurements.
7. Lifetime plasticity assessment.
8. Reproducible visual demonstration configuration.
9. Scientific report containing failures, uncertainty and limitations.
10. Updated HANDOFF.md and verified archive manifests.

Work incrementally and do not conceal failures behind a successful test suite.

## THE FUNDAMENTAL EXPERIMENT

We have given organisms articulated bodies, nervous systems, sensory inputs, energy constraints, and a physical environment.

Now determine whether they can develop useful motor coordination through experience or evolutionary optimization.

We are not attempting to recreate a specific animal's swimming technique.

We are investigating whether meaningful locomotion can emerge from the interaction of a controller, an articulated body, and its environment.

**Do not teach the creature a movement. Establish whether it can acquire the ability to move.**

The desired outcome is not a beautifully animated worm.

It is an organism whose movements have measurable physical consequences, whose coordination can improve under a clearly identified learning or optimization process, and whose competence survives a change of circumstances.