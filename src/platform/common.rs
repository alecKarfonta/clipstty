//! Common platform functionality shared across different operating systems.

/// Common platform utilities
pub struct PlatformUtils;

impl PlatformUtils {
    /// Get current platform name
    pub fn get_platform() -> &'static str {
        if cfg!(target_os = "linux") {
            "linux"
        } else if cfg!(target_os = "macos") {
            "macos"
        } else if cfg!(target_os = "windows") {
            "windows"
        } else {
            "unknown"
        }
    }

    /// Check if running on Linux
    pub fn is_linux() -> bool {
        cfg!(target_os = "linux")
    }

    /// Check if running on macOS
    pub fn is_macos() -> bool {
        cfg!(target_os = "macos")
    }

    /// Check if running on Windows
    pub fn is_windows() -> bool {
        cfg!(target_os = "windows")
    }
}
