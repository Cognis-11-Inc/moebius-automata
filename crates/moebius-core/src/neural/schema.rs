use crate::Error;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Axis {
    Bend,
    Twist,
    Jaw,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub joint: usize,
    pub axis: Axis,
    pub min: i32,
    pub max: i32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Anatomy {
    pub version: u32,
    pub spine: usize,
    pub jaws: usize,
    pub axes: Vec<Binding>,
}
impl Anatomy {
    pub fn reference() -> Self {
        Self::chain(12, 2).expect("reference anatomy")
    }
    pub fn chain(spine: usize, jaws: usize) -> Result<Self, Error> {
        if !(3..=24).contains(&spine) || jaws > 2 {
            return Err(Error::Config("neural anatomy limits".into()));
        }
        let mut axes = Vec::new();
        for joint in 0..spine {
            axes.push(Binding {
                joint,
                axis: Axis::Bend,
                min: -768,
                max: 768,
            });
            axes.push(Binding {
                joint,
                axis: Axis::Twist,
                min: -512,
                max: 512,
            });
        }
        for joint in spine..spine + jaws {
            axes.push(Binding {
                joint,
                axis: Axis::Jaw,
                min: -256,
                max: 768,
            });
        }
        Ok(Self {
            version: 1,
            spine,
            jaws,
            axes,
        })
    }
    pub fn validate(&self) -> Result<(), Error> {
        if *self != Self::chain(self.spine, self.jaws)? || self.version != 1 {
            return Err(Error::Config(
                "neural anatomy mapping does not match schema 1".into(),
            ));
        }
        Ok(())
    }
    pub fn sensors(&self) -> usize {
        self.axes.len() * 3 + 29
    }
    pub fn motors(&self) -> usize {
        self.axes.len() + 4
    } // axes, valve, body-local nozzle xyz
    pub fn sensor_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        for axis in &self.axes {
            for channel in ["angle", "rate", "effort"] {
                names.push(format!("joint_{}_{:?}_{channel}", axis.joint, axis.axis));
            }
        }
        names.extend(
            [
                "velocity_x",
                "velocity_y",
                "velocity_z",
                "acceleration_x",
                "acceleration_y",
                "acceleration_z",
                "angular_velocity_x",
                "angular_velocity_y",
                "angular_velocity_z",
                "energy",
                "secretion_feedstock",
                "last_intake",
                "last_metabolic_delta",
                "last_motor_cost",
                "local_nutrient",
                "chemical_xp",
                "chemical_xm",
                "chemical_yp",
                "chemical_ym",
                "chemical_zp",
                "chemical_zm",
                "contact_xp",
                "contact_xm",
                "contact_yp",
                "contact_ym",
                "contact_zp",
                "contact_zm",
                "jaw_load_left",
                "jaw_load_right",
            ]
            .map(str::to_string),
        );
        names
    }
}
