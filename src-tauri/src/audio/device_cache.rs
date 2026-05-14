//! One-time audio device probe + cache.
//! Eliminates the 1-3s WASAPI `supported_input_configs()` call on every recording.
//! Auto-refreshes when the default input device changes (e.g. headset unplugged).

use std::sync::Mutex;

use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{SampleFormat, SampleRate, StreamConfig};

/// Cached state that can be refreshed when the device changes.
struct Inner {
    device: cpal::Device,
    channels: u16,
    sample_format: SampleFormat,
    working_rate: u32,
    needs_downmix: bool,
}

pub struct AudioDeviceCache {
    inner: Mutex<Inner>,
}

impl AudioDeviceCache {
    /// Probe the default input device ONCE: enumerate supported configs,
    /// determine the best working rate, and cache everything.
    pub fn probe() -> Result<Self, String> {
        let inner = Self::probe_inner()?;
        Ok(Self {
            inner: Mutex::new(inner),
        })
    }

    fn probe_inner() -> Result<Inner, String> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| "no input device available".to_string())?;

        let dev_name = device.name().unwrap_or_default();
        tracing::info!("[AUDIO CACHE] Probing device: \"{dev_name}\"");

        // ── Enumerate supported configs (diagnostics, run once) ──
        match device.supported_input_configs() {
            Ok(configs) => {
                let list: Vec<String> = configs
                    .map(|c| {
                        format!(
                            "ch={} rate={}-{} fmt={:?}",
                            c.channels(),
                            c.min_sample_rate().0,
                            c.max_sample_rate().0,
                            c.sample_format(),
                        )
                    })
                    .collect();
                tracing::info!("[AUDIO CACHE] Supported configs ({}):", list.len());
                for s in &list {
                    tracing::info!("[AUDIO CACHE]   {s}");
                }
            }
            Err(e) => {
                tracing::warn!("[AUDIO CACHE] supported_input_configs failed: {e}");
            }
        }

        // ── Determine native config ──
        let native_cfg = device
            .default_input_config()
            .map_err(|e| format!("default_input_config: {e}"))?;

        let channels = native_cfg.channels();
        let sample_format = native_cfg.sample_format();
        let native_rate = native_cfg.sample_rate().0;

        tracing::info!(
            "[AUDIO CACHE] Native: ch={channels} rate={native_rate} fmt={sample_format:?}"
        );

        // ── Trial-build: which rate works? Prefer 16 kHz ──
        let rate_candidates: Vec<u32> = if native_rate == 16000 {
            vec![16000]
        } else {
            vec![16000, native_rate]
        };

        let mut working_rate = native_rate;

        for &rate in &rate_candidates {
            let stream_cfg = StreamConfig {
                channels,
                sample_rate: SampleRate(rate),
                buffer_size: cpal::BufferSize::Default,
            };

            let samples = std::sync::Arc::new(Mutex::new(Vec::<i16>::new()));
            let result = super::recorder::Recorder::try_build_stream(
                &device,
                &stream_cfg,
                channels,
                sample_format,
                samples,
            );

            match result {
                Ok(stream) => {
                    drop(stream);
                    working_rate = rate;
                    tracing::info!("[AUDIO CACHE] Trial stream OK at {rate} Hz — caching");
                    break;
                }
                Err(e) => {
                    tracing::info!("[AUDIO CACHE] Trial stream at {rate} Hz failed: {e}");
                }
            }
        }

        tracing::info!(
            "[AUDIO CACHE] Cached: ch={channels} rate={working_rate} fmt={sample_format:?} downmix={}",
            channels > 1
        );

        Ok(Inner {
            device,
            channels,
            sample_format,
            working_rate,
            needs_downmix: channels > 1,
        })
    }

    /// Build an input stream using the cached config.
    /// If the cached device is no longer available (e.g. headset unplugged),
    /// falls back to a full re-probe of the current default input device.
    pub fn build_stream(
        &self,
        samples: std::sync::Arc<Mutex<Vec<i16>>>,
    ) -> Result<(cpal::Stream, u32), String> {
        // First attempt with cached device+config
        {
            let inner = self.inner.lock().unwrap();
            let stream_cfg = StreamConfig {
                channels: inner.channels,
                sample_rate: SampleRate(inner.working_rate),
                buffer_size: cpal::BufferSize::Default,
            };
            let result = super::recorder::Recorder::try_build_stream(
                &inner.device,
                &stream_cfg,
                inner.channels,
                inner.sample_format,
                samples.clone(),
            );
            if let Ok(stream) = result {
                return Ok((stream, inner.working_rate));
            }
        }

        // Cached device gone (headset unplugged, etc.) — full re-probe
        tracing::warn!("[AUDIO CACHE] Cached device unavailable, re-probing default input device...");
        let fresh = Self::probe_inner()?;
        tracing::info!(
            "[AUDIO CACHE] Re-probed: ch={} rate={} fmt={:?}",
            fresh.channels, fresh.working_rate, fresh.sample_format
        );

        let stream_cfg = StreamConfig {
            channels: fresh.channels,
            sample_rate: SampleRate(fresh.working_rate),
            buffer_size: cpal::BufferSize::Default,
        };
        let stream = super::recorder::Recorder::try_build_stream(
            &fresh.device,
            &stream_cfg,
            fresh.channels,
            fresh.sample_format,
            samples,
        )
        .map_err(|e| format!("build_stream(retry): {e}"))?;

        let rate = fresh.working_rate;
        *self.inner.lock().unwrap() = fresh;

        Ok((stream, rate))
    }
}
