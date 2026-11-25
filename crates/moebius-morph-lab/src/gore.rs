//! Presentation meshes. They do not move the material books.
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, Mesh},
    prelude::Vec3,
    render::render_resource::PrimitiveTopology,
};

struct Buf {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

impl Buf {
    fn new() -> Self {
        Self {
            positions: vec![],
            normals: vec![],
            colors: vec![],
            indices: vec![],
        }
    }
    fn tri(&mut self, a: [f32; 3], b: [f32; 3], c: [f32; 3], color: [f32; 4]) {
        let edge0 = Vec3::from_array(b) - Vec3::from_array(a);
        let edge1 = Vec3::from_array(c) - Vec3::from_array(a);
        let n = edge0.cross(edge1).normalize_or_zero().to_array();
        let base = self.positions.len() as u32;
        self.positions.extend([a, b, c]);
        self.normals.extend([n, n, n]);
        self.colors.extend([color, color, color]);
        self.indices.extend([base, base + 1, base + 2]);
    }
    fn ribbon(&mut self, origin: Vec3, tip: Vec3, half: f32, color: [f32; 4]) {
        let side = Vec3::new(half, 0., half * 0.4);
        self.tri(
            origin.to_array(),
            (origin + side).to_array(),
            tip.to_array(),
            color,
        );
        self.tri(
            origin.to_array(),
            tip.to_array(),
            (origin - side).to_array(),
            color,
        );
    }
    fn mesh(self) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, self.colors);
        mesh.insert_indices(Indices::U32(self.indices));
        mesh
    }
}

/// Far too many drips. Centers are already camera-relative.
pub fn hemorrhage_field(sites: &[(Vec3, u64)], frame: u32) -> Mesh {
    let mut buf = Buf::new();
    let wet = [0.62, 0.01, 0.03, 1.];
    let dark = [0.28, 0.0, 0.02, 1.];
    for (center, reservoir) in sites {
        let count = 48 + (*reservoir).min(48) as u32;
        for i in 0..count {
            let angle = i as f32 * 2.399;
            let radius = 0.04 + (i % 7) as f32 * 0.035;
            let origin = *center + Vec3::new(angle.cos() * radius, 0.08, angle.sin() * radius);
            let swing = (frame as f32 * 0.17 + i as f32 * 0.35).sin();
            let length = 0.45 + swing.abs() * (0.9 + *reservoir as f32 * 0.035);
            let tip = origin
                + Vec3::new(
                    swing * 0.12,
                    -length,
                    (i as f32 * 1.3 + frame as f32 * 0.05).cos() * 0.1,
                );
            buf.ribbon(
                origin,
                tip,
                0.018 + (i % 4) as f32 * 0.008,
                if i % 3 == 0 { dark } else { wet },
            );
        }
    }
    buf.mesh()
}

pub fn droplets(points: &[Vec3]) -> Mesh {
    let mut buf = Buf::new();
    let color = [0.72, 0.38, 0.14, 1.];
    for point in points {
        let up = *point + Vec3::Y * 0.09;
        let a = *point + Vec3::X * 0.06;
        let b = *point - Vec3::X * 0.04 + Vec3::Z * 0.05;
        let c = *point - Vec3::Z * 0.06;
        buf.tri(up.to_array(), a.to_array(), b.to_array(), color);
        buf.tri(up.to_array(), b.to_array(), c.to_array(), color);
        buf.tri(up.to_array(), c.to_array(), a.to_array(), color);
        buf.tri(a.to_array(), c.to_array(), b.to_array(), color);
    }
    buf.mesh()
}

/// Lumpy wet mass plus a ridiculous drip count. One mesh for the visible sites.
pub fn food_gore(sites: &[(Vec3, f32, u64)], frame: u32) -> Mesh {
    let mut buf = Buf::new();
    let flesh = [0.38, 0.02, 0.035, 1.];
    let blood = [0.7, 0.02, 0.04, 1.];
    for (center, radius, id) in sites {
        for ring in 0..4 {
            let y = (ring as f32 / 3. - 0.5) * radius * 1.4;
            let girth = (1. - (ring as f32 / 3. - 0.5).abs() * 1.2).max(0.25) * radius;
            let wobble = 1. + 0.18 * ((ring + *id as usize) as f32).sin();
            let mut band = [Vec3::ZERO; 6];
            for (s, point) in band.iter_mut().enumerate() {
                let angle = s as f32 / 6. * std::f32::consts::TAU;
                *point = *center
                    + Vec3::new(
                        angle.cos() * girth * wobble,
                        y,
                        angle.sin() * girth * wobble,
                    );
            }
            if ring > 0 {
                // Bridged from the previous ring stored by redrawing a fan against the center.
            }
            for s in 0..6 {
                buf.tri(
                    center.to_array(),
                    band[s].to_array(),
                    band[(s + 1) % 6].to_array(),
                    flesh,
                );
            }
        }
        let drips = 28;
        for i in 0..drips {
            let phase = (frame as f32 * 0.11 + i as f32 * 0.37 + *id as f32 * 0.2)
                .sin()
                .abs();
            let origin = *center
                + Vec3::new(
                    ((i as f32) * 1.3).sin() * radius * 0.4,
                    0.,
                    ((i as f32) * 0.9).cos() * radius * 0.4,
                );
            let tip = origin
                + Vec3::new(
                    0.04 * (i as f32).cos(),
                    -(0.5 + phase * (1.3 + radius)),
                    0.04 * (i as f32).sin(),
                );
            buf.ribbon(origin, tip, 0.02, blood);
        }
    }
    buf.mesh()
}

/// Root stalks connecting world-owned sac attachments to their visible bodies.
pub fn substrate_roots(roots: &[(Vec3, Vec3)]) -> Mesh {
    let mut buf = Buf::new();
    for (root, tip) in roots {
        let axis = (*tip - *root).normalize_or_zero();
        let radius =
            moebius_morph::habitat::ROOT_RADIUS as f32 / moebius_morph::emission::SCALE as f32;
        let cross = if axis.cross(Vec3::X).length_squared() > 0.01 {
            axis.cross(Vec3::X)
        } else {
            axis.cross(Vec3::Y)
        };
        let side = cross.normalize_or_zero() * radius;
        let up = axis.cross(side).normalize_or_zero() * radius;
        for k in 0..8 {
            let a = k as f32 * std::f32::consts::TAU / 8.;
            let b = (k + 1) as f32 * std::f32::consts::TAU / 8.;
            let x = side * a.cos() + up * a.sin();
            let y = side * b.cos() + up * b.sin();
            buf.tri(
                (*root + x).to_array(),
                (*root + y).to_array(),
                (*tip + x * 0.65).to_array(),
                [0.35, 0.07, 0.08, 1.],
            );
            buf.tri(
                (*root + y).to_array(),
                (*tip + y * 0.65).to_array(),
                (*tip + x * 0.65).to_array(),
                [0.35, 0.07, 0.08, 1.],
            );
        }
    }
    buf.mesh()
}
