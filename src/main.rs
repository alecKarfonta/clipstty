//! STT Clippy - Main Application Entry Point
//!
//! This is the main entry point for the STT Clippy desktop application.

use std::process;
use std::sync::{Arc, Mutex};
use stt_clippy::{cleanup, init, get_config, Result};
use stt_clippy::services::{AudioService, STTService, VADService, VADMode};
use stt_clippy::core::config::ActivationMode;
use stt_clippy::services::hotkey::HotkeyService;
use stt_clippy::core::types::Hotkey;
use stt_clippy::ui::SystemTray;
use tracing::{info};

#[tokio::main]
async fn main() {
    // Initialize the application
    if let Err(e) = init(None, None) {
        eprintln!("Failed to initialize STT Clippy: {e}");
        process::exit(1);
    }

    // Run the main application
    if let Err(e) = run_app().await {
        eprintln!("Application error: {e}");
        process::exit(1);
    }

    // Cleanup before exit
    if let Err(e) = cleanup() {
        eprintln!("Failed to cleanup STT Clippy: {e}");
        process::exit(1);
    }
}

/// Main application loop
async fn run_app() -> Result<()> {
    info!("STT Clippy v{} starting...", env!("CARGO_PKG_VERSION"));

    let cfg = get_config().clone();

    // Minimal tray bootstrap
    let tray = SystemTray::new()?;
    if cfg.ui.show_tray { tray.show()?; }

    // Initialize STT with config
    let mut stt = STTService::new()?;
    stt.apply_config(cfg.stt.clone())?;

    // Initialize VAD using config
    let vad = VADService::new(
        cfg.audio.vad_sensitivity,
        cfg.audio.vad_timeout,
        match cfg.audio.activation_mode {
            ActivationMode::PushToTalk => VADMode::PushToTalk,
            ActivationMode::Toggle => VADMode::Toggle,
        },
    )?;

    // Enumerate audio devices and start/stop
    let mut audio = AudioService::new()?;
    if !cfg.audio.device_name.is_empty() {
        audio.select_input_device_by_name(Some(cfg.audio.device_name.clone()));
    }
    let devices = audio.get_devices()?;
    info!(available_devices = devices.len(), "Audio devices enumerated");

    // Attach VAD and register a simple handler for detections
    let vad_shared = Arc::new(Mutex::new(vad));
    audio.attach_vad(vad_shared);
    audio.on_vad_event(|res| {
        // In this handler you can trigger STT start/stop, buffer routing, etc.
        // For now, just log detections for visibility.
        if res.voice_detected {
            tracing::info!(confidence = res.confidence, duration_ms = res.duration_ms, "VAD: voice detected");
        }
    });

    // Hotkey wiring for Push-to-Talk / Toggle
    let mut hotkeys = HotkeyService::new()?;
    let primary = Hotkey::from_string(&cfg.hotkeys.primary)
        .unwrap_or_else(|_| Hotkey::new("S".to_string()).with_ctrl().with_alt());
    let _ = hotkeys.register_hotkey(&primary);

    match cfg.audio.activation_mode {
        ActivationMode::PushToTalk => {
            let audio_ref = Arc::new(Mutex::new(audio));
            let audio_press = audio_ref.clone();
            let audio_release = audio_ref.clone();
            hotkeys.on_press(&primary, Arc::new(move || {
                if let Ok(mut a) = audio_press.lock() {
                    let _ = a.start_capture();
                    a.set_ptt_gate(true);
                }
            }));
            hotkeys.on_release(&primary, Arc::new(move || {
                if let Ok(mut a) = audio_release.lock() {
                    a.set_ptt_gate(false);
                    let _ = a.stop_capture();
                }
            }));
            // Keep a reference to prevent drop
            let _keep_audio = audio_ref;
        }
        ActivationMode::Toggle => {
            // Single press toggles capture/gate
            let audio_ref = Arc::new(Mutex::new(audio));
            hotkeys.on_press(&primary, Arc::new(move || {
                if let Ok(mut a) = audio_ref.lock() {
                    let _ = a.toggle_capture();
                }
            }));
        }
    }

    // Placeholder event loop tick (simulate running app)
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    info!("STT Clippy started successfully!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use stt_clippy::init;

    #[tokio::test]
    async fn test_run_app() {
        let _ = init(None, None);
        let result = run_app().await;
        assert!(result.is_ok());
    }
}
