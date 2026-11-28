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
