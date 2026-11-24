//! Version2 composition. Saved coordinates describe anatomy, never mesh triangle numbers.
use crate::{
    compiler::{attachment, generate, ring},
    math::*,
    *,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    f32::consts::{PI, TAU},
    path::Path,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnatomySpec {
    pub version: u32,
    /// Original v1 trunk and genuine primary mouth. Its flagella list must be empty.
    pub body: WormSpec,
    pub components: Vec<Component>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Component {
    pub id: u64,
    /// Reserved body0; primary mouth1. Only body attachments implemented in v2.
    pub parent: u64,
    pub attachment: SurfaceCoordinate,
    /// Local yaw around normal and pitch away from the backwards trunk tangent.
    pub orientation: [f32; 2],
    pub shape: Shape,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum Shape {
    Flagellum {
        length: f32,
        radius: f32,
        taper: f32,
        curvature: f32,
        amplitude: f32,
        frequency: f32,
        phase: f32,
    },
    Spine {
        length: f32,
        radius: f32,
        taper: f32,
        curvature: f32,
    },
    SecretionNozzle {
        length: f32,
        radius: f32,
        aperture: f32,
        output: EmissionMetadata,
    },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmissionMetadata {
    pub material_channel: String,
    /// Only false is valid: anatomy does not grant material emission.
    pub operational: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OutputFrame {
    pub component_id: u64,
    pub position: [f32; 3],
    pub direction: [f32; 3],
    pub metadata: EmissionMetadata,
}
#[derive(Clone)]
pub struct CompiledAnatomy {
    spec: AnatomySpec,
    topology: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnatomicalPose {
    pub geometry: Organism,
    pub component_ids: Vec<u64>,
    pub outputs: Vec<OutputFrame>,
}
fn bound(v: f32, lo: f32, hi: f32) -> bool {
    v.is_finite() && (lo..=hi).contains(&v)
}
impl Component {
    pub fn flagellum(id: u64, station: f32, azimuth: f32) -> Self {
        Self {
            id,
            parent: 0,
            attachment: SurfaceCoordinate { station, azimuth },
            orientation: [0., 0.],
            shape: Shape::Flagellum {
                length: 1.2,
                radius: 0.025,
                taper: 0.92,
                curvature: 0.,
                amplitude: 0.14,
                frequency: 1.8,
                phase: azimuth,
            },
        }
    }
    pub fn kind(&self) -> &'static str {
        match self.shape {
            Shape::Flagellum { .. } => "Flagellum",
            Shape::Spine { .. } => "Spine",
            Shape::SecretionNozzle { .. } => "Nozzle (unbound)",
        }
    }
}
impl AnatomySpec {
    pub fn migrate(mut body: WormSpec) -> Result<Self, MorphError> {
        body.validate()?;
        let components = std::mem::take(&mut body.flagella)
            .into_iter()
            .enumerate()
            .map(|(i, f)| {
                let mut c =
                    Component::flagellum(i as u64 + 2, f.attachment.station, f.attachment.azimuth);
                c.shape = Shape::Flagellum {
                    length: f.length,
                    radius: f.radius,
                    taper: 0.92,
                    curvature: 0.,
                    amplitude: 0.14,
                    frequency: 1.8,
                    phase: f.phase,
                };
                c
            })
            .collect();
        let s = Self {
            version: 2,
            body,
            components,
        };
        s.validate()?;
        Ok(s)
    }
    pub fn from_seed(seed: u64) -> Self {
        Self::migrate(WormSpec::from_seed(seed)).expect("valid seed")
    }
    /// One reproducible construction witness; no species or simulation capability.
    pub fn showcase(seed: u64) -> Self {
        let mut s = Self::from_seed(seed);
        s.body.length = 2.2;
        s.body.radius = 0.36;
        s.body.ridges = None;
        for i in 0..8 {
            let mut c =
                Component::flagellum(i + 2, 0.18 + (i % 3) as f32 * 0.24, TAU * i as f32 / 8.);
            if let Shape::Flagellum { length, phase, .. } = &mut c.shape {
                *length = 0.75 + i as f32 * 0.11;
                *phase = i as f32 * 0.8;
            }
            c.orientation = [i as f32 * 0.09, -0.2 + i as f32 * 0.06];
            s.components.push(c);
        }
        s.components.drain(..2);
        let id = s.next_id();
        let mut spine = Component::flagellum(id, 0.7, PI * 0.5);
        spine.shape = Shape::Spine {
            length: 0.4,
            radius: 0.07,
            taper: 0.92,
            curvature: 0.1,
        };
        s.components.push(spine);
        let mut nozzle = Component::flagellum(s.next_id(), 0.65, PI * 1.5);
        nozzle.orientation = [0., 0.8];
        nozzle.shape = Shape::SecretionNozzle {
            length: 0.38,
            radius: 0.095,
            aperture: 0.60,
            output: EmissionMetadata {
                material_channel: "future_conserved_substrate".into(),
                operational: false,
            },
        };
        s.components.push(nozzle);
        s
    }
    /// New-game trunk: the generator's mouth, plus one secretion nozzle.
    /// Flagella, spines, and extra nozzles are added by the player.
    pub fn bare(seed: u64) -> Self {
        let mut s = Self::from_seed(seed);
        s.body.length = 2.2;
        s.body.radius = 0.36;
        s.body.ridges = None;
        s.components.clear();
        let mut nozzle = Component::flagellum(2, 0.65, PI * 1.5);
        nozzle.orientation = [0., 0.8];
        nozzle.shape = Shape::SecretionNozzle {
            length: 0.38,
            radius: 0.095,
            aperture: 0.60,
            output: EmissionMetadata {
                material_channel: "future_conserved_substrate".into(),
                operational: false,
            },
        };
        s.components.push(nozzle);
        s.validate().expect("bare trunk");
        s
    }
    /// Blood-producing body from the same generator. The envelope still
    /// requires a long trunk, so this is a swollen short body with a deep
    /// pore and root hairs, not a separate mesh language.
    pub fn blood_sac(seed: u64) -> Self {
        let mut s = Self::from_seed(seed);
        s.body.length = 2.05;
        s.body.radius = 0.40;
        s.body.aspect = 1.15;
        s.body.curvature = 0.18;
        s.body.radius_profile = [0.5, 1.15, 1.28, 1.02];
        s.body.mouth.aperture = 1.;
        s.body.mouth.width = 0.9;
        s.body.mouth.depth = 1.1;
        s.body.ridges = Some(crate::Ridges {
            count: 7,
            height: 0.025,
        });
        s.body.appearance.skin = [0.46, 0.02, 0.035];
        s.body.appearance.oral = [0.78, 0.01, 0.03];
        s.body.appearance.flagella = [0.32, 0.02, 0.04];
        s.body.appearance.roughness = 0.18;
        s.components.clear();
        for i in 0..5 {
            let mut c =
                Component::flagellum(i as u64 + 2, 0.28 + i as f32 * 0.08, TAU * i as f32 / 5.);
            if let Shape::Flagellum {
                length,
                radius,
                amplitude,
                ..
            } = &mut c.shape
            {
                *length = 0.24;
                *radius = 0.012;
                *amplitude = 0.05;
            }
            s.components.push(c);
        }
        s.validate().expect("blood sac");
        s
    }
    pub fn next_id(&self) -> u64 {
        self.components
            .iter()
            .map(|c| c.id)
            .max()
            .unwrap_or(1)
            .saturating_add(1)
    }
    pub fn validate(&self) -> Result<(), MorphError> {
        require(self.version == 2, "unsupported composition version")?;
        self.body.validate()?;
        require(
            self.body.flagella.is_empty(),
            "v2 flagella must be independent components",
        )?;
        require(
            self.components.len() <= 256,
            "component mesh budget exceeded (prototype budget, not anatomical ceiling)",
        )?;
        let mut ids = BTreeSet::from([0, 1]);
        for c in &self.components {
            require(
                c.id < u64::MAX && ids.insert(c.id),
                "duplicate or reserved component id",
            )?;
            require(
                c.parent == 0,
                "unsupported or dangling parent; only trunk attachment supported",
            )?;
            require(
                bound(c.attachment.station, 0.08, 0.85)
                    && bound(c.attachment.azimuth, 0., TAU)
                    && c.orientation.iter().all(|&v| bound(v, -PI, PI)),
                "invalid attachment coordinates or orientation",
            )?;
            let (l, r) = match &c.shape {
                Shape::Flagellum {
                    length,
                    radius,
                    taper,
                    curvature,
                    amplitude,
                    frequency,
                    phase,
                } => {
                    require(
                        bound(*taper, 0., 0.98)
                            && bound(*curvature, -0.4, 0.4)
                            && bound(*amplitude, 0., 0.25)
                            && bound(*frequency, 0., 4.)
                            && bound(*phase, -20., 20.),
                        "invalid flagellar animation or taper",
                    )?;
                    (*length, *radius)
                }
                Shape::Spine {
                    length,
                    radius,
                    taper,
                    curvature,
                } => {
                    require(
                        bound(*taper, 0., 0.98) && bound(*curvature, -0.4, 0.4),
                        "invalid spine shape",
                    )?;
                    (*length, *radius)
                }
                Shape::SecretionNozzle {
                    length,
                    radius,
                    aperture,
                    output,
                } => {
                    require(
                        bound(*aperture, 0.2, 0.8)
                            && !output.operational
                            && !output.material_channel.is_empty()
                            && output.material_channel.len() <= 128,
                        "invalid or falsely operational nozzle",
                    )?;
                    (*length, *radius)
                }
            };
            require(
                bound(l, 0.15, 3.) && bound(r, 0.008, 0.15) && l / r >= 2.,
                "component dimensions outside swept-surface envelope",
            )?;
        }
        Ok(())
    }
}
impl CompiledAnatomy {
    pub fn new(spec: AnatomySpec) -> Result<Self, MorphError> {
        spec.validate()?;
        let p = Pose::rest(&spec.body);
        let g = compose(&spec, p);
        g.geometry.audit()?;
        Ok(Self {
            spec,
            topology: g.geometry.topology_digest(),
        })
    }
    pub fn pose(&self, p: Pose) -> Result<AnatomicalPose, MorphError> {
        require(
            p.phase.is_finite()
                && p.phase.abs() < 100000.
                && [p.bend, p.mouth, p.flagella]
                    .iter()
                    .all(|v| (0. ..=1.).contains(v)),
            "invalid pose",
        )?;
        let g = compose(&self.spec, p);
        require(
            g.geometry.topology_digest() == self.topology,
            "pose changed topology",
        )?;
        Ok(g)
    }
    pub fn spec(&self) -> &AnatomySpec {
        &self.spec
    }
}
fn compose(s: &AnatomySpec, p: Pose) -> AnatomicalPose {
    let mut geometry = generate(&s.body, p);
    let mut outputs = vec![];
    for c in &s.components {
        let mount = attachment(&s.body, p, &geometry.surfaces[0], &c.attachment);
        let (yaw, pitch) = (c.orientation[0], c.orientation[1]);
        let backwards = add(
            mul(mount.tangent, -yaw.cos()),
            mul(mount.binormal, yaw.sin()),
        );
        let direction = unit(add(
            mul(mount.normal, 0.65 * pitch.cos() + 0.76 * pitch.sin()),
            mul(backwards, 0.76 * pitch.cos() - 0.65 * pitch.sin()),
        ));
        let (sx, sy) = frame(direction, mount.binormal);
        let (length, radius, taper, curve, amp, freq, phase, nozzle) = match &c.shape {
            Shape::Flagellum {
                length,
                radius,
                taper,
                curvature,
                amplitude,
                frequency,
                phase,
            } => (
                *length, *radius, *taper, *curvature, *amplitude, *frequency, *phase, None,
            ),
            Shape::Spine {
                length,
                radius,
                taper,
                curvature,
            } => (*length, *radius, *taper, *curvature, 0., 0., 0., None),
            Shape::SecretionNozzle {
                length,
                radius,
                aperture,
                output,
            } => (
                *length,
                *radius,
                0.15,
                0.,
                0.,
                0.,
                0.,
                Some((*aperture, output)),
            ),
        };
        let origin = mount.origin;
        let mut part = Surface::new(&format!("component_{}_{}", c.id, c.kind()));
        let color = match c.shape {
            Shape::Spine { .. } => [0.30, 0.24, 0.19],
            Shape::SecretionNozzle { .. } => [0.31, 0.075, 0.10],
            _ => s.body.appearance.flagella,
        };
        // Decorative collar for spine and nozzle only. The root stays on the
        // body triangle. Over the first quarter of the tube the section spreads
        // across the skin, flattens against the trunk, and takes the trunk colour.
        // It remains its own closed surface: not a weld, load path, or material
        // joint. Flagella keep the previous round tube. The nozzle lip and cavity
        // sit past this blend.
        let flesh = matches!(c.shape, Shape::Spine { .. } | Shape::SecretionNozzle { .. });
        let skin = s.body.appearance.skin;
        let flesh_normal = mount.normal;
        let flesh_weight = |u: f32| {
            if !flesh {
                return 0.;
            }
            let t = (u / 0.24).clamp(0., 1.);
            1. - t * t * (3. - 2. * t)
        };
        let cap = part.vertex(origin, [0.5, 0.], if flesh { skin } else { color });
        let mut prev = 0;
        let center = |u: f32| {
            let wave = p.flagella * length * amp * (PI * u).sin();
            let angle = p.phase * freq - u * TAU + phase;
            add(
                add(origin, mul(direction, length * u)),
                add(
                    mul(sx, wave * angle.sin() + curve * length * u * u),
                    mul(sy, wave * 0.55 * angle.cos()),
                ),
            )
        };
        for r in 0..=28 {
            let u = if nozzle.is_some() {
                r as f32 / 28.
            } else {
                r as f32 / 29.
            };
            let rad = radius * (1. - u * taper);
            let w = flesh_weight(u);
            let tone = if w == 0. {
                color
            } else {
                mix(skin, color, 1. - w)
            };
            let here = center(u);
            let start = ring(&mut part, here, (sx, sy), [rad, rad], u, 10, tone);
            if w > 0. {
                // Reshape inside the ring plane. The cap stays at the attachment,
                // so this collar does not become an inward cone.
                let projected = sub(flesh_normal, mul(direction, dot(flesh_normal, direction)));
                let spread = 1. + 0.95 * w;
                let count = start as usize;
                if dot(projected, projected) > 0.0625 {
                    let axis = unit(projected);
                    let keep = 1. - 0.66 * w;
                    for vertex in &mut part.positions[count..count + 10] {
                        let offset = sub(*vertex, here);
                        let along = dot(offset, axis);
                        let lateral = sub(offset, mul(axis, along));
                        *vertex = add(here, add(mul(lateral, spread), mul(axis, along * keep)));
                    }
                } else {
                    for vertex in &mut part.positions[count..count + 10] {
                        *vertex = add(here, mul(sub(*vertex, here), spread));
                    }
                }
            }
            if r == 0 {
                for j in 0..10 {
                    part.triangle(cap, start + (j + 1) % 10, start + j);
                }
            } else {
                part.bridge(prev, start, 10);
            }
            prev = start;
        }
        if let Some((aperture, metadata)) = nozzle {
            // Annular lip leads into a real recessed cavity; no material emission.
            for i in 0..=8 {
                let q = i as f32 / 8.;
                let rad = radius * (1. - taper) * aperture * (1. - 0.35 * q);
                let start = ring(
                    &mut part,
                    center(1. - 0.7 * q),
                    (sx, sy),
                    [rad, rad],
                    1. + q,
                    10,
                    mul(color, 0.35),
                );
                part.bridge(prev, start, 10);
                prev = start;
            }
            let base = part.vertex(center(0.3), [0.5, 2.], mul(color, 0.2));
            for j in 0..10 {
                part.triangle(prev + j, prev + (j + 1) % 10, base);
            }
            outputs.push(OutputFrame {
                component_id: c.id,
                position: center(1.),
                direction,
                metadata: metadata.clone(),
            });
        } else {
            let tip = part.vertex(center(1.), [0.5, 1.], color);
            for j in 0..10 {
                part.triangle(prev + j, prev + (j + 1) % 10, tip);
            }
        }
        part.normals();
        geometry.attachments.push(mount);
        geometry.surfaces.push(part);
    }
    AnatomicalPose {
        geometry,
        component_ids: s.components.iter().map(|c| c.id).collect(),
        outputs,
    }
}
pub fn anatomy_from_json(json: &str) -> Result<AnatomySpec, MorphError> {
    let v: serde_json::Value = serde_json::from_str(json).map_err(|e| MorphError(e.to_string()))?;
    if v.get("version").and_then(|v| v.as_u64()) == Some(1) {
        return AnatomySpec::migrate(from_json(json)?);
    }
    let s: AnatomySpec = serde_json::from_value(v).map_err(|e| MorphError(e.to_string()))?;
    s.validate()?;
    Ok(s)
}
pub fn load_anatomy(path: &Path) -> Result<AnatomySpec, MorphError> {
    anatomy_from_json(&std::fs::read_to_string(path).map_err(|e| MorphError(e.to_string()))?)
}
pub fn save_anatomy(path: &Path, s: &AnatomySpec) -> Result<(), MorphError> {
    s.validate()?;
    let data = serde_json::to_string_pretty(s).map_err(|e| MorphError(e.to_string()))?;
    crate::write_atomic(path, &data)
}
