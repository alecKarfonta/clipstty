//! Common types and data structures for STT Clippy.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for clipboard items
pub type ClipboardItemId = Uuid;

/// Audio sample type
pub type AudioSample = f32;

/// Audio buffer type
pub type AudioBuffer = Vec<AudioSample>;

/// STT result with confidence and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct STTResult {
    /// Transcribed text
    pub text: String,

    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,

    /// Language detected (ISO 639-1 code)
    pub language: Option<String>,

    /// Timestamp when transcription completed
    pub timestamp: DateTime<Utc>,

    /// Processing time in milliseconds
    pub processing_time_ms: u64,

    /// Model used for transcription
    pub model: String,

    /// Backend used for transcription
    pub backend: String,
}

/// Partial STT result for streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialSTTResult {
    /// Partial transcribed text
    pub text: String,

    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,

    /// Is this the final result?
    pub is_final: bool,

    /// Timestamp when partial result was generated
    pub timestamp: DateTime<Utc>,
}

/// Clipboard item with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    /// Unique identifier
    pub id: ClipboardItemId,

    /// Content of the clipboard item
    pub content: ClipboardContent,

    /// Source of the clipboard item
    pub source: ClipboardSource,

    /// Timestamp when item was created
    pub created_at: DateTime<Utc>,

    /// Timestamp when item was last accessed
    pub accessed_at: DateTime<Utc>,

    /// Number of times this item was accessed
    pub access_count: u64,

    /// Tags for categorization
    pub tags: Vec<String>,

    /// Is this item pinned?
    pub pinned: bool,

    /// Application context when created
    pub app_context: Option<AppContext>,

    /// Size in bytes
    pub size_bytes: usize,
}

/// Clipboard content with format information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardContent {
    /// Text content (if any)
    pub text: Option<String>,

    /// HTML content (if any)
    pub html: Option<String>,

    /// Rich text content (if any)
    pub rtf: Option<String>,

    /// Image data (if any)
    pub image: Option<ImageData>,

    /// File paths (if any)
    pub files: Option<Vec<String>>,

    /// Custom formats
    pub custom: std::collections::HashMap<String, Vec<u8>>,
}

/// Image data representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageData {
    /// Image format (PNG, JPEG, etc.)
    pub format: String,

    /// Image dimensions
    pub width: u32,
    pub height: u32,

    /// Image data
    pub data: Vec<u8>,
}

/// Source of clipboard item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClipboardSource {
    /// Created by STT transcription
    STT {
        /// STT result that created this item
        stt_result: STTResult,
    },

    /// Manually copied by user
    Manual,

    /// Imported from file
    Import {
        /// Source file path
        file_path: String,
    },

    /// Synced from another device
    Sync {
        /// Source device identifier
        device_id: String,
    },

    /// Other source
    Other(String),
}

/// Application context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppContext {
    /// Application name
    pub name: String,

    /// Application process ID
    pub pid: Option<u32>,

    /// Window title (if applicable)
    pub window_title: Option<String>,

    /// Application bundle identifier (macOS)
    pub bundle_id: Option<String>,

    /// Application executable path
    pub executable_path: Option<String>,
}

/// Hotkey combination
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Hotkey {
    /// Control key modifier
    pub ctrl: bool,

    /// Alt key modifier
    pub alt: bool,

    /// Shift key modifier
    pub shift: bool,

    /// Windows/Command key modifier
    pub meta: bool,

    /// Main key
    pub key: String,
}

/// Audio device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    /// Device name
    pub name: String,

    /// Device identifier
    pub id: String,

    /// Supported sample rates
    pub sample_rates: Vec<u32>,

    /// Supported channel counts
    pub channels: Vec<u16>,

    /// Is this the default device?
    pub is_default: bool,

    /// Device type
    pub device_type: AudioDeviceType,
}

/// Audio device type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AudioDeviceType {
    /// Input device (microphone)
    Input,

    /// Output device (speakers)
    Output,

    /// Both input and output
    Duplex,
}

/// VAD (Voice Activity Detection) result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VADResult {
    /// Is voice detected?
    pub voice_detected: bool,

    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,

    /// Timestamp when detection occurred
    pub timestamp: DateTime<Utc>,

    /// Duration of voice activity in milliseconds
    pub duration_ms: u64,
}

