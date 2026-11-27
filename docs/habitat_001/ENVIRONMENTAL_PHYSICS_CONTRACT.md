# HABITAT-001 experimental physical contract v1

Preregistered 2026-11-27, before implementation/outcomes. These are experimental
design assumptions, not measured properties of the fictional vessel.

## Cage and excluded claims

Isolated finite 3D chamber; no organism/flagellar actuation, chemistry, hull intake,
radiation, paid material exchange, historical migration or MORPH-006. Ship
force-free guards and old laws stay. This is a small isothermal weakly compressible
LBM reference, not a complete atmospheric/thermal/turbulence or resolved-body CFD
solver. Dense carrier is NOT a scalable enormous-ship architecture.

## Units and scale

SI: metre, second, kilogram; velocity m/s, acceleration m/s², force N, density
kg/m³, dynamic viscosity Pa·s (independent of density), pressure Pa, concentration
kg/m³, momentum kg·m/s, energy J. Temperature is an isothermal assumption; no
solved Kelvin field or thermal equation. No physical temperature from lattice
sound speed is inferred. One existing anatomical world unit is adopted as1m for
this experiment,1_000_000 micro-units per metre. Existing Position supplies a
normalized global origin; all numerical positions are local metre offsets.
Observer quantization never feeds the solver. Tick is existing16_666_667ns;
fixed configured1..8 substeps per tick, no wall-clock feedback/adaptive dt.

## Bulk reference

Cell-centred D3Q19 BGK populations, fixed deterministic cell/direction order.
Weights rest1/3, axes1/18, diagonals1/36; second-order polynomial equilibrium.
Each cell spans h=L/N; dt=tick/substeps. rho=sum(f), u_L=sum(f*c)/rho,
u_SI=u_L*h/dt, nu=mu/rho_reference, tau=0.5+3*nu*dt/h².
Viscosity is selected independently; no density-to-dynamic-viscosity inference.
Require0.501<=tau<=2, lattice Mach<=0.1, finite nonnegative populations,
relative density departures<=5percent; reject unstable steps atomically.
Physical bulk viscosity/real-gas equation of state not calibrated. Dynamic
pressure=(rho-rho_reference)*(h/dt)²/3; this is artificial-compressibility pressure,
not absolute atmospheric pressure or a thermal equation of state.

Default sealed six walls use halfway bounceback. Reflected population momentum
change is charged as external wall impulse. Periodic topology exists only as
an explicit numerical-reference control, never silently substituted for walls.
No pressure Poisson/global projection: local collision/streaming propagates
acoustic influence over time. It does not stop at an observer boundary.

## Gravity and inert spheres

Uniform explicit weak g, including zero and arbitrary direction, norm<=0.05m/s².
Carrier hydrostatic equilibrium is subtracted: no carrier gravity density-gradient
or stratification solve. Sphere closure: Fg=mass*g, Fb=-rho_local*volume*g,
Fdrag=6*pi*mu*r*(u-v). Buoyancy is a prescribed hydrostatic-pressure support force;
its impulse is explicitly charged to the external hydrostatic boundary reservoir,
NOT mislabeled as resolved pressure integration or an internal conserved force.
Carrier mass is finite; drag impulse is transferred oppositely to its sampled cell.
No infinite-reservoir carrier is claimed. Analytical reservoir controls, when used,
are separate scalar ODE references, not the closed chamber result.

Nearest-cell sampling, unresolved small spheres (r<=h/4), volume loading<=1percent,
Re_p=2*r*rho*|v-u|/mu<=0.1. No lubrication, added mass, Basset memory, excluded-volume
flow, lift, surface traction, torque or body/body collision is implemented.
The single body/sample-cell linear drag/gravity subsystem has an exact exponential
solution using reduced mass; update body velocity/position and cell impulse from
that solution. Sequential stable body-ID order is specified. Cell impulse uses
an equilibrium-difference population shift, preserving its old nonequilibrium part.
No ongoing prescribed displacement or thrust. Vacuum requires rho=mu=0; no fluid
populations/forces, exact constant-acceleration position/velocity integration.

Body wall contact is a bounded inelastic/specular impulse plus positional
projection at the step endpoint; it is a numerical contact constraint, not a
hydrodynamic force or precise curved-shape continuous collision solver. Wall
impulse and kinetic energy loss are recorded. Convergence controls avoid walls.

## Suspended species

At most4 dilute passive species. Identity, radius, material density and effective
diffusivity have consumers: Stokes terminal slip2*r²*(rho_p-rho)*g/(9*mu),
advection and diffusion. Fields store kg per cell; displayed concentration divides
by cell volume. Species loading<=0.1percent of carrier mass. This is one-way
terminal-slip closure: particle momentum/inertia/backreaction is NOT represented
by statistical fields. No claim of total momentum including tracers follows.
Explicit test spheres provide the independently validated two-way interaction.

