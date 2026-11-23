//! Viewer costumes. These meshes are not organism detections.
//! Small curved carapaces, ribbed conduits, and soft cores share one cell law.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::prelude::*;
use moebius_core::farm::{JOINTS, SPINE_JOINTS};
pub const BIND_START: f32 = -1.05;
pub const BIND_STEP: f32 = 2.0 / (SPINE_JOINTS - 1) as f32;

struct Sculpt {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
    colors: Vec<[f32; 4]>,
    tint: [f32; 4],
}

impl Sculpt {
    fn new() -> Self {
        Self {
            positions: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            indices: Vec::new(),
            colors: Vec::new(),
            tint: [1.0; 4],
        }
    }

    fn tube(&mut self, from: Vec3, to: Vec3, radius_a: f32, radius_b: f32, sides: u32) {
        let axis = to - from;
        let length = axis.length();
        if length < 1.0e-4 || sides < 3 {
            return;
        }
        let dir = axis / length;
        let helper = if dir.y.abs() > 0.85 { Vec3::X } else { Vec3::Y };
        let side = dir.cross(helper).normalize();
        let up = dir.cross(side).normalize();
        let base = self.positions.len() as u32;
        for ring in 0..2 {
            let center = if ring == 0 { from } else { to };
            let radius = if ring == 0 { radius_a } else { radius_b };
            for step in 0..sides {
                let angle = step as f32 / sides as f32 * std::f32::consts::TAU;
                let normal = side * angle.cos() + up * angle.sin();
                self.positions.push((center + normal * radius).to_array());
                self.colors.push(self.tint);
                self.normals.push(normal.to_array());
                self.uvs.push([step as f32 / sides as f32, ring as f32]);
            }
        }
        for step in 0..sides {
            let next = (step + 1) % sides;
            let a = base + step;
            let b = base + next;
            let c = base + sides + step;
            let d = base + sides + next;
            self.indices.extend([a, b, c, b, d, c]);
        }
    }

    /// Smooth swept conduit with a transported frame, including around bends.
    fn conduit(&mut self, path: &[(Vec3, f32)], sides: u32) {
        if path.len() < 2 {
            return;
        }
        let base = self.positions.len() as u32;
        let mut side = Vec3::X;
        for (ring, &(center, radius)) in path.iter().enumerate() {
            let prev = path[ring.saturating_sub(1)].0;
            let next = path[(ring + 1).min(path.len() - 1)].0;
            let dir = (next - prev).normalize_or_zero();
            side = (side - dir * side.dot(dir)).normalize_or_zero();
            if side.length_squared() < 0.5 {
                let helper = if dir.y.abs() > 0.85 { Vec3::Z } else { Vec3::Y };
                side = dir.cross(helper).normalize();
            }
            let up = dir.cross(side).normalize();
            for step in 0..sides {
                let angle = step as f32 / sides as f32 * std::f32::consts::TAU;
                let normal = side * angle.cos() + up * angle.sin();
                self.positions.push((center + normal * radius).to_array());
                self.normals.push(normal.to_array());
                self.colors.push(self.tint);
                self.uvs.push([step as f32 / sides as f32, ring as f32]);
            }
        }
        for ring in 0..path.len() as u32 - 1 {
            for step in 0..sides {
                let next = (step + 1) % sides;
                let a = base + ring * sides + step;
                let b = base + ring * sides + next;
                let c = a + sides;
                let d = b + sides;
                self.indices.extend([a, b, c, b, d, c]);
            }
        }
    }

    /// Ellipsoidal tissue beneath the external ribs. Smooth analytic normals.
    fn organ(&mut self, center: Vec3, size: Vec3) {
        let base = self.positions.len() as u32;
        let rings = 12;
        let sides = 20;
        for ring in 0..=rings {
            let latitude = ring as f32 / rings as f32 * std::f32::consts::PI;
            for step in 0..=sides {
                let angle = step as f32 / sides as f32 * std::f32::consts::TAU;
                let unit = Vec3::new(
                    latitude.sin() * angle.cos(),
                    latitude.cos(),
                    latitude.sin() * angle.sin(),
                );
                self.positions.push((center + unit * size).to_array());
                self.normals.push((unit / size).normalize().to_array());
                self.colors.push(self.tint);
                self.uvs
                    .push([step as f32 / sides as f32, ring as f32 / rings as f32]);
            }
        }
        for ring in 0..rings {
            for step in 0..sides {
                let a = base + ring * (sides + 1) + step;
                let b = a + 1;
                let c = a + sides + 1;
                let d = c + 1;
                self.indices.extend([a, b, c, b, d, c]);
            }
        }
    }

