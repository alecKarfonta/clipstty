//! Paste service for injecting text at the cursor position.

use crate::Result;
use enigo::{Enigo, Keyboard, Key, Direction, Settings};
use std::thread;
use std::time::Duration;
use arboard::Clipboard;

/// Paste service for injecting text at cursor position
pub struct PasteService {
    enigo: Enigo,
}

impl PasteService {
    /// Create a new paste service
    pub fn new() -> Result<Self> {
        let settings = Settings::default();
        let enigo = Enigo::new(&settings).map_err(|e| crate::core::error::ClipboardError::Write(format!("enigo init: {e}")))?;
        Ok(Self { enigo })
    }

    /// Inject text at cursor position
    pub fn inject_text(&mut self, text: &str) -> Result<()> {
        // Type text directly if supported
        self.enigo.text(text)
            .map_err(|e| crate::core::error::ClipboardError::Write(format!("text injection failed: {e}")))?;
        Ok(())
    }

    /// Check if paste injection is supported
    pub fn is_supported(&self) -> bool {
        true
    }

    /// Get fallback method
    pub fn get_fallback_method(&self) -> crate::core::config::FallbackMethod {
        crate::core::config::FallbackMethod::Clipboard
    }

    /// Copy text to clipboard and send paste keystroke, with small delay
    pub fn clipboard_paste(&mut self, text: &str, delay_ms: u64) -> Result<()> {
        let mut clipboard = Clipboard::new()
            .map_err(|e| crate::core::error::ClipboardError::Write(e.to_string()))?;
        clipboard
            .set_text(text.to_string())
            .map_err(|e| crate::core::error::ClipboardError::Write(e.to_string()))?;
        if delay_ms > 0 { thread::sleep(Duration::from_millis(delay_ms)); }
        // On macOS use Meta (Command), otherwise Ctrl
        #[cfg(target_os = "macos")] {
            let _ = self.enigo.key(Key::Meta, Direction::Press);
            let _ = self.enigo.key(Key::Unicode('v'), Direction::Click);
            let _ = self.enigo.key(Key::Meta, Direction::Release);
        }
        #[cfg(not(target_os = "macos"))] {
            let _ = self.enigo.key(Key::Control, Direction::Press);
            let _ = self.enigo.key(Key::Unicode('v'), Direction::Click);
            let _ = self.enigo.key(Key::Control, Direction::Release);
        }
        Ok(())
    }
}
