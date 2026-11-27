# HABITAT-001 — The Primordial Medium

2026-11-27 America/Toronto. **DONE: isolated experimental checkpoint.** The
bounded model is validated within the controls below; production atmosphere
scalability, resolved body hydrodynamics and canonical integration remain
UNRESOLVED. MORPH-005 remains PARTIAL with conventional stabilization paused;
MORPH-006 remains suspended. Source commit `8498c22894c269e31ad8d53d63116b9b3ecafa5b`.

## Owner, implementation and compatibility

`moebius_morph::primordial::Chamber` is the explicit, feature-gated numerical owner.
Its finite 3D D3Q19 isothermal carrier, conservative passive material fields,
deposited wall inventories and unresolved inert spheres evolve independently of
organisms. `primordial/{mod,transport,experiments,tests}.rs`, the
`habitat_environment` example and standalone Bevy `habitat-witness` implement and
exercise it. Density and nominal effective viscosity are independent SI inputs;
fixed tick16,666,667ns and existing Position/metric conversion are reused.

[Reuse audit](ENVIRONMENTAL_REUSE_AUDIT.md),
[architecture](HABITAT_ENVIRONMENT_ARCHITECTURE.md) and
[physical contract](ENVIRONMENTAL_PHYSICS_CONTRACT.md) specify inspected existing
producers/consumers, units, model alternatives, numerical methods, boundaries,
validity cages, authority and reservations. No canonical runtime, collision,
habitat guard, emission/material, anatomy, filament or nursery physics law changed.
No MaterialBook/secretion quanta conversion, ship history migration or second ship
admission path is installed. The source commit contains20 files/3516 added lines;
subsequent publication edits contain only reports, receipts, provenance and handoff.

## Experimental verdicts

All five final ordinary/counting/native commands completed after the clean suite,
without a competing controlled build/test. [Machine-readable results](HABITAT-001_RESULTS.json)
retain the measured values; raw per-tick traces and full bit-encoded initial/final
states are external evidence, not Git data. Every one of16 cases per executable
asserted exact every-tick replay and full final-state equality against its replay.
Independent analysis compared ordinary/counting full encoded states and physical
observations, plus native/demo full initial/final states. Those comparisons PASS.

| Experiment | Actual finding | Verdict and boundary |
|---|---|---|
| E01 equilibrium | Zero velocity/kinetic energy through120 ticks; carrier inventory stable | VALIDATED sealed uniform control |
| E02 vacuum | Constant velocity with zero carrier/viscosity | VALIDATED analytic free motion |
| E03 weak gravity | Three acceleration directions; position/velocity error below1e-12 SI | VALIDATED vacuum analytic reference |
| E04 drag | First step matches independent finite-cell reduced-mass exponential within1e-14m/s | VALIDATED local Stokes ODE; later fluid diffusion is not an infinite bath exponential |
| E05 buoyancy | Density10 rises,20 remains initially neutral,40 settles in carrier20kg/m³ | VALIDATED prescribed hydrostatic support closure; not resolved pressure traction |
| E06 particulate | Three distinct drifts and finite suspended+wall inventories; separate deposition case | VALIDATED terminal-slip statistical transport |
| E07 transport | Periodic analytic advection/diffusion spatial errors decrease8/16/32³; local clouds also transported in demo | VALIDATED bounded scalar controls |
| E08 disturbance | A, B and A+B controls propagate/dissipate; all-cell final nonadditivity RMS 1.085173e-09m/s | VALIDATED weak interaction; no biological forcing |
| E09 coupling | Opposite body/carrier impulse; actual wall impulse accounted | VALIDATED finite-cell exchange and wall budget |
| E10 refinement | Scalar/shear spatial errors decrease at each substep setting; vacuum exact at roundoff | VALIDATED bounded spatial trend; general temporal/PDE accuracy UNRESOLVED |
| E11 replay | All case replay assertions, ordinary/counting equivalence, native/demo full-state equality | VALIDATED same-source execution contract, no cross-toolchain guarantee |
| E12 scaling | Four independent factors,36 settings,18 exact dense/sparse physical pairs | VALIDATED measured short-window costs; enormous-volume scalability UNRESOLVED |

