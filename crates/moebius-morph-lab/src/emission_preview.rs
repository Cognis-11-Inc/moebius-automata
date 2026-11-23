//! Interactive observer of the secretion book.
//!
//! The preview clock is a fixed timestep. It is not the wrapping anatomical
//! animation phase. Trail lines drawn by the laboratory are visualization.

use moebius_morph::emission::{
    AuthoredTrajectory, EmissionStatus, Environment, MaterialBook, RigidPose, SCALE,
};
use moebius_morph::{AnatomySpec, CompiledAnatomy, MorphError, Pose};
use std::path::PathBuf;

pub const PHASE_TICK_CAP: u64 = 6_399_936;
pub const WEAK_ACCELERATION: [i64; 3] = [2_000, 0, 0];
pub const PREVIEW_FLOW: [i64; 3] = [0, 0, 30_000];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreviewCommand {
    ToggleEmission,
    ToggleClock,
    Rate(i64),
    StageReservoir { numerator: u64, denominator: u64 },
    Drag(i32),
    ToggleAcceleration,
    ToggleFlow,
    ToggleTrajectory,
    Reset,
    Save,
    Load,
}

pub struct PreviewControls {
    pub book: MaterialBook,
    pub staged_initial: u64,
    pub trajectory_on: bool,
    pub trajectory: AuthoredTrajectory,
    pub clock: bool,
    pub clock_armed: bool,
    pub accumulator: f32,
    pub dt: f32,
    pub body_rigid: RigidPose,
    pub note: String,
    pub path: PathBuf,
}

impl PreviewControls {
    pub fn from_spec(spec: &AnatomySpec, path: PathBuf) -> Result<Self, MorphError> {
        let mut book = MaterialBook::new(
            spec.body.seed,
            spec.body.seed,
            10_000,
            Environment::suspended(),
            4096,
        )?;
        book.launch_speed = 60_000;
        book.sync_nozzles(spec);
        Ok(Self {
            book,
            staged_initial: 10_000,
            trajectory_on: false,
            trajectory: AuthoredTrajectory {
                velocity_per_tick: [30_000, 8_000, 12_000],
                ..AuthoredTrajectory::stationary()
            },
            clock: false,
            clock_armed: false,
            accumulator: 0.,
            dt: 1. / 60.,
            body_rigid: RigidPose::identity(),
            note: "Material clock idle. E emits. C runs the clock without emission.".into(),
            path,
        })
    }

    /// Authored pose parameter. It stops at 99999 and does not wrap.
    pub fn phase_for_tick(tick: u64) -> f32 {
        (tick.min(PHASE_TICK_CAP) as f32) / 64.
    }

    pub fn command(
        &mut self,
        spec: &AnatomySpec,
        command: PreviewCommand,
    ) -> Result<(), MorphError> {
        match command {
            PreviewCommand::ToggleEmission => {
                self.book.emission_enabled = !self.book.emission_enabled;
                if self.book.emission_enabled {
                    self.clock = true;
                    self.clock_armed = true;
                    self.note = "Emission on. The material clock owns pose phase.".into();
                } else {
                    self.note = "Emission off. Admitted material still follows the clock.".into();
                }
            }
            PreviewCommand::ToggleClock => {
                self.clock = !self.clock;
                if self.clock {
                    self.clock_armed = true;
                }
                self.note = if self.clock {
                    "Material clock running.".into()
                } else {
                    "Material clock paused.".into()
                };
            }
            PreviewCommand::Rate(delta) => {
                let next = (self.book.rate as i64 + delta).clamp(1, 10_000) as u64;
                self.book.rate = next;
                self.note = format!("Secretion rate {next} quanta per admission.");
            }
            PreviewCommand::StageReservoir {
                numerator,
                denominator,
            } => {
                let scaled = self.staged_initial.saturating_mul(numerator) / denominator.max(1);
                self.staged_initial = scaled.clamp(1, 1_000_000_000);
                self.note = format!(
                    "Reset reservoir staged at {}. X applies it.",
                    self.staged_initial
                );
            }
            PreviewCommand::Drag(delta) => {
                let next = (i32::from(self.book.environment.drag_per_mille) + delta).clamp(0, 1000);
                self.book.environment.drag_per_mille = next as u16;
                self.note = format!("Ambient drag {next} per mille. Not an atmosphere.");
            }
            PreviewCommand::ToggleAcceleration => {
                let on = self.book.environment.acceleration == [0, 0, 0];
                self.book.environment.acceleration = if on { WEAK_ACCELERATION } else { [0, 0, 0] };
                self.note = if on {
                    "Weak +X acceleration on. This is not a floor and not gravity.".into()
                } else {
                    "Directional acceleration off.".into()
                };
            }
            PreviewCommand::ToggleFlow => {
                let on = self.book.environment.medium_flow == [0, 0, 0];
                self.book.environment.medium_flow = if on { PREVIEW_FLOW } else { [0, 0, 0] };
                self.note = if on {
                    "Uniform +Z medium flow on.".into()
                } else {
                    "Medium flow off.".into()
                };
            }
            PreviewCommand::ToggleTrajectory => {
                self.trajectory_on = !self.trajectory_on;
                self.note = if self.trajectory_on {
                    "Authored trajectory on. It is a test input, not locomotion.".into()
                } else {
                    "Authored trajectory off.".into()
                };
                self.refresh_rigid()?;
            }
            PreviewCommand::Reset => {
                let environment = self.book.environment;
                let rate = self.book.rate;
                let launch = self.book.launch_speed;
                self.book = MaterialBook::new(
                    spec.body.seed,
                    spec.body.seed,
                    self.staged_initial,
                    environment,
                    4096,
                )?;
                self.book.rate = rate;
                self.book.launch_speed = launch;
                self.book.sync_nozzles(spec);
                self.clock = false;
                self.clock_armed = false;
                self.accumulator = 0.;
                self.body_rigid = RigidPose::identity();
                self.note =
                    "Experiment reset. Emission off, tick 0, staged reservoir loaded.".into();
            }
            PreviewCommand::Save => {
                self.book.save(&self.path)?;
                self.note = format!("Material book saved {}", self.path.display());
            }
            PreviewCommand::Load => {
                let loaded = MaterialBook::load(&self.path)?;
                self.staged_initial = loaded.initial;
                self.book = loaded;
                self.clock = false;
                self.accumulator = 0.;
                self.refresh_rigid()?;
                self.note = format!("Material book loaded {}", self.path.display());
            }
        }
        Ok(())
    }

