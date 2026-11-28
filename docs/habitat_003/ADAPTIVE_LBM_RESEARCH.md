# Adaptive lattice Boltzmann: retained failed candidate

Exploratory native two-level D3Q19 implementation in scientific::lbm reuses the
legacy equilibrium, moments, directions and SI scaling. Acoustic scaling uses
hf=hc/2 and dtf=dtc/2, preserving c=h/dt. Physical nu requires
(tauf-1/2)=2*(tauc-1/2), with tau_l=1/2+3*nu*dt_l/h_l².

The authored pre-collision Chapman-Enskog estimate fneq_l proportional to
-tau_l*dt_l*D(feq) gives transfer factor (tau_target*dt_target)/
(tau_source*dt_source). Local rescale preserves equilibrium density/velocity
moments to roundoff. Restriction averages fine distributions then rescales;
prolongation is piecewise constant with that factor. One coarse predictor and
two fine steps use interpolated coarse ghosts. There is NO conservative repair.

CONSULTED Palabos2021 selected full-text slides, Dupuis/Chopard2003 abstract and
bibliography. Palabos uses transformed/raw nonequilibrium notation; slide15 was
rendered. The mapping to the locally stored pre-collision distribution was NOT
established. This is not a published-algorithm port or a validated stress closure.

Pilot right-traveling periodic wave, n16,dt0.001, relative amplitude1e-4, static
central refinement: first attempted step had carrier residual1.108082696e-7kg and
x-momentum residual-5.595405591e-6kg*m/s. Guard refused it; full committed digest
was unchanged. Candidate state is retained in raw summary. Uniform equilibrium
alone passes and is insufficient. No normalization, floor, clipping or forced
acceptance was introduced. More interface tuning is deferred; scalar/body/gravity
extensions of this failed candidate are NOT RUN. Final repeated failure is in the
qualification table. The general viability of published adaptive LBM is UNRESOLVED;
this rejection applies to the authored candidate only.
