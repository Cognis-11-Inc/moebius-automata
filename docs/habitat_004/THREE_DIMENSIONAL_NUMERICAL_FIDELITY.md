# Three-dimensional numerical fidelity

IMPLEMENTED fixed1m³ controls at8/12/16 cells in each axis: tensor-product scalar
diffusion, diagonal advection/diffusion, multiaxis divergence-free shear, oblique
linear waves and a manufactured three-dimensional vortex. Unlike003, every x/y/z
extent stays fixed during resolution sweeps. Uniform and central-refined cases use
new analytic CELL AVERAGES on actual leaves. Trigonometric integrals use sinc;
packet initialization uses8-point Gauss-Legendre quadrature with4-point discrepancy.
An independent Python evaluator uses analytic averages and math.erf for Gaussian
packets, not the Rust evaluator. Initialization, evolution and model-form error are
separate. Final qualification is pending at this implementation checkpoint.

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