Conservative face-paired first-order upwind flux plus centred Fick diffusion;
fixed face order. All outgoing coefficient sums*dt<=0.9 for positivity, otherwise
refuse the step. Normal carrier boundary flow is zero. Outward species slip
sticks to the corresponding wall inventory; diffusion has zero wall flux. Matter
stays inside the sealed system: suspended+six deposited wall stocks conserved.
No clipping/renormalization or unaccounted material sink. Upwind spreading is
numerical dissipation, measured separately from prescribed diffusivity.

Dense scalar reference and sparse active-face/map representation must agree
numerically with the same face order; carrier remains dense in both. Zero scalar
cells do not justify freezing carrier evolution. Sparse influence/activation must
include every adjacent flux face; no finite Witness radius participates.

## Conservation, replay and acceptance

Carrier mass and each species suspended+wall inventory: relative error<=1e-10
and absolute species error<=1e-16kg. Fluid+explicit-body momentum closes against
gravity, prescribed hydrostatic support and actual wall impulses, absolute
residual<=1e-10kg·m/s plus1e-9 relative to initial/accumulated impulse scale.
Drag loss>=0 for the exact pair ODE; wall loss>=0 for restitution<=1. Fluid
kinetic/acoustic energy and viscous shear decay measured, NOT a complete thermal
energy ledger or claim of total energy conservation.

Step is provisional until validation; errors preserve source state. Snapshot
encoding uses exact f64 bits with schema/source identity/digest checks, adapting
existing JSON/atomic helpers. Exact replay means same executable/target/features,
fixed dt/order and initial bits; no cross-compiler/architecture guarantee.
No new ship event/history format or old snapshot conversion.

E01 equilibrium; E02 vacuum; E03 analytic gravity; E04 drag ODE; E05 density/
neutral buoyancy; E06 species slip/conservation; E07 analytic periodic advection/
diffusion; E08 local/two disturbances and shear-wave decay; E09 two-way impulse;
E10 independent spatial/timestep convergence; E11 repeat/checkpoint; E12 independent
resolution/active-region/species/object factorials. Report nulls/failures, actual
error slopes and validity violations, not only PASS labels. Provisional development
budget: environmental update<=2ms mean/4ms P99 at a stated small reference grid,
leaving most16.667ms for future work. This is not a production resolution choice;
measure before recommending one.

## Consulted foundations, not local validation

BGK/halfway bounceback and unit separation follow the inspected sections of the
[OpenLB guide](https://www.openlb.net/wp-content/uploads/2011/12/olb_ug-0.4r3.pdf)
and its [unit-conversion API](https://openlb.net/DoxyGen/html/d6/d92/classolb_1_1UnitConverter.html).
The low-Re drag closure uses [MIT's Stokes-law discussion](https://ocw.mit.edu/courses/12-090-introduction-to-fluid-motions-sediment-transport-and-current-generated-sedimentary-structures-fall-2006/7841d9b1681d6748fa2f5cbc6d6f1cb2_ch3.pdf).
Flux-paired scalar conservation follows the principle described by the
[Clawpack authors](https://www.clawpack.org/riemann_book/html/Approximate_solvers.html).
The exact local reduced-mass composition, bounds, inventory/impulse accounting
and validity cage are authored here and need the local tests. Full provenance,
queries, partial access/timeouts and CONSULTED/IMPLEMENTED/VALIDATED separation
are recorded in docs/research/2026-11-27-habitat-001.json. No external source
certifies this implementation or the fictional atmosphere.

Before final suite: analytic vacuum position/velocity tolerance1e-12 SI; exact
single-cell pair drag reference tolerance1e-14m/s; initial neutral buoyancy
velocity tolerance1e-12m/s. Statistical settling centroid drift is compared with
terminal slip before substantial wall deposition, tolerance1e-7m plus1percent of
expected drift. Scalar/shear spatial errors must decrease on8/16/32 at matched
physical conditions. Report time-refinement differences rather than assuming
error against the PDE decreases monotonically at fixed coarse h (spatial/time
error cancellation is possible). Machine-floor exact gravity has no meaningful
fitted order. Sparse equivalence requires every stored physical mass bit and
wall stock to match the dense face-order reference; do not certify carrier sparsity.

Interpretation clarification: fixed BGK tau uses nu=mu/rho_reference, so mu is the
nominal effective dynamic viscosity at reference density; the carrier's implied
local dynamic viscosity varies with its small density departure. The unresolved
Stokes closure uses the configured nominal mu. The5percent density cage bounds
this distinction; no exact variable-density constant-dynamic-viscosity model is
claimed. Continuum/Kn<<1 is an external validity assumption: molecular mean free
path and rarefaction corrections are not represented or calibrated. Particle
sizes and material names in demonstrations are authored control classes, not a
measurement of the vessel's atmosphere.
