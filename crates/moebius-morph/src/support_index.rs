//! Exact support-point search over the existing convex hull vertices.
//! This changes search work, never the hull or its original tie ordering.
use parry3d::math::Vector;

struct Node {
    low: [f32; 3],
    high: [f32; 3],
    range: std::ops::Range<usize>,
    children: Option<(usize, usize)>,
}
// Seven fixed directions provide fourteen supporting planes. Unlike the rejected
// all-face node walk, this certificate has constant work and no support search.
const DIRECTIONS: [[f64; 3]; 7] = [
    [1., 0., 0.],
    [0., 1., 0.],
    [0., 0., 1.],
    [1., 1., 1.],
    [1., 1., -1.],
    [1., -1., 1.],
    [-1., 1., 1.],
];
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a.into_iter().zip(b).map(|(x, y)| x * y).sum()
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
pub(crate) struct ProjectionPlanes {
    planes: [([f64; 3], f64, f64, f64); 7],
}
impl ProjectionPlanes {
    pub fn swept(mut self, velocity: Vector, time: f32) -> Self {
        let velocity = velocity.to_array().map(f64::from);
        for (axis, lower, upper, margin) in &mut self.planes {
            let delta = dot(*axis, velocity) * f64::from(time);
            *lower += delta.min(0.);
            *upper += delta.max(0.);
            *margin += delta.abs() * 64. * f64::from(f32::EPSILON);
        }
        self
    }
    pub fn excludes(&self, low: Vector, high: Vector) -> bool {
        for &(axis, lower, upper, margin) in &self.planes {
            let min = (0..3)
                .map(|k| axis[k] * f64::from(if axis[k] >= 0. { low[k] } else { high[k] }))
                .sum::<f64>();
            let max = (0..3)
                .map(|k| axis[k] * f64::from(if axis[k] >= 0. { high[k] } else { low[k] }))
                .sum::<f64>();
            if min > upper + margin || max < lower - margin {
                return true;
            }
        }
        false
    }
}

