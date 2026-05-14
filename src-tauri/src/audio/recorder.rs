use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{BuildStreamError, SampleFormat, Stream, StreamConfig};

pub struct Recorder {
    samples: Arc<Mutex<Vec<i16>>>,
    stream: Option<Stream>,
    start_time: Option<Instant>,
    sample_rate: u32,
}

impl Recorder {
    pub fn new() -> Self {
        Self {
            samples: Arc::new(Mutex::new(Vec::new())),
            stream: None,
            start_time: None,
            sample_rate: 16000,
        }
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn sample_buffer(&self) -> Arc<Mutex<Vec<i16>>> {
        self.samples.clone()
    }

    pub fn drain_samples(&self) -> Vec<i16> {
        std::mem::take(&mut *self.samples.lock().unwrap())
    }

    /// Start recording using a PRE-CACHED device config.
    /// Zero WASAPI enumeration — builds the stream directly from cached parameters.
    pub fn start(
        &mut self,
        cache: &crate::audio::device_cache::AudioDeviceCache,
        stop_flag: &AtomicBool,
    ) -> Result<(), String> {
        if stop_flag.load(Ordering::SeqCst) {
            tracing::info!("[AUDIO] start() aborted: stop_flag set before init");
            return Err("cancelled".into());
        }

        let t0 = Instant::now();
        self.start_time = Some(Instant::now());

        let (stream, actual_rate) = cache
            .build_stream(self.samples.clone())
            .map_err(|e| {
                tracing::error!("[AUDIO] build_stream(cached) failed: {e}");
                e
            })?;

        stream.play().map_err(|e| {
            tracing::error!("[AUDIO] stream.play() failed: {e}");
            format!("failed to start audio stream: {e}")
        })?;

        let elapsed = (Instant::now() - t0).as_secs_f64() * 1000.0;
        tracing::info!(
            "[AUDIO] Recorder active at {actual_rate} Hz (cached, start: {elapsed:.0} ms)"
        );

        self.stream = Some(stream);
        self.sample_rate = actual_rate;
        Ok(())
    }

    /// Shared entry point: build a stream for the given format + channel count.
    pub(crate) fn try_build_stream(
        device: &cpal::Device,
        config: &StreamConfig,
        channels: u16,
        format: SampleFormat,
        samples: Arc<Mutex<Vec<i16>>>,
    ) -> Result<Stream, BuildStreamError> {
        match (format, channels) {
            (SampleFormat::I16, 1) => Self::try_i16_mono_stream(device, config, samples),
            (SampleFormat::I16, ch) => Self::try_i16_downmix_stream(device, config, ch, samples),
            (SampleFormat::F32, 1) => Self::try_f32_downmix_stream(device, config, 1, samples),
            (SampleFormat::F32, ch) => Self::try_f32_downmix_stream(device, config, ch, samples),
            _ => Err(BuildStreamError::StreamConfigNotSupported),
        }
    }

    fn try_i16_mono_stream(
        device: &cpal::Device,
        config: &StreamConfig,
        samples: Arc<Mutex<Vec<i16>>>,
    ) -> Result<Stream, BuildStreamError> {
        device.build_input_stream(
            config,
            move |data: &[i16], _info| {
                samples.lock().unwrap().extend_from_slice(data);
            },
            |err| tracing::error!("audio input error: {err}"),
            None,
        )
    }

    fn try_i16_downmix_stream(
        device: &cpal::Device,
        config: &StreamConfig,
        channels: u16,
        samples: Arc<Mutex<Vec<i16>>>,
    ) -> Result<Stream, BuildStreamError> {
        device.build_input_stream(
            config,
            move |data: &[i16], _info| {
                let mut buf = samples.lock().unwrap();
                if channels == 1 {
                    buf.extend_from_slice(data);
                } else {
                    let ch = channels as i32;
                    for chunk in data.chunks(channels as usize) {
                        let sum: i32 = chunk.iter().map(|&s| s as i32).sum();
                        buf.push((sum / ch) as i16);
                    }
                }
            },
            |err| tracing::error!("audio input error: {err}"),
            None,
        )
    }

    fn try_f32_downmix_stream(
        device: &cpal::Device,
        config: &StreamConfig,
        channels: u16,
        samples: Arc<Mutex<Vec<i16>>>,
    ) -> Result<Stream, BuildStreamError> {
        device.build_input_stream(
            config,
            move |data: &[f32], _info| {
                let mut buf = samples.lock().unwrap();
                if channels == 1 {
                    for &s in data {
                        let clamped = s.clamp(-1.0, 1.0);
                        buf.push((clamped * 32767.0) as i16);
                    }
                } else {
                    let n = channels as f32;
                    for chunk in data.chunks(channels as usize) {
                        let avg: f32 = chunk.iter().sum::<f32>() / n;
                        let clamped = avg.clamp(-1.0, 1.0);
                        buf.push((clamped * 32767.0) as i16);
                    }
                }
            },
            |err| tracing::error!("audio input error: {err}"),
            None,
        )
    }

    pub fn stop(&mut self) -> Vec<i16> {
        self.stream.take();
        self.start_time = None;

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
}

/// Result of a blocking recording session.
pub struct RecordedAudio {
    pub samples: Vec<i16>,
    pub duration_ms: u64,
    pub sample_rate: u32,
}

/// Blocking record wrapper. Uses the pre-cached audio device config —
/// zero WASAPI enumeration on the hot path.
///
/// Exit conditions (only two):
/// 1. `stop_flag` — second hotkey press (manual stop).
/// 2. `max_duration_sec` hard deadline — safety net against deadlock.
pub fn record_blocking(
    max_duration_sec: u64,
    stop_flag: Arc<AtomicBool>,
    cache: &crate::audio::device_cache::AudioDeviceCache,
) -> RecordedAudio {
    println!("=== [REC THREAD] record_blocking entered, max_dur={max_duration_sec}s ===");

    if stop_flag.load(Ordering::SeqCst) {
        println!("=== [REC THREAD] Stop flag set before init, aborting ===");
        tracing::info!("Recording aborted before cpal init (stop flag)");
        return RecordedAudio { samples: vec![], duration_ms: 0, sample_rate: 0 };
    }

    let mut recorder = Recorder::new();

    println!("=== [REC THREAD] Starting audio device (cached)... ===");
    if let Err(e) = recorder.start(cache, &stop_flag) {
        eprintln!("[REC ERROR] Recorder failed to start: {e}");
        tracing::error!("Recorder failed to start: {e}");
        return RecordedAudio { samples: vec![], duration_ms: 0, sample_rate: 0 };
    }
    let sample_rate = recorder.sample_rate();
    println!("=== [REC THREAD] Audio stream started at {sample_rate}Hz, entering poll loop ===");
    tracing::info!("Recorder started at {sample_rate}Hz (blocking thread)");

    let max_ms = (max_duration_sec + 10) * 1000;
    let deadline = Instant::now() + std::time::Duration::from_millis(max_ms);

    let mut loop_count: u64 = 0;
    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));
        loop_count += 1;

