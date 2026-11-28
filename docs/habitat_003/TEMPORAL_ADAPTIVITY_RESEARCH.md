# Temporal adaptivity and causal synchronization

IMPLEMENTED: fixed2:1 FV subcycling only. No local event queue or arbitrary local
timesteps. begin_subcycle creates a coarse full-step predictor and advances fine
leaves one half-step using old coarse ghosts. Fine-to-coarse integrated exchange
is recorded provisionally. At the second half-step coarse ghosts interpolate
old and predicted states at alpha1/2. Fine leaves advance again. The coarse final
update retains its coarse/coarse and wall fluxes but replaces predicted interface
exchange with the two actual fine half-step fluxes. Equal/opposite integrated
exchange supplies the conservative correction; no concentration renormalization.

The committed State cycle stays unchanged until finish_subcycle passes validation.
Pending stores stage1, work, predictor and coarse-interface exchange. Load/save
validate it by deterministically reconstructing the expected half-cycle from the
committed base. This is expensive and bounded; it proves causal provenance for this
two-stage experiment. Topology changes are refused while pending. Rejection at
finish preserves both base and pending work, allowing recovery.

Uniform equilibrium, moving scalar/momentum, restored pending continuation and
injected finish refusal have focused tests. Physical/temporal convergence and
performance belong to final experiments. A synchronous conservative baseline
exists; temporal adaptivity is not presumed faster. HYPERS event scheduling and
AMReX hierarchy subcycling were consulted; neither donor was built or reproduced.
No future authoritative neighbor state, skipped exchange, or hidden extrapolation
is allowed. General asynchronous interfaces remain PROPOSED/NOT RUN.
