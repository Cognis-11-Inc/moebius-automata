# Lane E: detached FP64 OpenCL carrier

PCI identifies NVIDIA GeForce RTX5060Ti. OpenCL reports16,715,481,088 global-memory
bytes (15.57GiB), driver620.73.04, FP64 and device availability. NVML fails with
library620.73 versus loaded kernel615.78.08. CUDA cuInit succeeds; nvcc13.5 exists.
Vulkan enumerates AMD Radeon610M/RADV on this run, not NVIDIA. Bevy already uses
wgpu, but its available Vulkan device is not the requested NVIDIA compute device.
OpenCL is the smallest direct installed API tested here, through standard-library
ctypes, no added Rust dependency or second permanent runtime stack. No driver or
machine settings changed. `perf` executable unavailable. Full probe logs retained.

Own kernels derived from the existing D3Q19 equations: collision, pull streaming
with sealed bounceback/periodic reference, per-cell finite/nonnegative/Mach/density
validation. FP64, explicit operation order, FP_CONTRACT OFF, CL1.2 with no relaxed
math. Three persistent population-sized buffers and one status buffer. No complete
population transfer each tick: small local-validation status is downloaded;
full populations transfer for initial/checkpoint/qualification only.

Measure kernel event times, host enqueue, synchronize/status transfer, initial
upload and checkpoint download at8/16/24/32³. Requested VRAM is exact buffer sum;
actual total device VRAM consumption is NOT available through failed NVML and is
not inferred from allocation requests. Driver/compiler log retained.

This is NOT a GPU environmental backend: sphere exchange, scalar transport,
ordered wall-impulse ledger, global inventory admission and full Chamber staging
are not integrated. Final inventory/equivalence is checked in the experiment,
not used as a per-tick authoritative GPU commit. No complete-GPU-tick acceleration
or adoption claim. Rendering contention is measured on the actually enumerated
render device; cross-device contention is not same-GPU competition evidence.

## Final measured status

Final8/16/24/32³120-step final population bits equal CPU, repeated final execution and restored+3-step continuation exact.32³ collide+stream event mean0.201400ms; host/local-validation envelope0.659009ms. No complete environmental GPU update measured. Initial silent-log overlap protocol failed; corrected19-sample process overlap confirms AMD Vulkan render and NVIDIA compute are distinct devices. Same-GPU contention remains NOT RUN.
