//! STT Clippy - Speech-to-Text with Smart Clipboard Management
//!
//! This library provides the core functionality for the STT Clippy desktop application,
//! including speech-to-text processing, clipboard management, and platform integration.

pub mod core;
pub mod services;
pub mod platform;
pub mod ui;

pub use core::config::Config;
pub use core::error::STTClippyError;
pub use services::stt::STTService;
pub use services::clipboard::ClipboardService;
pub use services::hotkey::HotkeyService;

/// Application version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application name
pub const APP_NAME: &str = "STT Clippy";

/// Default configuration file name
pub const CONFIG_FILE: &str = "stt-clippy.toml";

/// Default log file name
pub const LOG_FILE: &str = "stt-clippy.log";

/// Default clipboard history capacity
pub const DEFAULT_CLIPBOARD_CAPACITY: usize = 500;

/// Default STT model size
pub const DEFAULT_STT_MODEL: &str = "base";

/// Default hotkey combination
pub const DEFAULT_HOTKEY: &str = "Ctrl+Alt+S";

/// Default history palette hotkey
pub const DEFAULT_HISTORY_HOTKEY: &str = "Ctrl+Alt+H";

/// Default audio sample rate
pub const DEFAULT_SAMPLE_RATE: u32 = 16000;

/// Default VAD sensitivity
pub const DEFAULT_VAD_SENSITIVITY: f32 = 0.5;

/// Default VAD timeout in milliseconds
pub const DEFAULT_VAD_TIMEOUT: u64 = 2000;

/// Maximum audio buffer size in samples
pub const MAX_AUDIO_BUFFER_SIZE: usize = 1024 * 1024; // 1MB

/// Maximum clipboard item size in bytes
pub const MAX_CLIPBOARD_ITEM_SIZE: usize = 10 * 1024 * 1024; // 10MB

/// Maximum clipboard history items
pub const MAX_CLIPBOARD_HISTORY: usize = 10000;

/// Minimum supported Rust version
pub const MIN_RUST_VERSION: &str = "1.70.0";

/// Supported audio formats
pub const SUPPORTED_AUDIO_FORMATS: &[&str] = &["wav", "flac", "pcm"];

/// Supported STT models
pub const SUPPORTED_STT_MODELS: &[&str] = &["tiny", "base", "small", "medium", "large"];

/// Supported languages (ISO 639-1 codes)
pub const SUPPORTED_LANGUAGES: &[&str] = &[
    "en", "es", "fr", "de", "it", "pt", "ru", "ja", "ko", "zh",
    "ar", "hi", "nl", "sv", "da", "no", "fi", "pl", "tr", "uk"
];

/// Application result type
pub type Result<T> = std::result::Result<T, crate::core::error::STTClippyError>;

/// Initialize the STT Clippy library
///
/// This function should be called before using any other functionality.
/// It sets up logging, configuration, and platform-specific initialization.
///
/// # Arguments
///
/// * `config_path` - Optional path to configuration file
/// * `log_level` - Optional log level (default: "info")
///
/// # Returns
///
/// Returns a result indicating success or failure
///
/// # Examples
///
/// ```rust
/// use stt_clippy;
///
/// fn main() -> stt_clippy::Result<()> {
///     stt_clippy::init(None, None)?;
///     // ... rest of application
///     Ok(())
/// }
/// ```
pub fn init(config_path: Option<&str>, log_level: Option<&str>) -> Result<()> {
    // Initialize logging
    let log_level = log_level.unwrap_or("info");
    tracing_subscriber::fmt()
        .with_env_filter(format!("stt_clippy={}", log_level))
        .init();

    tracing::info!("Initializing STT Clippy v{}", VERSION);
    tracing::info!("Application: {}", APP_NAME);

    // Load configuration
    if let Some(path) = config_path {
        tracing::info!("Loading configuration from: {}", path);
        // TODO: Load configuration from file
    } else {
        tracing::info!("Using default configuration");
        // TODO: Load default configuration
    }

    // Initialize platform-specific components
    platform::init()?;

    tracing::info!("STT Clippy initialization complete");
    Ok(())
}

/// Cleanup and shutdown the STT Clippy library
///
/// This function should be called when shutting down the application
/// to ensure proper cleanup of resources.
///
/// # Returns
///
/// Returns a result indicating success or failure
pub fn cleanup() -> Result<()> {
    tracing::info!("Cleaning up STT Clippy");

    // Cleanup platform-specific components
    platform::cleanup()?;

    tracing::info!("STT Clippy cleanup complete");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
        assert_eq!(APP_NAME, "STT Clippy");
        assert_eq!(DEFAULT_CLIPBOARD_CAPACITY, 500);
        assert_eq!(DEFAULT_STT_MODEL, "base");
        assert_eq!(DEFAULT_HOTKEY, "Ctrl+Alt+S");
    }

    #[test]
    fn test_supported_models() {
        assert!(SUPPORTED_STT_MODELS.contains(&"base"));
        assert!(SUPPORTED_STT_MODELS.contains(&"large"));
        assert!(!SUPPORTED_STT_MODELS.contains(&"invalid"));
    }

    #[test]
    fn test_supported_languages() {
        assert!(SUPPORTED_LANGUAGES.contains(&"en"));
        assert!(SUPPORTED_LANGUAGES.contains(&"es"));
        assert!(!SUPPORTED_LANGUAGES.contains(&"invalid"));
    }
}
