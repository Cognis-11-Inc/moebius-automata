//! A bounded torque-driven kinematic rig and explicit reaction-mass mechanics.
//! Effective inertia is a model coefficient, not accounted organism biomass.
//! This is not a full articulated rigid-body or fluid solver.
use super::{
    Q,
    schema::{Anatomy, Axis},
};
use crate::{Dims, Error, Site};
use serde::{Deserialize, Serialize};
pub const INERTIA: i64 = 16;
const TRIG: i64 = 4096;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Body {
    pub angles: Vec<i32>,
    pub rates: Vec<i32>,
    pub efforts: Vec<i32>,
    pub loads: Vec<i32>,
    pub orientation: [i32; 3],
    pub angular_velocity: [i32; 3],
    pub momentum: [i64; 3],
    pub velocity: [i32; 3],
    pub acceleration: [i32; 3],
    pub centroid: [i32; 3],
}
impl Body {
    pub fn new(anatomy: &Anatomy) -> Self {
        let n = anatomy.axes.len();
        let mut body = Self {
            angles: vec![0; n],
            rates: vec![0; n],
            efforts: vec![0; n],
            loads: vec![0; n],
            orientation: [0; 3],
            angular_velocity: [0; 3],
            momentum: [0; 3],
            velocity: [0; 3],
            acceleration: [0; 3],
            centroid: [0; 3],
        };
        body.centroid = body.local_geometry(anatomy).1;
        body
    }
    pub fn step(
        &mut self,
        anatomy: &Anatomy,
        commands: &[i32],
        strengths: &[i32],
        drag: u32,
        contact: &[i32],
        ligament_stiffness: i32,
        ligament_rest: &[i32],
    ) -> [i64; 3] {
        let previous = self.velocity;
        let mut relative = [0i64; 3];
        for (index, binding) in anatomy.axes.iter().enumerate() {
            let commanded = (i64::from(commands[index]) * i64::from(strengths[index])
                / i64::from(Q))
            .clamp(-1024, 1024);
            // The ligament pulls toward a heritable rest angle. A nerve command
            // can hold the joint off that curve; it does not slam the stop.
            let rest = ligament_rest.get(index).copied().unwrap_or(0);
            let stiffness = i64::from(ligament_stiffness.clamp(2, 24));
            let spring = -i64::from(self.angles[index] - rest) / stiffness;
            let effort = (commanded + spring).clamp(-1024, 1024) as i32;
            self.efforts[index] = effort;
            self.loads[index] = contact.get(index).copied().unwrap_or(0).clamp(0, Q);
            let mut rate = (self.rates[index] * 3 / 4 + effort / 32).clamp(-16, 16);
            // Local unilateral contact opposes closing jaws. Spinal load is
            // exposed but complete capsule collision is a documented prerequisite.
            if binding.axis == Axis::Jaw && self.loads[index] > 0 && rate < 0 {
                rate = rate.max(-8);
            }
            let angle = (self.angles[index] + rate).clamp(binding.min, binding.max);
            if angle == binding.min || angle == binding.max {
                rate /= 4;
            }
            self.angles[index] = angle;
            self.rates[index] = rate;
            let axis = match binding.axis {
                Axis::Bend | Axis::Jaw => 2,
                Axis::Twist => 1,
            };
            let sign = if binding.axis == Axis::Jaw && binding.joint % 2 == 1 {
                -1
            } else {
                1
            };
            relative[axis] += i64::from(rate) * sign;
        }
        // Approximate internal counter-rotation; it never supplies COM thrust.
        for (axis, relative) in relative.into_iter().enumerate() {
            self.angular_velocity[axis] = (-relative / INERTIA) as i32;
            self.orientation[axis] = (self.orientation[axis] + self.angular_velocity[axis] + 3217)
                .rem_euclid(6434)
                - 3217;
        }
        let mut absorbed = [0; 3];
        for (axis, item) in absorbed.iter_mut().enumerate() {
            *item = self.momentum[axis] * i64::from(drag) / 1024;
            self.momentum[axis] -= *item;
            self.velocity[axis] = (self.momentum[axis] / INERTIA) as i32;
            self.acceleration[axis] = self.velocity[axis] - previous[axis];
        }
        // Anisotropic drag on the bending spine. Vacuum (drag 0) exchanges
        // nothing, so internal motion still cannot shove the center of mass.
        // The impulse put on the body is removed from the returned environment
        // transfer, and the caller adds that transfer to the environment ledger.
        if drag > 0 {
            let thrust = shape_impulse(self, anatomy, drag);
            for (axis, impulse) in thrust.into_iter().enumerate() {
                let next = (self.momentum[axis] + impulse).clamp(-4096, 4096);
                let applied = next - self.momentum[axis];
                self.momentum[axis] = next;
                absorbed[axis] -= applied;
                let velocity = (self.momentum[axis] / INERTIA) as i32;
                self.acceleration[axis] += velocity - self.velocity[axis];
                self.velocity[axis] = velocity;
            }
        }
        self.centroid = self.local_geometry(anatomy).1;
        absorbed
    }
    pub fn local_geometry(&self, anatomy: &Anatomy) -> (Vec<[i32; 3]>, [i32; 3]) {
        let mut matrix = [TRIG, 0, 0, 0, TRIG, 0, 0, 0, TRIG];
        let mut position = [0i64, -1720, 0];
        let step = 3277 / (anatomy.spine - 1) as i64;
        let mut points = Vec::new();
        for joint in 0..anatomy.spine {
            if joint > 0 {
                let delta = apply(matrix, [0, step, 0]);
                for axis in 0..3 {
                    position[axis] += delta[axis];
                }
            }
            points.push(position.map(|v| v as i32));
            let mut angles = [0; 3];
            for (index, binding) in anatomy
                .axes
                .iter()
                .enumerate()
                .filter(|(_, b)| b.joint == joint)
            {
                match binding.axis {
                    Axis::Bend => angles[2] = self.angles[index],
                    Axis::Twist => angles[1] = self.angles[index],
                    Axis::Jaw => {}
                }
            }
            matrix = multiply(matrix, rotation(angles));
        }
        let center = std::array::from_fn(|axis| {
            points.iter().map(|p| i64::from(p[axis])).sum::<i64>() / points.len() as i64
        });
        for point in &mut points {
            for axis in 0..3 {
                point[axis] -= center[axis] as i32;
            }
        }
        (points, center.map(|v| v as i32))
    }
    pub fn head_site(&self, anatomy: &Anatomy, site: Site, offset: [i32; 3], dims: Dims) -> Site {
        let (points, _) = self.local_geometry(anatomy);
        let point = points[anatomy.spine - 3].map(i64::from);
        let point = rotate(point, self.orientation);
        let base = [site.x, site.y, site.z];
        let extents = [dims.width, dims.height, dims.depth];
        let c: [u32; 3] = std::array::from_fn(|axis| {
            ((i64::from(base[axis]) * 1024 + i64::from(offset[axis]) + point[axis] + 512)
                .div_euclid(1024))
            .rem_euclid(i64::from(extents[axis])) as u32
        });
        Site {
            x: c[0],
            y: c[1],
            z: c[2],
        }
    }
    pub fn impulse(&mut self, impulse: [i64; 3]) {
        for (axis, value) in impulse.into_iter().enumerate() {
            let previous = self.velocity[axis];
            self.momentum[axis] += value;
            self.velocity[axis] = (self.momentum[axis] / INERTIA) as i32;
            self.acceleration[axis] += self.velocity[axis] - previous;
        }
    }
    pub fn can_impulse(&self, impulse: [i64; 3]) -> bool {
        self.momentum
            .iter()
            .zip(impulse)
            .all(|(p, i)| (p + i).unsigned_abs() <= 4096)
    }
    pub fn validate(&self, anatomy: &Anatomy) -> Result<(), Error> {
        let n = anatomy.axes.len();
        if self.velocity != self.momentum.map(|p| (p / INERTIA) as i32)
            || self.acceleration.iter().any(|v| v.unsigned_abs() > 512)
            || self.angular_velocity.iter().any(|v| v.unsigned_abs() > 128)
            || self.angles.len() != n
            || self.rates.len() != n
            || self.efforts.len() != n
            || self.loads.len() != n
            || self
                .angles
                .iter()
                .zip(&anatomy.axes)
                .any(|(a, b)| *a < b.min || *a > b.max)
            || self.rates.iter().any(|x| x.unsigned_abs() > 64)
            || self.efforts.iter().any(|x| x.unsigned_abs() > 2048)
            || self.loads.iter().any(|x| !(0..=Q).contains(x))
            || self.momentum.iter().any(|p| p.unsigned_abs() > 4096)
            || self.orientation.iter().any(|x| x.unsigned_abs() > 3217)
            || self.velocity.iter().any(|x| x.unsigned_abs() > 256)
        {
            return Err(Error::Invariant("invalid torque/body state".into()));
        }
        Ok(())
    }
}
/// Resistive drag on spine deformation. Normal drag is stronger than tangent
/// drag, so a traveling bend can leave a net impulse. A held pose does not.
/// This is a coarse authored coupling, not a Stokes solution.
fn shape_impulse(body: &Body, anatomy: &Anatomy, drag: u32) -> [i64; 3] {
    if anatomy.spine < 2 {
        return [0; 3];
    }
    let (points, _) = body.local_geometry(anatomy);
    let mut velocity = vec![[0i64; 3]; points.len()];
    for (index, binding) in anatomy.axes.iter().enumerate() {
        let rate = i64::from(body.rates[index]);
        if rate == 0 || binding.axis == Axis::Jaw {
            continue;
        }
        let mut probe = body.clone();
        let bumped = (body.angles[index] + 64).clamp(binding.min, binding.max);
        let delta = i64::from(bumped - body.angles[index]);
        if delta == 0 {
            continue;
        }
        probe.angles[index] = bumped;
        let (other, _) = probe.local_geometry(anatomy);
        for (slot, (from, to)) in velocity.iter_mut().zip(points.iter().zip(other.iter())) {
            for axis in 0..3 {
                slot[axis] += i64::from(to[axis] - from[axis]) * rate / delta;
            }
        }
    }
    for slot in &mut velocity {
        *slot = rotate(*slot, body.orientation);
    }
    let mut force = [0i64; 3];
    for index in 0..points.len().saturating_sub(1) {
        let tangent = rotate(
            std::array::from_fn(|axis| {
                i64::from(points[index + 1][axis]) - i64::from(points[index][axis])
            }),
            body.orientation,
        );
        let span = tangent
            .iter()
            .map(|value| value * value)
            .sum::<i64>()
            .max(1);
        let mut speed = [0i64; 3];
        for axis in 0..3 {
            speed[axis] = (velocity[index][axis] + velocity[index + 1][axis]) / 2;
        }
        let along = speed
            .iter()
            .zip(tangent)
            .map(|(value, axis)| value * axis)
            .sum::<i64>();
        let normal_drag = i64::from(drag.max(1));
        let tangent_drag = normal_drag / 4;
        for axis in 0..3 {
            let tangent_part = along * tangent[axis] / span;
            let normal_part = speed[axis] - tangent_part;
            force[axis] += (-tangent_drag * tangent_part - normal_drag * normal_part) / 128;
        }
    }
    force
}

