use crate::{MorphError, require};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WormSpec {
    pub version: u32,
    pub seed: u64,
    pub length: f32,
    pub radius: f32,
    pub aspect: f32,
    pub radius_profile: [f32; 4],
    pub curvature: f32,
    pub body_rings: usize,
    pub radial_segments: usize,
    pub mouth: Mouth,
    pub flagella: Vec<Flagellum>,
    pub ridges: Option<Ridges>,
    pub appearance: Appearance,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mouth {
    pub aperture: f32,
    pub width: f32,
    pub depth: f32,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SurfaceCoordinate {
    pub station: f32,
    pub azimuth: f32,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Flagellum {
    pub attachment: SurfaceCoordinate,
    pub length: f32,
    pub radius: f32,
    pub phase: f32,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ridges {
    pub count: u32,
    pub height: f32,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Appearance {
    pub skin: [f32; 3],
    pub oral: [f32; 3],
    pub flagella: [f32; 3],
    pub roughness: f32,
}
fn bounded(v: f32, lo: f32, hi: f32) -> bool {
    v.is_finite() && v >= lo && v <= hi
}
impl WormSpec {
    pub fn from_seed(seed: u64) -> Self {
        let mut state = seed;
        // Independently implemented SplitMix64; integer seed expansion only.
        let mut next = || {
            state = state.wrapping_add(0x9e3779b97f4a7c15);
            let mut z = state;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
            ((z ^ (z >> 31)) >> 40) as f32 / 16777216.
        };
        let length = 3. + next() * 3.;
        let radius = 0.22 + next() * 0.22;
        let mut spec = Self {
            version: 1,
            seed,
            length,
            radius,
            aspect: 0.72 + next() * 0.5,
            radius_profile: [
                0.12,
                0.65 + next() * 0.25,
                0.85 + next() * 0.25,
                0.80 + next() * 0.28,
            ],
            curvature: (next() - 0.5) * 0.7,
            body_rings: 64,
            radial_segments: 32,
            mouth: Mouth {
                aperture: 0.8,
                width: 0.55 + next() * 0.35,
                depth: 0.5 + next() * 0.6,
            },
            flagella: vec![],
            ridges: Some(Ridges {
                count: 12 + (next() * 9.) as u32,
                height: 0.04 + next() * 0.06,
            }),
            appearance: Appearance {
                skin: [
                    0.15 + next() * 0.12,
                    0.025 + next() * 0.025,
                    0.045 + next() * 0.045,
                ],
                oral: [0.022, 0.004, 0.009],
                flagella: [0.30, 0.19, 0.11],
                roughness: 0.36,
            },
        };
        for angle in [0., std::f32::consts::PI] {
            spec.flagella.push(Flagellum {
                attachment: SurfaceCoordinate {
                    station: 0.18,
                    azimuth: angle,
                },
                length: length * 0.36,
                radius: radius * 0.12,
                phase: angle,
            });
        }
        spec
    }
    pub fn validate(&self) -> Result<(), MorphError> {
        require(
            self.version == 1,
            "unsupported anatomical specification version",
        )?;
        require(
            bounded(self.length, 2., 8.)
                && bounded(self.radius, 0.12, 0.65)
                && self.length / self.radius >= 5.,
            "body length/radius outside supported geometry envelope",
        )?;
        require(
            bounded(self.aspect, 0.55, 1.5) && bounded(self.curvature, -0.6, 0.6),
            "invalid aspect or curvature",
        )?;
        require(
            self.radius_profile.iter().all(|&v| bounded(v, 0.08, 1.3)),
            "radius profile requires four positive finite samples",
        )?;
        require(
            (24..=128).contains(&self.body_rings) && (16..=64).contains(&self.radial_segments),
            "mesh resolution outside bounded budget",
        )?;
        require(
            bounded(self.mouth.aperture, 0., 1.)
                && bounded(self.mouth.width, 0.4, 0.95)
                && bounded(self.mouth.depth, 0.3, 1.2),
            "invalid mouth parameters",
        )?;
        require(
            self.flagella.len() <= 2,
            "first family supports at most two flagella",
        )?;
        for f in &self.flagella {
            require(
                bounded(f.attachment.station, 0.08, 0.85)
                    && bounded(f.attachment.azimuth, 0., std::f32::consts::TAU)
                    && bounded(f.length, 0.15, 3.)
                    && bounded(f.radius, 0.008, 0.09)
                    && bounded(f.phase, -20., 20.),
                "invalid flagellum specification",
            )?;
        }
        if let Some(r) = &self.ridges {
            require(
                (3..=32).contains(&r.count) && bounded(r.height, 0., 0.15),
                "invalid ridge descriptor",
            )?;
        }
        require(
            self.appearance
                .skin
                .iter()
                .chain(&self.appearance.oral)
                .chain(&self.appearance.flagella)
                .all(|&v| bounded(v, 0., 1.))
                && bounded(self.appearance.roughness, 0.1, 1.),
            "invalid appearance",
        )
    }
}
