# HABITAT-004 preregistration — 2026-11-27 local

## Containment first

PARTIAL is permitted; P0 first, independent P1 characterization next. No canonical
ship/biology/MORPH integration, new coordinate/material/history authority, imported
solver code/dependencies, driver/system tuning or deliberate hardware exhaustion.
Old adaptive LBM source and rejection remain unchanged. Old001/002/003 executables,
source capsules, archives and published verdicts remain authoritative historical
references. Every mandatory gate uses fail-closed execution; no failed gate may be
followed by commit/publication. No clipping, normalization, hidden thresholds or
changing tolerances after observation. Keep nulls and failures.

## Entry and reuse ruling

Clean main e02b720/remote equal, previous certified5d69505. Six001–003 archive and
manifest pairs rehashed; all760 members freshly restored from003 final archive.
92 Atlas notes read and checked. All001–003 markdown/JSON dossiers loaded and
retained. Eleven selected baseline complete checkpoints reproduce exactly:
diffusion/wave n16/32/64 refined monorate, shear32 mono/subcycle, largest tube,
original LBM rejection and002 buffered reference. Fresh shear mono0.528811ms,
subcycle2.896886ms. Mono measured stages: clone3968ns, topology305184ns,
numerical176980ns, validation40698ns. These are fresh windows, not replacements
for published timings. Invalid timing reduction on empty rejected-run samples was
retained and corrected to null mean/P99 with0 samples.

| Concern | Path/symbol | Status/input -> output/consumer | Authority/tests/reuse/missing seam |
|---|---|---|---|
| Native physics | primordial/scientific::{Chamber,State,Payload,Config} | active experimental selected owner; SI config/initial data -> committed leaves; habitat_scientific | retain sole scientific owner; extend configuration and kernels, not a third owner |
| FV | scientific/fv::{fluxes,advance,begin,finish} | first-order barotropic law, fixed2:1 pending protocol | original model/default remains reference; add separate explicit reconstruction/time choices |
| Geometry | scientific/mesh::{geometry,faces,refine,coarsen,validate} | actual leaves -> deterministic face graph; no shadow finest volume | reuse two-level geometry; cache derived immutable faces keyed by complete geometry inputs; recursive keys NOT implemented by relabeling |
| Controls | scientific/experiments.rs, habitat_scientific.rs |003 one-dimensional controls -> reports/checkpoints | retain reproducibility; add fixed-box3D and packet controls inside same responsibility |
| Observation | habitat-witness.rs Authority -> draw | legacy owner only, no scientific consumer | no duplicate state for rendering; native selected-owner adapter is a later dependency |
| Baseline/coupling | primordial/mod.rs couple, Species, Sphere; scalable | legacy typed closure, canonical-independent | preserve unchanged; body refusal stays until separately qualified |
| Provenance/archive | benchmark_integrity.py, f64 encoding/source identities | source capsule/executable -> verified replay | reuse capture conventions and Mermogon, extend scientific schema/config visibly |
| Hardware | local sysfs/lscpu/Vulkan/OpenCL prior probe | observed inventory, not theoretical performance | no permanent changes; absent perf/NVML access explicitly recorded |

Inspected governing UAP/HANDOFF, MORPH004 canonical ruling and MORPH005 containment,
current source/call sites, current root configuration/history and Atlas. No competing
AMR physical authority found beyond the two retained explicit experimental models.
Historical unmerged branches and inaccessible proprietary internals not exhaustively
inspected. Checkpoint schema/source changes intentionally refuse old003 payloads in
new binaries; old binaries remain available. CategoryI claims require complete
like-model payload/ledger equality, not headers with new metadata.

## Independently authored numerical candidates

Existing isothermal p=cs²rho, constant-mu vector-Laplacian and passive scalar law
retained. Separate Reconstruction::{Constant,Minmod} and Integrator::{Euler,Ssp2}.
Minmod primitive slopes use area-weighted one-sided normal neighbor differences
and physical distances. Reconstruct at actual subface centre; coarse/fine tangential
centroid displacement is explicit. Diffusion/viscosity retain existing centre-gradient
closure, not a simultaneous constitutive change. No universal second-order or TVD
claim for coarse/fine3D primitive reconstruction. Nonpositive/nonfinite reconstructed
states refuse the transaction. No corrective floor.

SSP2: U1=U0+dt L(U0,t); U2=U1+dt L(U1,t+dt); final=(U0+U2)/2.
Source/wall reservoirs and impulses receive the same convex combination. Topology
changes only after synchronized admission. Positivity remains tested admission,
not a theorem from naming SSP. Higher-order subcycling is refused until interface
qualification; original constant/Euler subcycle path is retained. Donors consulted:
van Leer1979 publisher metadata; Gottlieb/Shu/Tadmor2001 selected full text;
Clawpack limiter docs; Zeng2013/2014 nonuniform-limiter assumptions; AMReX docs.
D-levels and exact inspected evidence are recorded separately. No donor code port.

