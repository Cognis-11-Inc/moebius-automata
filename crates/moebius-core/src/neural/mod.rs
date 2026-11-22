//! Explicit neural experiment mode. Fixed-point neural dynamics, no gait library.
pub mod body;
mod controller;
pub mod schema;
pub mod sensing;
pub use controller::{
    ChemicalSensing, ControllerGenome, NEURONS, NeuralRules, NeuralState, Policy, Q,
};
