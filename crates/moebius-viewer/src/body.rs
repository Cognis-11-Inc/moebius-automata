//! Viewer costumes. These meshes are not organism detections.
//! The silhouette is elongated bone and pipe, not a sphere.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::prelude::*;

struct Sculpt {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
}

impl Sculpt {
    fn new() -> Self {
        Self {
            positions: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            indices: Vec::new(),
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
            self.indices.extend([a, c, b, b, c, d]);
        }
    }

    fn ribs(&mut self, center: Vec3, axis: Vec3, radius: f32, count: u32) {
        let dir = axis.normalize_or_zero();
        if dir.length_squared() < 0.5 {
            return;
        }
        let helper = if dir.y.abs() > 0.85 { Vec3::X } else { Vec3::Y };
        let side = dir.cross(helper).normalize();
        let up = dir.cross(side).normalize();
        for step in 0..count {
            let angle = step as f32 / count as f32 * std::f32::consts::TAU;
            let normal = side * angle.cos() + up * angle.sin();
            let root = center + normal * radius * 0.45;
            let tip = center + normal * radius;
            self.tube(root, tip, radius * 0.08, radius * 0.035, 5);
        }
    }

    fn mesh(self) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
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
    sculpt.mesh()
}

fn vertebral(sculpt: &mut Sculpt) {
    let joints = [
        (Vec3::new(0.0, -1.2, 0.0), 0.1),
        (Vec3::new(0.04, -0.72, 0.06), 0.2),
        (Vec3::new(-0.05, -0.22, -0.02), 0.14),
        (Vec3::new(0.06, 0.28, 0.08), 0.26),
        (Vec3::new(-0.02, 0.78, 0.0), 0.12),
        (Vec3::new(0.1, 1.22, -0.04), 0.05),
    ];
    for pair in joints.windows(2) {
        sculpt.tube(pair[0].0, pair[1].0, pair[0].1, pair[1].1, 7);
    }
    sculpt.ribs(joints[1].0, joints[2].0 - joints[1].0, 0.48, 7);
    sculpt.ribs(joints[3].0, joints[4].0 - joints[3].0, 0.62, 8);
    sculpt.tube(
        joints[3].0,
        joints[3].0 + Vec3::new(0.55, 0.15, 0.2),
        0.06,
        0.02,
        5,
    );
    sculpt.tube(
        joints[2].0,
        joints[2].0 + Vec3::new(-0.42, -0.1, 0.28),
        0.05,
        0.015,
        5,
    );
    let jaw = joints[5].0;
    sculpt.tube(jaw, jaw + Vec3::new(0.16, 0.34, 0.08), 0.045, 0.008, 5);
    sculpt.tube(jaw, jaw + Vec3::new(-0.14, 0.3, 0.1), 0.04, 0.008, 5);
    sculpt.tube(
        joints[4].0,
        joints[4].0 + Vec3::new(0.05, -0.62, 0.2),
        0.045,
        0.012,
        4,
    );
    sculpt.ribs(joints[2].0, joints[3].0 - joints[2].0, 0.36, 6);
}

fn curled(sculpt: &mut Sculpt) {
    let mut previous: Option<(Vec3, f32)> = None;
    for step in 0..7 {
        let t = step as f32 / 6.0;
        let angle = t * 2.4;
        let point = Vec3::new(angle.sin() * 0.55, t * 2.1 - 1.05, angle.cos() * 0.38);
        let radius = 0.08 + (t * std::f32::consts::PI).sin() * 0.16;
        if let Some((from, radius_a)) = previous {
            sculpt.tube(from, point, radius_a, radius, 6);
        }
        previous = Some((point, radius));
    }
    let head = Vec3::new(0.55_f32.sin() * 0.15, 1.05, 0.2);
    sculpt.ribs(head, Vec3::Y, 0.46, 6);
    sculpt.tube(head, head + Vec3::new(0.35, 0.45, -0.1), 0.07, 0.02, 5);
    sculpt.tube(
        Vec3::new(0.0, -1.05, 0.38),
        Vec3::new(-0.25, -1.45, 0.55),
        0.06,
        0.015,
        5,
    );
    sculpt.tube(head, head + Vec3::new(0.22, 0.28, 0.16), 0.04, 0.008, 5);
    sculpt.tube(head, head + Vec3::new(-0.18, 0.24, 0.18), 0.038, 0.008, 5);
    sculpt.tube(head, head + Vec3::new(0.02, -0.48, 0.32), 0.035, 0.01, 4);
}

fn thorax(sculpt: &mut Sculpt) {
    sculpt.tube(
        Vec3::new(-0.85, 0.0, 0.0),
        Vec3::new(0.9, 0.08, 0.05),
        0.16,
        0.22,
        8,
    );
    sculpt.tube(
        Vec3::new(0.9, 0.08, 0.05),
        Vec3::new(1.35, 0.22, -0.08),
        0.22,
        0.06,
        6,
    );
    sculpt.ribs(Vec3::new(0.05, 0.02, 0.0), Vec3::X, 0.58, 8);
    sculpt.tube(
        Vec3::new(-0.1, 0.12, 0.0),
        Vec3::new(-0.05, 0.72, 0.08),
        0.08,
        0.03,
        5,
    );
    sculpt.tube(
        Vec3::new(0.25, -0.05, 0.12),
        Vec3::new(0.4, -0.55, 0.35),
        0.07,
        0.02,
        5,
    );
    sculpt.tube(
        Vec3::new(-0.35, 0.0, -0.1),
        Vec3::new(-0.7, 0.35, -0.4),
        0.06,
        0.02,
        5,
    );
    let snout = Vec3::new(1.35, 0.22, -0.08);
    sculpt.tube(snout, snout + Vec3::new(0.28, 0.12, 0.1), 0.04, 0.008, 5);
    sculpt.tube(
        snout,
        snout + Vec3::new(0.26, -0.02, -0.12),
        0.036,
        0.008,
        5,
    );
    sculpt.ribs(Vec3::new(0.45, 0.06, 0.02), Vec3::X, 0.42, 6);
    sculpt.tube(
        Vec3::new(0.15, -0.02, -0.05),
        Vec3::new(0.2, -0.7, -0.25),
        0.05,
        0.012,
        4,
    );
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
    mesh.insert_indices(Indices::U32(sculpt.indices));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn costumes_are_elongated_pipes_not_spheres() {
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
