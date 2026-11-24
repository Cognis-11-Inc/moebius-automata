//! Endless generated bed for the ship windows. Presentation only.
//! It is not a biological signal and it is not part of any world digest.
//! The same recipe plays in the nursery viewer.

use std::num::NonZero;
use std::time::Duration;

use bevy::asset::Asset;
use bevy::audio::{
    AddAudioSource, AudioPlayer, AudioSink, ChannelCount, Decodable, PlaybackSettings, Sample,
    SampleRate, Source, Volume,
};
use bevy::prelude::*;
use bevy::reflect::TypePath;

#[derive(Clone, Debug)]
pub struct Drone {
    noise: u32,
    air: f32,
    phase: [f64; 8],
}

impl Default for Drone {
    fn default() -> Self {
        Self {
            noise: 0xA5A5_5A5A,
            air: 0.0,
            phase: [0.0; 8],
        }
    }
}

impl Drone {
    const RATE: f64 = 22_050.0;

    fn step(phase: &mut f64, hz: f64) -> f32 {
        *phase = (*phase + hz / Self::RATE).fract();
        (*phase * std::f64::consts::TAU).sin() as f32
    }

    pub fn next_sample(&mut self) -> f32 {
        let slow_a = Self::step(&mut self.phase[0], 0.021);
        let slow_b = Self::step(&mut self.phase[1], 0.013);
        let slow_c = Self::step(&mut self.phase[2], 0.007);
        let swell = slow_c.max(0.0).powi(2);
        let sub = Self::step(&mut self.phase[3], 32.7) * (0.18 + 0.05 * slow_a + 0.05 * swell);
        let low = Self::step(&mut self.phase[4], 49.0) * (0.07 + 0.025 * slow_b);
        let detune = Self::step(&mut self.phase[5], 49.63) * (0.07 + 0.025 * slow_b);
        let uneasy = Self::step(&mut self.phase[6], 32.7 * std::f64::consts::SQRT_2) * 0.045;
        let wander = Self::step(&mut self.phase[7], 74.0 + 16.0 * f64::from(slow_c)) * 0.035;
        self.noise = self
            .noise
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        let white = ((self.noise >> 16) as f32 / 32_768.0) - 1.0;
        let cutoff = 0.012 + 0.01 * (slow_a * 0.5 + 0.5);
        self.air += (white - self.air) * cutoff;
        let breath = 0.78 + 0.22 * slow_b;
        let mixed = (sub + low + detune + uneasy + wander + self.air * 0.16) * breath;
        let x = mixed.clamp(-1.2, 1.2);
        (x - x * x * x / 3.0) * 0.55
    }
}

#[derive(Asset, TypePath, Clone, Debug, Default)]
pub struct DarkAmbient;

pub struct DarkStream {
    drone: Drone,
}

impl Decodable for DarkAmbient {
    type Decoder = DarkStream;

    fn decoder(&self) -> Self::Decoder {
        DarkStream {
            drone: Drone::default(),
        }
    }
}

impl Iterator for DarkStream {
    type Item = Sample;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.drone.next_sample())
    }
}

impl Source for DarkStream {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> ChannelCount {
        NonZero::new(1).expect("one channel")
    }

    fn sample_rate(&self) -> SampleRate {
        NonZero::new(22_050).expect("rate")
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

#[derive(Resource)]
pub struct AmbientMix {
    pub play: bool,
    pub muted: bool,
    pub level: u8,
}

#[derive(Component)]
struct Bed;

pub fn register(app: &mut App, play: bool) {
    app.add_audio_source::<DarkAmbient>();
    app.insert_resource(AmbientMix {
        play,
        muted: false,
        level: 8,
    });
    app.add_systems(Update, (start, gain));
}

fn start(
    mut commands: Commands,
    mut sources: ResMut<Assets<DarkAmbient>>,
    mix: Res<AmbientMix>,
    existing: Query<Entity, With<Bed>>,
) {
    if !mix.play || !existing.is_empty() {
        return;
    }
    let handle = sources.add(DarkAmbient);
    // `AudioPlayer::new` only accepts the stock wav asset. A custom
    // decoder is the tuple struct itself.
    commands.spawn((
        AudioPlayer::<DarkAmbient>(handle),
        PlaybackSettings::ONCE.with_volume(Volume::Linear(if mix.level == 0 {
            0.0
        } else {
            0.32 * mix.level as f32
        })),
        Bed,
    ));
}

fn gain(mix: Res<AmbientMix>, mut sinks: Query<&mut AudioSink, With<Bed>>) {
    let volume = if !mix.play || mix.muted || mix.level == 0 {
        0.0
    } else {
        0.32 * mix.level as f32
    };
    for mut sink in &mut sinks {
        sink.set_volume(Volume::Linear(volume));
    }
}

pub fn adjust(keys: &ButtonInput<KeyCode>, mix: &mut AmbientMix, mute: KeyCode, volume: bool) {
    if keys.just_pressed(mute) {
        mix.muted = !mix.muted;
    }
    if volume && keys.just_pressed(KeyCode::BracketLeft) {
        mix.level = mix.level.saturating_sub(1);
    }
    if volume && keys.just_pressed(KeyCode::BracketRight) {
        mix.level = (mix.level + 1).min(10);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drone_keeps_going_and_does_not_repeat_a_short_bar() {
        let mut a = Drone::default();
        let mut b = Drone::default();
        let first: Vec<f32> = (0..22_050).map(|_| a.next_sample()).collect();
        let again: Vec<f32> = (0..22_050).map(|_| b.next_sample()).collect();
        assert_eq!(first, again);
        assert!(first.iter().all(|s| s.is_finite() && s.abs() <= 1.0));
        let energy = first.iter().map(|s| s.abs()).sum::<f32>() / first.len() as f32;
        assert!(energy > 0.02 && energy < 0.45, "energy {energy}");
        let later: Vec<f32> = (0..22_050).map(|_| a.next_sample()).collect();
        assert_ne!(first, later);
        let stream = DarkAmbient.decoder();
        assert!(stream.total_duration().is_none());
        let mut stream = stream;
        assert!(stream.nth(22_050 * 30).is_some());
    }
}
