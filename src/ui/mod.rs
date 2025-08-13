//! User interface components and system tray integration.

pub mod tray;
pub mod settings;
pub mod history_palette;

pub use tray::SystemTray;
pub use settings::SettingsWindow;
pub use history_palette::HistoryPalette;
