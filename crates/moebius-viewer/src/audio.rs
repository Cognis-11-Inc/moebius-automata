//! Viewer-only sound. Samples are not part of the state hash.
//! Byte identity is not claimed across platforms or libm builds.
//!
//! `synth_wav` is the old one-second population burst. It stays under test
//! and is not played. The hall and the committed-run picture play
//! `DarkAmbient`: one generated bed that does not end and does not restart.

#[cfg(test)]
const RATE: u32 = 22_050;
#[cfg(test)]
const FRAMES: usize = 22_050;

#[cfg(test)]
pub fn synth_wav(tick: u64, live: u32, dead: u32, field: u64, waste: u64) -> Vec<u8> {
    let fundamental = 33.0 + (live % 19) as f32;
    let tritone = fundamental * std::f32::consts::SQRT_2;
    let scrape = 48.0 + (dead % 17) as f32 * 2.5;
    let grit = 0.05 + ((waste % 23) as f32) * 0.003 + ((field % 29) as f32) * 0.000_8;
    let extinct = live == 0;
    let mut samples = Vec::with_capacity(FRAMES);
    for index in 0..FRAMES {
        let time = index as f32 / RATE as f32;
        let mut sample = (std::f32::consts::TAU * fundamental * time).sin() * 0.32;
        sample += (std::f32::consts::TAU * tritone * time).sin() * 0.2;
        sample += (std::f32::consts::TAU * scrape * time).sin().signum() * 0.07;
        let noise = ((tick
            .wrapping_mul(0x9E37_79B9_7F4A_7C15)
            .wrapping_add((index as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9)))
            >> 40) as u32;
        sample += ((noise % 1_024) as f32 / 1_024.0 - 0.5) * grit * 2.0;
        let period = 160 + (live % 90) as usize + (dead % 11) as usize;
        let phase = index % period;
        if phase < 5 {
            sample += 0.5 * (1.0 - phase as f32 / 5.0);
        }
        if extinct {
            sample = sample * 0.55 + (std::f32::consts::TAU * 27.5 * time).sin() * 0.16;
        }
        let crushed = (sample * 12.0).round() / 12.0;
        // Fade each short call in and out; no click at the clip boundaries.
        let fade_in = (time / 0.15).clamp(0.0, 1.0);
        let remaining = (FRAMES - 1 - index) as f32 / RATE as f32;
        let fade_out = (remaining / 0.30).clamp(0.0, 1.0);
        let envelope = (fade_in * std::f32::consts::FRAC_PI_2).sin().powi(2)
            * (fade_out * std::f32::consts::FRAC_PI_2).sin().powi(2);
        let bounded = crushed.clamp(-0.95, 0.95) * envelope;
        samples.push((bounded * i16::MAX as f32) as i16);
    }
    wav_pcm16(RATE, &samples)
}

#[cfg(test)]
fn wav_pcm16(rate: u32, samples: &[i16]) -> Vec<u8> {
    let data_len = samples.len() * 2;
    let mut out = Vec::with_capacity(44 + data_len);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&u32::try_from(36 + data_len).unwrap_or(36).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&rate.to_le_bytes());
    out.extend_from_slice(&(rate * 2).to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&u32::try_from(data_len).unwrap_or(0).to_le_bytes());
    for sample in samples {
        out.extend_from_slice(&sample.to_le_bytes());
    }
    out
}

/// Endless presentation bed. Each voice keeps its own phase, so a wrap is
/// exactly one cycle and the tone does not click or collapse into a short loop.
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

use std::num::NonZero;
use std::time::Duration;

use bevy::asset::Asset;
use bevy::audio::{AddAudioSource, ChannelCount, Decodable, Sample, SampleRate, Source};
use bevy::prelude::App;
use bevy::reflect::TypePath;

/// One endless bed. Playback uses `Once` because the stream itself does not end.
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

pub fn register(app: &mut App) {
    app.add_audio_source::<DarkAmbient>();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synth_is_deterministic_and_bounded() {
        let first = synth_wav(40, 12, 3, 90_000, 400);
        let second = synth_wav(40, 12, 3, 90_000, 400);
        assert_eq!(first, second);
        assert!(first.starts_with(b"RIFF"));
        assert!(first.len() < 100_000);
        let silent_population = synth_wav(40, 0, 12, 90_000, 400);
        assert_ne!(first, silent_population);
    }

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
        assert_eq!(stream.channels().get(), 1);
        let mut stream = stream;
        assert!(stream.nth(22_050 * 30).is_some());
    }
}
