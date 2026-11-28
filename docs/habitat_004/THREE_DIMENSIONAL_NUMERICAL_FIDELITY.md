# Three-dimensional numerical fidelity

IMPLEMENTED fixed1m³ controls at8/12/16 cells in each axis: tensor-product scalar
diffusion, diagonal advection/diffusion, multiaxis divergence-free shear, oblique
linear waves and a manufactured three-dimensional vortex. Unlike003, every x/y/z
extent stays fixed during resolution sweeps. Uniform and central-refined cases use
new analytic CELL AVERAGES on actual leaves. Trigonometric integrals use sinc;
packet initialization uses8-point Gauss-Legendre quadrature with4-point discrepancy.
An independent Python evaluator uses analytic averages and math.erf for Gaussian
packets, not the Rust evaluator. Initialization, evolution and model-form error are
separate. Final qualification is recorded below and in HABITAT-004_RESULTS.md.

The vortex is Taylor-Green shaped, u=A(sinx cosy cosz,-cosx siny cosz,0), with
A=.005 exp(-3nu*k²t). Constant density, p=cs²rho and vector-Laplacian viscosity
require external F=rho*u·grad u for this exact manufactured solution. Components
Fx=rho*A²*k*sinx*cosx*cos²z, Fy=rho*A²*k*siny*cosy*cos²z,Fz=0. They are integrated
as cell averages and tracked in manufactured_impulse. This is NOT an unforced
exact Taylor-Green vortex or a thermal-energy equation. A separate nonzero integral
test verifies external impulse bookkeeping, beyond a zero-sum periodic example.

Spherical propagation, unforced vortical reference, arbitrary oblique scattering,
interacting localized3D pulses and general rotational equivalence remain NOT RUN
unless the final matrix establishes them. Passing periodic analytic controls does
not imply arbitrary3D biological fluid dynamics.

The oblique-wave reference now solves the exact LINEARIZED VISCOUS density/momentum
system, including initial mean/second harmonic. It remains an approximation to the
nonlinear atmosphere. The earlier inviscid comparator is preserved in implementation
archivev1, superseded before qualification. Packet translated-Gaussian error fields
are labeled kinematic comparators; matched numerical references govern acceptance.

## Final measured qualification

| Control / scheme | Error n8 | n12 | n16 | Trend / all within bound | Family verdict |
|---|---|---|---|---|---|
| Diffusion3 / reference | 0.000722409 | 0.000322142 | 0.000180021 | True / True | QUALIFIED |
| Diffusion3 / combined | 0.000726268 | 0.000326223 | 0.000184182 | True / True | QUALIFIED |
| Diagonal / reference | 0.0017173 | 0.00119822 | 0.000915094 | True / True | QUALIFIED |
| Diagonal / combined | 0.000712936 | 0.000371764 | 0.000233261 | True / True | QUALIFIED |
| Shear3 / reference | 0.195719 | 0.146853 | 0.1161 | True / False | REJECTED |
| Shear3 / combined | 0.0865857 | 0.0412659 | 0.0307621 | True / True | QUALIFIED |
| Oblique / reference | 0.332 | 0.261298 | 0.211591 | True / False | REJECTED |
| Oblique / combined | 0.184551 | 0.107745 | 0.0729962 | True / False | REJECTED |
| Vortex / reference | 0.133661 | 0.0997363 | 0.0787329 | True / True | QUALIFIED |
| Vortex / combined | 0.0679535 | 0.0340596 | 0.0203404 | True / True | QUALIFIED |

Normalized density error is relative to the wave amplitude rho0×1e-4, not rho0. Shear/vortex use prescribed velocity amplitudes; scalar uses C0/2. All t=.05s. HOC n8 acoustic failure is retained; n12/16 meet the bound. The forced vortex is one manufactured flow, not independent unforced turbulence evidence.

Refined combined errors:

| Control | n8 | n12 | n16 |
|---|---|---|---|
| Diffusion3 | 0.000921491 | 0.000429071 | 0.00024093 |
| Shear3 | 0.0812105 | 0.0396432 | 0.0287846 |
| Oblique | 0.173351 | 0.101287 | 0.0686385 |
