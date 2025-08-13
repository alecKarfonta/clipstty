//! Integration tests for STT Clippy.

use stt_clippy::{init, cleanup, Result};

#[tokio::test]
async fn test_application_lifecycle() -> Result<()> {
    // Test initialization
    init(None, None)?;
    
    // Test cleanup
    cleanup()?;
    
    Ok(())
}

#[test]
fn test_configuration_defaults() {
    use stt_clippy::core::config::Config;
    
    let config = Config::new();
    
    // Verify default values
    assert_eq!(config.audio.sample_rate, 16000);
    assert_eq!(config.stt.model_size, "base");
    assert_eq!(config.clipboard.max_history, 500);
    assert_eq!(config.hotkeys.primary, "Ctrl+Alt+S");
}

#[test]
fn test_hotkey_parsing() {
    use stt_clippy::core::types::Hotkey;
    
    // Test valid hotkey parsing
    let hotkey = Hotkey::from_string("Ctrl+Alt+S").unwrap();
    assert!(hotkey.ctrl);
    assert!(hotkey.alt);
    assert!(!hotkey.shift);
    assert!(!hotkey.meta);
    assert_eq!(hotkey.key, "S");
    
    // Test string conversion
    let string_repr = hotkey.to_string();
    assert_eq!(string_repr, "Ctrl+Alt+S");
}

#[test]
fn test_clipboard_item_creation() {
    use stt_clippy::core::types::{ClipboardItem, ClipboardContent, ClipboardSource, STTResult};
    
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
    assert!(item.content.is_text_only());
}
