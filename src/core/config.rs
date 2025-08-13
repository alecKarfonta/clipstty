//! Configuration management for STT Clippy.

use crate::{
    DEFAULT_CLIPBOARD_CAPACITY, DEFAULT_HISTORY_HOTKEY, DEFAULT_HOTKEY, DEFAULT_SAMPLE_RATE,
    DEFAULT_STT_MODEL, DEFAULT_VAD_SENSITIVITY, DEFAULT_VAD_TIMEOUT, MAX_CLIPBOARD_HISTORY,
    SUPPORTED_LANGUAGES, SUPPORTED_STT_MODELS,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Audio configuration
    #[serde(default)]
    pub audio: AudioConfig,

    /// STT configuration
    #[serde(default)]
    pub stt: STTConfig,

    /// Clipboard configuration
    #[serde(default)]
    pub clipboard: ClipboardConfig,

    /// Hotkey configuration
    #[serde(default)]
    pub hotkeys: HotkeyConfig,

    /// Paste configuration
    #[serde(default)]
    pub paste: PasteConfig,

    /// Privacy configuration
    #[serde(default)]
    pub privacy: PrivacyConfig,

    /// UI configuration
    #[serde(default)]
    pub ui: UIConfig,
}

/// Audio configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AudioConfig {
    /// Audio sample rate
    #[serde(default = "default_sample_rate")]
    pub sample_rate: u32,

    /// Number of audio channels
    #[serde(default = "default_channels")]
    pub channels: u16,

    /// Voice Activity Detection sensitivity
    #[serde(default = "default_vad_sensitivity")]
    pub vad_sensitivity: f32,

    /// VAD timeout in milliseconds
    #[serde(default = "default_vad_timeout")]
    pub vad_timeout: u64,

    /// Enable noise reduction
    #[serde(default = "default_noise_reduction")]
    pub noise_reduction: bool,

    /// Audio device name (empty for default)
    #[serde(default)]
    pub device_name: String,
}

/// STT configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct STTConfig {
    /// STT backend to use
    #[serde(default = "default_stt_backend")]
    pub backend: String,

    /// STT model size
    #[serde(default = "default_stt_model")]
    pub model_size: String,

    /// Language for STT (empty for auto-detection)
    #[serde(default)]
    pub language: String,

    /// Enable punctuation
    #[serde(default = "default_enable_punctuation")]
    pub enable_punctuation: bool,

    /// Enable capitalization
    #[serde(default = "default_enable_capitalization")]
    pub enable_capitalization: bool,

    /// API key for cloud STT services
    #[serde(default)]
    pub api_key: String,

    /// API endpoint for cloud STT services
    #[serde(default)]
    pub api_endpoint: String,
}

/// Clipboard configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClipboardConfig {
    /// Maximum number of clipboard history items
    #[serde(default = "default_clipboard_capacity")]
    pub max_history: usize,

    /// Automatically save clipboard items
    #[serde(default = "default_auto_save")]
    pub auto_save: bool,

    /// Enable clipboard monitoring
    #[serde(default = "default_monitor_clipboard")]
    pub monitor_clipboard: bool,

    /// Maximum clipboard item size in bytes
    #[serde(default = "default_max_item_size")]
    pub max_item_size: usize,
}

/// Hotkey configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HotkeyConfig {
    /// Primary STT activation hotkey
    #[serde(default = "default_primary_hotkey")]
    pub primary: String,

    /// History palette hotkey
    #[serde(default = "default_history_hotkey")]
    pub history_palette: String,

    /// Quick access hotkeys for recent clips
    #[serde(default = "default_quick_access_hotkeys")]
    pub quick_access: Vec<String>,

    /// Enable global hotkeys
    #[serde(default = "default_enable_global_hotkeys")]
    pub enable_global: bool,
}

/// Paste configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PasteConfig {
    /// Paste mode
    #[serde(default = "default_paste_mode")]
    pub mode: PasteMode,

    /// Fallback method when paste injection fails
    #[serde(default = "default_fallback_method")]
    pub fallback: FallbackMethod,

    /// Restore previous clipboard after paste
    #[serde(default = "default_restore_clipboard")]
    pub restore_clipboard: bool,

    /// Delay before paste injection in milliseconds
    #[serde(default = "default_paste_delay")]
    pub delay: u64,
}

