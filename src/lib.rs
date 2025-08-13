//! STT Clippy - Speech-to-Text with Smart Clipboard Management
//!
//! This library provides the core functionality for the STT Clippy desktop application,
//! including speech-to-text processing, clipboard management, and platform integration.

pub mod core;
pub mod platform;
pub mod services;
pub mod ui;

pub use core::config::Config;
pub use core::error::STTClippyError;
pub use services::clipboard::ClipboardService;
pub use services::hotkey::HotkeyService;
pub use services::stt::STTService;
use std::path::PathBuf;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::prelude::*;
use once_cell::sync::OnceCell;

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
    "en", "es", "fr", "de", "it", "pt", "ru", "ja", "ko", "zh", "ar", "hi", "nl", "sv", "da", "no",
    "fi", "pl", "tr", "uk",
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
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Initialize the library
///     // ... rest of application
///     Ok(())
/// }
/// ```
pub fn init(config_path: Option<&str>, log_level: Option<&str>) -> Result<()> {
    // Initialize logging with rotating file and stdout
    let log_level = log_level.unwrap_or("info");
    let appender_dir = default_log_dir()?;
    std::fs::create_dir_all(&appender_dir)?;
    let file_appender = tracing_appender::rolling::daily(&appender_dir, LOG_FILE);
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_timer(UtcTime::rfc_3339())
        .with_target(false)
        .with_ansi(true);
    let file_layer = tracing_subscriber::fmt::layer()
        .with_timer(UtcTime::rfc_3339())
        .with_target(false)
        .with_ansi(false)
        .with_writer(non_blocking);

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(format!("stt_clippy={log_level}")))
        .with(stdout_layer)
        .with(file_layer)
        .init();

    tracing::info!("Initializing STT Clippy v{}", VERSION);
    tracing::info!("Application: {}", APP_NAME);

    // Load configuration
    let resolved_path: PathBuf = match config_path {
        Some(path) => PathBuf::from(path),
        None => default_config_path()?,
    };

    let config = match std::fs::metadata(&resolved_path) {
        Ok(_) => {
            tracing::info!("Loading configuration from: {}", resolved_path.display());
            Config::from_file(&resolved_path)?
        }
        Err(_) => {
            tracing::info!(
                "No configuration found. Creating default at: {}",
                resolved_path.display()
            );
            let cfg = Config::new();
            if let Some(parent) = resolved_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            cfg.save_to_file(&resolved_path)?;
            cfg
        }
    };

    set_global_config(config);

    // Initialize platform-specific components
    platform::init()?;

    tracing::info!("STT Clippy initialization complete");
    Ok(())
}

fn default_config_path() -> Result<PathBuf> {
    #[allow(unused_mut)]
    let mut path: PathBuf;
    #[cfg(not(target_os = "linux"))]
    {
        let dirs = directories::ProjectDirs::from("com", "sttclippy", "stt-clippy")
            .ok_or_else(|| crate::core::error::ConfigError::FileNotFound("config dir".into()))?;
        path = dirs.config_dir().to_path_buf();
    }
    #[cfg(target_os = "linux")]
    {
        let dirs = directories::BaseDirs::new()
            .ok_or_else(|| crate::core::error::ConfigError::FileNotFound("base dir".into()))?;
        let config_home = dirs.config_dir();
        path = config_home.join("stt-clippy");
    }
    Ok(path.join(CONFIG_FILE))
}

fn default_log_dir() -> Result<PathBuf> {
    #[allow(unused_mut)]
    let mut path: PathBuf;
    #[cfg(not(target_os = "linux"))]
    {
        let dirs = directories::ProjectDirs::from("com", "sttclippy", "stt-clippy")
            .ok_or_else(|| crate::core::error::ConfigError::FileNotFound("log dir".into()))?;
        path = dirs.data_dir().to_path_buf();
    }
    #[cfg(target_os = "linux")]
    {
        let dirs = directories::BaseDirs::new()
            .ok_or_else(|| crate::core::error::ConfigError::FileNotFound("base dir".into()))?;
        let data_home = dirs.data_dir();
        path = data_home.join("stt-clippy");
    }
    Ok(path)
}

static GLOBAL_CONFIG: OnceCell<core::config::Config> = OnceCell::new();

pub fn get_config() -> &'static core::config::Config {
    GLOBAL_CONFIG.get().expect("config not initialized")
}

fn set_global_config(cfg: core::config::Config) {
    let _ = GLOBAL_CONFIG.set(cfg);
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
