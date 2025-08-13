//! Speech-to-Text service for processing audio and generating transcriptions.

use crate::{core::config::STTConfig, core::types::*, Result, SUPPORTED_STT_MODELS};
use std::time::Instant;
use tracing::{info, warn};

#[cfg(feature = "local-stt")]
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

/// Backend interface for STT engines
pub trait STTBackend {
    fn transcribe(&mut self, audio: &[AudioSample], cfg: &STTConfig, model: &str) -> Result<STTResult>;
}

#[cfg(feature = "local-stt")]
struct LocalWhisperBackend;

#[cfg(feature = "local-stt")]
impl STTBackend for LocalWhisperBackend {
    fn transcribe(&mut self, audio: &[AudioSample], cfg: &STTConfig, model: &str) -> Result<STTResult> {
        let start_time = Instant::now();

        let model_path = std::env::var("WHISPER_MODEL_PATH").map_err(|_| {
            crate::core::error::STTError::ModelNotFound(
                "Environment variable WHISPER_MODEL_PATH not set".to_string(),
            )
        })?;

        let ctx_params = WhisperContextParameters::default();
        let ctx = WhisperContext::new_with_params(&model_path, ctx_params).map_err(|e| {
            crate::core::error::STTError::ModelLoad(format!(
                "Failed to load model at '{}': {}",
                model_path, e
            ))
        })?;
        let mut state = ctx.create_state().map_err(|e| {
            crate::core::error::STTError::BackendInit(format!(
                "Failed to create whisper state: {}",
                e
            ))
        })?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_n_threads(num_cpus::get() as i32);
        params.set_translate(false);
        params.set_language(if cfg.language.is_empty() { None } else { Some(cfg.language.as_str()) });

        state
            .full(params, audio)
            .map_err(|e| crate::core::error::STTError::Processing(format!(
                "Whisper processing failed: {}",
                e
            )))?;

        let num_segments = state
            .full_n_segments()
            .map_err(|e| crate::core::error::STTError::Processing(format!(
                "Failed to read segments: {}",
                e
            )))?;

        let mut text = String::new();
        for i in 0..num_segments {
            let seg = state
                .full_get_segment_text(i)
                .map_err(|e| crate::core::error::STTError::Processing(format!(
                    "Failed to read segment {}: {}",
                    i, e
                )))?;
            text.push_str(&seg);
        }

        let processing_ms = start_time.elapsed().as_millis() as u64;
        info!(backend = "local", model = %model, duration_ms = processing_ms, "Local transcription completed");

        Ok(STTResult::new(text, 1.0, model.to_string(), "local".to_string()).with_processing_time(processing_ms))
    }
}

/// STT service for managing speech-to-text processing
pub struct STTService {
    config: STTConfig,
    selected_model: String,
    backend: String,
}

impl STTService {
    /// Create a new STT service
    pub fn new() -> Result<Self> {
        let config = STTConfig::new();
        let selected_model = config.model_size.clone();
        let backend = config.backend.clone();
        info!(model = %selected_model, backend = %backend, "Initializing STTService with defaults");
        Ok(Self {
            config,
            selected_model,
            backend,
        })
    }

    /// Process audio and generate transcription
    pub fn transcribe(&mut self, audio: &[AudioSample]) -> Result<STTResult> {
        match self.backend.as_str() {
            "local" => {
                #[cfg(feature = "local-stt")]
                {
                    let mut backend = LocalWhisperBackend;
                    backend.transcribe(audio, &self.config, &self.selected_model)
                }
                #[cfg(not(feature = "local-stt"))]
                {
                    Err(crate::core::error::STTError::BackendInit(
                        "local backend requested but 'local-stt' feature is disabled".to_string(),
                    )
                    .into())
                }
            }
            other => Err(crate::core::error::STTError::BackendInit(format!(
                "Unsupported STT backend: {other}"
            ))
            .into()),
        }
    }

    /// Get available STT models
    pub fn get_models(&self) -> Result<Vec<STTModel>> {
        // Basic enumeration from supported list; file paths unknown at this stage
        let models = SUPPORTED_STT_MODELS
            .iter()
            .map(|size| STTModel {
                name: format!("whisper-{size}"),
                size: (*size).to_string(),
                file_path: String::new(),
                file_size: 0,
                languages: crate::SUPPORTED_LANGUAGES.iter().map(|s| s.to_string()).collect(),
                version: "1".to_string(),
                downloaded: false,
                download_progress: None,
            })
            .collect();
        Ok(models)
    }

    /// Select an STT model size (e.g., "tiny", "base") and remember it
    pub fn select_model(&mut self, model_size: &str) -> Result<()> {
        if !SUPPORTED_STT_MODELS.contains(&model_size) {
            return Err(crate::core::error::STTError::ModelNotFound(model_size.to_string()).into());
        }
        info!(model = %model_size, "STTService select_model called");
        self.selected_model = model_size.to_string();
        Ok(())
    }

    /// Update configuration and apply to service state
    pub fn apply_config(&mut self, cfg: STTConfig) -> Result<()> {
        if !SUPPORTED_STT_MODELS.contains(&cfg.model_size.as_str()) {
            return Err(crate::core::error::ConfigError::InvalidValue(format!(
                "Unsupported STT model: {}",
                cfg.model_size
            ))
            .into());
        }

        if !cfg.language.is_empty() && !crate::SUPPORTED_LANGUAGES.contains(&cfg.language.as_str()) {
            warn!(language = %cfg.language, "Unsupported language provided; continuing with value");
        }

        info!(backend = %cfg.backend, model = %cfg.model_size, "STTService apply_config called");
        self.backend = cfg.backend.clone();
        self.selected_model = cfg.model_size.clone();
        self.config = cfg;
        Ok(())
    }
}
