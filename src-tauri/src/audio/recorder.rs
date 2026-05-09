use std::sync::{Arc, Mutex};
use std::time::Instant;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig};

use crate::audio::StopReason;

pub struct RecorderConfig {
    pub silence_threshold_rms: f64,
    pub silence_timeout_ms: u64,
    pub max_duration_sec: u64,
}

impl Default for RecorderConfig {
    fn default() -> Self {
        Self {
            silence_threshold_rms: 500.0,
            silence_timeout_ms: 3000,
            max_duration_sec: 60,
        }
    }
}

pub struct Recorder {
    samples: Arc<Mutex<Vec<i16>>>,
    stream: Option<Stream>,
    start_time: Option<Instant>,
    sample_rate: u32,
    config: RecorderConfig,
    silence_start: Option<Instant>,
}

impl Recorder {
    pub fn new(config: RecorderConfig) -> Self {
        Self {
            samples: Arc::new(Mutex::new(Vec::new())),
            stream: None,
            start_time: None,
            sample_rate: 16000,
            config,
            silence_start: None,
        }
    }

    pub fn start(&mut self) -> Result<(), String> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or("no input device available")?;

        let config = StreamConfig {
            channels: 1,
            sample_rate: cpal::SampleRate(16000),
            buffer_size: cpal::BufferSize::Default,
        };

        let samples = self.samples.clone();
        self.start_time = Some(Instant::now());
        self.sample_rate = 16000;
        self.silence_start = None;

        let stream = device
            .build_input_stream(
                &config,
                move |data: &[i16], _info| {
                    samples.lock().unwrap().extend_from_slice(data);
                },
                |err| {
                    tracing::error!("audio input error: {err}");
                },
                None,
            )
            .map_err(|e| format!("failed to build input stream: {e}"))?;

        stream.play().map_err(|e| format!("failed to start stream: {e}"))?;
        self.stream = Some(stream);

        Ok(())
    }

    pub fn stop(&mut self) -> Vec<i16> {
        self.stream.take();
        self.start_time = None;
        self.silence_start = None;

        let samples = self.samples.lock().unwrap().clone();
        self.samples.lock().unwrap().clear();
        samples
    }

    pub fn current_rms(&self) -> f64 {
        let samples = self.samples.lock().unwrap();
        if samples.is_empty() {
            return 0.0;
        }

        let window = (self.sample_rate as usize / 10).min(samples.len());
        let recent = &samples[samples.len() - window..];

        let sum_sq: f64 = recent.iter().map(|&s| (s as f64) * (s as f64)).sum();
        (sum_sq / recent.len() as f64).sqrt()
    }

    pub fn sample_count(&self) -> usize {
        self.samples.lock().unwrap().len()
    }

    pub fn elapsed_ms(&self) -> u64 {
        match &self.start_time {
            Some(t) => t.elapsed().as_millis() as u64,
            None => 0,
        }
    }

    /// Check if recording should stop for silence or max duration
    pub fn check_stop_reason(&mut self) -> Option<StopReason> {
        let elapsed_ms = self.elapsed_ms();

        // Check max duration
        if elapsed_ms >= self.config.max_duration_sec * 1000 {
            return Some(StopReason::MaxDuration);
        }

        // Check silence
        let rms = self.current_rms();
        if rms < self.config.silence_threshold_rms {
            if self.silence_start.is_none() {
                self.silence_start = Some(Instant::now());
            }
            let silence_ms = self.silence_start.unwrap().elapsed().as_millis() as u64;
            if silence_ms >= self.config.silence_timeout_ms {
                return Some(StopReason::Silence);
            }
        } else {
            self.silence_start = None;
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_recorder() {
        let rec = Recorder::new(RecorderConfig::default());
        assert_eq!(rec.sample_count(), 0);
        assert_eq!(rec.current_rms(), 0.0);
        assert_eq!(rec.elapsed_ms(), 0);
    }

    #[test]
    fn test_rms_with_known_samples() {
        let rec = Recorder {
            samples: Arc::new(Mutex::new(vec![100i16, -100, 100, -100, 50, -50, 0, 0])),
            stream: None,
            start_time: None,
            sample_rate: 16000,
            config: RecorderConfig::default(),
            silence_start: None,
        };
        let rms = rec.current_rms();
        assert!(rms > 0.0);
        assert!(rms < 32768.0);
    }

    #[test]
    fn test_max_duration_stop() {
        let mut rec = Recorder::new(RecorderConfig {
            max_duration_sec: 0, // Immediate
            ..Default::default()
        });
        rec.start_time = Some(Instant::now());
        assert_eq!(rec.check_stop_reason(), Some(StopReason::MaxDuration));
    }

    #[test]
    fn test_silence_stop() {
        let mut rec = Recorder::new(RecorderConfig {
            silence_threshold_rms: 10.0,
            silence_timeout_ms: 0, // Immediate
            max_duration_sec: 60,
        });
        rec.start_time = Some(Instant::now());
        // With empty samples, RMS should be 0 < threshold
        assert_eq!(rec.check_stop_reason(), Some(StopReason::Silence));
    }

    #[test]
    fn test_no_stop_with_signal() {
        let mut rec = Recorder {
            samples: Arc::new(Mutex::new(vec![1000i16, -1000, 1000, -1000])),
            stream: None,
            start_time: Some(Instant::now()),
            sample_rate: 16000,
            config: RecorderConfig {
                silence_threshold_rms: 10.0,
                silence_timeout_ms: 10000, // 10s
                max_duration_sec: 60,
            },
            silence_start: None,
        };
        // RMS is high, shouldn't trigger silence
        assert_eq!(rec.check_stop_reason(), None);
    }
}
