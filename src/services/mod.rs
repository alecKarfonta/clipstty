//! Services module containing the core business logic services.

pub mod audio;
pub mod clipboard;
pub mod hotkey;
pub mod paste;
pub mod stt;
pub mod vad;
pub mod voice_commands;
pub mod audio_archive;
pub mod audio_storage;
pub mod audio_menu;
pub mod transcription_log;
pub mod transcription_deduplication;
pub mod transcription_search;
pub mod transcription_analytics;

pub use audio::AudioService;
pub use clipboard::ClipboardService;
pub use hotkey::HotkeyService;
pub use paste::PasteService;
pub use stt::STTService;
pub use vad::{VADService, VADMode};
pub use voice_commands::{VoiceCommandEngine, VoiceCommand, CommandCategory, CommandResult, VoiceCommandError};
pub use audio_archive::{AudioArchiveService, AudioError, RecordingSession};
pub use audio_storage::FileAudioStorage;
pub use audio_menu::AudioRecordingMenu;
pub use transcription_log::{TranscriptionLogService, TranscriptEntry, TranscriptError};
pub use transcription_deduplication::TranscriptDeduplicator;
pub use transcription_search::TranscriptIndexer;
pub use transcription_analytics::TranscriptAnalytics;
