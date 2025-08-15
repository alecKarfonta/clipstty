//! Speech-to-Text service for processing audio and generating transcriptions.

use crate::{core::config::STTConfig, core::types::*, Result, SUPPORTED_STT_MODELS};
use std::time::Instant;
use tracing::{info, warn, debug, error};

#[cfg(feature = "local-stt")]
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperState};

/// Backend interface for STT engines
pub trait STTBackend {
    fn transcribe(&mut self, audio: &[AudioSample], cfg: &STTConfig, model: &str) -> Result<STTResult>;
}

#[cfg(feature = "local-stt")]
struct LocalWhisperBackend {
    ctx: WhisperContext,
    state: WhisperState,
    model_path: String,
}

#[cfg(feature = "local-stt")]
impl LocalWhisperBackend {
    fn new(cfg: &STTConfig) -> Result<Self> {
        // Styled log helpers
        const C_CLASS: &str = "\x1b[35m"; // magenta
        const C_FUNC: &str = "\x1b[36m"; // cyan
        const C_VAR: &str = "\x1b[33m"; // yellow
        const C_VAL: &str = "\x1b[32m"; // green
        const C_RESET: &str = "\x1b[0m";

        let model_path = std::env::var("WHISPER_MODEL_PATH").map_err(|_| {
            crate::core::error::STTError::ModelNotFound(
                "Environment variable WHISPER_MODEL_PATH not set".to_string(),
            )
        })?;
        let model_path = model_path.trim().to_string();
        if model_path.is_empty() {
            return Err(crate::core::error::STTError::ModelNotFound(
                "WHISPER_MODEL_PATH is empty".to_string(),
            ).into());
        }
        let meta = std::fs::metadata(&model_path)
            .map_err(|e| crate::core::error::STTError::ModelLoad(format!("Cannot access model '{}': {}", model_path, e)))?;
        if !meta.is_file() || meta.len() == 0 {
            return Err(crate::core::error::STTError::ModelLoad(format!(
                "Model path '{}' is not a regular non-empty file",
                model_path
            )).into());
        }

        let mut ctx_params = WhisperContextParameters::default();
        let use_gpu_env = std::env::var("WHISPER_USE_GPU").ok();
        let use_gpu = match use_gpu_env.as_deref() {
            Some("0") | Some("false") | Some("False") => false,
            Some(_) => true,
            None => {
                #[cfg(target_os = "macos")] { true }
                #[cfg(not(target_os = "macos"))] { false }
            }
        };
        if use_gpu { ctx_params.use_gpu(true); }

        info!(target: "stt", "[{}LocalWhisperBackend{}].{}new{} {}model_path{}={}{}{}, {}use_gpu{}={}{}{}",
            C_CLASS, C_RESET, C_FUNC, C_RESET,
            C_VAR, C_RESET, C_VAL, model_path, C_RESET,
            C_VAR, C_RESET, C_VAL, use_gpu, C_RESET,
        );

        let ctx = WhisperContext::new_with_params(&model_path, ctx_params).map_err(|e| {
            crate::core::error::STTError::ModelLoad(format!(
                "Failed to load model at '{}': {}",
                model_path, e
            ))
        })?;
        let state = ctx.create_state().map_err(|e| {
            crate::core::error::STTError::BackendInit(format!(
                "Failed to create whisper state: {}",
                e
            ))
        })?;

        Ok(Self { ctx, state, model_path })
    }
}

