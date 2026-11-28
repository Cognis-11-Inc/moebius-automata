# FORGE computational baseline

Current primary inventory uses lscpu/sysfs/PCI/Vulkan/OpenCL rather than assumed
marketing specifications. Ryzen9 7950X,16 physical cores/32 logical CPUs, two observed
L3 cache-sharing instances and available system memory are recorded with exact raw
commands. Local compiler/OS/driver identities are archived. DIMM layout and certain
privileged counters remain unresolved. NVIDIA management failed; OpenCL device
execution is a separate observation, not proof NVML works or free VRAM is known.

New independent hardware executable supplies portable/explicit256-bit FP64 and
FP32 arithmetic, four working-set sizes32KiB..64MiB, sequential/strided/dependent-LCG
reads and copies, and persistent1/2/4/8/16/32-worker arithmetic jobs. It estimates
at most192MiB array residency. All arithmetic is verified against scalar references;
verification is outside timed kernels. Black-box scalar baseline inhibits vectorization
and adds barriers. Derived operation/payload-byte rates are not DRAM or hardware ceilings.

Separate OpenCL FP32/FP64/copy probes reuse the installed ctypes interface; old
LBM program setup is untimed and never evolved. Strict contraction-off math and
complete output checks establish only these microkernels. Driver build and transfers
are measured separately. No full atmosphere, GPU reduction, mixed precision or
concurrent AMD renderer is qualified by a microbenchmark. Final ordinary measurements
run after clean source certification; earlier device probes are retained as pilots.

## Final measurements

| FP64 array bytes | Portable / AVX2 time | Scalar blackbox / AVX2 | Derived AVX2 GFLOP/s |
|---|---|---|---|
| 32768 | 2.02012 | 8.02675 | 43.4133 |
| 262144 | 1.34583 | 5.22095 | 27.6835 |
| 4194304 | 1.26192 | 5.01725 | 26.1337 |
| 67108864 | 0.975107 | 1.89628 | 9.46136 |

| Workers | Mean job ms | Setup ms | Exact output |
|---|---|---|---|
| 1 | 8.68204 | 5.72811 | True |
| 2 | 4.29286 | 2.12349 | True |
| 4 | 1.69502 | 1.68283 | True |
| 8 | 1.0338 | 1.542 | True |
| 16 | 1.42604 | 1.41621 | True |
| 32 | 1.60217 | 1.27035 | True |

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

All whole-array scalar checks passed. Three timed windows retained; host verification is outside them. FP32/FP64 here are arithmetic lanes, not fluid precision qualification. Device/global-memory enumeration is not freeVRAM. Workload/thermal/clock variation prevents a machine-ceiling claim. CPU affinity affects only these processes.

Inventory reports 32,689,651,712 bytes host RAM; DIMM/channel details unresolved. CPU0 L1d32 KiB/L1i32 KiB/L2 1 MiB shared by logical0/16, L3 32 MiB shared0–7/16–23; lscpu reports two L3 instances total64 MiB. NVIDIA PCIe point sample x8 at2.5 GT/s, maximum sysfs fields16/32 GT/s: this idle sample is not loaded link throughput. Storage identities are recorded but IO benchmarking NOT RUN. GPU copy rates are nominal traffic of repeatedly resident buffers and may include device-cache effects; they are not measured DRAM bandwidth or a hardware ceiling. No counter confirms cache residency.
