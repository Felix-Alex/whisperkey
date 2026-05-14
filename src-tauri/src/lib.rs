pub mod app_state;
pub mod asr;
pub mod audio;
pub mod config;
pub mod crypto;
pub mod error;
pub mod history;
pub mod hotkey;
pub mod inject;
pub mod ipc;
pub mod license;
pub mod llm;
pub mod log;
pub mod pipeline;
pub mod util;

use app_state::AppState;
use hotkey::HotkeyEvent;
use tauri::Emitter;

fn preview(s: &str, max_bytes: usize) -> &str {
    let end = max_bytes.min(s.len());
    if s.is_char_boundary(end) { &s[..end] } else {
        let trunc = (0..end).rev().find(|&i| s.is_char_boundary(i)).unwrap_or(0);
        &s[..trunc]
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    log::init("info");
    println!("[STARTUP] WhisperKey v0.1.0 — log system ready");

    // ── Load config ──
    let paths = crate::util::paths::AppPaths::new();
    println!("[STARTUP] AppPaths: config={}", paths.config.display());
    let config_store = config::persist::ConfigStore::load(paths.config.clone())
        .expect("failed to load config");
    println!("[STARTUP] ConfigStore loaded");
    let license_store = license::LicenseStore::new(&paths);

    // ── Initialize prompt cache from user prompt files ──
    crate::llm::prompts::init_prompts(&paths.prompts_dir);
    println!("[STARTUP] Prompt cache initialized");

    // ── Read hotkey from config ──
    let hotkey_cfg = {
        let cfg = config_store.read().expect("locked config");
        let hotkey = &cfg.hotkey;
        let raw_str = format!("{}+{}", hotkey.modifiers.join("+"), hotkey.key);
        println!("=== PROBE-HK-1: config raw: modifiers={:?} key=\"{}\" formatted=\"{raw_str}\" ===", hotkey.modifiers, hotkey.key);
        if let Some(hk) = hotkey::HotkeyConfig::from_string(&raw_str) {
            println!("=== PROBE-HK-2: from_string SUCCESS → {hk} ===");
            hk
        } else {
            println!("=== PROBE-HK-2: from_string FAILED for \"{raw_str}\" — FALLBACK to Alt+J ===");
            hotkey::HotkeyConfig::new(
                vec![hotkey::Modifier::Alt],
                "J",
            )
        }
    };
    println!("[STARTUP] Hotkey: {hotkey_cfg}");

    // ── Broadcast channel for hotkey events ──
    let (hotkey_tx, hotkey_rx) = tokio::sync::broadcast::channel::<HotkeyEvent>(16);

    // ── One-time audio device probe (cached for all future recordings) ──
    println!("[STARTUP] Probing audio device (one-time cpal enumeration)...");
    let device_cache = std::sync::Arc::new(
        audio::device_cache::AudioDeviceCache::probe()
            .expect("failed to probe audio device")
    );
    println!("[STARTUP] Audio device cache ready");

    // ── Open history database ──
    let history_db = std::sync::Arc::new(std::sync::Mutex::new(
        history::db::HistoryDb::open(&paths.history_db)
            .expect("failed to open history db")
    ));
    println!("[STARTUP] History DB opened at {}", paths.history_db.display());

    // ── App state ──
    let app_state = AppState::new(
        config_store.clone(),
        license_store.clone(),
        hotkey_tx.clone(),
        device_cache.clone(),
        history_db.clone(),
    );

    // Clone components for the pipeline task (before app_state is moved into manage())
    let pipeline_config_store = config_store.clone();
    let pipeline_license_store = license_store.clone();
    let pipeline_asr_registry = app_state.asr_registry.clone();
    let pipeline_llm_registry = app_state.llm_registry.clone();
    let pipeline_device_cache = device_cache.clone();
    let pipeline_history_db = history_db.clone();

    // ── Start hotkey listener (OS thread, independent of Tauri runtime) ──
    let hotkey_handle = hotkey::registrar::start(hotkey_cfg, hotkey_tx.clone());
    app_state.set_hotkey_handle(hotkey_handle);
    println!("[STARTUP] Hotkey listener thread spawned");

    tauri::Builder::default()
        .manage(app_state)
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {}))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            ipc::config::cmd_config_get,
            ipc::config::cmd_config_set,
            ipc::config::cmd_llm_set_config,
            ipc::config::cmd_asr_set_config,
            ipc::config::cmd_config_save,
            ipc::config::cmd_llm_test_connection,
            ipc::config::cmd_asr_test_connection,
            ipc::config::cmd_hotkey_restart,
            ipc::recording::cmd_recording_toggle,
            ipc::recording::cmd_recording_get_state,
            ipc::recording::cmd_set_output_mode,
            ipc::history::cmd_history_list,
            ipc::history::cmd_history_delete,
            ipc::history::cmd_history_clear,
            ipc::license::cmd_license_status,
            ipc::license::cmd_license_activate,
            ipc::license::cmd_app_quit,
            ipc::license::cmd_app_open_logs_folder,
            ipc::config::cmd_custom_prompt_get,
            ipc::config::cmd_custom_prompt_set,
        ])
        .setup(move |_app| {
            println!("[STARTUP] Tauri setup callback invoked");
            let app_handle = _app.handle().clone();

            // Set panic hook
            std::panic::set_hook(Box::new(|info| {
                eprintln!("PANIC: {info}");
                tracing::error!("Panic: {info}");
                let crash_dir = dirs::data_dir()
                    .unwrap_or_default()
                    .join("WhisperKey")
                    .join("logs")
                    .join("crash");
                let _ = std::fs::create_dir_all(&crash_dir);
                let ts = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let path = crash_dir.join(format!("crash-{ts}.txt"));
                let _ = std::fs::write(&path, format!("{info}\n"));
            }));

            // ── Spawn pipeline task on the Tauri runtime ──
            let mut hotkey_rx = hotkey_rx;

            tauri::async_runtime::spawn(async move {
                use crate::pipeline::{transcribe_audio, process_with_llm};
                use crate::llm::r#trait::OutputMode;
                use crate::history::db::NewHistoryEntry;
                use std::sync::atomic::{AtomicBool, Ordering};
                use std::sync::{Arc, Mutex};
                use tokio::sync::oneshot;

                let save_history = |mode: &str, raw: &str, processed: &str, dur_ms: u64, asr_p: &str, llm_p: &str| {
                    let db = pipeline_history_db.lock().unwrap_or_else(|e| {
                        tracing::error!("[PIPELINE] History mutex poisoned, recovering");
                        e.into_inner()
                    });
                    let entry = NewHistoryEntry {
                        mode: mode.to_string(),
                        raw_text: raw.to_string(),
                        processed_text: processed.to_string(),
                        duration_ms: dur_ms,
                        app_name: String::new(),
                        app_title: String::new(),
                        asr_provider: asr_p.to_string(),
                        llm_provider: llm_p.to_string(),
                        injected: true,
                    };
                    if let Err(e) = db.add(&entry) {
                        tracing::error!("[PIPELINE] Failed to save history: {e:?}");
                    } else {
                        tracing::info!("[PIPELINE] History saved: mode={mode}");
                    }
                };

                println!("=== [PIPELINE LOOP] Task started ===");

                let mut recording_active = false;
                let mut current_stop_flag: Option<Arc<AtomicBool>> = None;

                loop {
                    match hotkey_rx.recv().await {
                        Ok(HotkeyEvent::Triggered) => {
                            println!("=== PROBE-PIPE-1: HotkeyEvent::Triggered received in pipeline loop! ===");
                            tracing::info!("[PIPELINE] HotkeyTriggered");

                            // Toggle stop if already recording
                            if recording_active {
                                tracing::info!("[PIPELINE] Sending stop signal");
                                if let Some(ref flag) = current_stop_flag {
                                    flag.store(true, Ordering::SeqCst);
                                }
                                recording_active = false;
                                current_stop_flag = None;
                                continue;
                            }

                            // ── Begin recording cycle ──
                            recording_active = true;
                            let stop_flag = Arc::new(AtomicBool::new(false));
                            current_stop_flag = Some(stop_flag.clone());

                            // Snapshot config for this cycle
                            let (max_dur, asr_cfg, llm_cfg, mode) = {
                                let cfg = pipeline_config_store.read().unwrap_or_else(|e| {
                                    tracing::error!("[PIPELINE] config lock poisoned: {e}");
                                    panic!("config lock poisoned");
                                });
                                (cfg.audio.max_duration_sec as u64, cfg.asr.clone(), cfg.llm.clone(), cfg.output_mode.clone())
                            };
                            // Decrypt API keys (stored DPAPI-encrypted at rest) for this cycle
                            let asr_cfg = {
                                let mut cfg = asr_cfg;
                                if !cfg.api_key.is_empty() {
                                    match crate::config::secrets::get_asr_key(&pipeline_config_store) {
                                        Ok(plain) => cfg.api_key = plain,
                                        Err(e) => tracing::error!("[PIPELINE] Failed to decrypt ASR key: {e:?}"),
                                    }
                                }
                                if !cfg.api_secret.is_empty() {
                                    match crate::config::secrets::get_asr_secret(&pipeline_config_store) {
                                        Ok(plain) => cfg.api_secret = plain,
                                        Err(e) => tracing::error!("[PIPELINE] Failed to decrypt ASR secret: {e:?}"),
                                    }
                                }
                                cfg
                            };
                            let llm_cfg = {
                                let mut cfg = llm_cfg;
                                if !cfg.api_key.is_empty() {
                                    match crate::config::secrets::get_llm_key(&pipeline_config_store) {
                                        Ok(plain) => cfg.api_key = plain,
                                        Err(e) => tracing::error!("[PIPELINE] Failed to decrypt LLM key: {e:?}"),
                                    }
                                }
                                if !cfg.api_secret.is_empty() {
                                    match crate::config::secrets::get_llm_secret(&pipeline_config_store) {
                                        Ok(plain) => cfg.api_secret = plain,
                                        Err(e) => tracing::error!("[PIPELINE] Failed to decrypt LLM secret: {e:?}"),
                                    }
                                }
                                cfg
                            };

                            let om = OutputMode::parse(&mode).unwrap_or(OutputMode::Raw);
                            tracing::info!("[PIPELINE] Cycle start: mode={mode}, max_dur={max_dur}s");

                            // ═══════════════════════════════════════════
                            // RAW MODE: streaming ASR (real-time text)
                            // ═══════════════════════════════════════════
                            if !om.requires_llm() {
                                // If ASR provider is not volcengine, fall back to batch
                                if asr_cfg.provider != "volcengine" {
                                    // Batch ASR for non-volcengine providers
                                    let (record_tx, mut record_rx) = oneshot::channel();
                                    {
                                        let flag = stop_flag.clone();
                                        let cache = Arc::clone(&pipeline_device_cache);
                                        tokio::task::spawn_blocking(move || {
                                            let result = crate::audio::recorder::record_blocking(max_dur, flag, &cache);
                                            let _ = record_tx.send(result);
                                        });
                                    }

                                    let recorded = loop {
                                        tokio::select! {
                                            event = hotkey_rx.recv() => {
                                                match event {
                                                    Ok(HotkeyEvent::Triggered) => { stop_flag.store(true, Ordering::SeqCst); }
                                                    _ => {}
                                                }
                                            }
                                            result = &mut record_rx => {
                                                break result.unwrap_or_else(|_| crate::audio::recorder::RecordedAudio { samples: vec![], duration_ms: 0, sample_rate: 0 });
                                            }
                                        }
                                    };

                                    if recorded.samples.is_empty() {
                                        tracing::warn!("[PIPELINE] No samples, skipping");
                                        recording_active = false;
                                        current_stop_flag = None;
                                        continue;
                                    }

                                    let asr_text = match transcribe_audio(&recorded, &pipeline_asr_registry, &asr_cfg).await {
                                        Ok(t) => t,
                                        Err(e) => {
                                            tracing::error!("[PIPELINE] ASR failed: {e:?}");
                                            recording_active = false;
                                            current_stop_flag = None;
                                            continue;
                                        }
                                    };

                                    tracing::info!("[PIPELINE] Injecting [{}…]", preview(&asr_text, 60));
                                    {
                                        let text = asr_text.clone();
                                        if let Err(e) = tokio::task::spawn_blocking(move || crate::inject::inject(&text)).await.unwrap_or_else(|_| Err(crate::error::AppError::Internal)) {
                                            tracing::error!("[PIPELINE] Inject failed: {e:?}");
                                        }
                                    }

                                    save_history("raw", &asr_text, &asr_text, recorded.duration_ms, &asr_cfg.provider, "none");

                                    recording_active = false;
                                    current_stop_flag = None;
                                    continue;
                                }

                                // ── Volcengine streaming ASR ──
                                use crate::asr::volcengine::VolcengineStreamer;
                                use std::time::{Instant, Duration};

                                let stream_start = Instant::now();
                                let sample_buf: Arc<Mutex<Vec<i16>>> = Arc::new(Mutex::new(Vec::new()));

                                // Start recorder in spawn_blocking (Recorder not Send, created inside)
                                let (rate_tx, rate_rx) = oneshot::channel();
                                let rec_stop = stop_flag.clone();
                                let rec_cache = Arc::clone(&pipeline_device_cache);
                                let rec_sample_buf = sample_buf.clone();
                                let max_duration = max_dur;
                                let rec_handle = tokio::task::spawn_blocking(move || {
                                    let mut recorder = crate::audio::recorder::Recorder::new();
                                    if let Err(e) = recorder.start(&rec_cache, &rec_stop) {
                                        tracing::error!("[PIPELINE] Recorder start: {e}");
                                        let _ = rate_tx.send(Err(()));
                                        return;
                                    }
                                    let rate = recorder.sample_rate();
                                    let _ = rate_tx.send(Ok(rate));

                                    let max_ms = (max_duration + 10) * 1000;
                                    let deadline = Instant::now() + Duration::from_millis(max_ms);
                                    loop {
                                        std::thread::sleep(Duration::from_millis(200));
                                        // Drain internal buffer to shared buffer
                                        let drained = recorder.drain_samples();
                                        if !drained.is_empty() {
                                            rec_sample_buf.lock().unwrap().extend_from_slice(&drained);
                                        }
                                        if rec_stop.load(Ordering::SeqCst) || Instant::now() > deadline {
                                            break;
                                        }
                                    }
                                    let remaining = recorder.stop();
                                    rec_sample_buf.lock().unwrap().extend_from_slice(&remaining);
                                });

                                // Wait for recorder to start
                                let sample_rate = match rate_rx.await {
                                    Ok(Ok(rate)) => rate,
                                    _ => {
                                        tracing::error!("[PIPELINE] Recorder failed to start");
                                        stop_flag.store(true, Ordering::SeqCst);
                                        let _ = rec_handle.await;
                                        recording_active = false;
                                        current_stop_flag = None;
                                        continue;
                                    }
                                };

                                // Start volcengine streamer
                                let mut streamer = match VolcengineStreamer::connect(&asr_cfg, sample_rate).await {
                                    Ok(s) => s,
                                    Err(e) => {
                                        tracing::error!("[PIPELINE] Streamer connect: {e:?}");
                                        stop_flag.store(true, Ordering::SeqCst);
                                        let _ = rec_handle.await;
                                        recording_active = false;
                                        current_stop_flag = None;
                                        continue;
                                    }
                                };

                                // Streaming loop
                                let mut last_partial = String::new();
                                let mut partial_seq = 0u32;
                                loop {
                                    let chunk: Vec<i16> = {
                                        let mut buf = sample_buf.lock().unwrap();
                                        std::mem::take(&mut *buf)
                                    };

                                    let stopped = stop_flag.load(Ordering::SeqCst);
                                    let rec_finished = rec_handle.is_finished();
                                    let should_stop = stopped || rec_finished;

                                    // Send accumulated audio (non-final — finalize() sends the final marker)
                                    if !chunk.is_empty() {
                                        let pcm_bytes: Vec<u8> = chunk.iter()
                                            .flat_map(|s| s.to_le_bytes())
                                            .collect();
                                        let pcm_len = pcm_bytes.len();
                                        tracing::debug!("[PIPELINE] sending {} PCM bytes (seq continuation)", pcm_len);
                                        if let Err(e) = streamer.send_audio(&pcm_bytes, false).await {
                                            tracing::error!("[PIPELINE] send_audio: {e:?}");
                                            stop_flag.store(true, Ordering::SeqCst);
                                            break;
                                        }
                                    }

                                    // Check for partial text — inject delta in real-time
                                    match streamer.try_recv_text().await {
                                        Ok(Some(text)) if !text.is_empty() && text != last_partial => {
                                            partial_seq += 1;
                                            tracing::info!("[PIPELINE] partial #{partial_seq}: '{text}'");
                                            // Inject only the new portion (delta) at cursor
                                            if text.starts_with(&last_partial) && text.len() > last_partial.len() {
                                                let delta: String = text[last_partial.len()..].to_string();
                                                tracing::debug!("[PIPELINE] injecting delta: '{delta}'");
                                                if let Err(e) = tokio::task::spawn_blocking(move || crate::inject::inject(&delta)).await.unwrap_or_else(|_| Err(crate::error::AppError::Internal)) {
                                                    tracing::error!("[PIPELINE] delta inject failed: {e:?}");
                                                }
                                            }
                                            last_partial = text.clone();
                                            let _ = app_handle.emit("asr-partial-text", serde_json::json!({
                                                "text": text,
                                                "seq": partial_seq
                                            }));
                                        }
                                        Err(e) => {
                                            tracing::warn!("[PIPELINE] try_recv: {e:?}");
                                        }
                                        _ => {}
                                    }

                                    // Check for stop hotkey
                                    if !stopped {
                                        match hotkey_rx.try_recv() {
                                            Ok(HotkeyEvent::Triggered) => {
                                                stop_flag.store(true, Ordering::SeqCst);
                                                tracing::info!("[PIPELINE] Stop hotkey during streaming");
                                            }
                                            _ => {}
                                        }
                                    }

                                    if should_stop {
                                        break;
                                    }

                                    tokio::time::sleep(Duration::from_millis(200)).await;
                                }

                                // Finalize and get final text
                                let final_text = match streamer.finalize().await {
                                    Ok(t) => t,
                                    Err(e) => {
                                        tracing::error!("[PIPELINE] finalize: {e:?}");
                                        if !last_partial.is_empty() { last_partial.clone() } else {
                                            recording_active = false;
                                            current_stop_flag = None;
                                            continue;
                                        }
                                    }
                                };

                                let _ = app_handle.emit("asr-partial-text", serde_json::json!({
                                    "text": final_text,
                                    "seq": partial_seq + 1,
                                    "final": true
                                }));

                                let _ = rec_handle.await;

                                // Inject final delta (remaining text after last partial)
                                let final_delta: String = if final_text.starts_with(&last_partial) && final_text.len() > last_partial.len() {
                                    final_text[last_partial.len()..].to_string()
                                } else if !final_text.is_empty() {
                                    final_text.clone()
                                } else {
                                    String::new()
                                };
                                if !final_delta.is_empty() {
                                    tracing::info!("[PIPELINE] Injecting final delta [{}…]", preview(&final_delta, 60));
                                    if let Err(e) = tokio::task::spawn_blocking(move || crate::inject::inject(&final_delta)).await.unwrap_or_else(|_| Err(crate::error::AppError::Internal)) {
                                        tracing::error!("[PIPELINE] Inject failed: {e:?}");
                                    }
                                } else {
                                    tracing::info!("[PIPELINE] All text already injected during streaming");
                                }

                                let stream_dur_ms = stream_start.elapsed().as_millis() as u64;
                                save_history("raw", &final_text, &final_text, stream_dur_ms, &asr_cfg.provider, "none");

                                recording_active = false;
                                current_stop_flag = None;
                                tracing::info!("[PIPELINE] Raw streaming cycle complete");
                                continue;
                            }

                            // ═══════════════════════════════════════════
                            // NON-RAW MODE: batch pipeline (record → ASR → LLM → inject)
                            // ═══════════════════════════════════════════

                            // Phase 1: Record
                            let (record_tx, mut record_rx) = oneshot::channel();
                            {
                                let flag = stop_flag.clone();
                                let cache = Arc::clone(&pipeline_device_cache);
                                tokio::task::spawn_blocking(move || {
                                    let result = crate::audio::recorder::record_blocking(
                                        max_dur, flag, &cache,
                                    );
                                    let _ = record_tx.send(result);
                                });
                            }

                            let recorded = loop {
                                tokio::select! {
                                    event = hotkey_rx.recv() => {
                                        match event {
                                            Ok(HotkeyEvent::Triggered) => {
                                                tracing::info!("[PIPELINE] Stop hotkey — stopping recording");
                                                stop_flag.store(true, Ordering::SeqCst);
                                            }
                                            Ok(HotkeyEvent::RegisterFailed) => {
                                                tracing::error!("[PIPELINE] RegisterFailed during recording");
                                            }
                                            Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                                                tracing::warn!("[PIPELINE] Lagged by {n} during recording");
                                            }
                                            Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                                                stop_flag.store(true, Ordering::SeqCst);
                                            }
                                        }
                                    }
                                    result = &mut record_rx => {
                                        break result.unwrap_or_else(|_| crate::audio::recorder::RecordedAudio {
                                            samples: vec![],
                                            duration_ms: 0,
                                            sample_rate: 0,
                                        });
                                    }
                                }
                            };

                            if recorded.samples.is_empty() {
                                tracing::warn!("[PIPELINE] Recording produced no samples, skipping");
                                recording_active = false;
                                current_stop_flag = None;
                                continue;
                            }

                            let was_manual_stop = stop_flag.load(Ordering::SeqCst);
                            tracing::info!("[PIPELINE] Recorded: {} samples, {}ms (manual_stop={was_manual_stop})",
                                recorded.samples.len(), recorded.duration_ms);

                            // Phase 2: ASR
                            let asr_text = match transcribe_audio(&recorded, &pipeline_asr_registry, &asr_cfg).await {
                                Ok(t) => { tracing::info!("[PIPELINE] ASR result: [{}]", t); t }
                                Err(e) => {
                                    tracing::error!("[PIPELINE] ASR failed: {e:?}");
                                    recording_active = false;
                                    current_stop_flag = None;
                                    continue;
                                }
                            };

                            // Phase 3: LLM
                            let final_text = match process_with_llm(&asr_text, &om, &pipeline_llm_registry, &llm_cfg, &pipeline_license_store).await {
                                Ok(t) => { tracing::info!("[PIPELINE] LLM done"); t }
                                Err(e) => {
                                    tracing::error!("[PIPELINE] LLM failed: {e:?}, falling back to raw");
                                    asr_text.clone()
                                }
                            };

                            // Phase 4: Inject
                            tracing::info!("[PIPELINE] Injecting [{}…]", preview(&final_text, 60));
                            {
                                let text = final_text.clone();
                                match tokio::task::spawn_blocking(move || crate::inject::inject(&text)).await {
                                    Ok(Ok(r)) => tracing::info!("[PIPELINE] Inject: {r:?}"),
                                    Ok(Err(e)) => tracing::error!("[PIPELINE] Inject failed: {e:?}"),
                                    Err(join_err) => tracing::error!("[PIPELINE] Inject task panicked: {join_err:?}"),
                                }
                            }

                            save_history(&mode, &asr_text, &final_text, recorded.duration_ms, &asr_cfg.provider, &llm_cfg.provider);

                            recording_active = false;
                            current_stop_flag = None;
                            tracing::info!("[PIPELINE] Batch cycle complete");
                        }
                        Ok(HotkeyEvent::RegisterFailed) => {
                            println!("=== PROBE-PIPE-0: HotkeyEvent::RegisterFailed received! ===");
                            tracing::error!("[PIPELINE] Hotkey registration failed");
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                            tracing::warn!("[PIPELINE] Hotkey receiver lagged by {n}");
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                            tracing::error!("[PIPELINE] Hotkey channel closed");
                            break;
                        }
                    }
                }
                println!("=== [PIPELINE LOOP] Exiting ===");
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    // tauri::run() is blocking — if we reach here the event loop has exited
    println!("[STARTUP] Tauri event loop exited");
    // hotkey_handle is stored in AppState — dropped automatically when AppState is dropped
    println!("[STARTUP] Exiting");
}