pub(crate) struct SupportIndex {
    points: Vec<Vector>,
    order: Vec<usize>,
    nodes: Vec<Node>,
    planes: Vec<(Vector, f64, f64)>,
    dop: [[f64; 2]; 7],
    coordinate_bound: f64,
    last_separating_plane: std::sync::atomic::AtomicUsize,
}
impl SupportIndex {
    pub fn new(points: &[Vector]) -> Self {
        let mut index = Self {
            points: points.to_vec(),
            order: (0..points.len()).collect(),
            nodes: vec![],
            planes: vec![],
            dop: DIRECTIONS.map(|n| {
                points
                    .iter()
                    .fold([f64::INFINITY, f64::NEG_INFINITY], |bounds, p| {
                        let projection = dot(n, p.to_array().map(f64::from));
                        [bounds[0].min(projection), bounds[1].max(projection)]
                    })
            }),
            coordinate_bound: points
                .iter()
                .map(|p| f64::from(p.abs().max_element()))
                .fold(0., f64::max),
            last_separating_plane: std::sync::atomic::AtomicUsize::new(0),
        };
        index.build(0..points.len());
        index
    }
    /// Project immutable hull bounds into the composite frame. The inverse
    /// transpose accounts for finite-precision rotations; an outward allowance
    /// covers f32 transform evaluation and coordinate-scaled roundoff.
    pub fn projection_planes(
        &self,
        pose: &parry3d::math::Pose,
        root: &parry3d::bounding_volume::Aabb,
        limit: f32,
        roundoff: f32,
    ) -> Option<ProjectionPlanes> {
        let columns = [Vector::X, Vector::Y, Vector::Z]
            .map(|axis| (pose.rotation * axis).to_array().map(f64::from));
        let cofactors = [
            cross(columns[1], columns[2]),
            cross(columns[2], columns[0]),
            cross(columns[0], columns[1]),
        ];
        let determinant = dot(columns[0], cofactors[0]);
        if !determinant.is_finite() || determinant.abs() < 1e-10 {
            return None;
        }
        let translation = pose.translation.to_array().map(f64::from);
        let scale = self.coordinate_bound
            + f64::from(pose.translation.abs().max_element())
            + f64::from(root.mins.abs().max(root.maxs.abs()).max_element())
            + 1.;
        let planes = std::array::from_fn(|i| {
            let n = DIRECTIONS[i];
            let axis = std::array::from_fn(|k| {
                (cofactors[0][k] * n[0] + cofactors[1][k] * n[1] + cofactors[2][k] * n[2])
                    / determinant
            });
            let norm = dot(axis, axis).sqrt();
            let offset = dot(axis, translation);
            let margin =
                (f64::from(limit) + f64::from(roundoff) + scale * 128. * f64::from(f32::EPSILON))
                    * norm;
            (
                axis,
                self.dop[i][0] + offset,
                self.dop[i][1] + offset,
                margin,
            )
        });
        planes
            .iter()
            .all(|(axis, lo, hi, margin)| {
                axis.iter().all(|v| v.is_finite())
                    && lo.is_finite()
                    && hi.is_finite()
                    && margin.is_finite()
            })
            .then_some(ProjectionPlanes { planes })
    }
    pub fn with_normals(mut self, directions: impl IntoIterator<Item = Vector>) -> Self {
        for normal in directions {
            if !normal.is_finite() {
                continue;
            }
            let n = normal.to_array().map(f64::from);
            let dot = |p: Vector| {
                p.to_array()
                    .into_iter()
                    .zip(n)
                    .map(|(x, y)| f64::from(x) * y)
                    .sum::<f64>()
            };
            let upper = self
                .points
                .iter()
                .map(|&p| dot(p))
                .fold(f64::NEG_INFINITY, f64::max);
            let norm = n.into_iter().map(|x| x * x).sum::<f64>().sqrt();
            if norm > 0. {
                self.planes.push((normal, upper, norm));
            }
        }
        self
    }
    /// A separating support plane certifies distance greater than the query
    /// threshold. All hull vertices supply its outward offset; no face index or
    /// approximate surface is substituted. Uncertain cases keep the old GJK.
    pub fn separates_triangle(
        &self,
        pose: &parry3d::math::Pose,
        triangle: &parry3d::shape::Triangle,
        limit: f32,
        roundoff: f32,
    ) -> bool {
        let points = [pose * triangle.a, pose * triangle.b, pose * triangle.c];
        let last = self
            .last_separating_plane
            .load(std::sync::atomic::Ordering::Relaxed);
        if self
            .planes
            .get(last)
            .is_some_and(|plane| Self::plane_separates(&points, *plane, limit, roundoff))
        {
            return true;
        }
        for (index, &plane) in self.planes.iter().enumerate() {
            if index != last && Self::plane_separates(&points, plane, limit, roundoff) {
                self.last_separating_plane
                    .store(index, std::sync::atomic::Ordering::Relaxed);
                return true;
            }
        }
        false
    }
    fn plane_separates(
        points: &[Vector; 3],
        (normal, upper, norm): (Vector, f64, f64),
        limit: f32,
        roundoff: f32,
    ) -> bool {
        #[cfg(feature = "allocation-attribution")]
        crate::profiling::work(6, 1);
        let n = normal.to_array().map(f64::from);
        let lower = points
            .iter()
            .map(|p| {
                p.to_array()
                    .into_iter()
                    .zip(n)
                    .map(|(x, y)| f64::from(x) * y)
                    .sum::<f64>()
            })
            .fold(f64::INFINITY, f64::min);
        let magnitude = upper.abs().max(lower.abs()).max(1.);
        let clear = lower - upper
            > (f64::from(limit) + f64::from(roundoff)) * norm
                + magnitude * 32. * f64::from(f32::EPSILON);
        #[cfg(feature = "allocation-attribution")]
        if clear {
            crate::profiling::work(7, 1);
        }
        clear
    }
    fn build(&mut self, range: std::ops::Range<usize>) -> usize {
        let mut low = [f32::INFINITY; 3];
        let mut high = [f32::NEG_INFINITY; 3];
        for &i in &self.order[range.clone()] {
            let p = self.points[i].to_array();
            for k in 0..3 {
                low[k] = low[k].min(p[k]);
                high[k] = high[k].max(p[k]);
            }
        }
        let id = self.nodes.len();
        self.nodes.push(Node {
            low,
            high,
            range: range.clone(),
            children: None,
        });
        if range.len() > 12 {
            let axis = (0..3)
                .max_by(|&a, &b| (high[a] - low[a]).total_cmp(&(high[b] - low[b])))
                .unwrap();
            let points = &self.points;
            self.order[range.clone()].sort_unstable_by(|&a, &b| {
                points[a][axis].total_cmp(&points[b][axis]).then(a.cmp(&b))
            });
            let mid = range.start + range.len() / 2;
            let a = self.build(range.start..mid);
            let b = self.build(mid..range.end);
            self.nodes[id].children = Some((a, b));
        }
        id
    }
    // f64 evaluation plus an outward allowance for any f32 dot rounding and
    // underflow. Ambiguous bounds visit the node. Nonfinite directions fall back.
    fn upper(node: &Node, d: [f64; 3]) -> f64 {
        #[cfg(feature = "allocation-attribution")]
        crate::profiling::work(8, 1);
        let mut sum = 0.;
        let mut magnitude = 0.;
        for (k, direction) in d.into_iter().enumerate() {
            let p = if direction >= 0. {
                node.high[k]
            } else {
                node.low[k]
            };
            sum += f64::from(p) * direction;
            magnitude += f64::from(node.low[k].abs().max(node.high[k].abs())) * direction.abs();
        }
        sum + magnitude * (16. * f64::from(f32::EPSILON)) + 16. * f64::from(f32::MIN_POSITIVE)
    }
    pub fn point(&self, dir: Vector) -> Vector {
        if !dir.is_finite() || self.points.iter().take(1).any(|p| !p.is_finite()) {
            return parry3d::utils::point_cloud_support_point(dir, &self.points);
        }
        let mut best = (0, self.points[0].dot(dir));
        let bound_dir = dir.to_array().map(f64::from);
        self.visit(
            0,
            Self::upper(&self.nodes[0], bound_dir),
            dir,
            bound_dir,
            &mut best,
        );
        self.points[best.0]
    }
    fn visit(
        &self,
        id: usize,
        upper: f64,
        dir: Vector,
        bound_dir: [f64; 3],
        best: &mut (usize, f32),
    ) {
        #[cfg(feature = "allocation-attribution")]
        crate::profiling::work(5, 1);
        let node = &self.nodes[id];
        if upper < f64::from(best.1) {
            #[cfg(feature = "allocation-attribution")]
            crate::profiling::work(9, 1);
            return;
        }
        if let Some((a, b)) = node.children {
            let ua = Self::upper(&self.nodes[a], bound_dir);
            let ub = Self::upper(&self.nodes[b], bound_dir);
            if ua >= ub {
                self.visit(a, ua, dir, bound_dir, best);
                self.visit(b, ub, dir, bound_dir, best);
            } else {
                self.visit(b, ub, dir, bound_dir, best);
                self.visit(a, ua, dir, bound_dir, best);
            }
        } else {
            #[cfg(feature = "allocation-attribution")]
            crate::profiling::work(4, node.range.len() as u64);
            for &i in &self.order[node.range.clone()] {
                let value = self.points[i].dot(dir);
                if value > best.1 || (value == best.1 && i < best.0) {
                    *best = (i, value);
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixed_projection_node_rejection_is_conservative_under_rotation_and_translation() {
        use parry3d::{
            bounding_volume::Aabb,
            math::{Pose, Rot3},
            shape::{ConvexPolyhedron, Cuboid},
        };
        let points = (0..120)
            .map(|i| {
                let t = i as f32 * 2.399963;
                let y = 1. - 2. * (i as f32 + 0.5) / 120.;
                let r = (1. - y * y).sqrt();
                Vector::new(r * t.cos() * 0.25, y, r * t.sin() * 0.15)
            })
            .collect::<Vec<_>>();
        let hull = ConvexPolyhedron::from_convex_hull(&points).unwrap();
        let index = SupportIndex::new(hull.points());
        let mut rejects = 0;
        for origin in [
            Vector::ZERO,
            Vector::new(-8., 4., -2.),
            Vector::splat(100000.),
        ] {
            for step in 0..31 {
                let pose = Pose::from_parts(
                    origin,
                    Rot3::from_rotation_y(step as f32 * 0.17)
                        * Rot3::from_rotation_z(step as f32 * 0.11),
                );
                let root = Aabb::new(origin - Vector::splat(4.), origin + Vector::splat(4.));
                let planes = index.projection_planes(&pose, &root, 0.001, 0.001).unwrap();
                for vertex in hull.points() {
                    let p = pose * *vertex;
                    assert!(
                        !planes.excludes(p - Vector::splat(0.005), p + Vector::splat(0.005)),
                        "excluded an actual hull vertex"
                    );
                }
                assert!(!planes.excludes(root.mins, root.maxs));
                let velocity = Vector::new(-3., 0.7, 2.);
                let swept = index
                    .projection_planes(&pose, &root, 0.001, 0.001)
                    .unwrap()
                    .swept(velocity, 1.);
                for fraction in [0., 0.125, 0.5, 0.875, 1.] {
                    for vertex in hull.points() {
                        let p = pose * *vertex + velocity * fraction;
                        assert!(
                            !swept.excludes(p - Vector::splat(0.005), p + Vector::splat(0.005)),
                            "excluded swept actual vertex"
                        );
                    }
                }

                for k in -30..=30 {
                    let center = origin
                        + Vector::new(
                            k as f32 * 0.071,
                            (k % 7) as f32 * 0.091,
                            (k % 11) as f32 * 0.057,
                        );
                    let half = Vector::new(0.015, 0.02, 0.015);
                    if planes.excludes(center - half, center + half) {
                        rejects += 1;
                        let box_pose = Pose::from_translation(center);
                        let distance =
                            parry3d::query::distance(&pose, &hull, &box_pose, &Cuboid::new(half))
                                .unwrap();
                        assert!(
                            distance > 0.001,
                            "false rejection at step {step}, offset {k}, distance {distance}"
                        );
                    }
                }
            }
        }
        assert!(rejects > 0);
    }

    #[test]
    fn support_plane_rejections_preserve_exhaustive_triangle_distances() {
        use parry3d::{
            math::Pose,
            shape::{ConvexPolyhedron, Triangle},
        };
        let points = (0..120)
            .map(|i| {
                let t = i as f32 * 2.399963;
                let y = 1. - 2. * (i as f32 + 0.5) / 120.;
                let r = (1. - y * y).sqrt();
                Vector::new(r * t.cos(), y, r * t.sin())
            })
            .collect::<Vec<_>>();
        let hull = ConvexPolyhedron::from_convex_hull(&points).unwrap();
        let index =
            SupportIndex::new(hull.points()).with_normals(hull.faces().iter().map(|f| f.normal));
        assert!(!index.planes.is_empty());
        let mut rejected = 0;
        for k in -100..=100 {
            let t = k as f32 * 0.021;
            let triangle = Triangle::new(
                Vector::new(t, -0.2, 0.),
                Vector::new(t, 0.2, 0.),
                Vector::new(t, 0., 0.2),
            );
            let distance = parry3d::query::details::distance_support_map_support_map(
                &Pose::IDENTITY,
                &hull,
                &triangle,
            );
            for limit in [0.0001, 0.01, 0.1, 0.5, 2.] {
                if index.separates_triangle(&Pose::IDENTITY, &triangle, limit, 0.001) {
                    rejected += 1;
                    assert!(
                        distance > limit,
                        "distance {distance}, limit {limit}, step {k}"
                    );
                }
            }
        }
        assert!(rejected > 0);
        let crossing = Triangle::new(
            Vector::new(-2., 0., 0.),
            Vector::new(2., 0., 0.),
            Vector::new(0., 2., 0.),
        );
        assert!(!index.separates_triangle(&Pose::IDENTITY, &crossing, 0.001, 0.001));
    }

    #[test]
    fn conservative_tree_preserves_original_support_points_and_ties() {
        let mut points = vec![Vector::ZERO, Vector::ZERO];
        let mut state = 73u64;
        for _ in 0..1200 {
            let p = std::array::from_fn(|_| {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                ((state >> 32) as i32) as f32 / i32::MAX as f32
            });
            points.push(Vector::from_array(p));
        }
        for scale in [1e-18, 1., 1e18] {
            let scaled = points.iter().map(|p| *p * scale).collect::<Vec<_>>();
            let index = SupportIndex::new(&scaled);
            for k in -2000..=2000 {
                let d = Vector::new(
                    k as f32 * 0.17,
                    (k % 17) as f32 * 0.03,
                    (k % 31) as f32 * 0.21,
                );
                assert_eq!(
                    index.point(d).to_array().map(f32::to_bits),
                    parry3d::utils::point_cloud_support_point(d, &scaled)
                        .to_array()
                        .map(f32::to_bits)
                );
            }
        }
    }
}
