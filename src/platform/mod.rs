//! Platform-specific implementations for different operating systems.

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;

pub mod common;

use crate::Result;

/// Initialize platform-specific components
pub fn init() -> Result<()> {
    #[cfg(target_os = "linux")]
    linux::init()?;
    
    #[cfg(target_os = "macos")]
    macos::init()?;
    
    #[cfg(target_os = "windows")]
    windows::init()?;
    
    Ok(())
}

/// Cleanup platform-specific components
pub fn cleanup() -> Result<()> {
    #[cfg(target_os = "linux")]
    linux::cleanup()?;
    
    #[cfg(target_os = "macos")]
    macos::cleanup()?;
    
    #[cfg(target_os = "windows")]
    windows::cleanup()?;
    
    Ok(())
}
