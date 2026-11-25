//! Transactional component editor; the native UI and headless gates use these actions.
use moebius_morph::*;
use std::{
    f32::consts::{PI, TAU},
    path::PathBuf,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComponentField {
    Length,
    Radius,
    Station,
    Azimuth,
    Yaw,
    Pitch,
    Taper,
    Curvature,
    Amplitude,
    Frequency,
    Phase,
    Aperture,
    Capacity,
}
impl ComponentField {
    pub const ALL: [Self; 13] = [
        Self::Length,
        Self::Radius,
        Self::Station,
        Self::Azimuth,
        Self::Yaw,
        Self::Pitch,
        Self::Taper,
        Self::Curvature,
        Self::Amplitude,
        Self::Frequency,
        Self::Phase,
        Self::Aperture,
        Self::Capacity,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Length => "Length",
            Self::Radius => "Thickness",
            Self::Station => "Attachment station",
            Self::Azimuth => "Attachment angle",
            Self::Yaw => "Local yaw",
            Self::Pitch => "Local pitch",
            Self::Taper => "Taper",
            Self::Curvature => "Curvature",
            Self::Amplitude => "Animation amplitude",
            Self::Frequency => "Animation frequency",
            Self::Phase => "Animation phase",
            Self::Aperture => "Nozzle aperture",
            Self::Capacity => "Food pouch capacity",
        }
    }
    pub fn value(self, c: &Component) -> Option<f32> {
        match self {
            Self::Station => return Some(c.attachment.station),
            Self::Azimuth => return Some(c.attachment.azimuth),
            Self::Yaw => return Some(c.orientation[0]),
            Self::Pitch => return Some(c.orientation[1]),
            _ => {}
        }
        match (&c.shape, self) {
            (
                Shape::Flagellum { length, .. }
                | Shape::Spine { length, .. }
                | Shape::SecretionNozzle { length, .. }
                | Shape::FoodPouch { length, .. },
                Self::Length,
            ) => Some(*length),
            (
                Shape::Flagellum { radius, .. }
                | Shape::Spine { radius, .. }
                | Shape::SecretionNozzle { radius, .. }
                | Shape::FoodPouch { radius, .. },
                Self::Radius,
            ) => Some(*radius),
            (Shape::Flagellum { taper, .. } | Shape::Spine { taper, .. }, Self::Taper) => {
                Some(*taper)
            }
            (
                Shape::Flagellum { curvature, .. } | Shape::Spine { curvature, .. },
                Self::Curvature,
            ) => Some(*curvature),
            (Shape::Flagellum { amplitude, .. }, Self::Amplitude) => Some(*amplitude),
            (Shape::Flagellum { frequency, .. }, Self::Frequency) => Some(*frequency),
            (Shape::Flagellum { phase, .. }, Self::Phase) => Some(*phase),
            (Shape::SecretionNozzle { aperture, .. }, Self::Aperture) => Some(*aperture),
            (Shape::FoodPouch { capacity, .. }, Self::Capacity) => Some(*capacity as f32),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub enum Edit {
    SelectNext,
    SelectPrevious,
    AddFlagellum,
    AddSpine,
    AddNozzle,
    AddFoodPouch,
    Duplicate,
    Remove,
    Field(usize),
    Adjust(f32),
    BodyLength(f32),
    BodyRadius(f32),
    Mouth(f32),
    Seed,
    Showcase,
    Save,
    Reload,
}
pub struct CompositionSession {
    pub spec: AnatomySpec,
    pub compiled: CompiledAnatomy,
    pub selected: Option<u64>,
    pub field: ComponentField,
    pub animation: bool,
    pub mouth_cycle: bool,
    pub phase: f32,
    pub path: PathBuf,
    pub status: String,
    pub last_edit_micros: u128,
}
impl CompositionSession {
    pub fn new(spec: AnatomySpec, path: PathBuf) -> Result<Self, MorphError> {
        Ok(Self {
            compiled: CompiledAnatomy::new(spec.clone())?,
            selected: spec.components.first().map(|c| c.id),
            spec,
            field: ComponentField::Length,
            animation: true,
            mouth_cycle: true,
            phase: 0.,
            path,
            status: "Anatomical preview; no physical or neural binding".into(),
            last_edit_micros: 0,
        })
    }
    pub fn pose(&self) -> Pose {
        Pose {
            phase: self.phase,
            bend: 0.8,
            mouth: if self.mouth_cycle {
                (0.5 + 0.48 * (self.phase * 1.2).sin()).clamp(0., 1.)
            } else {
                self.spec.body.mouth.aperture
            },
            flagella: 1.,
        }
    }
    pub fn geometry(&self) -> Result<Organism, MorphError> {
        Ok(self.compiled.pose(self.pose())?.geometry)
    }
    pub fn component(&self) -> Option<&Component> {
        self.spec
            .components
            .iter()
            .find(|c| Some(c.id) == self.selected)
    }
    pub fn edit(&mut self, e: Edit) -> Result<(), MorphError> {
        let started = std::time::Instant::now();
        let mut candidate = self.spec.clone();
        let mut selected = self.selected;
        match e {
            Edit::SelectNext | Edit::SelectPrevious => {
                let n = candidate.components.len();
                if n > 0 {
                    let i = candidate
                        .components
                        .iter()
                        .position(|c| Some(c.id) == selected)
                        .unwrap_or(0);
                    selected = Some(
                        candidate.components[if matches!(e, Edit::SelectNext) {
                            (i + 1) % n
                        } else {
                            (i + n - 1) % n
                        }]
                        .id,
                    );
                }
                self.selected = selected;
                return Ok(());
            }
            Edit::Field(i) => {
                self.field = ComponentField::ALL[i % ComponentField::ALL.len()];
                return Ok(());
            }
            Edit::Save => {
                save_anatomy(&self.path, &self.spec)?;
                self.status = format!("Saved {}", self.path.display());
                return Ok(());
            }
            Edit::Reload => candidate = load_anatomy(&self.path)?,
            Edit::Seed => candidate = AnatomySpec::from_seed(candidate.body.seed.wrapping_add(1)),
            Edit::Showcase => candidate = AnatomySpec::showcase(candidate.body.seed),
            Edit::AddFoodPouch => {
                candidate.add_food_pouch();
                selected = candidate.components.last().map(|c| c.id);
            }
            Edit::AddFlagellum | Edit::AddSpine | Edit::AddNozzle => {
                let id = candidate.next_id();
                let mut c = Component::flagellum(id, 0.35, (id as f32 * 1.7).rem_euclid(TAU));
                c.shape = match e {
                    Edit::AddSpine => Shape::Spine {
                        length: 0.4,
                        radius: 0.06,
                        taper: 0.92,
                        curvature: 0.,
                    },
                    Edit::AddNozzle => Shape::SecretionNozzle {
                        length: 0.38,
                        radius: 0.09,
                        aperture: 0.6,
                        output: EmissionMetadata {
                            material_channel: "future_conserved_substrate".into(),
                            operational: false,
                        },
                    },
                    _ => c.shape,
                };
                candidate.components.push(c);
                selected = Some(id);
            }
            Edit::Duplicate => {
                let mut c = self
                    .component()
                    .ok_or_else(|| MorphError("select a component first".into()))?
                    .clone();
                c.id = candidate.next_id();
                c.attachment.azimuth = (c.attachment.azimuth + 0.4).rem_euclid(TAU);
                selected = Some(c.id);
                candidate.components.push(c);
            }
            Edit::Remove => candidate.components.retain(|c| Some(c.id) != selected),
            Edit::BodyLength(d) => {
                candidate.body.length = (candidate.body.length + d * 0.1)
                    .clamp((candidate.body.radius * 5.).max(2.), 8.)
            }
            Edit::BodyRadius(d) => {
                candidate.body.radius = (candidate.body.radius + d * 0.01)
                    .clamp(0.12, (candidate.body.length / 5.).min(0.65))
            }
            Edit::Mouth(d) => {
                candidate.body.mouth.aperture =
                    (candidate.body.mouth.aperture + d * 0.1).clamp(0., 1.)
            }
            Edit::Adjust(d) => {
                let c = candidate
                    .components
                    .iter_mut()
                    .find(|c| Some(c.id) == selected)
                    .ok_or_else(|| MorphError("select a component first".into()))?;
                match self.field {
                    ComponentField::Station => {
                        c.attachment.station = (c.attachment.station + d * 0.025).clamp(0.08, 0.85)
                    }
                    ComponentField::Azimuth => {
                        c.attachment.azimuth = (c.attachment.azimuth + d * 0.1).rem_euclid(TAU)
                    }
                    ComponentField::Yaw => {
                        c.orientation[0] = (c.orientation[0] + d * 0.1).clamp(-PI, PI)
                    }
                    ComponentField::Pitch => {
                        c.orientation[1] = (c.orientation[1] + d * 0.1).clamp(-PI, PI)
                    }
                    ComponentField::Capacity => {
                        let Shape::FoodPouch { capacity, .. } = &mut c.shape else {
                            return Err(MorphError("capacity applies to a food pouch".into()));
                        };
                        *capacity = (*capacity as f32 + d * 128.).clamp(64., 8192.) as u64;
                    }
                    ComponentField::Radius if matches!(c.shape, Shape::FoodPouch { .. }) => {
                        if let Shape::FoodPouch { radius, .. } = &mut c.shape {
                            *radius = (*radius + d * 0.01).clamp(0.08, 0.5);
                        }
                    }
                    ComponentField::Length if matches!(c.shape, Shape::FoodPouch { .. }) => {
                        if let Shape::FoodPouch { length, .. } = &mut c.shape {
                            *length = (*length + d * 0.05).clamp(0.3, 2.);
                        }
                    }
                    field => {
                        let (v, step, lo, hi) = match (&mut c.shape, field) {
                            (
                                Shape::Flagellum { length, .. }
                                | Shape::Spine { length, .. }
                                | Shape::SecretionNozzle { length, .. }
                                | Shape::FoodPouch { length, .. },
                                ComponentField::Length,
                            ) => (length, 0.05, 0.15, 3.),
                            (
                                Shape::Flagellum { radius, .. }
                                | Shape::Spine { radius, .. }
                                | Shape::SecretionNozzle { radius, .. }
                                | Shape::FoodPouch { radius, .. },
                                ComponentField::Radius,
                            ) => (radius, 0.005, 0.008, 0.15),
                            (
                                Shape::Flagellum { taper, .. } | Shape::Spine { taper, .. },
                                ComponentField::Taper,
                            ) => (taper, 0.02, 0., 0.98),
                            (
                                Shape::Flagellum { curvature, .. } | Shape::Spine { curvature, .. },
                                ComponentField::Curvature,
                            ) => (curvature, 0.025, -0.4, 0.4),
                            (Shape::Flagellum { amplitude, .. }, ComponentField::Amplitude) => {
                                (amplitude, 0.01, 0., 0.25)
                            }
                            (Shape::Flagellum { frequency, .. }, ComponentField::Frequency) => {
                                (frequency, 0.1, 0., 4.)
                            }
                            (Shape::Flagellum { phase, .. }, ComponentField::Phase) => {
                                (phase, 0.1, -20., 20.)
                            }
                            (Shape::SecretionNozzle { aperture, .. }, ComponentField::Aperture) => {
                                (aperture, 0.025, 0.2, 0.8)
                            }
                            _ => {
                                return Err(MorphError(
                                    "parameter does not apply to selected component".into(),
                                ));
                            }
                        };
                        *v = (*v + d * step).clamp(lo, hi);
                    }
                }
            }
        }
        let compiled = CompiledAnatomy::new(candidate.clone())?;
        // Audit the current deformed witness too before replacing accepted state.
        compiled.pose(self.pose())?.geometry.audit()?;
        if !candidate.components.iter().any(|c| Some(c.id) == selected) {
            selected = candidate.components.first().map(|c| c.id);
        }
        self.spec = candidate;
        self.compiled = compiled;
        self.selected = selected;
        if matches!(e, Edit::Mouth(_)) {
            self.mouth_cycle = false;
        }
        self.last_edit_micros = started.elapsed().as_micros();
        self.status = format!("Accepted {e:?} in {} us", self.last_edit_micros);
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn food_pouch_is_editable_storage_anatomy_not_a_nozzle() {
        let mut editor =
            CompositionSession::new(AnatomySpec::bare(77), PathBuf::from("unused-pouch.json"))
                .unwrap();
        let outputs = editor.geometry().unwrap().surfaces.len();
        editor.edit(Edit::AddFoodPouch).unwrap();
        assert_eq!(editor.spec.food_capacity(), 1024);
        assert_eq!(editor.geometry().unwrap().surfaces.len(), outputs + 1);
        editor
            .edit(Edit::Field(
                ComponentField::ALL
                    .iter()
                    .position(|f| *f == ComponentField::Capacity)
                    .unwrap(),
            ))
            .unwrap();
        editor.edit(Edit::Adjust(1.)).unwrap();
        assert_eq!(editor.spec.food_capacity(), 1152);
        assert_eq!(
            editor.compiled.pose(editor.pose()).unwrap().outputs.len(),
            1
        );
        let json = serde_json::to_string(&editor.spec).unwrap();
        assert_eq!(anatomy_from_json(&json).unwrap(), editor.spec);
    }
    #[test]
    fn individual_edits_deletion_duplication_persistence_and_invalid_reload() {
        let path =
            std::env::temp_dir().join(format!("morph002-editor-{}.json", std::process::id()));
        let mut s = CompositionSession::new(AnatomySpec::showcase(4), path.clone()).unwrap();
        let neighbor = s.spec.components[1].clone();
        let original = s.component().unwrap().clone();
        for i in 0..11 {
            s.edit(Edit::Field(i)).unwrap();
            s.edit(Edit::Adjust(1.)).unwrap();
        }
        assert_ne!(*s.component().unwrap(), original);
        assert_eq!(s.spec.components[1], neighbor);
        s.edit(Edit::Duplicate).unwrap();
        let duplicate = s.selected;
        assert_ne!(duplicate, Some(original.id));
        s.edit(Edit::Remove).unwrap();
        assert!(!s.spec.components.iter().any(|c| Some(c.id) == duplicate));
        for _ in 0..12 {
            s.edit(Edit::AddFlagellum).unwrap();
            s.edit(Edit::Duplicate).unwrap();
            s.edit(Edit::Remove).unwrap();
            s.geometry().unwrap().audit().unwrap();
        }
        s.edit(Edit::AddSpine).unwrap();
        s.edit(Edit::AddNozzle).unwrap();
        s.edit(Edit::Field(11)).unwrap();
        s.edit(Edit::Adjust(-1.)).unwrap();
        s.edit(Edit::Save).unwrap();
        let spec = s.spec.clone();
        let digest = s.geometry().unwrap().digest();
        s.edit(Edit::Seed).unwrap();
        s.edit(Edit::Reload).unwrap();
        assert_eq!(spec, s.spec);
        assert_eq!(digest, s.geometry().unwrap().digest());
        std::fs::write(&path, "{\"version\":2,\"components\":[]}").unwrap();
        assert!(s.edit(Edit::Reload).is_err());
        assert_eq!(spec, s.spec);
        assert_eq!(digest, s.geometry().unwrap().digest());
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn refused_geometry_edit_does_not_replace_accepted_state() {
        let mut s = CompositionSession::new(AnatomySpec::showcase(9), PathBuf::from("unused.json"))
            .unwrap();
        s.edit(Edit::AddNozzle).unwrap();
        s.edit(Edit::Field(0)).unwrap();
        let spec = s.spec.clone();
        let selected = s.selected;
        // Minimum nozzle length at this thickness violates aspect envelope.
        assert!(s.edit(Edit::Adjust(-100.)).is_err());
        assert_eq!(spec, s.spec);
        assert_eq!(selected, s.selected);
    }
}
