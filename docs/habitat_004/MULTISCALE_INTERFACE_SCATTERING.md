# Localized multiscale interface scattering

IMPLEMENTED normal plane-Gaussian packet recipes on fixed[1,.03125,.03125]m
periodic domains. The transverse field is uniform; these are intentionally planar
interface controls, separate from the genuine3D cubes. Coarse x32/64 keeps all
physical extents fixed. Roots with x>=.5 refine by2; incident centres .25/rightward
and .75/leftward cross the interface before t=.12s. Width.05 and selected.025m
controls, homogeneous coarse/fine references and one double-interface slab are
registered before qualification. Final outcomes are recorded below.

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

## Final measured qualification

| Case | Excess reflected / incident | Transmission peak error | Phase (m) | Arrival (s) | Bounds |
|---|---|---|---|---|---|
| packet-32--1-reference-refined | 0.000217231 | 0.127023 | 0.015625 | 0.0019 | True |
| packet-32--1-combined-refined | 0.00102216 | 0.15174 | 0.015625 | 0.0032 | False |
| packet-32-1-reference-refined | 3.36862e-05 | 0.174055 | 0 | 0.0013 | False |
| packet-32-1-combined-refined | 4.06311e-05 | 0.197322 | 0 | 0.0002 | False |
| packet-64--1-reference-refined | 0.000223288 | 0.108626 | 0.0078125 | 0.0009 | True |
| packet-64--1-combined-refined | 0.000642306 | 0.0932184 | 0.0078125 | 0.0016 | True |
| packet-64-1-reference-refined | 2.87473e-05 | 0.150489 | 0.0078125 | 0.0008 | False |
| packet-64-1-combined-refined | 3.8229e-05 | 0.12644 | 0 | 0.0003 | True |
| narrow-64--1-combined-refined | 0.000493864 | 0.140997 | 0.0078125 | 0.0016 | True |
| narrow-64-1-combined-refined | UNRESOLVED | 0.191516 | 0 | 0.0001 | None |
| double-interface-64-reference | 5.56015e-05 | 0.143878 | 0 | 0.0007 | True |
| double-interface-64-combined | 3.20786e-05 | 0.120598 | 0 | 0.0018 | True |

Characteristic reflection subtracts the homogeneous control at the SAME upstream resolution. Transmission is compared against homogeneous fine. Global native-versus-fine reflected-characteristic difference and ordinary coarse/fine damping remain in SCALING.json; they are not interchangeable with interface reflection. Width .025 rightward lacks its registered matching homogeneous coarse control: UNRESOLVED, never PASS. Double-interface actual x limits are .40625/.59375m, derived from selected roots. Thin plane-symmetric periodic box, t=.12s; no oblique/moving-interface inference.