## P0 controls and preregistered bounds

Physical SI units; existing carrier/scalar inventory tolerance1e-14+1e-10 initial,
accounted momentum1e-10+1e-9 impulse scale; density deviation<=5percent,
Mach<=0.1 and current conservative CFL<=0.9 unchanged. Any failed step preserves
committed logical digest; cache/telemetry has no mutation rights over physics.

Fixed3D boxes: [1,1,1]m at n8/12/16 in EACH axis, h=1/n; dt=0.0005s,
t=0.05s initially. Every resolution preserves all three dimensions. Uniform and
central physical refinement cube[0.25,0.75]^3 are explicit scenarios. Refined initial
fields are newly evaluated as analytic cell averages on actual leaves, never just
constant-prolonged coarse samples. Trigonometric averages use sinc factors; packet
Gaussian integrals use explicit quadrature with a separately recorded order check.

-3D diffusion: C=C0*(1+0.5*cos(kx)*cos(ky)*cos(kz)*exp(-3Dk²t)), k=2pi/m,
 C0=1e-4kg/m³,D=0.01m²/s,u=0,rho20kg/m³. Scalar normalized volume L1<=0.15
 and error decreases over3h for a qualified spatial family.
-Diagonal advection: product cosine translated by u=[0.03,0.02,0.01]m/s,
 D=0.001, exact corresponding advection/diffusion; same normalized L1 bound.
-Multiaxis shear: u=a sin(k*(x+y+z))*exp(-3nu*k²t), a=[.006,-.003,-.003]m/s,
 a dot[1,1,1]=0, rho constant, nu=mu/rho=.005. Thus nonlinear advection vanishes
 analytically. Velocity normalized L1<=0.15, spatial error decreases. First-order
 normal-momentum acoustic damping may FAIL this control; no threshold relaxation.
-Oblique linear wave: integer wavevector[1,1,1], density amplitude1e-4,
 u=cs*amplitude*unit(k)*cos(phase-cs*|k|t). Linear approximation has nonzero model
 error at finite amplitude. Normalized density L2<=0.15, not exact nonlinear truth.
-Manufactured vortex: divergence-free Taylor-Green-shaped u=(A sinx cosy cosz,
 -A cosx siny cosz,0), A=.005*exp(-3nu*k²t). Explicit external forcing rho*u·grad u
 makes this a manufactured solution of the DECLARED isothermal model. It is not
 an unforced exact Taylor-Green vortex. Force is configured, versioned, integrated
 and included in external impulse accounting. Velocity L1<=0.15; source quadrature
 and initialization errors recorded. No total thermal-energy claim.

