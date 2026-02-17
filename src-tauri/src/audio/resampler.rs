/// Simple linear resampler for converting audio sample rates.
/// Converts from source sample rate to target sample rate (typically 16kHz for Vosk).
pub struct Resampler {
    source_rate: u32,
    target_rate: u32,
    accumulator: f64,
}

impl Resampler {
    pub fn new(source_rate: u32, target_rate: u32) -> Self {
        Self {
            source_rate,
            target_rate,
            accumulator: 0.0,
        }
    }

    /// Returns true if resampling is needed (rates differ).
    pub fn needs_resampling(&self) -> bool {
        self.source_rate != self.target_rate
    }

    /// Resample i16 PCM samples from source rate to target rate.
    pub fn resample(&mut self, input: &[i16]) -> Vec<i16> {
        if !self.needs_resampling() {
            return input.to_vec();
        }

        let ratio = self.source_rate as f64 / self.target_rate as f64;
        let estimated_len = (input.len() as f64 / ratio) as usize + 1;
        let mut output = Vec::with_capacity(estimated_len);

        for &sample in input {
            self.accumulator += 1.0;
            if self.accumulator >= ratio {
                self.accumulator -= ratio;
                output.push(sample);
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_resampling_needed() {
        let mut r = Resampler::new(16000, 16000);
        assert!(!r.needs_resampling());
        let input = vec![100, 200, 300];
        assert_eq!(r.resample(&input), input);
    }

    #[test]
    fn test_downsample_48k_to_16k() {
        let mut r = Resampler::new(48000, 16000);
        assert!(r.needs_resampling());
        // 48kHz -> 16kHz is 3:1 ratio
        let input: Vec<i16> = (0..48).collect();
        let output = r.resample(&input);
        // Should produce roughly 16 samples from 48
        assert!(output.len() >= 15 && output.len() <= 17);
    }
}
