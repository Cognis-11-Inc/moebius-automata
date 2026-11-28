# Independent finite-volume competitor

IMPLEMENTED isothermal low-Mach FV with integrated leaf inventories, shared face
fluxes, explicit viscosity/diffusion, gravity, sealed free-slip or periodic boundaries,
and passive species/terminal slip. No thermal equation, full compressible stress,
projection, turbulence closure, rarefied collision model or resolved body surface.
The material constants are authored controls inherited in scale from001; they
are not laboratory measurements or a calibrated ship atmosphere.

Uniform/refined equilibrium and nonuniform scalar/momentum conservation tests
pass. Final controls compare analytical sinusoidal diffusion, advection phase,
shear decay, linear acoustic waves, weak hydrostatic profiles and uniform-fine
references. Refinement starts with constant prolongation of sampled coarse
initial data. Error includes that initialization approximation. Same inventory
budgets apply to all grids; numerical errors may fail their registered thresholds.

Positive finite state and low-Mach/density validity are admission conditions,
not an accuracy proof. The boundary reaction ledger is necessary for momentum
accounting. Passive bodies currently fail the envelope instead of being frozen
or advanced by an unqualified closure. Existing001 body tests are legacy evidence.
No claim of end-to-end feeding, gill physics, chemistry or canonical integration.