        if loop_count % 10 == 0 {
            let sc = recorder.sample_count();
            let rms = recorder.current_rms();
            println!("=== [REC THREAD] alive: loop={loop_count}, samples={sc}, rms={rms:.0} ===");
        }

        // Exit condition 1: manual stop (second hotkey press)
        if stop_flag.load(Ordering::SeqCst) {
            println!("=== [REC THREAD] Stop flag detected! Breaking loop ===");
            tracing::info!("Stop reason: Hotkey toggle (manual)");
            break;
        }

        // Exit condition 2: hard deadline safety net
        if Instant::now() > deadline {
            println!("=== [REC THREAD] Max duration reached, forcing stop ===");
            tracing::warn!("Max duration reached, forcing stop");
            break;
        }
    }

    println!("=== [REC THREAD] Stopping stream and collecting samples... ===");
    let duration = recorder.elapsed_ms();
    let samples = recorder.stop();
    println!(
        "=== [REC THREAD] Done: samples={}, duration_ms={duration}, rate={sample_rate}Hz ===",
        samples.len(),
    );
    RecordedAudio { samples, duration_ms: duration, sample_rate }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_recorder() {
        let rec = Recorder::new();
        assert_eq!(rec.sample_count(), 0);
        assert_eq!(rec.current_rms(), 0.0);
        assert_eq!(rec.elapsed_ms(), 0);
        assert_eq!(rec.sample_rate(), 16000);
    }

    #[test]
    fn test_rms_with_known_samples() {
        let rec = Recorder {
            samples: Arc::new(Mutex::new(vec![100i16, -100, 100, -100, 50, -50, 0, 0])),
            stream: None,
            start_time: None,
            sample_rate: 16000,
        };
        let rms = rec.current_rms();
        assert!(rms > 0.0);
        assert!(rms < 32768.0);
    }
}