fn sin_cos(angle: i32) -> (i64, i64) {
    let mut a = (i64::from(angle) * 4 + 12868).rem_euclid(25736) - 12868;
    let mut sign = 1;
    if a > 6434 {
        a = 12868 - a;
        sign = -1;
    } else if a < -6434 {
        a = -12868 - a;
        sign = -1;
    }
    let x2 = a * a / TRIG;
    let x3 = x2 * a / TRIG;
    let x4 = x2 * x2 / TRIG;
    let x5 = x4 * a / TRIG;
    let x6 = x4 * x2 / TRIG;
    (
        a - x3 / 6 + x5 / 120,
        sign * (TRIG - x2 / 2 + x4 / 24 - x6 / 720),
    )
}
fn rotation(angles: [i32; 3]) -> [i64; 9] {
    let (sx, cx) = sin_cos(angles[0]);
    let (sy, cy) = sin_cos(angles[1]);
    let (sz, cz) = sin_cos(angles[2]);
    multiply(
        multiply(
            [cz, -sz, 0, sz, cz, 0, 0, 0, TRIG],
            [cy, 0, sy, 0, TRIG, 0, -sy, 0, cy],
        ),
        [TRIG, 0, 0, 0, cx, -sx, 0, sx, cx],
    )
}
fn multiply(a: [i64; 9], b: [i64; 9]) -> [i64; 9] {
    std::array::from_fn(|index| {
        let row = index / 3;
        let column = index % 3;
        (0..3)
            .map(|k| a[row * 3 + k] * b[k * 3 + column])
            .sum::<i64>()
            / TRIG
    })
}
fn apply(matrix: [i64; 9], v: [i64; 3]) -> [i64; 3] {
    std::array::from_fn(|axis| (0..3).map(|i| matrix[axis * 3 + i] * v[i]).sum::<i64>() / TRIG)
}
pub fn rotate(v: [i64; 3], angles: [i32; 3]) -> [i64; 3] {
    apply(rotation(angles), v)
}
pub fn local(v: [i32; 3], angles: [i32; 3]) -> [i32; 3] {
    let m = rotation(angles);
    let transpose = [m[0], m[3], m[6], m[1], m[4], m[7], m[2], m[5], m[8]];
    apply(transpose, v.map(i64::from)).map(|x| x as i32)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Jet {
    pub owner: u64,
    pub position: [i64; 3],
    pub momentum: [i64; 3],
    pub life: u8,
}
impl Jet {
    pub fn site(&self, dims: Dims) -> Site {
        let extents = [dims.width, dims.height, dims.depth];
        let c: [u32; 3] = std::array::from_fn(|a| {
            ((self.position[a] + 512).div_euclid(1024)).rem_euclid(i64::from(extents[a])) as u32
        });
        Site {
            x: c[0],
            y: c[1],
            z: c[2],
        }
    }
    pub fn advance(&mut self, dims: Dims) {
        let extents = [dims.width, dims.height, dims.depth];
        for (axis, extent) in extents.into_iter().enumerate() {
            self.position[axis] =
                (self.position[axis] + self.momentum[axis]).rem_euclid(i64::from(extent) * 1024);
        }
        self.life = self.life.saturating_sub(1);
    }
}
