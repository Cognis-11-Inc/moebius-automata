Contact dispatch adaptation in crates/moebius-morph/src/collision.rs follows
Parry 0.27.0 (Dimforge and contributors), Apache-2.0, from:
- src/query/contact/contact_support_map_support_map.rs
- src/query/contact/contact_composite_shape_shape.rs
- src/query/default_query_dispatcher.rs

Changes: reuse EPA capacity and memoize identical queries; adapt dispatch to the
existing immutable shapes. The dependency remains pinned at 0.27.0. License copy:
Parry-0.27.0-Apache-2.0.txt. No claim of upstream endorsement or upstream changes.

IV filtered linear traversal also adapts:
- src/query/shape_cast/shape_cast_composite_shape_shape.rs

Changes: retain original traversal costs, transforms and leaf dispatcher; insert
conservative swept projection-bound exclusions and observer operation counters.
