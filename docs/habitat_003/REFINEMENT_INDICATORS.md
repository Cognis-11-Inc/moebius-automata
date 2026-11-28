# Physical refinement indicators

IMPLEMENTED optional versioned Indicators in Config: relative density jump,
velocity jump in m/s, scalar concentration jump in kg/m³, and LBM normalized
nonequilibrium norm. Each is divided by its explicit threshold; max score>=1
refines the associated coarse root. Fine roots coarsen only when all child scores
remain below coarsen_fraction for quiet_cycles committed cycles. Thresholds and
quiet history are checkpointed and hashed. Refinement capacity refusal rejects
the transaction rather than dropping material or silently changing thresholds.

Tests exercise quiet-cycle hysteresis and physical indicator state. The example
qualification plan uses a static refinement window to isolate numerical errors;
it does not establish the moving refinement strategy's scientific adequacy.
Actual error estimation, density/shear/vorticity-based moving adaptation and
threshold sensitivity qualification remain incomplete. The current gradients
are indicators, not certified truncation-error estimates. No camera, zoom, mesh
visibility, renderer precision or viewer trajectory is accepted as indicator input.