Normal acoustic packet: [1,.03125,.03125]m, coarse roots32/64 in x, transverse roots
n/32 each, fixed physical dimensions, dt=0.0001s,t=.12s. Static interface x=.5m,
rightward packet x0=.25 (coarse->fine) or leftward x0=.75 (fine->coarse), Gaussian
width .05m, relative amplitude1e-4, rho'=rho0*amplitude*G,u_n=direction*cs*amplitude*G.
Use homogeneous coarse and homogeneous fine controls with newly evaluated initial
averages and same physical duration. Two-level geometry only. Characteristic
q+/-=(rho'/rho0 +/- u_n/cs)/2 separates modes in the linear regime. Measure peak,
phase/centroid, density/velocity probe arrival, shape L1, frequency/dissipation proxies,
and linear acoustic energy diagnostic cs²rho'^2/(2rho0)+rho0*u²/2 (not thermal energy).
Reference difference is measured on a common finest diagnostic sampling grid only.

Registered interface bounds: excess reflected peak<=0.10 incident amplitude;
transmitted peak relative difference<=0.15 versus homogeneous fine; peak displacement
<=one coarse h; arrival discrepancy<=2h/cs. All must pass for this bounded interface
qualification. Ordinary homogeneous damping is reported separately. No extrapolation
to oblique/moving interfaces. Packet quadrature discrepancy order4 versus8 normalized
by amplitude<=1e-6; otherwise initialization gate FAIL. No post-hoc packet tuning.

Ablations constant/Euler, Minmod/Euler, constant/SSP2, Minmod/SSP2 first on uniform
controls, then refined. Intended higher-order wave improvement must reduce matched
error by at least1.5x and meet the0.15 bound; failure remains FAIL. Steep scalar slab
transport must remain nonnegative, conserve, and not create values outside its initial
extrema beyond1e-12 absolute; otherwise report FAIL, never clip.

## P1 proof and measurement boundaries

Persistent topology is derived only: geometry identity includes ordered leaf keys,
h, roots and periodicity. Payload changes do not invalidate it; refinement/coarsening
must invalidate/rebuild. Private immutable cached faces cannot become material state.
Cached and rebuilt operation order must yield exact complete physical state/digest
for the SAME new schema. Deliberate stale/missing-neighbor mutation must expose
invalidation failures and preserve committed state. No recursive hierarchy claim.

Profile current stages before replacing representations. Hardware inventory distinguishes
available information, unavailable privileged counters/NVML access and observed execution.
Microbenchmarks use bounded<=192MiB arrays and total memory estimates, process-local
workers/affinity only. Scalar/vector memory kernels are NOT full environmental speedups.
Ordinary timing repeated3 windows where feasible, all samples/no silent outlier removal;
instrumented results separately labeled. Match physical duration/dt/accuracy in temporal
comparisons. Cache cost, full transaction, checkpoint and validation remain included
in appropriate totals; no validation removal to force timings.

## Publication gate

Focused falsification/static gates -> immutable source/executable/evidence capsule with
verified Mermogon identity, hashes, restoration and execution -> authorized source
commit -> clean recorded full workspace suite -> final scientific runs -> independent
analysis and failure matrix -> versioned final archive/restore -> metadata publication
and remote verification. No canonical/source changes outside authorized responsibilities.
All66 experiment rows receive honest status; unimplemented dependencies are NOT RUN.

Pre-pilot resource clarification: the normal packet uses periodic transverse lengths
0.03125m (one/two coarse transverse cells at n32/64), preserving those physical lengths
across resolution. This intentionally plane-symmetric control is separate from genuine
3D cube tests. It avoids an unnecessary16fold transverse workload; no new packet result
was observed before this change. 3D scalar norms integrate |C-Cexact|V/(C0*0.5*Vbox);
velocity L1 integrates norm(u-uexact)V/(peak prescribed vector amplitude*Vbox); wave
density L2 is RMS divided by rho0*1e-4. Errors use analytic CELL AVERAGES, not centres.
Additional fixed-mesh temporal control: oblique wave n12, dt.002/.001/.0005 versus
.000125 numerical time reference, all t=.05s. Decreasing same-grid endpoint difference
is required; no analytic spatial-error cancellation is advertised as temporal order.
Steep scalar case t=.5s at n16; sustained3D advection t=2.5s at n8, instrumented
heap observations distinct from ordinary timings. Isotropic background scaling h=.125m,
roots8/16/24 peraxis, fixed fine cube[.25,.75]^3, t=.01s; actual volumes1/8/27m³.

Prequalification plan extension (after four20-step smoke pilots, before final runs):
add three ordinary cached/rebuilt higher-order pairs to test exact equivalence for
Minmod/SSP2, not just the first-order path. Add matched static/physical-indicator
3D diagonal-scalar runs, n8,t=2.5s,dt.0005. Dynamic thresholds: density.001, velocity
.01m/s,scalar2e-5kg/m³,nonequilibrium.01,coarsen fraction.4,quiet8cycles. Same scalar
error<=.15 and inventory budgets; record topology changes and lost-resolution error.
This is a two-level translated sinusoidal field, not a localized cloud or arbitrary-depth
tracking/split/merge qualification. Initial113-case plan retained separately; final
plan121 cases. These controls were added for missing proof, not after observing
final candidate measurements. No tolerance changed.

Reference correction BEFORE any final qualification (implementation archivev1 retained):
the initial oblique comparator omitted viscosity. Use the linearized viscous equations
d(delta-rho)/dt=-div(P), dP/dt=-cs²grad(delta-rho)+nu*Laplacian(P). For each mode
K, gamma=nu*K²/2,omega=sqrt((cs*K)²-gamma²). Evolve cosine/sine coefficients with
exp(-gamma*t) times the damped two-variable oscillator. Include the mean/second
harmonic of the prescribed initial nonlinear rho*u product; full nonlinear PDE error
remains O(amplitude²). Restrict the reference to underdamped configured modes. All
error bounds stay unchanged. Uniform-cell mean momentum and viscous energy decay
have separate tests; an inviscid reference cannot pass the latter.

The translated Gaussian returned for packet density/velocity norms is explicitly a
KINEMATIC comparator, not an exact viscous evolution. Only homogeneous-fine matched
viscous numerical controls qualify scattering. Independent Python computes the same
linear viscous wave and uses erf Gaussian integrals. This correction changes reference
interpretation/code and source identity before qualification, not acceptance limits.

Scattering decomposition clarified before final execution: isolate reflection upstream
by subtracting the homogeneous control with the SAME upstream resolution (coarse for
rightward coarse->fine; fine for leftward fine->coarse), over the upstream half-box.
Also retain the original global difference versus finest reference and homogeneous
coarse/fine peak difference to expose ordinary bulk damping. Transmission/phase/arrival
bounds remain against homogeneous fine; no bound changed. Narrow-packet controls
without a matched homogeneous-coarse run cannot isolate coarse-upstream reflection;
that component must be marked UNRESOLVED rather than silently using fine instead.