/// Audio processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Sample rate in Hz
    pub sample_rate: u32,

    /// Number of channels
    pub channels: u16,

    /// Buffer size in samples
    pub buffer_size: usize,

    /// Enable noise reduction
    pub noise_reduction: bool,

    /// Enable echo cancellation
    pub echo_cancellation: bool,

    /// Enable automatic gain control
    pub auto_gain_control: bool,
}

/// STT model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct STTModel {
    /// Model name
    pub name: String,

    /// Model size (tiny, base, small, medium, large)
    pub size: String,

    /// Model file path
    pub file_path: String,

    /// Model file size in bytes
    pub file_size: u64,

    /// Supported languages
    pub languages: Vec<String>,

    /// Model version
    pub version: String,

    /// Is model downloaded?
    pub downloaded: bool,

    /// Download progress (0.0 to 1.0)
    pub download_progress: Option<f32>,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// CPU usage percentage
    pub cpu_usage: f32,

    /// Memory usage in bytes
    pub memory_usage: u64,

    /// Audio latency in milliseconds
    pub audio_latency_ms: u64,

    /// STT processing time in milliseconds
    pub stt_processing_time_ms: u64,

    /// Total end-to-end latency in milliseconds
    pub total_latency_ms: u64,

    /// Timestamp when metrics were collected
    pub timestamp: DateTime<Utc>,
}

/// Application statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppStats {
    /// Total number of transcriptions
    pub total_transcriptions: u64,

    /// Total transcription time in seconds
    pub total_transcription_time: u64,

    /// Average transcription accuracy
    pub average_accuracy: f32,

    /// Total clipboard items created
    pub total_clipboard_items: u64,

    /// Most used applications
    pub most_used_apps: Vec<(String, u64)>,

    /// Most used languages
    pub most_used_languages: Vec<(String, u64)>,

    /// Application uptime in seconds
    pub uptime_seconds: u64,
}

impl ClipboardItem {
    /// Create a new clipboard item
    pub fn new(content: ClipboardContent, source: ClipboardSource) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            content,
            source,
            created_at: now,
            accessed_at: now,
            access_count: 0,
            tags: Vec::new(),
            pinned: false,
            app_context: None,
            size_bytes: 0, // Will be calculated
        }
    }

    /// Mark item as accessed
    pub fn mark_accessed(&mut self) {
        self.accessed_at = Utc::now();
        self.access_count += 1;
    }

    /// Add a tag to the item
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// Remove a tag from the item
    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
    }

    /// Check if item has a specific tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(&tag.to_string())
    }

    /// Toggle pinned status
    pub fn toggle_pinned(&mut self) {
        self.pinned = !self.pinned;
    }
}

impl ClipboardContent {
    /// Create new text content
    pub fn new_text(text: String) -> Self {
        Self {
            text: Some(text),
            html: None,
            rtf: None,
            image: None,
            files: None,
            custom: std::collections::HashMap::new(),
        }
    }

    /// Get primary text content
    pub fn get_text(&self) -> Option<&str> {
        self.text.as_deref()
    }

    /// Check if content is text-only
    pub fn is_text_only(&self) -> bool {
        self.text.is_some()
            && self.html.is_none()
            && self.rtf.is_none()
            && self.image.is_none()
            && self.files.is_none()
            && self.custom.is_empty()
    }

    /// Check if content is empty
    pub fn is_empty(&self) -> bool {
        self.text.is_none()
            && self.html.is_none()
            && self.rtf.is_none()
            && self.image.is_none()
            && self.files.is_none()
            && self.custom.is_empty()
    }
}

impl Hotkey {
    /// Create a new hotkey
    pub fn new(key: String) -> Self {
        Self {
            ctrl: false,
            alt: false,
            shift: false,
            meta: false,
            key,
        }
    }

    /// Add control modifier
    pub fn with_ctrl(mut self) -> Self {
        self.ctrl = true;
        self
    }

    /// Add alt modifier
    pub fn with_alt(mut self) -> Self {
        self.alt = true;
        self
    }

    /// Add shift modifier
    pub fn with_shift(mut self) -> Self {
        self.shift = true;
        self
    }