Statistical settling over4.00000008s gives downward drifts0.400000008mm (small),
6.399996099mm (large),1.066666687mm (light), versus terminal-slip references0.400000008,
6.400000128,1.066666688mm. Maximum centroid error4.029e-09m; all within the
preregistered1e-7m+1percent drift tolerance. These are authored material classes,
not measured properties of the fictional vessel.

Across the16 recorded ordinary cases, maximum carrier residual is2.594e-13kg,
species residual2.118e-21kg, and total accounted momentum residual norm1.241e-13
kg·m/s. These satisfy the declared inventory and impulse tolerances. Momentum
includes explicit spheres, carrier, external gravity, prescribed hydrostatic
support and actual wall impulses. Statistical species carry no momentum equation.
Drag dissipation and restitution loss are tracked separately; viscous shear and
local-flow kinetic energy decrease. There is no full thermal/total-energy ledger.
Endpoint sphere wall projection is a numerical constraint; accurate continuous
sphere contact or resolved body-boundary convergence is not validated.

At one substep scalar L2 errors are9.75808e-8,5.10521e-8,2.55921e-8kg/m³ for8/16/32³;
observed spatial orders0.935 and0.996. Shear relative errors are5.825,1.437,0.133percent.
Apparent shear orders2.02/3.43 are this control's trend, not a universal order proof.
At fixed h, additional substeps do not monotonically reduce error against the PDE;
coarse8³ shear remains around6percent. Exact vacuum gravity errors are at roundoff,
so a fitted temporal order would be meaningless. No instability was hidden or
unconverged case relabeled as a production accuracy certificate.

## Costs and model selection

Reference:0.8m cube,16³, two species, nine inert spheres,120 ordinary release ticks.
Environmental update mean**1.077040ms**, empirical P99**1.095660ms**, max1.121410ms.
P99 uses sorted index floor(0.99*(n-1)). The provisional2ms mean/4ms P99 development
budget passes here; future collision/biology/cognition coexistence is NOT tested.
Mean components: carrier0.452914ms, transport0.322477ms, validation0.288786ms,
clone0.011087ms, coupling0.001222ms. Allocation-counted updates request3,113,969bytes
per tick; final live5,209,411bytes, process peak live118,809,641bytes. Observed ordinary
RSS peak127,144KiB includes harness snapshot/replay allocation retention, not only
resident physical state. No allocation optimization success is claimed.

Native update mean1.246968ms/P991.812580ms; drawing submission CPU mean0.290433ms;
frame mean16.969697ms/P9932.180066ms/max101.428358ms after the initial zero interval.
Frames include scheduling/render/wait; no GPU timings. These are short single-host
windows, no frame-smoothness guarantee. Final actual capture was inspected:
legible ASCII SI labels, chamber, concentration/explicit-sphere distinction,
gravity and sampled flow. Original font/HUD failures and corrected captures remain
archived. Keyboard/mouse acceptance by the Operator is untested.

[Scaling table](HABITAT-001_SCALING.json) includes component timings, sample cost,
requested/live/peak bytes and occupied scalar cells for every setting. Dense means
for8/12/16/24/32³ are0.114/0.383/0.909/3.036/7.256ms. Sparse scalar means are
0.298/0.979/2.092/5.552/11.235ms: **no speedup in these nonempty resolution controls**.
Zero-active sparse is slightly cheaper, but diffusion activates tails and the
carrier always remains dense. At16³ dense active-region0/1/4/16 costs
0.869/0.880/0.889/0.908ms; species0/1/2/4 costs0.729/0.879/0.987/1.203ms;
objects0/1/16/64/256 costs0.877/0.878/0.880/0.892/0.935ms. These are controlled
factorials, not a production scaling law or proposed ship resolution.

## Capability and rejected-model record