    fn tendon(&mut self, points: [Vec3; 4], radius: f32) {
        let path: Vec<_> = (0..=16)
            .map(|step| {
                let t = step as f32 / 16.0;
                let u = 1.0 - t;
                let point = points[0] * u.powi(3)
                    + points[1] * (3.0 * u * u * t)
                    + points[2] * (3.0 * u * t * t)
                    + points[3] * t.powi(3);
                (point, radius * (1.0 - 0.65 * t))
            })
            .collect();
        self.conduit(&path, 10);
    }

    fn mesh(self) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, self.colors);
        mesh.insert_indices(Indices::U32(self.indices));
        mesh
    }
}

pub fn creature_mesh(variant: u8) -> Mesh {
    let mut sculpt = Sculpt::new();
    match variant % 3 {
        0 => vertebral(&mut sculpt),
        1 => curled(&mut sculpt),
        _ => thorax(&mut sculpt),
    }
    let mut mesh = sculpt.mesh();
    let positions = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .unwrap()
        .as_float3()
        .unwrap();
    let mut indices = Vec::new();
    let mut weights = Vec::new();
    for position in positions {
        let joint = ((position[1] - BIND_START) / BIND_STEP).clamp(0.0, (SPINE_JOINTS - 1) as f32);
        let lower = joint.floor() as u16;
        let upper = (lower + 1).min((SPINE_JOINTS - 1) as u16);
        let blend = joint.fract();
        indices.push([lower, upper, 0, 0]);
        weights.push([1.0 - blend, blend, 0.0, 0.0]);
    }
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_JOINT_INDEX,
        bevy::mesh::VertexAttributeValues::Uint16x4(indices),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, weights);
    mesh
}

// Vertex colors separate pale armor, dark flexible conduits, and warm tissue.
const ARMOR: [f32; 4] = [0.62, 0.51, 0.37, 1.0];
const TISSUE: [f32; 4] = [0.16, 0.085, 0.065, 1.0];
const HOSE: [f32; 4] = [0.045, 0.070, 0.063, 1.0];

fn vertebral(sculpt: &mut Sculpt) {
    larva(sculpt, 0.0, 1.0);
}

fn curled(sculpt: &mut Sculpt) {
    larva(sculpt, 0.75, 0.82);
}

fn thorax(sculpt: &mut Sculpt) {
    larva(sculpt, -0.32, 1.22);
}

fn centerline(t: f32, curl: f32) -> Vec3 {
    Vec3::new(
        curl * (t * 2.5).sin() * 0.32,
        (t - 0.5) * 1.50,
        0.10 * (t * 3.0).sin(),
    )
}

