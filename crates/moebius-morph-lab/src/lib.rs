//! Editor command model, usable without a window. The morphology engine owns geometry.
use moebius_morph::{CompiledWorm, MorphError, Organism, Pose, WormSpec};
use std::path::PathBuf;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Seed,
    Length,
    Thickness,
    Mouth,
    FlagLength,
    FlagCount,
    Curvature,
}
impl Field {
    pub const ALL: [Self; 7] = [
        Self::Seed,
        Self::Length,
        Self::Thickness,
        Self::Mouth,
        Self::FlagLength,
        Self::FlagCount,
        Self::Curvature,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Seed => "Seed",
            Self::Length => "Body length",
            Self::Thickness => "Body thickness",
            Self::Mouth => "Mouth aperture",
            Self::FlagLength => "Flagella length",
            Self::FlagCount => "Flagella count",
            Self::Curvature => "Spine curvature",
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub enum Action {
    Adjust(f32),
    Select(usize),
    NextSeed,
    ToggleAnimation,
    ToggleMouthCycle,
    CycleFlagCount,
    ToggleRidges,
    Save,
    Reload,
}
pub struct Session {
    pub spec: WormSpec,
    pub compiled: CompiledWorm,
    pub field: Field,
    pub animation: bool,
    pub mouth_cycle: bool,
    pub phase: f32,
    pub path: PathBuf,
    pub status: String,
}
impl Session {
    pub fn new(spec: WormSpec, path: PathBuf) -> Result<Self, MorphError> {
        Ok(Self {
            compiled: CompiledWorm::new(spec.clone())?,
            spec,
            field: Field::Mouth,
            animation: true,
            mouth_cycle: true,
            phase: 0.,
            path,
            status: "Local anatomy. No simulation motor binding.".into(),
        })
    }
    pub fn pose(&self) -> Pose {
        Pose {
            phase: self.phase,
            bend: 0.8,
            mouth: if self.mouth_cycle {
                (0.5 + 0.48 * (self.phase * 1.2).sin()).clamp(0., 1.)
            } else {
                self.spec.mouth.aperture
            },
            flagella: 1.,
        }
    }
    pub fn geometry(&self) -> Result<Organism, MorphError> {
        self.compiled.pose(self.pose())
    }
    pub fn action(&mut self, action: Action) -> Result<(), MorphError> {
        let mut candidate = self.spec.clone();
        match action {
            Action::Select(i) => {
                self.field = Field::ALL[i % Field::ALL.len()];
                return Ok(());
            }
            Action::ToggleAnimation => {
                self.animation = !self.animation;
                return Ok(());
            }
            Action::ToggleMouthCycle => {
                self.mouth_cycle = !self.mouth_cycle;
                return Ok(());
            }
            Action::Save => {
                moebius_morph::save(&self.path, &self.spec)?;
                self.status = format!("SAVED {}", self.path.display());
                return Ok(());
            }
            Action::Reload => {
                candidate = moebius_morph::load(&self.path)?;
            }
            Action::NextSeed => candidate = WormSpec::from_seed(self.spec.seed.wrapping_add(1)),
            Action::CycleFlagCount => set_count(&mut candidate, (self.spec.flagella.len() + 1) % 3),
            Action::ToggleRidges => {
                candidate.ridges = if candidate.ridges.is_some() {
                    None
                } else {
                    WormSpec::from_seed(candidate.seed).ridges
                };
            }
            Action::Adjust(delta) => match self.field {
                Field::Seed => {
                    candidate = WormSpec::from_seed(if delta < 0. {
                        candidate.seed.saturating_sub(1)
                    } else {
                        candidate.seed.wrapping_add(1)
                    })
                }
                Field::Length => {
                    candidate.length =
                        (candidate.length + delta * 0.15).clamp((candidate.radius * 5.).max(2.), 8.)
                }
                Field::Thickness => {
                    candidate.radius = (candidate.radius + delta * 0.02)
                        .clamp(0.12, (candidate.length / 5.).min(0.65))
                }
                Field::Mouth => {
                    candidate.mouth.aperture =
                        (candidate.mouth.aperture + delta * 0.1).clamp(0., 1.)
                }
                Field::FlagLength => {
                    for f in &mut candidate.flagella {
                        f.length = (f.length + delta * 0.1).clamp(0.15, 3.);
                    }
                }
                Field::FlagCount => {
                    let count = (candidate.flagella.len() as i32 + if delta < 0. { -1 } else { 1 })
                        .clamp(0, 2) as usize;
                    set_count(&mut candidate, count);
                }
                Field::Curvature => {
                    candidate.curvature = (candidate.curvature + delta * 0.04).clamp(-0.6, 0.6)
                }
            },
        }
        // Compile before changing any accepted definition or displayed geometry.
        let compiled = CompiledWorm::new(candidate.clone())?;
        if matches!(action, Action::Adjust(_)) && self.field == Field::Mouth {
            self.mouth_cycle = false;
        }
        self.spec = candidate;
        self.compiled = compiled;
        self.status = match action {
            Action::Reload => format!("RELOADED {}", self.path.display()),
            _ => "Definition validated and compiled.".into(),
        };
        Ok(())
    }
}
fn set_count(s: &mut WormSpec, count: usize) {
    let defaults = WormSpec::from_seed(s.seed).flagella;
    while s.flagella.len() < count {
        s.flagella.push(defaults[s.flagella.len()].clone());
    }
    s.flagella.truncate(count);
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn editing_and_persistence_reproduce_the_same_live_structure() {
        let path = std::env::temp_dir().join(format!("morph-editor-{}.json", std::process::id()));
        let mut s = Session::new(WormSpec::from_seed(142), path.clone()).unwrap();
        s.action(Action::ToggleAnimation).unwrap();
        s.action(Action::ToggleMouthCycle).unwrap();
        for i in 1..7 {
            s.action(Action::Select(i)).unwrap();
            s.action(Action::Adjust(1.)).unwrap();
        }
        s.action(Action::Save).unwrap();
        let spec = s.spec.clone();
        let digest = s.geometry().unwrap().digest();
        s.action(Action::NextSeed).unwrap();
        assert_ne!(spec, s.spec);
        s.action(Action::Reload).unwrap();
        assert_eq!(spec, s.spec);
        assert_eq!(digest, s.geometry().unwrap().digest());
        std::fs::write(&path, "{\"version\":99}").unwrap();
        assert!(s.action(Action::Reload).is_err());
        assert_eq!(spec, s.spec);
        assert_eq!(digest, s.geometry().unwrap().digest());
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn flag_count_add_remove_and_open_close_are_real_topology_and_geometry_edits() {
        let mut s = Session::new(WormSpec::from_seed(72), PathBuf::from("unused.json")).unwrap();
        s.action(Action::CycleFlagCount).unwrap();
        assert_eq!(s.geometry().unwrap().surfaces.len(), 1);
        s.action(Action::CycleFlagCount).unwrap();
        assert_eq!(s.geometry().unwrap().surfaces.len(), 2);
        s.action(Action::CycleFlagCount).unwrap();
        assert_eq!(s.geometry().unwrap().surfaces.len(), 3);
        s.action(Action::Select(3)).unwrap();
        s.action(Action::Adjust(-10.)).unwrap();
        let closed = s.geometry().unwrap();
        s.action(Action::Adjust(10.)).unwrap();
        let opened = s.geometry().unwrap();
        assert_ne!(closed.digest(), opened.digest());
        assert_eq!(closed.topology_digest(), opened.topology_digest());
        opened.audit().unwrap();
        closed.audit().unwrap();
    }
}

/// Preview frequencies are1,0.7,1.2 and1.8; ten base turns close every cycle.
/// Wrapping at one base turn creates a visible discontinuity in slower channels.
pub fn advance_phase(phase: f32, seconds: f32) -> f32 {
    (phase + seconds * 0.9).rem_euclid(std::f32::consts::TAU * 10.)
}
#[cfg(test)]
mod continuity_tests {
    use super::*;
    #[test]
    fn preview_wrap_is_continuous_for_body_mouth_and_flagella() {
        let mut s = Session::new(WormSpec::from_seed(51), PathBuf::from("unused.json")).unwrap();
        s.phase = std::f32::consts::TAU * 10. - 0.0001;
        let before = s.geometry().unwrap();
        let aperture = s.pose().mouth;
        s.phase = advance_phase(s.phase, 0.0002 / 0.9);
        let after = s.geometry().unwrap();
        assert!(s.phase < 0.001);
        assert!((aperture - s.pose().mouth).abs() < 0.001);
        for (a, b) in before.surfaces.iter().zip(&after.surfaces) {
            let max = a
                .positions
                .iter()
                .zip(&b.positions)
                .map(|(a, b)| {
                    a.iter()
                        .zip(b)
                        .map(|(a, b)| (a - b).powi(2))
                        .sum::<f32>()
                        .sqrt()
                })
                .fold(0f32, f32::max);
            assert!(max < 0.002, "loop boundary jump in {}: {max}", a.name);
        }
    }
}
