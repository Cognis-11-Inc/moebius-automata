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

## Final measurements

| Kernel | Elements | Kernel mean ms | Host dispatch+finish ms | Upload ms | Download ms | Derived GFLOP/s |
|---|---|---|---|---|---|---|
| micro_f64 | 16384 | 0.020736 | 0.0480497 | 0.271239 | 0.08233 | 101.136 |
| micro_f64 | 262144 | 0.204352 | 0.235149 | 0.808716 | 0.815696 | 164.199 |
| micro_f64 | 1048576 | 0.786741 | 0.817433 | 1.9096 | 2.28077 | 170.6 |
| micro_f32 | 16384 | 0.006112 | 0.02144 | 0.052449 | 0.03605 | 343.12 |
| micro_f32 | 262144 | 0.007968 | 0.056263 | 0.360539 | 0.435568 | 4211.15 |
| micro_f32 | 1048576 | 0.021408 | 0.0505697 | 0.999155 | 1.06275 | 6269.51 |
| micro_copy | 16384 | 0.00357333 | 0.01681 | 0.03801 | 0.03397 | 0 |
| micro_copy | 262144 | 0.00606933 | 0.03326 | 0.301359 | 0.301018 | 0 |
| micro_copy | 1048576 | 0.0145813 | 0.0436263 | 1.61447 | 1.6832 | 0 |

Actual detached outputs match separately rounded FP32 and strict FP64 scalar recurrences bitwise. Full pressure/scalar tails/source ledgers remain FP64 CPU; no reduced-precision fluid lane was tested.