fn larva(sculpt: &mut Sculpt, curl: f32, girth: f32) {
    // Continuous soft abdomen, tapering into a small segmented tail.
    sculpt.tint = TISSUE;
    let body: Vec<_> = (0..=28)
        .map(|step| {
            let t = step as f32 / 28.0;
            let radius =
                (0.014 + 0.19 * (t * std::f32::consts::PI).sin().max(0.0).powf(0.8)) * girth;
            (centerline(t, curl), radius)
        })
        .collect();
    sculpt.conduit(&body, 16);
    // Nested curved hoops resemble external ribs rather than radial spikes.
    sculpt.tint = ARMOR;
    for rib in 0..9 {
        let t = 0.16 + rib as f32 * 0.075;
        let center = centerline(t, curl);
        let width = (0.065 + 0.16 * (t * std::f32::consts::PI).sin()) * girth;
        let path: Vec<_> = (0..=20)
            .map(|step| {
                let angle = -0.25 + step as f32 / 20.0 * (std::f32::consts::TAU - 0.50);
                let p = center
                    + Vec3::new(
                        angle.sin() * width,
                        0.026 * angle.cos(),
                        angle.cos() * width * 0.86,
                    );
                (p, 0.022 + 0.009 * (t * std::f32::consts::PI).sin())
            })
            .collect();
        sculpt.conduit(&path, 8);
    }
    // Overlapping vertebral plates over the dorsal cable.
    for plate in 0..8 {
        let t = 0.18 + plate as f32 * 0.085;
        let center = centerline(t, curl) + Vec3::new(0.0, 0.0, -0.17 * girth);
        sculpt.organ(center, Vec3::new(0.085 * girth, 0.065, 0.045));
    }
    let head = centerline(0.92, curl);
    sculpt.organ(head, Vec3::new(0.17 * girth, 0.22, 0.15));
    // Dark port and short paired feeding mandibles; no long spear tips.
    sculpt.tint = HOSE;
    sculpt.organ(
        head + Vec3::new(0.0, 0.14, 0.10),
        Vec3::new(0.085, 0.07, 0.06),
    );
    for sign in [-1.0, 1.0] {
        let root = centerline(0.25, curl) + Vec3::new(sign * 0.13 * girth, 0.0, 0.10);
        let end = head + Vec3::new(sign * 0.12 * girth, -0.04, 0.03);
        sculpt.tendon(
            [
                root,
                root + Vec3::new(sign * 0.32, 0.08, 0.18),
                end + Vec3::new(sign * 0.25, -0.15, 0.20),
                end,
            ],
            0.043,
        );
        sculpt.tint = HOSE;
    }
    // One flexible trailing conduit, ending in a rounded taper.
    let tail = centerline(0.03, curl);
    sculpt.tendon(
        [
            tail,
            tail + Vec3::new(-0.10, -0.16, 0.06),
            tail + Vec3::new(-0.24, -0.22, 0.16),
            tail + Vec3::new(-0.21, -0.34, 0.18),
        ],
        0.035,
    );
}

pub fn mandible_mesh(sign: f32) -> Mesh {
    let mut sculpt = Sculpt::new();
    sculpt.tint = ARMOR;
    sculpt.tendon(
        [
            Vec3::ZERO,
            Vec3::new(sign * 0.13, 0.10, 0.05),
            Vec3::new(sign * 0.10, 0.28, 0.17),
            Vec3::new(-sign * 0.08, 0.25, 0.12),
        ],
        0.045,
    );
    sculpt.tint = HOSE;
    sculpt.organ(Vec3::ZERO, Vec3::splat(0.055));
    sculpt.mesh()
}

pub fn rig_bind_positions() -> Vec<Vec3> {
    let mut positions: Vec<_> = (0..SPINE_JOINTS)
        .map(|index| Vec3::Y * (BIND_START + index as f32 * BIND_STEP))
        .collect();
    let head = positions[SPINE_JOINTS - 3];
    positions.push(head + Vec3::new(-0.10, 0.15, 0.08));
    positions.push(head + Vec3::new(0.10, 0.15, 0.08));
    debug_assert_eq!(positions.len(), JOINTS);
    positions
}

pub fn nutrient_mesh() -> Mesh {
    let mut sculpt = Sculpt::new();
    for (center, size) in [
        (Vec3::ZERO, Vec3::new(0.75, 0.90, 0.65)),
        (Vec3::new(0.45, 0.1, 0.0), Vec3::new(0.57, 0.64, 0.53)),
        (Vec3::new(-0.3, -0.35, 0.15), Vec3::new(0.62, 0.55, 0.56)),
        (Vec3::new(0.0, 0.53, -0.1), Vec3::new(0.50, 0.52, 0.49)),
    ] {
        sculpt.organ(center, size);
    }
    sculpt.mesh()
}