| Capability | Status | Evidence or missing condition |
|---|---|---|
| Finite carrier inertia, local velocity/dynamic pressure, viscosity | IMPLEMENTED; bounded VALIDATED | D3Q19 controls; low-Mach/density/tau cage; no absolute gas EOS |
| Uniform gravity, inert spheres, finite-cell drag reaction, buoyancy | IMPLEMENTED; experimental VALIDATED | Analytic and impulse controls; hydrostatic support external approximation |
| Statistical species, conservative transport/diffusion/wall stocks | IMPLEMENTED; bounded VALIDATED | Actual size/density/slip consumers; inventory and grid tests |
| Dense/sparse scalar storage, bounded sampling | IMPLEMENTED; VALIDATED equivalence | Same face order; dense carrier persists; sparse speed benefit REJECTED for measured nonempty controls |
| Source-bound checkpoints and observer neutrality | IMPLEMENTED; VALIDATED | Bit-exact states, guards, restored execution; no source migration |
| Thermal/radiation/chemistry/nutrient/hull exchange | RESERVED | Documented authority/energy/material seams only, no decorative runtime implementation |
| Future flagellar force/torque/energy interface | RESERVED | Nursery reuse inspected; no propulsion installed |
| Full compressible thermal gas or pressure-projection solver | NOT SELECTED | Larger equations/solver obligations; BGK selected for bounded controls, no empirical superiority claim |
| Existing uniform Environment as spatial atmosphere | REJECTED for this role | Parcel-only uniform contract cannot supply spatial carrier reaction; canonical use preserved |
| Sparse carrier, long-range production activation, resolved body hydrodynamics | UNRESOLVED | Separate convergence and architecture experiments required |
| Canonical ship adoption | RESERVED | Requires separately authorized versioned physical contract |

Nominal mu is independent of reference density, but fixed nu implies local carrier
mu changes with the small density departure. Stokes drag uses nominalmu. Carrier
hydrostatic background is subtracted; no carrier gravity stratification. Statistical
species are dilute one-way terminal-slip fields; no backreaction/inertial settling
transient. Spheres are unresolved point coupling, with nearest-cell self-flow and
no lubrication, added mass, lift, Basset term, body/body collision or torque.
Continuum/Kn<<1 is assumed, not calibrated. No thermal convection, chemistry,
resources consumed, organisms, actuation or hull matter creation is claimed.

## Reproducibility and publication

Clean commit above passed `cargo test --offline --workspace --all-targets --all-features`:
**274 passed,0 failed,0 ignored**,155.778s. Eleven focused tests, strict workspace
all-target/all-feature Clippy, formatting and nine Python integrity tests pass.
Reaction/source/atomicity intentional violations fail assertions (exit101), then
restored tests pass. All initial parser/style failures remain evidence. See
[clean-source receipt](HABITAT-001_VALIDATION.json) and
[focused receipt](HABITAT-001_FOCUSED_VALIDATION.json). Certification covers recorded
Rust/Cargo sources, not later documentation edits or hosted CI.

[Research provenance](../research/2026-11-27-habitat-001.json) preserves search intent,
exact links/identifiers, selected-text versus abstract-only access, timeout,
authored divergences and CONSULTED/IMPLEMENTED/VALIDATED status. A cited paper does
not certify the local model.

Raw working copy `/home/cognis/habitat-001-20261127`; standalone plots in `plots/`,
Witness capture `final-native-record/chamber.png`. The implementation archive
[receipt](HABITAT-001_STORAGE.json) retains303files/861,617,777memberbytes with verified
restoration. The immutable final qualification supplement is recorded in
[qualification storage](HABITAT-001_QUALIFICATION_STORAGE.json). Its227files/89,619,901memberbytes retain final
traces, references to implementation-archived captured-source binaries, clean validation, independent analysis/plot
scripts and figures, restored replay/source-refusal proof and Atlas snapshots.
The restored certified binary replayed the restored final demo for120ticks and
reproduced the complete final checkpoint exactly. Qualification archive SHA256:
`b8db989946e774a5b5da34e2c966a1d18be34e436e982d9bb4d34422f67f46ab`.
Both archives remain on verified Mermogon UUID D0A02EEBA02ED82C; previous versions
and local copies retained. Cargo targets/plotting installation are excluded
regenerable caches. Atlas navigation is updated; live source and archive receipts
retain authority. No bulky raw additions enter Git.

Next dependency: a separately scoped experiment for unresolved body hydrodynamics
or sparse carrier/conservative activation; canonical integration and appendage
actuation need explicit Operator authorization. No MORPH-006 work commenced.