    pub fn begin_tick(&mut self) -> Result<(), MorphError> {
        self.book.step()?;
        self.refresh_rigid()
    }

    pub fn emit_current(
        &mut self,
        anatomy: &CompiledAnatomy,
        pose: Pose,
        selected: Option<u64>,
    ) -> Result<EmissionStatus, MorphError> {
        self.book.sync_nozzles(anatomy.spec());
        self.refresh_rigid()?;
        let Some(nozzle_id) = choose_nozzle(&self.book, selected) else {
            self.book.last_status = EmissionStatus::UnknownNozzle { nozzle_id: 0 };
            return Ok(self.book.last_status);
        };
        let posed = anatomy.pose(pose)?;
        let frame = posed
            .outputs
            .iter()
            .find(|frame| frame.component_id == nozzle_id)
            .ok_or_else(|| MorphError("selected nozzle has no output frame".into()))?;
        let placed = moebius_morph::emission::place_output(frame, self.body_rigid)?;
        let status = self.book.try_emit(moebius_morph::emission::EmitRequest {
            nozzle_id,
            origin: placed.origin,
            direction: placed.direction,
            quantity: self.book.rate,
            launch_speed: self.book.launch_speed,
            channel: &frame.metadata.material_channel,
        });
        Ok(status)
    }

    pub fn refresh_rigid(&mut self) -> Result<(), MorphError> {
        self.body_rigid = if self.trajectory_on {
            self.trajectory.at(self.book.tick)?
        } else {
            RigidPose::identity()
        };
        Ok(())
    }

    pub fn ledger_line(&self) -> String {
        match self.book.ledger() {
            Ok(ledger) => format!(
                "tick {} | reservoir {} | emitted {} | in world {} | sinks {} | residual {} | drag {} | accel {:?} | {}",
                self.book.tick,
                ledger.reservoir,
                ledger.initial - ledger.reservoir,
                ledger.in_world,
                ledger.sinks,
                ledger.residual,
                self.book.environment.drag_per_mille,
                self.book.environment.acceleration,
                self.note
            ),
            Err(error) => format!("LEDGER REFUSED: {error}"),
        }
    }
}

fn choose_nozzle(book: &MaterialBook, selected: Option<u64>) -> Option<u64> {
    if let Some(id) = selected
        && book.registered.contains(&id)
    {
        return Some(id);
    }
    book.registered.iter().next().copied()
}

