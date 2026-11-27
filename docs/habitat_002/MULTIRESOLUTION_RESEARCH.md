# Lane G: conservative multiresolution research

Independent scalar-only finite-volume interface probe in
`python/habitat_002_multiresolution.py`. Shared face flux preserves inventory across
coarse .025m/fine .0125m cells, with a uniformly fine reference. Cell-average
Gaussian initial inventory, constant advection and Fick diffusion are explicit.
Restriction sums mass; volume-weighted prolongation preserves mass but loses
subcell information (counterexample retained). This is a mathematical transport
probe, not another environmental owner or an adopted runtime path.

D3Q19 multiresolution NOT IMPLEMENTED. Changing h at fixeddt changes tau-.5 by
h^-2. Matching viscosity through dt/subcycling changes the relation between
moments and physical velocity. Naive population interpolation cannot establish
non-equilibrium stress scaling or acoustic interface transmission/reflection.
Mass/momentum restriction, nonequilibrium reconstruction, conservative flux ledger,
compatible temporal synchronization and source-bound representation transitions
need a separate coupled proof. No pressure-wave transmission, reflection, interface
attenuation or hydrodynamic saving result is claimed from the scalar probe.

Do not substitute it for B/C/D/E/F. Final scalar mass/L1/timing results are retained
separately and cannot certify atmospheric multiresolution. There is no Witness-
dependent refinement or approximate activation threshold in any adopted candidate.
