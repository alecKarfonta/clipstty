//! Services module containing the core business logic services.

pub mod audio;
pub mod stt;
pub mod clipboard;
pub mod hotkey;
pub mod paste;

pub use audio::AudioService;
pub use stt::STTService;
pub use clipboard::ClipboardService;
pub use hotkey::HotkeyService;
pub use paste::PasteService;
