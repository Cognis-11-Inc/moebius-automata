# Hybrid physical regimes — conditional dossier and arithmetic probe

CONSULTED NASA CFD/DSMC report, SPARTA, HYPERS, WarpX and granular/particle methods.
No local DSMC, PIC, DEM or particle/continuum exchange is implemented. A mode name
or donor success cannot establish a physical regime in this project.

Authored dilute hard-sphere probe uses number density n=rho/molecule_mass,
mean free path lambda=1/(sqrt(2)*pi*diameter²*n), Kn=lambda/L and expected particles
per cell n*L³. It is a conditional arithmetic control, not a calibrated gas model.
Input molecule identity, collision law and temperature-dependent transport are
UNKNOWN for the intended atmosphere. `HYBRID_REGIME_PROBE.json` retains inputs,
units and outputs; it does not decide canonical physical law. High Kn calls for
kinetic qualification; low Kn alone does not validate the isothermal FV closure.

Before hybrid coupling: identify which representation owns mass/momentum/energy,
conservative handoff across boundaries, particle weights and fluctuation/error
budgets, moment restriction, particle reconstruction, reaction ownership and clock
synchronization. Test equilibrium detailed balance, flux cancellation, interface
reflection, variance versus sampling and costs across Kn regimes. Dense particulate
contact, surface deposition, fluid/solid transitions and gill uptake each need
distinct constitutive and boundary qualification; none is supplied by this dossier.
