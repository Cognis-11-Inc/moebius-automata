//! Pure presentation navigation, using the existing metric Position; no world mutation rights.
use bevy::{math::DVec3, prelude::*};
use moebius_morph::{emission::SCALE, runtime::Position};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigationMode {
    Flight,
    Orbit,
}
pub struct Navigation {
    pub position: Position,
    pub rotation: Quat,
    pub velocity: DVec3,
    pub fractional: DVec3,
    pub mode: NavigationMode,
    pub pivot: Position,
    pub distance: f64,
    pub speed: f64,
    pub sensitivity: f32,
}
impl Default for Navigation {
    fn default() -> Self {
        Self::new()
    }
}
impl Navigation {
    pub fn new() -> Self {
        let transform = Transform::from_xyz(4., 2., 5.).looking_at(Vec3::ZERO, Vec3::Y);
        Self {
            position: Position::from_absolute([4 * SCALE, 2 * SCALE, 5 * SCALE]).unwrap(),
            rotation: transform.rotation,
            velocity: DVec3::ZERO,
            fractional: DVec3::ZERO,
            mode: NavigationMode::Flight,
            pivot: Position::from_absolute([0; 3]).unwrap(),
            distance: 5.,
            speed: 3.,
            sensitivity: 0.003,
        }
    }
    fn translate(&mut self, delta: DVec3) {
        self.fractional += delta * SCALE as f64;
        let shift = self.fractional.round();
        self.fractional -= shift;
        self.position = self
            .position
            .offset(shift.to_array().map(|v| v as i64))
            .expect("camera within supported metric envelope");
    }
    pub fn fly(&mut self, local: Vec3, seconds: f64, multiplier: f64, roll: f32) {
        if self.mode != NavigationMode::Flight {
            return;
        }
        let target =
            (self.rotation * local.clamp_length_max(1.)).as_dvec3() * self.speed * multiplier;
        let rate = 8.;
        let decay = (-rate * seconds).exp();
        let old = self.velocity;
        let displacement = target * seconds + (old - target) * (1. - decay) / rate;
        self.velocity = target + (old - target) * decay;
        self.translate(displacement);
        self.rotation = (self.rotation * Quat::from_rotation_z(roll * seconds as f32)).normalize();
    }
    pub fn look(&mut self, delta: Vec2) {
        let old = self.rotation;
        self.rotation = (self.rotation
            * Quat::from_rotation_y(-delta.x * self.sensitivity)
            * Quat::from_rotation_x(-delta.y * self.sensitivity))
        .normalize();
        if self.mode == NavigationMode::Orbit {
            let offset = Vec3::from_array(
                self.position
                    .relative(self.pivot)
                    .unwrap()
                    .map(|v| v as f32 / SCALE as f32),
            );
            let moved = (self.rotation * old.conjugate()) * offset;
            self.position = self
                .pivot
                .offset(
                    (moved.as_dvec3() * SCALE as f64)
                        .round()
                        .to_array()
                        .map(|v| v as i64),
                )
                .unwrap();
        }
    }
    pub fn orbit_target(&mut self, pivot: Position) {
        self.pivot = pivot;
        let offset = self.position.relative(pivot).unwrap();
        self.distance = offset
            .iter()
            .map(|v| (*v as f64 / SCALE as f64).powi(2))
            .sum::<f64>()
            .sqrt()
            .max(0.1);
    }
    fn orbit_position(&mut self) {
        let offset = (self.rotation * Vec3::Z).as_dvec3() * self.distance;
        self.position = self
            .pivot
            .offset((offset * SCALE as f64).round().to_array().map(|v| v as i64))
            .unwrap();
        self.fractional = DVec3::ZERO;
    }
    pub fn focus(&mut self, target: Position, size: f64) {
        self.pivot = target;
        self.distance = (size * 2.5).clamp(0.3, 10000.);
        self.orbit_position();
        self.velocity = DVec3::ZERO;
    }
    pub fn switch(&mut self) {
        self.velocity = DVec3::ZERO;
        self.mode = match self.mode {
            NavigationMode::Flight => {
                self.pivot = self
                    .position
                    .offset(
                        ((self.rotation * Vec3::NEG_Z).as_dvec3() * self.distance * SCALE as f64)
                            .round()
                            .to_array()
                            .map(|v| v as i64),
                    )
                    .unwrap();
                NavigationMode::Orbit
            }
            NavigationMode::Orbit => NavigationMode::Flight,
        };
    }
    pub fn pan(&mut self, delta: Vec2) {
        let d =
            (self.rotation * Vec3::new(-delta.x, delta.y, 0.)).as_dvec3() * self.distance * 0.002;
        self.translate(d);
        self.pivot = self
            .pivot
            .offset((d * SCALE as f64).round().to_array().map(|v| v as i64))
            .unwrap();
    }
    pub fn wheel(&mut self, delta: f32) {
        if self.mode == NavigationMode::Flight {
            self.speed = (self.speed * (delta as f64 * 0.15).exp()).clamp(0.02, 100.);
        } else {
            let before = self.distance;
            self.distance = (self.distance * (-delta as f64 * 0.1).exp()).clamp(0.1, 10000.);
            let offset = self
                .position
                .relative(self.pivot)
                .unwrap()
                .map(|v| (v as f64 * self.distance / before).round() as i64);
            self.position = self.pivot.offset(offset).unwrap();
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn camera_displacement_is_frame_rate_independent_and_decelerates() {
        let mut a = Navigation::new();
        let mut b = Navigation::new();
        for _ in 0..30 {
            a.fly(Vec3::NEG_Z, 1. / 30., 1., 0.);
        }
        for _ in 0..120 {
            b.fly(Vec3::NEG_Z, 1. / 120., 1., 0.);
        }
        let d = a.position.relative(b.position).unwrap();
        assert!(d.iter().all(|v| v.abs() <= 2), "{d:?}");
        let before = a.velocity.length();
        a.fly(Vec3::ZERO, 1., 1., 0.);
        assert!(a.velocity.length() < before * 0.001);
    }
    #[test]
    fn modes_focus_pivot_roll_and_speed_keep_stable_orientation() {
        let mut n = Navigation::new();
        let r = n.rotation;
        let p = n.position;
        n.switch();
        assert_eq!(n.position, p);
        assert_eq!(n.rotation, r);
        n.look(Vec2::new(60., -20.));
        n.pan(Vec2::new(3., 4.));
        let pivot = n.pivot;
        let far = n.distance;
        n.wheel(2.);
        assert!(n.distance < far);
        assert_eq!(pivot, n.pivot);
        let r = n.rotation;
        let p = n.position;
        n.switch();
        assert_eq!(p, n.position);
        assert_eq!(r, n.rotation);
        n.focus(
            Position::from_absolute([100 * SCALE, 3 * SCALE, 0]).unwrap(),
            2.,
        );
        assert_eq!(r, n.rotation);
        n.fly(Vec3::Y, 1., 4., 0.5);
        assert_ne!(n.rotation, r);
        let speed = n.speed;
        n.wheel(1.);
        assert!(n.speed > speed);
    }
}