pub fn world_point(micro: [i64; 3]) -> [f32; 3] {
    micro.map(|value| value as f32 / SCALE as f32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use moebius_morph::Shape;

    fn preview() -> (PreviewControls, CompiledAnatomy, AnatomySpec) {
        let spec = AnatomySpec::showcase(20261123);
        let compiled = CompiledAnatomy::new(spec.clone()).unwrap();
        let controls =
            PreviewControls::from_spec(&spec, PathBuf::from("/tmp/unused-book.json")).unwrap();
        (controls, compiled, spec)
    }

    #[test]
    fn phase_schedule_stops_without_wrapping() {
        assert_eq!(PreviewControls::phase_for_tick(0), 0.);
        assert_eq!(PreviewControls::phase_for_tick(64), 1.);
        assert_eq!(PreviewControls::phase_for_tick(PHASE_TICK_CAP), 99999.);
        assert_eq!(PreviewControls::phase_for_tick(u64::MAX), 99999.);
    }

    #[test]
    fn disabled_clock_does_not_spend_the_reservoir() {
        let (mut controls, compiled, spec) = preview();
        assert!(!controls.book.emission_enabled);
        let initial = controls.book.reservoir;
        controls
            .command(&spec, PreviewCommand::ToggleClock)
            .unwrap();
        controls.begin_tick().unwrap();
        let pose = pose_at(&spec, controls.book.tick);
        let status = controls.emit_current(&compiled, pose, None).unwrap();
        assert_eq!(status, EmissionStatus::Disabled);
        assert_eq!(controls.book.reservoir, initial);
        assert!(controls.book.parcels.is_empty());
        assert_eq!(controls.book.residual().unwrap(), 0);
    }

    #[test]
    fn emitted_material_stays_when_the_authored_body_moves() {
        let (mut controls, compiled, spec) = preview();
        controls.book.launch_speed = 0;
        controls
            .command(&spec, PreviewCommand::ToggleEmission)
            .unwrap();
        controls.begin_tick().unwrap();
        let pose = pose_at(&spec, controls.book.tick);
        let status = controls.emit_current(&compiled, pose, None).unwrap();
        assert!(status.admitted());
        assert!(
            controls
                .book
                .runtime_operational(controls.book.parcels[0].nozzle_id)
        );
        let birth = controls.book.parcels[0].position;
        controls
            .command(&spec, PreviewCommand::ToggleTrajectory)
            .unwrap();
        controls.begin_tick().unwrap();
        let pose = pose_at(&spec, controls.book.tick);
        controls.emit_current(&compiled, pose, None).unwrap();
        assert_eq!(controls.book.parcels[0].position, birth);
        assert_ne!(controls.book.parcels[1].position, birth);
        assert_eq!(controls.book.residual().unwrap(), 0);
        let flag = spec.components.iter().any(|component| {
            matches!(
                component.shape,
                Shape::SecretionNozzle {
                    output: moebius_morph::EmissionMetadata {
                        operational: true,
                        ..
                    },
                    ..
                }
            )
        });
        assert!(!flag);
    }

    #[test]
    fn reservoir_request_beyond_the_balance_is_rejected() {
        let (mut controls, compiled, spec) = preview();
        controls
            .command(
                &spec,
                PreviewCommand::StageReservoir {
                    numerator: 5,
                    denominator: 10_000,
                },
            )
            .unwrap();
        assert_eq!(controls.staged_initial, 5);
        controls.command(&spec, PreviewCommand::Reset).unwrap();
        controls.book.rate = 3;
        controls.book.launch_speed = 0;
        controls
            .command(&spec, PreviewCommand::ToggleEmission)
            .unwrap();
        controls.begin_tick().unwrap();
        let pose = pose_at(&spec, controls.book.tick);
        assert!(
            controls
                .emit_current(&compiled, pose, None)
                .unwrap()
                .admitted()
        );
        controls.begin_tick().unwrap();
        let pose = pose_at(&spec, controls.book.tick);
        let status = controls.emit_current(&compiled, pose, None).unwrap();
        assert!(matches!(
            status,
            EmissionStatus::ReservoirExhausted {
                requested: 3,
                available: 2
            }
        ));
        assert_eq!(controls.book.reservoir, 2);
        assert_eq!(controls.book.in_world().unwrap(), 3);
        assert_eq!(controls.book.residual().unwrap(), 0);
    }

    #[test]
    fn save_and_load_keep_the_same_quanta() {
        let (mut controls, compiled, spec) = preview();
        let path =
            std::env::temp_dir().join(format!("morph003-preview-{}.json", std::process::id()));
        controls.path = path.clone();
        controls
            .command(&spec, PreviewCommand::ToggleEmission)
            .unwrap();
        controls.begin_tick().unwrap();
        let pose = pose_at(&spec, controls.book.tick);
        controls.emit_current(&compiled, pose, None).unwrap();
        controls.command(&spec, PreviewCommand::Save).unwrap();
        let hash = controls.book.state_hash().unwrap();
        controls.command(&spec, PreviewCommand::Reset).unwrap();
        assert!(controls.book.parcels.is_empty());
        controls.command(&spec, PreviewCommand::Load).unwrap();
        assert_eq!(controls.book.state_hash().unwrap(), hash);
        assert!(!controls.clock);
        std::fs::remove_file(path).unwrap();
    }

    fn pose_at(spec: &AnatomySpec, tick: u64) -> Pose {
        let mut pose = Pose::rest(&spec.body);
        pose.phase = PreviewControls::phase_for_tick(tick);
        pose.bend = 0.8;
        pose.flagella = 1.;
        pose
    }
}
