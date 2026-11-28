# Localized multiscale interface scattering

IMPLEMENTED normal plane-Gaussian packet recipes on fixed[1,.03125,.03125]m
periodic domains. The transverse field is uniform; these are intentionally planar
interface controls, separate from the genuine3D cubes. Coarse x32/64 keeps all
physical extents fixed. Roots with x>=.5 refine by2; incident centres .25/rightward
and .75/leftward cross the interface before t=.12s. Width.05 and selected.025m
controls, homogeneous coarse/fine references and one double-interface slab are
registered before qualification. Final outcomes pending at this checkpoint.

In the linear isothermal regime, q±=(rho'/rho0±u_n/cs)/2 separates directions.
Python independently reads final physical payloads and compares characteristic
profiles on a common finest diagnostic sampling grid. Excess reflection is native
minus homogeneous-fine reflected characteristic, normalized by nominal incident
amplitude. Transmission compares outgoing peaks, peak displacement and probe-arrival
time. The leftward probe is atx=.35, rightwardx=.65. Ordinary numerical damping
is reported separately. See protocol for all fixed acceptance bounds.

Linear acoustic energy integrates cs²rho'^2/(2rho0)+rho0*|u|²/2. It is an approximate
small-perturbation diagnostic, not conserved total thermal energy. Periodic Gaussian
tails are finite, and quadrature error is separately recorded. General oblique,
moving-interface and wavelength-resolved transfer functions remain unqualified;
two widths alone do not establish the complete frequency response.