#[cfg(feature = "local-stt")]
impl STTBackend for LocalWhisperBackend {
    fn transcribe(&mut self, audio: &[AudioSample], cfg: &STTConfig, model: &str) -> Result<STTResult> {
        let start_time = Instant::now();

        // Styled log helpers
        const C_CLASS: &str = "\x1b[35m"; // magenta
        const C_FUNC: &str = "\x1b[36m"; // cyan
        const C_VAR: &str = "\x1b[33m"; // yellow
        const C_VAL: &str = "\x1b[32m"; // green
        const C_RESET: &str = "\x1b[0m";

        debug!(target: "stt", "[{}LocalWhisperBackend{}].{}transcribe{} {}model_path{}={}{}{}, {}model{}={}{}{}",
            C_CLASS, C_RESET, C_FUNC, C_RESET,
            C_VAR, C_RESET, C_VAL, self.model_path, C_RESET,
            C_VAR, C_RESET, C_VAL, model, C_RESET,
        );

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        let threads = std::env::var("WHISPER_THREADS")
            .ok()
            .and_then(|v| v.parse::<i32>().ok())
            .filter(|&n| n > 0)
            .unwrap_or(num_cpus::get() as i32);
        params.set_n_threads(threads);
        debug!(target: "stt", "[{}STTService{}].{}transcribe{} {}threads{}={}{}{}, {}language{}={}{}{}",
            C_CLASS, C_RESET, C_FUNC, C_RESET,
            C_VAR, C_RESET, C_VAL, threads, C_RESET,
            C_VAR, C_RESET, C_VAL, if cfg.language.is_empty() { "auto" } else { &cfg.language }, C_RESET,
        );
        params.set_translate(false);
        // Force English by default to avoid language auto-detection overhead unless overridden
        let lang = if cfg.language.is_empty() { "en" } else { cfg.language.as_str() };
        params.set_language(Some(lang));

        debug!(target: "stt", "[{}STTService{}].{}transcribe{} {}audio_len{}={}{}{} samples",
            C_CLASS, C_RESET, C_FUNC, C_RESET,
            C_VAR, C_RESET, C_VAL, audio.len(), C_RESET
        );
        if let Err(e) = self.state.full(params, audio) {
            error!(target: "stt", "[{}STTService{}].{}transcribe{} {}error{}={}{}{}",
                C_CLASS, C_RESET, C_FUNC, C_RESET,
                C_VAR, C_RESET, C_VAL, e, C_RESET
            );
            return Err(crate::core::error::STTError::Processing(format!("Whisper processing failed: {}", e)).into());
        }

        let num_segments = self.state
            .full_n_segments()
            .map_err(|e| crate::core::error::STTError::Processing(format!(
                "Failed to read segments: {}",
                e
            )))?;

        let mut text = String::new();
        for i in 0..num_segments {
            let seg = self.state
                .full_get_segment_text(i)
                .map_err(|e| crate::core::error::STTError::Processing(format!(
                    "Failed to read segment {}: {}",
                    i, e
                )))?;
            text.push_str(&seg);
        }

        let processing_ms = start_time.elapsed().as_millis() as u64;
        let audio_sec = (audio.len() as f64) / 16000.0_f64;
        let wall_sec = (processing_ms as f64) / 1000.0_f64;
        let rtf = if audio_sec > 0.0 { wall_sec / audio_sec } else { 0.0 };
        debug!(target: "stt", "[{}STTService{}].{}transcribe{} completed {}backend{}={}{}{}, {}model{}={}{}{}, {}duration_ms{}={}{}{}, {}audio_s{}={}{}{}, {}wall_s{}={}{}{}, {}rtf{}={}{}{}",
            C_CLASS, C_RESET, C_FUNC, C_RESET,
            C_VAR, C_RESET, C_VAL, "local", C_RESET,
            C_VAR, C_RESET, C_VAL, model, C_RESET,
            C_VAR, C_RESET, C_VAL, processing_ms, C_RESET,
            C_VAR, C_RESET, C_VAL, format!("{:.3}", audio_sec), C_RESET,
            C_VAR, C_RESET, C_VAL, format!("{:.3}", wall_sec), C_RESET,
            C_VAR, C_RESET, C_VAL, format!("{:.3}", rtf), C_RESET,
        );

        Ok(STTResult::new(text, 1.0, model.to_string(), "local".to_string()).with_processing_time(processing_ms))
    }
}

/// STT service for managing speech-to-text processing
pub struct STTService {
    config: STTConfig,
    selected_model: String,
    backend: String,
    #[cfg(feature = "local-stt")]
    local_backend: Option<LocalWhisperBackend>,
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
            #[cfg(feature = "local-stt")]
            local_backend: None,
        })
    }

    /// Process audio and generate transcription
    pub fn transcribe(&mut self, audio: &[AudioSample]) -> Result<STTResult> {
        match self.backend.as_str() {
            "local" => {
                #[cfg(feature = "local-stt")]
                {
                    if self.local_backend.is_none() {
                        self.local_backend = Some(LocalWhisperBackend::new(&self.config)?);
                    }
                    if let Some(b) = self.local_backend.as_mut() {
                        b.transcribe(audio, &self.config, &self.selected_model)
                    } else {
                        Err(crate::core::error::STTError::BackendInit("local backend init failed".to_string()).into())
                    }
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
