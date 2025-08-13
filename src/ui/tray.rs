//! System tray integration for STT Clippy.

use crate::Result;
use tracing::info;

/// System tray interface
pub struct SystemTray {
    // TODO: Implement system tray
}

impl SystemTray {
    /// Create a new system tray
    pub fn new() -> Result<Self> {
        // TODO: Initialize system tray
        Ok(Self {})
    }

    /// Show the system tray
    pub fn show(&self) -> Result<()> {
        info!("Tray show called");
        Ok(())
    }

    /// Hide the system tray
    pub fn hide(&self) -> Result<()> {
        info!("Tray hide called");
        Ok(())
    }

    /// Update tray icon
    pub fn update_icon(&self, _icon_name: &str) -> Result<()> {
        info!(icon = %_icon_name, "Tray update_icon called");
        Ok(())
    }
}