/// Folded edible tissue: an asymmetric fluted spindle, pale constrictions and
/// exposed vessels. Material amount controls its volume in the live picture.
pub fn food_clot_mesh() -> Mesh {
    let mut s = Sculpt::new();
    s.tint = [0.32, 0.055, 0.08, 1.0];
    let path: Vec<_> = (0..=10)
        .map(|i| {
            let t = i as f32 / 10.0;
            (
                Vec3::new(
                    (t * 6.0).sin() * 0.12,
                    (t - 0.5) * 1.8,
                    (t * 4.0).cos() * 0.1,
                ),
                0.025 + (t * std::f32::consts::PI).sin() * 0.48,
            )
        })
        .collect();
    s.conduit(&path, 10);
    s.tint = [0.66, 0.39, 0.34, 1.0];
    for ring in 1..8 {
        let t = ring as f32 / 8.0;
        let center = Vec3::new(
            (t * 6.0).sin() * 0.12,
            (t - 0.5) * 1.8,
            (t * 4.0).cos() * 0.1,
        );
        let radius = (t * std::f32::consts::PI).sin() * 0.48;
        let fold: Vec<_> = (0..=12)
            .map(|i| {
                let a = i as f32 / 12.0 * std::f32::consts::TAU;
                (
                    center + Vec3::new(a.cos() * radius, a.sin() * 0.045, a.sin() * radius),
                    0.028,
                )
            })
            .collect();
        s.conduit(&fold, 5);
    }
    s.tint = [0.20, 0.025, 0.04, 1.0];
    for sign in [-1.0, 1.0] {
        let vessel: Vec<_> = path
            .iter()
            .map(|(p, r)| (*p + Vec3::new(sign * r * 0.7, 0.0, r * 0.65), 0.035))
            .collect();
        s.conduit(&vessel, 6);
    }
    s.mesh()
}

pub fn bone_cage(extent: Vec3, thickness: f32) -> Mesh {
    let mut sculpt = Sculpt::new();
    let half = extent * 0.5;
    let corners = [
        Vec3::new(-half.x, -half.y, -half.z),
        Vec3::new(half.x, -half.y, -half.z),
        Vec3::new(half.x, half.y, -half.z),
        Vec3::new(-half.x, half.y, -half.z),
        Vec3::new(-half.x, -half.y, half.z),
        Vec3::new(half.x, -half.y, half.z),
        Vec3::new(half.x, half.y, half.z),
        Vec3::new(-half.x, half.y, half.z),
    ];
    let edges = [
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0),
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];
    let radius = thickness.max(0.02);
    for (a, b) in edges {
        sculpt.tube(corners[a], corners[b], radius, radius, 4);
    }
    sculpt.mesh()
}

pub fn write_threads(mesh: &mut Mesh, segments: &[([f32; 3], [f32; 3])], radius: f32) {
    mesh.remove_attribute(Mesh::ATTRIBUTE_JOINT_INDEX);
    mesh.remove_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT);
    let mut sculpt = Sculpt::new();
    if segments.is_empty() {
        sculpt.tube(Vec3::ZERO, Vec3::Y * 0.001, 0.001, 0.001, 3);
    } else {
        for (start, end) in segments.iter().take(4_096) {
            sculpt.tube(
                Vec3::from_array(*start),
                Vec3::from_array(*end),
                radius,
                radius * 0.65,
                4,
            );
        }
    }
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, sculpt.positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, sculpt.normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, sculpt.uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, sculpt.colors);
    mesh.insert_indices(Indices::U32(sculpt.indices));
}

