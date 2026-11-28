# SIMD and persistent CPU parallelism

IMPLEMENTED in an isolated hardware example: portable loops, black-box scalar
reference, and runtime-checked256-bit float/double intrinsics on an AVX2-capable
machine. Local ISA availability AND execution correctness are checked. FP32/FP64
storage/arithmetic lanes are distinct; no atmospheric mixed precision is introduced.
AVX-512 is present in inventory but execution/performance is NOT RUN. Online Rust
docs version differs from the installed compiler; actual build identity governs.

Persistent std-thread workers own disjoint synthetic array chunks, process multiple
channel jobs without respawning per job, and verify against scalar results outside
timing. Setup/scheduling are separate measurements; no worker has atmospheric mutation
rights. Full-environment parallel face work/reductions are NOT RUN. Process-local
CPU masks may compare unique-core versus SMT-paired sets; this is not permanent
OS tuning or proof every worker stayed pinned. Placement is documented as restricted
allowed CPUs, not a stronger topology claim. No data races or unordered atomic
physical reductions are introduced. Observe actual throughput, not core-count promises.

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

Eight-worker process-local affinity comparison:

| Affinity | Mean job ms | All3 window ns |
|---|---|---|
| physical8 | 1.99631 | [2340669, 1843602, 1804661] |
| smt4pairs | 2.679 | [2741988, 2709997, 2585008] |

Single3-window sample per affinity condition, no scheduling superiority claim. Explicit256-bit arithmetic and whole-executable YMMmul instructions observed; portable baseline contains compiler-selected code. Blackbox scalar incurs barriers; no pure-vectorization factor is asserted for the atmosphere.
