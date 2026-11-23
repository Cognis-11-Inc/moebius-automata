use crate::{MorphError, Surface, WormSpec, math::*, require};
use serde::{Deserialize, Serialize};
use std::f32::consts::{PI, TAU};
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pose {
    pub phase: f32,
    pub bend: f32,
    pub mouth: f32,
    pub flagella: f32,
}
impl Pose {
    pub fn rest(spec: &WormSpec) -> Self {
        Self {
            phase: 0.,
            bend: 0.,
            mouth: spec.mouth.aperture,
            flagella: 0.,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AttachmentFrame {
    pub origin: V,
    pub normal: V,
    pub tangent: V,
    pub binormal: V,
    pub triangle: [u32; 3],
    pub barycentric: [f32; 3],
    pub capability: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Organism {
    pub surfaces: Vec<Surface>,
    pub attachments: Vec<AttachmentFrame>,
    pub mouth_ring: Vec<u32>,
    pub mouth_center: V,
    pub throat_base: u32,
    pub mechanical_binding: Option<String>,
    pub deformation: String,
}
impl Organism {
    pub fn digest(&self) -> String {
        let mut h = blake3::Hasher::new();
        for s in &self.surfaces {
            h.update(s.digest().as_bytes());
        }
        for a in &self.attachments {
            for v in a
                .origin
                .iter()
                .chain(&a.normal)
                .chain(&a.tangent)
                .chain(&a.binormal)
            {
                h.update(&v.to_bits().to_le_bytes());
            }
        }
        h.finalize().to_hex().to_string()
    }
    pub fn topology_digest(&self) -> String {
        let mut h = blake3::Hasher::new();
        for s in &self.surfaces {
            h.update(s.name.as_bytes());
            for i in &s.indices {
                h.update(&i.to_le_bytes());
            }
        }
        h.finalize().to_hex().to_string()
    }
    pub fn audit(&self) -> Result<Vec<crate::MeshAudit>, MorphError> {
        let audits = self
            .surfaces
            .iter()
            .map(Surface::audit)
            .collect::<Result<Vec<_>, _>>()?;
        require(
            self.attachments.len() + 1 == self.surfaces.len(),
            "attachment/component mismatch",
        )?;
        for (frame, part) in self.attachments.iter().zip(&self.surfaces[1..]) {
            let point = frame
                .triangle
                .iter()
                .zip(frame.barycentric)
                .fold([0.; 3], |a, (&i, w)| {
                    add(a, mul(self.surfaces[0].positions[i as usize], w))
                });
            require(
                length(sub(frame.origin, point)) < 1e-5
                    && length(sub(part.positions[0], frame.origin)) < 1e-5,
                "detached flagellum root",
            )?;
            require(
                (dot(frame.normal, frame.tangent)).abs() < 1e-4
                    && (length(frame.normal) - 1.).abs() < 1e-4
                    && (length(frame.tangent) - 1.).abs() < 1e-4,
                "invalid attachment frame",
            )?;
            require(
                frame.capability == "decorative_unbound",
                "unimplemented physical capability",
            )?;
        }
        Ok(audits)
    }
}
#[derive(Clone)]
pub struct CompiledWorm {
    spec: WormSpec,
    topology: String,
}
impl CompiledWorm {
    pub fn new(spec: WormSpec) -> Result<Self, MorphError> {
        spec.validate()?;
        let geometry = generate(&spec, Pose::rest(&spec));
        geometry.audit()?;
        Ok(Self {
            spec,
            topology: geometry.topology_digest(),
        })
    }
    pub fn spec(&self) -> &WormSpec {
        &self.spec
    }
    /// Parametric surface controls remain live; poses never alter the definition.
    pub fn pose(&self, pose: Pose) -> Result<Organism, MorphError> {
        require(
            pose.phase.is_finite()
                && pose.phase.abs() < 100000.
                && (0. ..=1.).contains(&pose.bend)
                && (0. ..=1.).contains(&pose.mouth)
                && (0. ..=1.).contains(&pose.flagella),
            "invalid deformation pose",
        )?;
        let geometry = generate(&self.spec, pose);
        require(
            geometry.topology_digest() == self.topology,
            "pose changed topology",
        )?;
        Ok(geometry)
    }
}
fn center(s: &WormSpec, p: Pose, t: f32) -> V {
    let envelope = (PI * t).sin();
    [
        s.length
            * (s.curvature * 0.08 * envelope
                + p.bend * 0.035 * (TAU * t - p.phase).sin() * envelope),
        s.length * p.bend * 0.018 * (TAU * t + p.phase * 0.7).sin() * envelope,
        (t - 0.5) * s.length,
    ]
}
fn tangent(s: &WormSpec, p: Pose, t: f32) -> V {
    unit(sub(center(s, p, t + 0.0001), center(s, p, t - 0.0001)))
}
fn radius(s: &WormSpec, t: f32, angle: f32) -> f32 {
    let axis = t * 3.;
    let i = (axis.floor() as usize).min(2);
    let a = axis - i as f32;
    let a = a * a * (3. - 2. * a);
    let mut r = s.radius * (s.radius_profile[i] * (1. - a) + s.radius_profile[i + 1] * a);
    if let Some(ridge) = &s.ridges {
        r *= 1.
            + ridge.height
                * (0.5 + 0.5 * (TAU * ridge.count as f32 * t).cos()).powi(6)
                * (0.55 + 0.45 * angle.sin().max(0.));
    }
    r
}
pub(crate) fn ring(
    mesh: &mut Surface,
    c: V,
    axes: (V, V),
    radii: [f32; 2],
    v: f32,
    sides: usize,
    color: V,
) -> u32 {
    let (x, y) = axes;
    let [rx, ry] = radii;
    let start = mesh.positions.len() as u32;
    for j in 0..sides {
        let a = j as f32 / sides as f32 * TAU;
        mesh.vertex(
            add(c, add(mul(x, rx * a.cos()), mul(y, ry * a.sin()))),
            [j as f32 / sides as f32, v],
            color,
        );
    }
    start
}
pub(crate) fn generate(s: &WormSpec, p: Pose) -> Organism {
    let n = s.radial_segments;
    let mut body = Surface::new("body_lips_and_recessed_throat");
    let mut side = [1., 0., 0.];
    let tail = body.vertex(center(s, p, 0.), [0.5, 0.], s.appearance.skin);
    let mut previous = 0;
    for i in 0..=s.body_rings {
        let t = i as f32 / s.body_rings as f32;
        let (x, y) = frame(tangent(s, p, t), side);
        side = x;
        let c = center(s, p, t);
        let start = body.positions.len() as u32;
        for j in 0..n {
            let a = j as f32 / n as f32 * TAU;
            let r = radius(s, t, a);
            let tone = 0.90 + 0.10 * (TAU * 14. * t).cos();
            body.vertex(
                add(c, add(mul(x, r * a.cos()), mul(y, r * s.aspect * a.sin()))),
                [j as f32 / n as f32, t],
                mul(s.appearance.skin, tone),
            );
        }
        if i == 0 {
            for j in 0..n as u32 {
                body.triangle(tail, start + (j + 1) % n as u32, start + j);
            }
        } else {
            body.bridge(previous, start, n);
        }
        previous = start;
    }
    let head = center(s, p, 1.);
    let z = tangent(s, p, 1.);
    let (x, y) = frame(z, side);
    let outer = s.radius * s.radius_profile[3];
    let inner_x = outer * s.mouth.width;
    let inner_y = outer * s.aspect * (0.018 + 0.72 * p.mouth);
    for i in 1..=5 {
        let q = i as f32 / 5.;
        let rx = outer * (1. - q) + inner_x * q;
        let ry = outer * s.aspect * (1. - q) + inner_y * q;
        let c = add(head, mul(z, outer * (0.12 * q + 0.12 * (PI * q).sin())));
        let start = ring(
            &mut body,
            c,
            (x, y),
            [rx, ry],
            1. + q * 0.1,
            n,
            mix(s.appearance.skin, [0.32, 0.065, 0.085], q * 0.35),
        );
        body.bridge(previous, start, n);
        previous = start;
    }
    let mouth_ring = (previous..previous + n as u32).collect();
    let mouth_center = add(head, mul(z, outer * 0.12));
    for i in 1..=12 {
        let q = i as f32 / 12.;
        let c = add(mouth_center, mul(z, -outer * s.mouth.depth * q));
        let rx = inner_x * (1. - q) + outer * 0.22 * q;
        let ry = inner_y * (1. - q) + outer * 0.22 * q;
        let color = mix(mul(s.appearance.skin, 0.45), s.appearance.oral, q.sqrt());
        let start = ring(&mut body, c, (x, y), [rx, ry], 1.1 + q * 0.2, n, color);
        body.bridge(previous, start, n);
        previous = start;
    }
    let base = body.vertex(
        add(mouth_center, mul(z, -outer * s.mouth.depth)),
        [0.5, 1.3],
        s.appearance.oral,
    );
    for j in 0..n as u32 {
        body.triangle(previous + j, previous + (j + 1) % n as u32, base);
    }
    body.normals();
    let mut attachments = vec![];
    let mut appendages = vec![];
    for (id, f) in s.flagella.iter().enumerate() {
        let mount = attachment(s, p, &body, &f.attachment);
        let AttachmentFrame {
            origin,
            normal,
            tangent,
            binormal,
            ..
        } = mount;
        let direction = unit(sub(mul(normal, 0.65), mul(tangent, 0.76)));
        let (sx, sy) = frame(direction, binormal);
        let mut part = Surface::new(&format!("flagellum_{id}"));
        let cap = part.vertex(origin, [0.5, 0.], s.appearance.flagella);
        let mut prev = 0;
        for r in 0..=28 {
            let u = r as f32 / 29.;
            let wave = p.flagella * f.length * 0.14 * (PI * u).sin();
            let angle = p.phase * 1.8 - u * TAU + f.phase;
            let c = add(
                add(origin, mul(direction, f.length * u)),
                add(
                    mul(sx, wave * angle.sin()),
                    mul(sy, wave * 0.55 * angle.cos()),
                ),
            );
            let rad = f.radius * (1. - u * 0.92);
            let start = ring(
                &mut part,
                c,
                (sx, sy),
                [rad, rad],
                u,
                10,
                s.appearance.flagella,
            );
            if r == 0 {
                for j in 0..10 {
                    part.triangle(cap, start + (j + 1) % 10, start + j);
                }
            } else {
                part.bridge(prev, start, 10);
            }
            prev = start;
        }
        let tip = part.vertex(
            add(origin, mul(direction, f.length)),
            [0.5, 1.],
            s.appearance.flagella,
        );
        for j in 0..10 {
            part.triangle(prev + j, prev + (j + 1) % 10, tip);
        }
        part.normals();
        attachments.push(mount);
        appendages.push(part);
    }
    let mut surfaces = vec![body];
    surfaces.extend(appendages);
    Organism {
        surfaces,
        attachments,
        mouth_ring,
        mouth_center,
        throat_base: base,
        mechanical_binding: None,
        deformation: "continuous parametric CPU preview; no production motor binding".into(),
    }
}

pub(crate) fn attachment(
    s: &WormSpec,
    p: Pose,
    body: &Surface,
    coord: &crate::SurfaceCoordinate,
) -> AttachmentFrame {
    let n = s.radial_segments;
    let a = coord.station * s.body_rings as f32;
    let i = a.floor() as u32;
    let u = a - i as f32;
    let b = coord.azimuth.rem_euclid(TAU) / TAU * n as f32;
    let j = b.floor() as u32;
    let v = b - j as f32;
    let first = 1 + i * n as u32 + j;
    let next = 1 + i * n as u32 + (j + 1) % n as u32;
    let (tri, weights) = if u + v <= 1. {
        ([first, next, first + n as u32], [1. - u - v, v, u])
    } else {
        (
            [next, next + n as u32, first + n as u32],
            [1. - u, u + v - 1., 1. - v],
        )
    };
    let interpolate = |array: &[V]| {
        tri.iter()
            .zip(weights)
            .fold([0.; 3], |p, (&i, w)| add(p, mul(array[i as usize], w)))
    };
    let origin = interpolate(&body.positions);
    let normal = unit(interpolate(&body.normals));
    let t = tangent(s, p, coord.station);
    let tangent = unit(sub(t, mul(normal, dot(t, normal))));
    let binormal = unit(cross(normal, tangent));
    AttachmentFrame {
        origin,
        normal,
        tangent,
        binormal,
        triangle: tri,
        barycentric: weights,
        capability: "decorative_unbound".into(),
    }
}