/// A single implicit surface makes neighboring deposits grow into necks and
/// sheets. Geometry is derived from the conserved field; fusion adds no stock.
pub fn fused_flesh_mesh(
    deposits: &[moebius_core::farm::FleshDeposit],
    dims: moebius_core::Dims,
) -> Mesh {
    use std::collections::{BTreeSet, HashMap};
    type Grid = [i32; 3];
    let mut field: HashMap<Grid, f32> = HashMap::new();
    for deposit in deposits {
        let center = [
            deposit.site.x as i32 * 2,
            deposit.site.y as i32 * 2,
            deposit.site.z as i32 * 2,
        ];
        let weight = (deposit.amount as f32).cbrt();
        let radius = 2.2 + (weight - 1.0) * 0.35;
        let extent = radius.ceil() as i32;
        for z in -extent..=extent {
            for y in -extent..=extent {
                for x in -extent..=extent {
                    let distance2 = (x * x + y * y + z * z) as f32;
                    let kernel = (1.0 - distance2 / (radius * radius)).max(0.0);
                    if kernel == 0.0 {
                        continue;
                    }
                    let point = [center[0] + x, center[1] + y, center[2] + z];
                    *field.entry(point).or_default() += weight * kernel * kernel;
                }
            }
        }
    }
    let mut cells = BTreeSet::new();
    for point in field.keys() {
        for z in -1..=0 {
            for y in -1..=0 {
                for x in -1..=0 {
                    cells.insert([point[0] + x, point[1] + y, point[2] + z]);
                }
            }
        }
    }
    let sample = |p: Grid| field.get(&p).copied().unwrap_or(0.0);
    let gradient = |p: Grid| {
        Vec3::new(
            sample([p[0] - 1, p[1], p[2]]) - sample([p[0] + 1, p[1], p[2]]),
            sample([p[0], p[1] - 1, p[2]]) - sample([p[0], p[1] + 1, p[2]]),
            sample([p[0], p[1], p[2] - 1]) - sample([p[0], p[1], p[2] + 1]),
        )
    };
    let half = Vec3::new(
        (dims.width - 1) as f32,
        (dims.height - 1) as f32,
        (dims.depth - 1) as f32,
    ) * crate::layout::SPACING
        * 0.5;
    let corners = [
        [0, 0, 0],
        [1, 0, 0],
        [1, 1, 0],
        [0, 1, 0],
        [0, 0, 1],
        [1, 0, 1],
        [1, 1, 1],
        [0, 1, 1],
    ];
    let tetrahedra = [
        [0, 5, 1, 6],
        [0, 1, 2, 6],
        [0, 2, 3, 6],
        [0, 3, 7, 6],
        [0, 7, 4, 6],
        [0, 4, 5, 6],
    ];
    let mut sculpt = Sculpt::new();
    for cell in cells {
        let points = corners.map(|offset| {
            [
                cell[0] + offset[0],
                cell[1] + offset[1],
                cell[2] + offset[2],
            ]
        });
        let values = points.map(sample);
        if values.iter().all(|v| *v < 0.72) || values.iter().all(|v| *v >= 0.72) {
            continue;
        }
        let normals = points.map(gradient);
        let edge = |a: usize, b: usize| {
            let t = ((0.72 - values[a]) / (values[b] - values[a])).clamp(0.0, 1.0);
            let p = Vec3::new(
                points[a][0] as f32,
                points[a][1] as f32,
                points[a][2] as f32,
            )
            .lerp(
                Vec3::new(
                    points[b][0] as f32,
                    points[b][1] as f32,
                    points[b][2] as f32,
                ),
                t,
            );
            (
                p * (crate::layout::SPACING * 0.5) - half,
                normals[a].lerp(normals[b], t).normalize_or_zero(),
            )
        };
        for tetra in tetrahedra {
            let inside: Vec<_> = tetra
                .iter()
                .copied()
                .filter(|i| values[*i] >= 0.72)
                .collect();
            let outside: Vec<_> = tetra
                .iter()
                .copied()
                .filter(|i| values[*i] < 0.72)
                .collect();
            match inside.len() {
                1 => emit_tissue_triangle(
                    &mut sculpt,
                    [
                        edge(inside[0], outside[0]),
                        edge(inside[0], outside[1]),
                        edge(inside[0], outside[2]),
                    ],
                ),
                3 => emit_tissue_triangle(
                    &mut sculpt,
                    [
                        edge(outside[0], inside[0]),
                        edge(outside[0], inside[1]),
                        edge(outside[0], inside[2]),
                    ],
                ),
                2 => {
                    let a = edge(inside[0], outside[0]);
                    let b = edge(inside[0], outside[1]);
                    let c = edge(inside[1], outside[0]);
                    let d = edge(inside[1], outside[1]);
                    emit_tissue_triangle(&mut sculpt, [a, b, c]);
                    emit_tissue_triangle(&mut sculpt, [b, d, c]);
                }
                _ => {}
            }
        }
    }
    if sculpt.positions.is_empty() {
        sculpt.tube(Vec3::ZERO, Vec3::Y * 0.001, 0.001, 0.001, 3);
    }
    sculpt.mesh()
}

