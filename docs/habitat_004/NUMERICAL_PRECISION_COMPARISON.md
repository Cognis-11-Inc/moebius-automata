# Numerical precision comparison

Atmospheric FV and original LBM remain FP64. No FP32 or mixed-precision atmospheric
model is implemented or qualified. All existing inventory/error budgets remain.
Separate hardware kernels exercise FP32 and FP64 recurrences, with contraction off
for OpenCL and exact scalar verification. Those kernels have no physical density,
scalar tail, pressure, interface or conservation accounting; they cannot qualify
reduced precision for the atmosphere. No factor from their throughput may be used
as a whole-simulation acceleration estimate.

Future precision lanes must specify each stored variable, arithmetic/reduction
precision, conversion point, rounding mechanism, tail preservation, source ledger
and checkpoint identity. Test long-duration inventory drift and interface errors,
not merely a visually similar field. Mixed precision remains NOT RUN.
