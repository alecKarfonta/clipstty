//! Error types and error handling for STT Clippy.

use thiserror::Error;

/// Main error type for STT Clippy
#[derive(Error, Debug)]
pub enum STTClippyError {
    /// Audio-related errors
    #[error("Audio error: {0}")]
    Audio(#[from] AudioError),

    /// STT-related errors
    #[error("STT error: {0}")]
    STT(#[from] STTError),

    /// Clipboard-related errors
    #[error("Clipboard error: {0}")]
    Clipboard(#[from] ClipboardError),

    /// Hotkey-related errors
    #[error("Hotkey error: {0}")]
    Hotkey(#[from] HotkeyError),

    /// Paste injection errors
    #[error("Paste error: {0}")]
    Paste(#[from] PasteError),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// Platform-specific errors
    #[error("Platform error: {0}")]
    Platform(#[from] PlatformError),

    /// Database errors
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Other errors
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Audio-related errors
#[derive(Error, Debug)]
pub enum AudioError {
    #[error("Failed to initialize audio device: {0}")]
    DeviceInit(String),
    
    #[error("Failed to start audio capture: {0}")]
    CaptureStart(String),
    
    #[error("Failed to stop audio capture: {0}")]
    CaptureStop(String),
    
    #[error("Audio device not found: {0}")]
    DeviceNotFound(String),
    
    #[error("Unsupported audio format: {0}")]
    UnsupportedFormat(String),
    
    #[error("Audio buffer overflow")]
    BufferOverflow,
    
    #[error("VAD initialization failed: {0}")]
    VADInit(String),
}

/// STT-related errors
#[derive(Error, Debug)]
pub enum STTError {
    #[error("Failed to load STT model: {0}")]
    ModelLoad(String),
    
    #[error("Failed to initialize STT backend: {0}")]
    BackendInit(String),
    
    #[error("STT processing failed: {0}")]
    Processing(String),
    
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),
    
    #[error("STT timeout")]
    Timeout,
}

/// Clipboard-related errors
#[derive(Error, Debug)]
pub enum ClipboardError {
    #[error("Failed to read clipboard: {0}")]
    Read(String),
    
    #[error("Failed to write clipboard: {0}")]
    Write(String),
    
    #[error("Clipboard format not supported: {0}")]
    UnsupportedFormat(String),
    
    #[error("Clipboard item too large: {0} bytes")]
    ItemTooLarge(usize),
    
    #[error("Failed to access clipboard history: {0}")]
    HistoryAccess(String),
}

/// Hotkey-related errors
#[derive(Error, Debug)]
pub enum HotkeyError {
    #[error("Failed to register hotkey: {0}")]
    Registration(String),
    
    #[error("Failed to unregister hotkey: {0}")]
    Unregistration(String),
    
    #[error("Hotkey conflict: {0}")]
    Conflict(String),
    
    #[error("Unsupported hotkey combination: {0}")]
    UnsupportedCombination(String),
}

/// Paste injection errors
#[derive(Error, Debug)]
pub enum PasteError {
    #[error("Failed to inject text: {0}")]
    Injection(String),
    
    #[error("Paste injection not supported on this platform")]
    NotSupported,
    
    #[error("Accessibility permissions required")]
    PermissionsRequired,
    
    #[error("Target application not found")]
    TargetNotFound,
}

/// Configuration errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to load configuration file: {0}")]
    Load(String),
    
    #[error("Failed to parse configuration: {0}")]
    Parse(String),
    
    #[error("Invalid configuration value: {0}")]
    InvalidValue(String),
    
    #[error("Configuration file not found: {0}")]
    FileNotFound(String),
}

/// Platform-specific errors
#[derive(Error, Debug)]
pub enum PlatformError {
    #[error("Platform not supported: {0}")]
    NotSupported(String),
    
    #[error("Platform initialization failed: {0}")]
    Init(String),
    
    #[error("Platform cleanup failed: {0}")]
    Cleanup(String),
    
    #[error("Platform-specific operation failed: {0}")]
    Operation(String),
}

/// Database errors
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Failed to connect to database: {0}")]
    Connection(String),
    
    #[error("Failed to execute query: {0}")]
    Query(String),
    
    #[error("Failed to create table: {0}")]
    TableCreation(String),
    
    #[error("Failed to insert data: {0}")]
    Insert(String),
    
    #[error("Failed to update data: {0}")]
    Update(String),
    
    #[error("Failed to delete data: {0}")]
    Delete(String),
}

impl From<String> for STTClippyError {
    fn from(err: String) -> Self {
        STTClippyError::Unknown(err)
    }
}

impl From<&str> for STTClippyError {
    fn from(err: &str) -> Self {
        STTClippyError::Unknown(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let stt_error: STTClippyError = io_error.into();
        
        match stt_error {
            STTClippyError::Io(_) => (),
            _ => panic!("Expected Io error"),
        }
    }

    #[test]
    fn test_string_error_conversion() {
        let error: STTClippyError = "Test error".into();
        
        match error {
            STTClippyError::Unknown(msg) => assert_eq!(msg, "Test error"),
            _ => panic!("Expected Unknown error"),
        }
    }
}
