# Intra-query parallelism results — VII

**REJECTED exact candidate; no runtime integration.** Existing authenticated
V fixtures reconstruct canonical whole body/pipe geometry from IV-derived worlds.
Snapshot digest, tick, collider/radius fingerprints, pipe revision and expected
hit are validated by fixtures::probe before experiments. Old source/binaries
and immutable V/VI archives remain retained; source identity is never bypassed.

## Attribution and dependency ruling

`cast` first computes exact initial contact. Constant-rotation queries use
FilteredLinearDispatcher::composite_cast: the same SAH BVH/find_best uses current
best TOI, ray/AABB costs, strict leaf selection and early termination. Nonlinear
queries use ordered Lipschitz interval subdivision, conservative bounds, GJK
support/triangle work and exact final contact. within_distance can stop as soon
as the bound is satisfied. reusable_contact selects deepest contact using strict
less-than in BVH visitation order; equal-depth/tie normal choice can depend on
that order. GJK simplex/support selection and floating-point operation order
are result-affecting. Recovery samples and solver admission remain ordered.

Existing IV detailed kernel attribution is reused, not remeasured: support work,
triangle candidates and BVH traversal dominate selected nonlinear fixtures;
100-step GJK saturation and EPA were not the measured cause there. Local Parry
0.27 public interfaces and exact wrappers were inspected. Splitting ordered leaf
traversal cannot be claimed exact by simply minimizing independently returned TOIs.

## Deliberate falsification

The research module reuses VI's bounded persistent Executor, one pool per assay,
<=64 component tasks, worker-local scratch, no recovery executor nesting. Each
component runs the unchanged cast kernel at authenticated poses; receipts are
consumed in anatomy order and strict minimum TOI selected. Work/error ordering
is not assumed equivalent to the whole compound.

| Fixture | Workers | Exact /10 | Candidate mean ms |
|---|---:|---:|---:|
| eight | 1 | 0/10 | 20.629 |
| eight | 2 | 0/10 | 12.199 |
| eight | 4 | 0/10 | 7.698 |
| eight | 8 | 0/10 | 7.457 |
| fresh | 1 | 0/10 | 57.214 |
| fresh | 2 | 0/10 | 57.027 |
| fresh | 4 | 0/10 | 57.207 |
| fresh | 8 | 0/10 | 57.377 |
| endurance | 1 | 10/10 | 0.322 |
| endurance | 2 | 10/10 | 0.088 |
| endurance | 4 | 10/10 | 0.178 |
| endurance | 8 | 10/10 | 0.185 |

Eight and fresh differ in TOI bits, normals and points at every worker count;
endurance fixture0 matches. Thus80/120 results fail the full exact-hit contract.
The faster component result is not a qualified speedup. Whole-compound radius
bounds, interval subdivision and contact/traversal selection differ even though
leaf kernels are unchanged. All mismatch payloads are retained; none is used to
mutate runtime state. This one selected endurance query is not the370-query batch.

Canonical probe is one cold query while candidate runs have five rounds with
persistent worker scratch; the one-worker fallback uses fresh part workspaces.
That timing is not a matched warm speedup or worker-scaling certificate. Errors
captured by these fixtures are absent; general lazy-error/tie equivalence remains
UNVALIDATED. Alternative conservative-bounds parallelism is PROPOSED only. No
exact runtime candidate reaches continuation or full-matrix gates.