fn emit_tissue_triangle(sculpt: &mut Sculpt, mut vertices: [(Vec3, Vec3); 3]) {
    let face = (vertices[1].0 - vertices[0].0).cross(vertices[2].0 - vertices[0].0);
    if face.length_squared() < 1.0e-14 {
        return;
    }
    let normal = vertices.iter().map(|(_, n)| *n).sum::<Vec3>();
    if face.dot(normal) < 0.0 {
        vertices.swap(1, 2);
    }
    let base = sculpt.positions.len() as u32;
    for (position, normal) in vertices {
        sculpt.positions.push(position.to_array());
        sculpt.normals.push(normal.to_array());
        sculpt.colors.push([1.0; 4]);
        sculpt.uvs.push([0.0; 2]);
    }
    sculpt.indices.extend([base, base + 1, base + 2]);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn surface_components(mesh: &Mesh) -> usize {
        use std::collections::{BTreeMap, BTreeSet};
        fn root(parents: &[usize], mut index: usize) -> usize {
            while parents[index] != index {
                index = parents[index];
            }
            index
        }
        let mut vertices = BTreeMap::new();
        let mut parents = Vec::new();
        let mapped: Vec<_> = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .unwrap()
            .as_float3()
            .unwrap()
            .iter()
            .map(|p| {
                let key = p.map(|v| (v * 1_000_000.0).round() as i64);
                *vertices.entry(key).or_insert_with(|| {
                    let id = parents.len();
                    parents.push(id);
                    id
                })
            })
            .collect();
        let indices: Vec<_> = mesh.indices().unwrap().iter().collect();
        for triangle in indices.as_chunks::<3>().0 {
            let a = root(&parents, mapped[triangle[0]]);
            for index in &triangle[1..] {
                let b = root(&parents, mapped[*index]);
                parents[b] = a;
            }
        }
        (0..parents.len())
            .map(|index| root(&parents, index))
            .collect::<BTreeSet<_>>()
            .len()
    }

    #[test]
    fn fused_tissue_is_one_smooth_surface_and_empty_gaps_stay_open() {
        use moebius_core::{Dims, Site, farm::FleshDeposit};
        let dims = Dims {
            width: 8,
            height: 8,
            depth: 8,
        };
        let deposits = [
            FleshDeposit {
                site: Site { x: 1, y: 1, z: 1 },
                amount: 1,
            },
            FleshDeposit {
                site: Site { x: 2, y: 1, z: 1 },
                amount: 1,
            },
        ];
        let mesh = fused_flesh_mesh(&deposits, dims);
        assert_eq!(surface_components(&mesh), 1);
        for normal in mesh
            .attribute(Mesh::ATTRIBUTE_NORMAL)
            .unwrap()
            .as_float3()
            .unwrap()
        {
            let n = Vec3::from_array(*normal);
            assert!(n.is_finite() && n.length() > 0.9);
        }
        let separated = [
            deposits[0].clone(),
            FleshDeposit {
                site: Site { x: 5, y: 1, z: 1 },
                amount: 1,
            },
        ];
        assert_eq!(surface_components(&fused_flesh_mesh(&separated, dims)), 2);
    }

    #[test]
    fn costumes_have_finite_elongated_biomechanical_geometry() {
        for variant in 0..3 {
            let mesh = creature_mesh(variant);
            let positions = mesh
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .and_then(|values| values.as_float3())
                .expect("positions");
            assert!(positions.len() > 40);
            let mut min = Vec3::splat(f32::MAX);
            let mut max = Vec3::splat(f32::MIN);
            let mut radii = Vec::new();
            for position in positions {
                let point = Vec3::from_array(*position);
                min = min.min(point);
                max = max.max(point);
                radii.push(point.length());
            }
            let span = max - min;
            assert!(span.y > span.x || span.x > span.y * 1.2 || span.z > 0.6);
            let mean = radii.iter().sum::<f32>() / radii.len() as f32;
            let spread = radii
                .iter()
                .map(|radius| (radius - mean).abs())
                .sum::<f32>()
                / radii.len() as f32;
            assert!(spread > 0.05, "variant {variant} is too spherical");
            assert!(
                positions
                    .iter()
                    .all(|position| position.iter().all(|value| value.is_finite()))
            );
        }
    }
}
