# Temporal computational economy

Original fixed2:1 subcycling remains unchanged numerically and slower in fresh
baseline windows. A read-only intrusive attribution helper separately measures base
clone, face construction, half-step flux evaluation, begin, pending validation,
one pending encoding, finish including validation and final validation. Pending
validation actually reconstructs begin and encodes BOTH expected/provided pending
states. Numbers from separate invocations are non-additive; they are not a single
critical-path stack or hardware-counter attribution. The helper verifies unchanged
committed digest. Ordinary transaction timings do not include this extra diagnostic.

Registered comparisons use identical3D diffusion initial data/refinement, target
.05s, coarse dt.001 and fine dt.0005, with explicit mono/fine/subcycle frequencies.
Report ms/interval and cost per physical second, and require the same fidelity
budget. No speedup is claimed merely because transactions have different dt.
No persistent predictors, reduced validation, optimized subcycling or asynchronous
clock system is implemented. Higher-order subcycling refuses pending interface proof.

## Final measured qualification

| Case | dt (s) | Scalar normalized L1 | Transaction ms | Transaction seconds / physical second |
|---|---|---|---|---|
| temporal-coarse | 0.001 | 0.000914888 | 0.851056 | 0.851056 |
| temporal-fine | 0.0005 | 0.00091819 | 0.836902 | 1.6738 |
| temporal-subcycle | 0.001 | 0.000915282 | 12.7094 | 12.7094 |

Same physical duration .05s; first-order schemes, same refined cube and scalar law. Fine monorate and subcycle have different temporal treatment; no speed recommendation without fidelity comparison. Separate non-additive attribution (ns, one pending encoding bytes):

```json
{
  "base_clone_ns": 12390,
  "begin_ns": 1981240,
  "face_construction_ns": 1214479,
  "final_validation_ns": 144270,
  "finish_including_validation_ns": 10173047,
  "half_step_flux_ns": 531220,
  "one_pending_encoding_ns": 1920580,
  "pending_encoded_bytes": 359260,
  "physical_digest_unchanged": true,
  "scope": "intrusive separate invocations, non-additive; committed state untouched",
  "validation_including_reconstruction_and_two_encodings_ns": 7808969
}
```

Fixed n12 combined-wave same-space endpoint differences against dt=.000125:

| dt | Normalized density difference |
|---|---|
| 0.002 | 0.00127913 |
| 0.001 | 0.000324668 |
| 0.0005 | 7.88396e-05 |

These are numerical-reference differences; decreasing error does not prove asymptotic formal order. No optimized subcycle candidate was implemented.

The monorate recipes use the new derived topology cache; the original subcycle path rebuilds topology. The 7.593× timed-transaction/physical-time cost ratio versus fine monorate compares the complete currently available paths, not an isolated temporal algorithm with equal caching. Source inspection and non-additive probes separate repeated pending reconstruction/serialization and topology costs; no optimized subcycle was tested.

Timing scope: transaction durations cover the selected Chamber clone/topology/numerical/source/admission/commit path. Seconds per physical second are DERIVED from those timed transactions and configured dt, not total recorder process wall time. Untimed field metrics, digest recording, trace/checkpoint IO and restoration are excluded; no complete application/renderer/history-service throughput is claimed.
