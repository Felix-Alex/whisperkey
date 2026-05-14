pub mod state_machine;

/// Encode recorded audio to WAV and run ASR.
pub async fn transcribe_audio(
    recorded: &crate::audio::recorder::RecordedAudio,
    asr_registry: &crate::asr::AsrRegistry,
    asr_config: &crate::config::schema::AsrConfig,
) -> Result<String, crate::error::AppError> {
    let wav_bytes = crate::audio::encoder::encode_wav(&recorded.samples, recorded.sample_rate)?;

    let provider = asr_registry
        .get(&asr_config.provider)
        .ok_or(crate::error::AppError::AsrAuth)?;

    let resp = provider.transcribe(wav_bytes, asr_config).await?;
    Ok(resp.text)
}

/// Run LLM processing for non-raw modes.
pub async fn process_with_llm(
    text: &str,
    mode: &crate::llm::r#trait::OutputMode,
    llm_registry: &crate::llm::LlmRegistry,
    llm_config: &crate::config::schema::LlmConfig,
    license_store: &crate::license::LicenseStore,
) -> Result<String, crate::error::AppError> {
    // Gate: non-raw modes require activation
    if mode.requires_llm() && !license_store.is_unlocked() {
        return Err(crate::error::AppError::LicenseRequired);
    }

    let provider = llm_registry
        .get(&llm_config.provider)
        .ok_or(crate::error::AppError::LlmAuth)?;

    let system_prompt = crate::llm::prompts::render_prompt(mode, text);

    let result = provider
        .chat(&system_prompt, text, &llm_config.api_key, &llm_config.base_url, &llm_config.model)
        .await?;

    // Polish mode: validate output length
    if *mode == crate::llm::r#trait::OutputMode::Polish {
        Ok(crate::llm::validate_polish_output(text, &result))
    } else {
        Ok(result)
    }
}