/// Privacy configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PrivacyConfig {
    /// Data retention period (e.g., "30d", "1y")
    #[serde(default = "default_data_retention")]
    pub data_retention: String,

    /// Automatically expire old data
    #[serde(default = "default_auto_expiry")]
    pub auto_expiry: bool,

    /// List of sensitive applications to exclude
    #[serde(default)]
    pub sensitive_apps: Vec<String>,

    /// Encrypt clipboard history at rest
    #[serde(default = "default_encrypt_storage")]
    pub encrypt_storage: bool,

    /// Enable usage analytics (opt-in)
    #[serde(default = "default_enable_analytics")]
    pub enable_analytics: bool,
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UIConfig {
    /// Show system tray icon
    #[serde(default = "default_show_tray")]
    pub show_tray: bool,

    /// Start minimized to tray
    #[serde(default = "default_start_minimized")]
    pub start_minimized: bool,

    /// Theme (light, dark, auto)
    #[serde(default = "default_theme")]
    pub theme: String,

    /// Enable notifications
    #[serde(default = "default_enable_notifications")]
    pub enable_notifications: bool,

    /// Notification duration in seconds
    #[serde(default = "default_notification_duration")]
    pub notification_duration: u64,
}

/// Paste mode enumeration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum PasteMode {
    /// Copy to clipboard only
    Clipboard,
    /// Paste at cursor only
    Paste,
    /// Both copy and paste
    #[default]
    Both,
}

/// Fallback method enumeration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum FallbackMethod {
    /// Use clipboard only
    #[default]
    Clipboard,
    /// Show manual paste instructions
    Manual,
    /// Show error notification
    Error,
}

