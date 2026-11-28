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
