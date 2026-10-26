//! Viewer-only sound. Samples are a pure function of committed counts.
//! They use f32 and are not part of the state hash. Byte identity is not
//! claimed across platforms or libm builds.

const RATE: u32 = 22_050;
const FRAMES: usize = 22_050;

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
        let bounded = crushed.clamp(-0.95, 0.95);
        samples.push((bounded * i16::MAX as f32) as i16);
    }
    wav_pcm16(RATE, &samples)
}

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
}