impl Config {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self {
            audio: AudioConfig::new(),
            stt: STTConfig::new(),
            clipboard: ClipboardConfig::new(),
            hotkeys: HotkeyConfig::new(),
            paste: PasteConfig::new(),
            privacy: PrivacyConfig::new(),
            ui: UIConfig::new(),
        }
    }

    /// Load configuration from file
    pub fn from_file(path: &PathBuf) -> crate::Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::core::error::ConfigError::Load(e.to_string()))?;

        let config: Config = toml::from_str(&content)
            .map_err(|e| crate::core::error::ConfigError::Parse(e.to_string()))?;

        config.validate()?;
        Ok(config)
    }

    /// Save configuration to file
    pub fn save_to_file(&self, path: &PathBuf) -> crate::Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| crate::core::error::ConfigError::Parse(e.to_string()))?;

        std::fs::write(path, content)
            .map_err(|e| crate::core::error::ConfigError::Load(e.to_string()))?;

        Ok(())
    }

    /// Validate configuration values
    pub fn validate(&self) -> crate::Result<()> {
        // Validate STT model
        if !SUPPORTED_STT_MODELS.contains(&self.stt.model_size.as_str()) {
            return Err(crate::core::error::ConfigError::InvalidValue(format!(
                "Unsupported STT model: {}",
                self.stt.model_size
            ))
            .into());
        }

        // Validate language if specified
        if !self.stt.language.is_empty()
            && !SUPPORTED_LANGUAGES.contains(&self.stt.language.as_str())
        {
            return Err(crate::core::error::ConfigError::InvalidValue(format!(
                "Unsupported language: {}",
                self.stt.language
            ))
            .into());
        }

        // Validate clipboard capacity
        if self.clipboard.max_history > MAX_CLIPBOARD_HISTORY {
            return Err(crate::core::error::ConfigError::InvalidValue(format!(
                "Clipboard history capacity too large: {}",
                self.clipboard.max_history
            ))
            .into());
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

// Default value functions
fn default_sample_rate() -> u32 {
    DEFAULT_SAMPLE_RATE
}
fn default_channels() -> u16 {
    1
}
fn default_vad_sensitivity() -> f32 {
    DEFAULT_VAD_SENSITIVITY
}
fn default_vad_timeout() -> u64 {
    DEFAULT_VAD_TIMEOUT
}
fn default_noise_reduction() -> bool {
    true
}
fn default_stt_backend() -> String {
    "local".to_string()
}
fn default_stt_model() -> String {
    DEFAULT_STT_MODEL.to_string()
}
fn default_enable_punctuation() -> bool {
    true
}
fn default_enable_capitalization() -> bool {
    true
}
fn default_clipboard_capacity() -> usize {
    DEFAULT_CLIPBOARD_CAPACITY
}
fn default_auto_save() -> bool {
    true
}
fn default_monitor_clipboard() -> bool {
    true
}
fn default_max_item_size() -> usize {
    10 * 1024 * 1024
} // 10MB
fn default_primary_hotkey() -> String {
    DEFAULT_HOTKEY.to_string()
}
fn default_history_hotkey() -> String {
    DEFAULT_HISTORY_HOTKEY.to_string()
}
fn default_quick_access_hotkeys() -> Vec<String> {
    vec![
        "Alt+1".to_string(),
        "Alt+2".to_string(),
        "Alt+3".to_string(),
    ]
}
fn default_enable_global_hotkeys() -> bool {
    true
}
fn default_paste_mode() -> PasteMode {
    PasteMode::Both
}
fn default_fallback_method() -> FallbackMethod {
    FallbackMethod::Clipboard
}
fn default_restore_clipboard() -> bool {
    true
}
fn default_paste_delay() -> u64 {
    100
}
fn default_data_retention() -> String {
    "30d".to_string()
}
fn default_auto_expiry() -> bool {
    true
}
fn default_encrypt_storage() -> bool {
    true
}
fn default_enable_analytics() -> bool {
    false
}
fn default_show_tray() -> bool {
    true
}
fn default_start_minimized() -> bool {
    false
}
fn default_theme() -> String {
    "auto".to_string()
}
fn default_enable_notifications() -> bool {
    true
}
fn default_notification_duration() -> u64 {
    5
}

impl AudioConfig {
    pub fn new() -> Self {
        Self {
            sample_rate: DEFAULT_SAMPLE_RATE,
            channels: 1,
            vad_sensitivity: DEFAULT_VAD_SENSITIVITY,
            vad_timeout: DEFAULT_VAD_TIMEOUT,
            noise_reduction: true,
            device_name: String::new(),
        }
    }
}

impl STTConfig {
    pub fn new() -> Self {
        Self {
            backend: "local".to_string(),
            model_size: DEFAULT_STT_MODEL.to_string(),
            language: String::new(),
            enable_punctuation: true,
            enable_capitalization: true,
            api_key: String::new(),
            api_endpoint: String::new(),
        }
    }
}

impl ClipboardConfig {
    pub fn new() -> Self {
        Self {
            max_history: DEFAULT_CLIPBOARD_CAPACITY,
            auto_save: true,
            monitor_clipboard: true,
            max_item_size: 10 * 1024 * 1024,
        }
    }
}

impl HotkeyConfig {
    pub fn new() -> Self {
        Self {
            primary: DEFAULT_HOTKEY.to_string(),
            history_palette: DEFAULT_HISTORY_HOTKEY.to_string(),
            quick_access: vec![
                "Alt+1".to_string(),
                "Alt+2".to_string(),
                "Alt+3".to_string(),
            ],
            enable_global: true,
        }
    }
}

impl PasteConfig {
    pub fn new() -> Self {
        Self {
            mode: PasteMode::Both,
            fallback: FallbackMethod::Clipboard,
            restore_clipboard: true,
            delay: 100,
        }
    }
}

impl PrivacyConfig {
    pub fn new() -> Self {
        Self {
            data_retention: "30d".to_string(),
            auto_expiry: true,
            sensitive_apps: Vec::new(),
            encrypt_storage: true,
            enable_analytics: false,
        }
    }
}

impl UIConfig {
    pub fn new() -> Self {
        Self {
            show_tray: true,
            start_minimized: false,
            theme: "auto".to_string(),
            enable_notifications: true,
            notification_duration: 5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::new();
        assert_eq!(config.audio.sample_rate, DEFAULT_SAMPLE_RATE);
        assert_eq!(config.stt.model_size, DEFAULT_STT_MODEL);
        assert_eq!(config.clipboard.max_history, DEFAULT_CLIPBOARD_CAPACITY);
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::new();

        // Valid config should pass validation
        assert!(config.validate().is_ok());

        // Invalid STT model should fail validation
        config.stt.model_size = "invalid".to_string();
        assert!(config.validate().is_err());

        // Reset to valid value
        config.stt.model_size = DEFAULT_STT_MODEL.to_string();

        // Invalid language should fail validation
        config.stt.language = "invalid".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::new();
        let toml_string = toml::to_string(&config).unwrap();
        let parsed_config: Config = toml::from_str(&toml_string).unwrap();

        assert_eq!(config.audio.sample_rate, parsed_config.audio.sample_rate);
        assert_eq!(config.stt.model_size, parsed_config.stt.model_size);
    }
}