    /// Add meta modifier
    pub fn with_meta(mut self) -> Self {
        self.meta = true;
        self
    }
}

impl std::fmt::Display for Hotkey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut parts = Vec::new();

        if self.ctrl {
            parts.push("Ctrl");
        }
        if self.alt {
            parts.push("Alt");
        }
        if self.shift {
            parts.push("Shift");
        }
        if self.meta {
            parts.push("Meta");
        }

        parts.push(&self.key);
        write!(f, "{}", parts.join("+"))
    }
}

impl Hotkey {
    /// Parse from string representation
    pub fn from_string(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split('+').collect();
        if parts.is_empty() {
            return Err("Empty hotkey string".to_string());
        }

        let mut hotkey = Hotkey::new(parts.last().unwrap().to_string());

        for part in &parts[..parts.len() - 1] {
            match part.to_lowercase().as_str() {
                "ctrl" => hotkey.ctrl = true,
                "alt" => hotkey.alt = true,
                "shift" => hotkey.shift = true,
                "meta" => hotkey.meta = true,
                _ => return Err(format!("Unknown modifier: {part}")),
            }
        }

        Ok(hotkey)
    }
}

impl STTResult {
    /// Create a new STT result
    pub fn new(text: String, confidence: f32, model: String, backend: String) -> Self {
        Self {
            text,
            confidence,
            language: None,
            timestamp: Utc::now(),
            processing_time_ms: 0,
            model,
            backend,
        }
    }

    /// Set language
    pub fn with_language(mut self, language: String) -> Self {
        self.language = Some(language);
        self
    }

    /// Set processing time
    pub fn with_processing_time(mut self, time_ms: u64) -> Self {
        self.processing_time_ms = time_ms;
        self
    }

    /// Check if result is high confidence
    pub fn is_high_confidence(&self) -> bool {
        self.confidence >= 0.8
    }

    /// Check if result is low confidence
    pub fn is_low_confidence(&self) -> bool {
        self.confidence < 0.6
    }
}

impl PartialSTTResult {
    /// Create a new partial result
    pub fn new(text: String, confidence: f32, is_final: bool) -> Self {
        Self {
            text,
            confidence,
            is_final,
            timestamp: Utc::now(),
        }
    }

    /// Mark as final result
    pub fn mark_final(mut self) -> Self {
        self.is_final = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_item_creation() {
        let content = ClipboardContent::new_text("Hello, World!".to_string());
        let source = ClipboardSource::STT {
            stt_result: STTResult::new(
                "Hello, World!".to_string(),
                0.95,
                "base".to_string(),
                "local".to_string(),
            ),
        };

        let item = ClipboardItem::new(content, source);

        assert_eq!(item.access_count, 0);
        assert!(!item.pinned);
        assert!(item.tags.is_empty());
    }

    #[test]
    fn test_hotkey_creation() {
        let hotkey = Hotkey::new("S".to_string()).with_ctrl().with_alt();

        assert!(hotkey.ctrl);
        assert!(hotkey.alt);
        assert!(!hotkey.shift);
        assert!(!hotkey.meta);
        assert_eq!(hotkey.key, "S");
    }

    #[test]
    fn test_hotkey_string_conversion() {
        let hotkey = Hotkey::new("S".to_string()).with_ctrl().with_alt();

        let string_repr = hotkey.to_string();
        assert_eq!(string_repr, "Ctrl+Alt+S");

        let parsed = Hotkey::from_string(&string_repr).unwrap();
        assert_eq!(parsed, hotkey);
    }

    #[test]
    fn test_stt_result_confidence() {
        let high_conf = STTResult::new(
            "Hello".to_string(),
            0.9,
            "base".to_string(),
            "local".to_string(),
        );

        let low_conf = STTResult::new(
            "Hello".to_string(),
            0.5,
            "base".to_string(),
            "local".to_string(),
        );

        assert!(high_conf.is_high_confidence());
        assert!(low_conf.is_low_confidence());
    }

    #[test]
    fn test_clipboard_content_methods() {
        let content = ClipboardContent::new_text("Test".to_string());

        assert!(content.is_text_only());
        assert!(!content.is_empty());
        assert_eq!(content.get_text(), Some("Test"));
    }
}
